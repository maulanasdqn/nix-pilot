//! Deployment job management

use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::executor::DeployExecutor;
use super::types::{
    DeployJob, DeployJobId, DeployJobSummary, DeployPhase, DeployRequest, DeployStatus,
    GenerationInfo, RollbackRequest,
};

/// Manager for deployment jobs
pub struct DeployManager {
    jobs: Arc<RwLock<HashMap<DeployJobId, DeployJob>>>,
    executor: DeployExecutor,
    max_concurrent_jobs: usize,
}

impl DeployManager {
    pub fn new(executor: DeployExecutor, max_concurrent_jobs: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            executor,
            max_concurrent_jobs,
        }
    }

    /// List all deployment jobs
    pub async fn list(&self) -> Vec<DeployJobSummary> {
        let jobs = self.jobs.read().await;
        jobs.values().map(DeployJobSummary::from).collect()
    }

    /// Get a specific job
    pub async fn get(&self, id: &DeployJobId) -> Result<DeployJob> {
        let jobs = self.jobs.read().await;
        jobs.get(id)
            .cloned()
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))
    }

    /// Create a new deployment job
    pub async fn create(&self, request: DeployRequest) -> Result<DeployJob> {
        let job = DeployJob::new(request);
        let id = job.id.clone();

        let mut jobs = self.jobs.write().await;

        // Check concurrent job limit
        let running_count = jobs
            .values()
            .filter(|j| j.status == DeployStatus::Running)
            .count();

        if running_count >= self.max_concurrent_jobs {
            return Err(NpError::Other(format!(
                "Maximum concurrent deployment jobs ({}) reached",
                self.max_concurrent_jobs
            )));
        }

        jobs.insert(id.clone(), job.clone());
        Ok(job)
    }

    /// Start a deployment job
    pub async fn start(&self, id: &DeployJobId, output_tx: mpsc::Sender<OutputLine>) -> Result<()> {
        // Get the current generation before deployment (for rollback info)
        let previous_gen = {
            let jobs = self.jobs.read().await;
            let job = jobs
                .get(id)
                .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

            self.executor
                .get_current_generation(&job.request.target)
                .await
                .ok()
        };

        // Update job status to running
        {
            let mut jobs = self.jobs.write().await;
            let job = jobs
                .get_mut(id)
                .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

            if job.status != DeployStatus::Pending {
                return Err(NpError::Other(format!(
                    "Job {} is not in pending state",
                    id
                )));
            }

            job.start();
            job.previous_generation = previous_gen;
        }

        // Get the request for the deployment
        let request = {
            let jobs = self.jobs.read().await;
            let job = jobs
                .get(id)
                .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;
            job.request.clone()
        };

        // Create channels for phase updates and output
        let (phase_tx, mut phase_rx) = mpsc::channel::<DeployPhase>(10);
        let output_tx_clone = output_tx.clone();

        // Clone for the background tasks
        let jobs = self.jobs.clone();
        let id_clone = id.clone();

        // Spawn a task to update job phase
        let jobs_phase = jobs.clone();
        let id_phase = id.clone();
        tokio::spawn(async move {
            while let Some(phase) = phase_rx.recv().await {
                let mut jobs = jobs_phase.write().await;
                if let Some(job) = jobs.get_mut(&id_phase) {
                    job.set_phase(phase);
                }
            }
        });

        // Spawn a task to collect output
        let (line_tx, mut line_rx) = mpsc::channel::<OutputLine>(100);
        let jobs_output = jobs.clone();
        let id_output = id.clone();
        tokio::spawn(async move {
            while let Some(line) = line_rx.recv().await {
                // Store in job
                {
                    let mut jobs = jobs_output.write().await;
                    if let Some(job) = jobs.get_mut(&id_output) {
                        job.add_output(line.content.clone());
                    }
                }
                // Forward to output channel
                let _ = output_tx_clone.send(line).await;
            }
        });

        // Run the deployment
        let result = self.executor.deploy(&request, line_tx, phase_tx).await;

        // Update job with result
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&id_clone) {
            match result {
                Ok(exit_code) => {
                    job.complete(exit_code);

                    // If deployment failed and rollback is enabled, attempt rollback
                    if exit_code != 0 && request.rollback_on_failure {
                        if let Some(prev_gen) = job.previous_generation {
                            let _ = output_tx
                                .send(OutputLine::stderr(format!(
                                    "Deployment failed, rolling back to generation {}",
                                    prev_gen
                                )))
                                .await;

                            // Attempt rollback (best effort)
                            let rollback_req = RollbackRequest {
                                target: request.target.clone(),
                                generation: Some(prev_gen),
                            };
                            let (rollback_tx, _) = mpsc::channel(100);
                            let _ = self.executor.rollback(&rollback_req, rollback_tx).await;
                        }
                    }
                }
                Err(e) => {
                    job.fail(e.to_string());
                }
            }
        }

        Ok(())
    }

    /// Cancel a deployment job
    pub async fn cancel(&self, id: &DeployJobId) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

        if job.status == DeployStatus::Completed || job.status == DeployStatus::Failed {
            return Err(NpError::Other(format!("Job {} is already finished", id)));
        }

        job.cancel();
        Ok(())
    }

    /// Delete a completed job
    pub async fn delete(&self, id: &DeployJobId) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

        if job.status == DeployStatus::Running {
            return Err(NpError::Other(format!("Cannot delete running job {}", id)));
        }

        jobs.remove(id);
        Ok(())
    }

    /// Rollback a target to a previous generation
    pub async fn rollback(
        &self,
        request: RollbackRequest,
        tx: mpsc::Sender<OutputLine>,
    ) -> Result<i32> {
        self.executor.rollback(&request, tx).await
    }

    /// List generations on a target
    pub async fn list_generations(
        &self,
        target: &crate::ssh::SshTarget,
    ) -> Result<Vec<GenerationInfo>> {
        self.executor.list_generations(target).await
    }

    /// Clean up old completed jobs
    pub async fn cleanup(&self, max_age_hours: u64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        let mut jobs = self.jobs.write().await;
        jobs.retain(|_, job| {
            // Keep running or pending jobs
            if job.status == DeployStatus::Running || job.status == DeployStatus::Pending {
                return true;
            }

            // Keep recent completed jobs
            if let Some(completed_at) = job.completed_at {
                completed_at > cutoff
            } else {
                job.created_at > cutoff
            }
        });
    }

    /// Get output for a job (streaming via channel)
    pub async fn subscribe_output(
        &self,
        id: &DeployJobId,
    ) -> Result<(DeployJob, mpsc::Receiver<OutputLine>)> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?
            .clone();

        // Create a channel for future output
        let (tx, rx) = mpsc::channel(100);

        // If job is already done, send existing output
        if job.status != DeployStatus::Running && job.status != DeployStatus::Pending {
            let output_lines = job.output_lines.clone();
            tokio::spawn(async move {
                for line in output_lines.iter() {
                    let _ = tx.send(OutputLine::stdout(line.clone())).await;
                }
            });
        }

        Ok((job, rx))
    }
}

impl Default for DeployManager {
    fn default() -> Self {
        Self::new(DeployExecutor::default(), 4)
    }
}

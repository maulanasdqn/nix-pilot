//! Job management for tracking installation jobs

use crate::error::{NpError, Result};
use crate::nix::OutputLine;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::installer::NixosAnywhereInstaller;
use super::types::{InstallJob, InstallPhase, InstallRequest, JobId, JobStatus, JobSummary};

/// Manager for installation jobs
pub struct JobManager {
    jobs: Arc<RwLock<HashMap<JobId, InstallJob>>>,
    installer: NixosAnywhereInstaller,
    max_concurrent_jobs: usize,
}

impl JobManager {
    pub fn new(installer: NixosAnywhereInstaller, max_concurrent_jobs: usize) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            installer,
            max_concurrent_jobs,
        }
    }

    /// List all jobs
    pub async fn list(&self) -> Vec<JobSummary> {
        let jobs = self.jobs.read().await;
        jobs.values().map(JobSummary::from).collect()
    }

    /// Get a specific job
    pub async fn get(&self, id: &JobId) -> Result<InstallJob> {
        let jobs = self.jobs.read().await;
        jobs.get(id)
            .cloned()
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))
    }

    /// Create a new installation job
    pub async fn create(&self, request: InstallRequest) -> Result<InstallJob> {
        let job = InstallJob::new(request);
        let id = job.id.clone();

        let mut jobs = self.jobs.write().await;

        // Check concurrent job limit
        let running_count = jobs
            .values()
            .filter(|j| j.status == JobStatus::Running)
            .count();

        if running_count >= self.max_concurrent_jobs {
            return Err(NpError::Other(format!(
                "Maximum concurrent jobs ({}) reached",
                self.max_concurrent_jobs
            )));
        }

        jobs.insert(id.clone(), job.clone());
        Ok(job)
    }

    /// Start an installation job
    pub async fn start(
        &self,
        id: &JobId,
        output_tx: mpsc::Sender<OutputLine>,
    ) -> Result<()> {
        // Update job status to running
        {
            let mut jobs = self.jobs.write().await;
            let job = jobs
                .get_mut(id)
                .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

            if job.status != JobStatus::Pending {
                return Err(NpError::Other(format!(
                    "Job {} is not in pending state",
                    id
                )));
            }

            job.start();
        }

        // Get the request for the installation
        let request = {
            let jobs = self.jobs.read().await;
            let job = jobs
                .get(id)
                .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;
            job.request.clone()
        };

        // Create channels for phase updates and output
        let (phase_tx, mut phase_rx) = mpsc::channel::<InstallPhase>(10);
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

        // Run the installation
        let result = if request.vm_test {
            self.installer.vm_test(&request, line_tx).await
        } else {
            self.installer.install(&request, line_tx, phase_tx).await
        };

        // Update job with result
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&id_clone) {
            match result {
                Ok(exit_code) => {
                    job.complete(exit_code);
                }
                Err(e) => {
                    job.fail(e.to_string());
                }
            }
        }

        Ok(())
    }

    /// Cancel a job
    pub async fn cancel(&self, id: &JobId) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

        if job.status == JobStatus::Completed || job.status == JobStatus::Failed {
            return Err(NpError::Other(format!(
                "Job {} is already finished",
                id
            )));
        }

        job.cancel();
        Ok(())
    }

    /// Delete a completed job
    pub async fn delete(&self, id: &JobId) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        let job = jobs
            .get(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?;

        if job.status == JobStatus::Running {
            return Err(NpError::Other(format!(
                "Cannot delete running job {}",
                id
            )));
        }

        jobs.remove(id);
        Ok(())
    }

    /// Clean up old completed jobs
    pub async fn cleanup(&self, max_age_hours: u64) {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);

        let mut jobs = self.jobs.write().await;
        jobs.retain(|_, job| {
            // Keep running or pending jobs
            if job.status == JobStatus::Running || job.status == JobStatus::Pending {
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
    pub async fn subscribe_output(&self, id: &JobId) -> Result<(InstallJob, mpsc::Receiver<OutputLine>)> {
        let jobs = self.jobs.read().await;
        let job = jobs
            .get(id)
            .ok_or_else(|| NpError::JobNotFound(id.to_string()))?
            .clone();

        // Create a channel for future output (for running jobs)
        let (tx, rx) = mpsc::channel(100);

        // If job is already done, send existing output
        if job.status != JobStatus::Running && job.status != JobStatus::Pending {
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

impl Default for JobManager {
    fn default() -> Self {
        Self::new(NixosAnywhereInstaller::default(), 4)
    }
}

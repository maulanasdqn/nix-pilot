use crate::auth::AuthState;
use np_core::{
    CoreConfig, DeployExecutor, DeployManager, FlakeManager, JobManager, MachineManager,
    NixExecutor, NixosAnywhereInstaller,
};
use std::sync::Arc;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<CoreConfig>,
    pub nix: Arc<NixExecutor>,
    pub machines: Arc<MachineManager>,
    pub flakes: Arc<FlakeManager>,
    pub jobs: Arc<JobManager>,
    pub deploys: Arc<DeployManager>,
    pub auth: AuthState,
}

impl AppState {
    pub fn new(config: CoreConfig) -> Self {
        let nix = NixExecutor::new(config.nix_path.clone())
            .with_timeout(config.command_timeout_secs);

        let machines = MachineManager::new(config.machines_dir.clone());

        // Create a separate NixExecutor for FlakeManager
        let flake_nix = NixExecutor::new(config.nix_path.clone())
            .with_timeout(config.command_timeout_secs);
        let flakes = FlakeManager::new(config.flakes_dir.clone(), flake_nix);

        // Create job manager for installations
        let installer = NixosAnywhereInstaller::new();
        let jobs = JobManager::new(installer, config.max_concurrent_jobs);

        // Create deploy manager
        let deploy_executor = DeployExecutor::new();
        let deploys = DeployManager::new(deploy_executor, config.max_concurrent_jobs);

        // Create auth state
        let auth = AuthState::from_env();

        Self {
            config: Arc::new(config),
            nix: Arc::new(nix),
            machines: Arc::new(machines),
            flakes: Arc::new(flakes),
            jobs: Arc::new(jobs),
            deploys: Arc::new(deploys),
            auth,
        }
    }

    /// Initialize the state (load data from disk)
    pub async fn init(&self) -> np_core::Result<()> {
        self.machines.load().await?;
        Ok(())
    }
}

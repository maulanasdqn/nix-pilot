mod dashboard;
pub mod deploy;
pub mod flakes;
pub mod install;
pub mod machines;
pub mod nix;
mod not_found;
pub mod services;

pub use dashboard::DashboardPage;
pub use deploy::{DeployProgressPage, DeployWizardPage};
pub use flakes::{AddFlakePage, FlakeDetailPage, FlakeListPage};
pub use install::InstallWizardPage;
pub use machines::{AddMachinePage, MachineDetailPage, MachineListPage};
pub use nix::NixOperationsPage;
pub use not_found::NotFoundPage;
pub use services::{ServiceDetailPage, ServiceListPage, ServiceLogsPage};

mod dashboard;
pub mod flakes;
mod login;
pub mod nix;
mod not_found;
mod rebuild;
pub mod secrets;
pub mod services;
pub mod settings;

pub use dashboard::DashboardPage;
pub use flakes::{AddFlakePage, FlakeDetailPage, FlakeListPage};
pub use login::LoginPage;
pub use nix::NixOperationsPage;
pub use not_found::NotFoundPage;
pub use rebuild::RebuildPage;
pub use secrets::{AddSecretPage, KeysPage, SecretsListPage};
pub use services::{ServiceDetailPage, ServiceListPage, ServiceLogsPage};
pub use settings::SettingsPage;

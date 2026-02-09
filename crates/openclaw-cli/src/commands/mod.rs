//! CLI command implementations.

pub mod admin;
pub mod completion;
pub mod config;
pub mod configure;
pub mod daemon;
pub mod doctor;
pub mod gateway;
pub mod onboard;
pub mod status;

pub use admin::run_admin;
pub use completion::run_completion;
pub use config::run_config;
pub use configure::run_configure;
pub use daemon::run_daemon;
pub use doctor::run_doctor;
pub use gateway::run_gateway;
pub use onboard::run_onboard;
pub use status::run_status;

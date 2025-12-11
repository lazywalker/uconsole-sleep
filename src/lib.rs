//! Console Sleep Service Library
//! Pure Rust implementation with zero external dependencies

pub mod config;
pub mod error;
pub mod hardware;
pub mod power_mode;

pub use config::Config;
pub use error::Error;
pub use hardware::CpuFreqConfig;
pub use hardware::wifi::WifiConfig;
pub use hardware::*;
pub use power_mode::PowerMode;
pub use power_mode::{enter_saving_mode, exit_saving_mode};

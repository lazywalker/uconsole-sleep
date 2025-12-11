//! Console Sleep Service Library
//! Pure Rust implementation with zero external dependencies

pub mod config;
pub mod error;
pub mod hardware;
pub mod logger;
pub mod power_mode;

pub use config::Config;
pub use cpu::CpuFreqConfig;
pub use error::Error;
pub use hardware::*;
pub use power_mode::PowerMode;
pub use power_mode::{enter_saving_mode, exit_saving_mode};
pub use wifi::WifiConfig;

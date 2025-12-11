//! Hardware detection modules

pub mod backlight;
pub mod cpu;
pub mod drm_panel;
pub mod framebuffer;
pub mod internal_kb;
pub mod power_key;
pub mod wifi;

pub use backlight::find_backlight;
pub use cpu::CpuFreqConfig;
pub use drm_panel::find_drm_panel;
pub use framebuffer::find_framebuffer;
pub use internal_kb::find_internal_kb;
pub use power_key::find_power_key;
pub use wifi::find_default_rfkill_path;

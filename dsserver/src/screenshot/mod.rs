#[cfg(target_os = "windows")]
pub mod win32;
pub use win32::Screen;

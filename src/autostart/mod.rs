#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{is_enabled, set_enabled};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{is_enabled, set_enabled};

/// Keeps the login item pointing at this binary if the app was moved since it was enabled.
pub fn refresh() {
    if is_enabled()
        && let Err(e) = set_enabled(true)
    {
        eprintln!("launch at login: {e}");
    }
}

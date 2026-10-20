use std::{env, io};

use windows_registry::CURRENT_USER;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "Deezer Presence";

pub fn is_enabled() -> bool {
    CURRENT_USER
        .open(RUN_KEY)
        .and_then(|key| key.get_string(VALUE))
        .is_ok()
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    let key = CURRENT_USER.create(RUN_KEY).map_err(io::Error::other)?;
    if enabled {
        let exe = env::current_exe()?;
        key.set_string(VALUE, format!("\"{}\"", exe.display()))
            .map_err(io::Error::other)
    } else if is_enabled() {
        key.remove_value(VALUE).map_err(io::Error::other)
    } else {
        Ok(())
    }
}

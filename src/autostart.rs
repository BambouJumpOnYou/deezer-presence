use std::path::PathBuf;
use std::{env, fs, io};

const LABEL: &str = "io.github.bamboujumponyou.deezer-presence";

fn plist_path() -> Option<PathBuf> {
    env::home_dir().map(|home| {
        home.join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist"))
    })
}

pub fn is_enabled() -> bool {
    plist_path().is_some_and(|p| p.exists())
}

pub fn set_enabled(enabled: bool) -> io::Result<()> {
    let path = plist_path().ok_or_else(|| io::Error::other("no home directory"))?;
    if !enabled {
        return match fs::remove_file(&path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        };
    }
    let exe = env::current_exe()?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(&path, plist(&exe.to_string_lossy()))
}

/// Keeps the LaunchAgent pointing at this binary if the app was moved since it was enabled.
pub fn refresh() {
    if is_enabled()
        && let Err(e) = set_enabled(true)
    {
        eprintln!("launch at login: {e}");
    }
}

fn plist(exe: &str) -> String {
    let exe = exe
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
"#
    )
}

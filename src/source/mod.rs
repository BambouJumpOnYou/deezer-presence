use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
compile_error!("only macOS is supported for now");

#[derive(Debug, Clone, PartialEq)]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub playing: bool,
    pub duration_ms: Option<i64>,
    /// Elapsed position, valid at `observed_at_ms` (epoch).
    pub elapsed_ms: i64,
    pub observed_at_ms: i64,
}

impl NowPlaying {
    pub fn started_at_ms(&self) -> i64 {
        self.observed_at_ms - self.elapsed_ms
    }
}

/// Sends `Some` on every change of Deezer's state, `None` when Deezer stops reporting.
pub fn spawn(tx: Sender<Option<NowPlaying>>) {
    #[cfg(target_os = "macos")]
    macos::spawn(tx);
}

/// Stops the platform helper process, if any.
pub fn shutdown() {
    #[cfg(target_os = "macos")]
    macos::shutdown();
}

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

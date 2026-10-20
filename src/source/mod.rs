use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
mod windows;

#[cfg(not(any(target_os = "macos", windows)))]
compile_error!("only macOS and Windows are supported");

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

    /// Same track and state, ignoring small drift of the computed start time (a seek counts as a change).
    pub fn same_moment(&self, other: &Self) -> bool {
        self.title == other.title
            && self.artist == other.artist
            && self.playing == other.playing
            && (self.started_at_ms() - other.started_at_ms()).abs() < 2000
    }
}

/// Sends `Some` on every change of Deezer's state, `None` when Deezer stops reporting.
pub fn spawn(tx: Sender<Option<NowPlaying>>) {
    #[cfg(target_os = "macos")]
    macos::spawn(tx);
    #[cfg(windows)]
    windows::spawn(tx);
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

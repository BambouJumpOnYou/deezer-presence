use std::env;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use super::{NowPlaying, now_ms};

const DEEZER_BUNDLE_ID: &str = "com.deezer.deezer-desktop";

static ADAPTER: Mutex<Option<Child>> = Mutex::new(None);

#[derive(Deserialize)]
struct Event {
    payload: Payload,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    bundle_identifier: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    playing: Option<bool>,
    duration_micros: Option<f64>,
    elapsed_time_micros: Option<f64>,
    timestamp_epoch_micros: Option<f64>,
    playback_rate: Option<f64>,
}

impl Payload {
    fn into_now_playing(self) -> Option<NowPlaying> {
        if self.bundle_identifier.as_deref() != Some(DEEZER_BUNDLE_ID) {
            return None;
        }
        let playing = self.playing.unwrap_or(false);
        let now = now_ms();
        let mut elapsed_ms = (self.elapsed_time_micros.unwrap_or(0.0) / 1000.0) as i64;
        if let Some(ts) = self.timestamp_epoch_micros {
            let rate = if playing {
                self.playback_rate.unwrap_or(1.0)
            } else {
                0.0
            };
            elapsed_ms += ((now as f64 - ts / 1000.0) * rate) as i64;
        }
        Some(NowPlaying {
            title: self.title?,
            artist: self.artist.unwrap_or_default(),
            album: self.album,
            playing,
            duration_ms: self.duration_micros.map(|d| (d / 1000.0) as i64),
            elapsed_ms: elapsed_ms.max(0),
            observed_at_ms: now,
        })
    }
}

fn adapter_dir() -> PathBuf {
    if let Ok(dir) = env::var("MEDIAREMOTE_ADAPTER_DIR") {
        return PathBuf::from(dir);
    }
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from));
    exe_dir
        .iter()
        .flat_map(|dir| {
            [
                dir.join("../Resources/mediaremote-adapter"),
                dir.join("mediaremote-adapter"),
            ]
        })
        .find(|dir| dir.exists())
        .unwrap_or_else(|| {
            PathBuf::from(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/vendor/mediaremote-adapter"
            ))
        })
}

pub fn spawn(tx: Sender<Option<NowPlaying>>) {
    thread::spawn(move || {
        let dir = adapter_dir();
        loop {
            if let Err(e) = stream(&dir, &tx) {
                eprintln!("mediaremote-adapter: {e}");
            }
            if tx.send(None).is_err() {
                return;
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}

fn stream(dir: &Path, tx: &Sender<Option<NowPlaying>>) -> std::io::Result<()> {
    let mut child = Command::new("/usr/bin/perl")
        .arg(dir.join("mediaremote-adapter.pl"))
        .arg(dir.join("MediaRemoteAdapter.framework"))
        .args([
            "stream",
            "--no-diff",
            "--no-artwork",
            "--micros",
            "--debounce=300",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    *ADAPTER.lock().unwrap() = Some(child);

    for line in BufReader::new(stdout).lines() {
        let line = line?;
        let Ok(event) = serde_json::from_str::<Event>(&line) else {
            continue;
        };
        if tx.send(event.payload.into_now_playing()).is_err() {
            break;
        }
    }
    shutdown();
    Err(std::io::Error::other("adapter exited"))
}

pub fn shutdown() {
    if let Some(mut child) = ADAPTER.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

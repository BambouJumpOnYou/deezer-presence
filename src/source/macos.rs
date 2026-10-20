use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{env, fs, thread};

use serde::Deserialize;

use super::{NowPlaying, now_ms};

/// Built from macos/helper.m by build.rs.
const HELPER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/helper.dylib"));

/// Loads the helper into /usr/bin/perl, which is allowed to use MediaRemote.
const PERL_LOADER: &str = r#"
use DynaLoader;
my $handle = DynaLoader::dl_load_file($ARGV[0], 0) or die "cannot load helper\n";
my $symbol = DynaLoader::dl_find_symbol($handle, "deezer_presence_poll") or die "helper symbol missing\n";
DynaLoader::dl_install_xsub("main::poll", $symbol);
poll();
"#;

static HELPER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);

#[derive(Deserialize)]
struct Payload {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: Option<f64>,
    elapsed: Option<f64>,
    timestamp: Option<f64>,
    rate: Option<f64>,
}

impl Payload {
    fn into_now_playing(self) -> Option<NowPlaying> {
        let title = self.title.filter(|t| !t.is_empty())?;
        let rate = self.rate.unwrap_or(0.0);
        let now = now_ms();
        let mut elapsed_ms = (self.elapsed.unwrap_or(0.0) * 1000.0) as i64;
        if let Some(ts) = self.timestamp {
            elapsed_ms += ((now as f64 - ts * 1000.0) * rate) as i64;
        }
        Some(NowPlaying {
            title,
            artist: self.artist.unwrap_or_default(),
            album: self.album.filter(|a| !a.is_empty()),
            playing: rate > 0.0,
            duration_ms: self.duration.map(|d| (d * 1000.0) as i64),
            elapsed_ms: elapsed_ms.max(0),
            observed_at_ms: now,
        })
    }
}

/// Writes the embedded helper to the cache directory, once per helper version.
fn install_helper() -> io::Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    HELPER.hash(&mut hasher);
    let dir = env::home_dir()
        .ok_or_else(|| io::Error::other("no home directory"))?
        .join("Library/Caches/deezer-presence");
    let path = dir.join(format!("helper-{:016x}.dylib", hasher.finish()));
    if !path.exists() {
        fs::create_dir_all(&dir)?;
        for old in fs::read_dir(&dir)?.flatten() {
            if old.file_name().to_string_lossy().starts_with("helper-") {
                let _ = fs::remove_file(old.path());
            }
        }
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, HELPER)?;
        fs::rename(&tmp, &path)?;
    }
    Ok(path)
}

pub fn spawn(tx: Sender<Option<NowPlaying>>) {
    thread::spawn(move || {
        loop {
            if let Err(e) = install_helper().and_then(|helper| stream(&helper, &tx)) {
                eprintln!("now playing helper: {e}");
            }
            if tx.send(None).is_err() {
                return;
            }
            thread::sleep(Duration::from_secs(5));
        }
    });
}

fn stream(helper: &Path, tx: &Sender<Option<NowPlaying>>) -> io::Result<()> {
    let mut child = Command::new("/usr/bin/perl")
        .args(["-e", PERL_LOADER])
        .arg(helper)
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("piped stdout");
    *HELPER_PROCESS.lock().unwrap() = Some(child);

    let mut last: Option<NowPlaying> = None;
    for line in BufReader::new(stdout).lines() {
        let Ok(payload) = serde_json::from_str::<Payload>(&line?) else {
            continue;
        };
        let current = payload.into_now_playing();
        let unchanged = match (&current, &last) {
            (Some(a), Some(b)) => a.same_moment(b),
            (None, None) => true,
            _ => false,
        };
        if !unchanged {
            if tx.send(current.clone()).is_err() {
                break;
            }
            last = current;
        }
    }
    shutdown();
    Err(io::Error::other("helper exited"))
}

pub fn shutdown() {
    if let Some(mut child) = HELPER_PROCESS.lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

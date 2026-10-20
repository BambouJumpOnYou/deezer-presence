use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as SessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus,
};

use super::{NowPlaying, now_ms};

const POLL: Duration = Duration::from_secs(1);
/// Milliseconds between 1601-01-01 (Windows epoch) and 1970-01-01.
const WINDOWS_TO_UNIX_MS: i64 = 11_644_473_600_000;

pub fn spawn(tx: Sender<Option<NowPlaying>>) {
    thread::spawn(move || {
        let mut manager = None;
        let mut last = None;
        loop {
            if manager.is_none() {
                manager = SessionManager::RequestAsync()
                    .and_then(|op| op.join())
                    .inspect_err(|e| eprintln!("media sessions unavailable: {e}"))
                    .ok();
            }
            let current = manager.as_ref().and_then(|m| {
                read(m)
                    .inspect_err(|e| eprintln!("media session read failed: {e}"))
                    .ok()
                    .flatten()
            });
            let unchanged = match (&current, &last) {
                (Some(a), Some(b)) => a.same_moment(b),
                (None, None) => true,
                _ => false,
            };
            if !unchanged {
                if tx.send(current.clone()).is_err() {
                    return;
                }
                last = current;
            }
            thread::sleep(POLL);
        }
    });
}

/// Windows lists every player separately, so Deezer is found even when another app has focus.
fn read(manager: &SessionManager) -> windows::core::Result<Option<NowPlaying>> {
    let mut deezer = Vec::new();
    for session in &manager.GetSessions()? {
        if session
            .SourceAppUserModelId()?
            .to_string_lossy()
            .to_lowercase()
            .contains("deezer")
        {
            deezer.push(session);
        }
    }
    let playing = |s: &Session| {
        s.GetPlaybackInfo()
            .and_then(|i| i.PlaybackStatus())
            .is_ok_and(|status| status == PlaybackStatus::Playing)
    };
    let Some(session) = deezer.iter().find(|s| playing(s)).or(deezer.first()) else {
        return Ok(None);
    };
    now_playing(session, playing(session))
}

fn now_playing(session: &Session, playing: bool) -> windows::core::Result<Option<NowPlaying>> {
    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let title = props.Title()?.to_string_lossy();
    if title.is_empty() {
        return Ok(None);
    }
    let album = props.AlbumTitle()?.to_string_lossy();

    let timeline = session.GetTimelineProperties()?;
    let ticks_to_ms = |ticks: i64| ticks / 10_000;
    let duration_ms = ticks_to_ms(timeline.EndTime()?.Duration - timeline.StartTime()?.Duration);
    let position_ms = ticks_to_ms(timeline.Position()?.Duration);
    let updated_ms = ticks_to_ms(timeline.LastUpdatedTime()?.UniversalTime) - WINDOWS_TO_UNIX_MS;

    let now = now_ms();
    let mut elapsed_ms = position_ms;
    if playing && updated_ms > 0 && updated_ms <= now {
        elapsed_ms += now - updated_ms;
    }
    if duration_ms > 0 {
        elapsed_ms = elapsed_ms.min(duration_ms);
    }

    Ok(Some(NowPlaying {
        title,
        artist: props.Artist()?.to_string_lossy(),
        album: (!album.is_empty()).then_some(album),
        playing,
        duration_ms: (duration_ms > 0).then_some(duration_ms),
        elapsed_ms: elapsed_ms.max(0),
        observed_at_ms: now,
    }))
}

use discord_rich_presence::activity::{
    Activity, ActivityType, Assets, Button, StatusDisplayType, Timestamps,
};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};

use crate::deezer::TrackInfo;
use crate::i18n::t;
use crate::source::NowPlaying;

/// Discord IPC client that connects lazily and reconnects when Discord restarts.
pub struct Presence {
    client: DiscordIpcClient,
    connected: bool,
}

impl Presence {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: DiscordIpcClient::new(client_id),
            connected: false,
        }
    }

    fn ensure_connected(&mut self) -> bool {
        if !self.connected {
            self.connected = self.client.connect().is_ok();
        }
        self.connected
    }

    pub fn set(&mut self, track: &NowPlaying, info: &TrackInfo) {
        if !self.ensure_connected() {
            return;
        }
        let start = track.started_at_ms();
        let mut timestamps = Timestamps::new().start(start);
        if let Some(duration) = track.duration_ms {
            timestamps = timestamps.end(start + duration);
        }

        let large_text = track.album.as_deref().unwrap_or(&track.title);
        let mut assets = Assets::new().large_text(large_text);
        if let Some(cover) = &info.cover_url {
            assets = assets.large_image(cover);
        }

        let mut activity = Activity::new()
            .activity_type(ActivityType::Listening)
            .status_display_type(StatusDisplayType::State)
            .details(&track.title)
            .state(&track.artist)
            .assets(assets)
            .timestamps(timestamps);
        if let Some(link) = &info.link {
            activity = activity
                .details_url(link)
                .buttons(vec![Button::new(t().listen_on_deezer, link)]);
        }

        let sent = self.client.set_activity(activity);
        self.check_reply(sent);
    }

    pub fn clear(&mut self) {
        if self.connected {
            let sent = self.client.clear_activity();
            self.check_reply(sent);
        }
    }

    fn check_reply(&mut self, sent: Result<(), discord_rich_presence::error::Error>) {
        let reply = sent.and_then(|()| self.client.recv());
        match reply {
            Ok((_, body)) if body["evt"] == "ERROR" => {
                eprintln!("discord rejected activity: {}", body["data"]["message"])
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("discord ipc error: {e}");
                self.connected = false;
            }
        }
    }
}

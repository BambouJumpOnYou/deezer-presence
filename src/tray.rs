use tray_icon::menu::{CheckMenuItem, Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::autostart;
use crate::i18n::t;
use crate::source::NowPlaying;

const SIZE: u32 = 36;
const PLAYING_BARS: [f32; 5] = [10.0, 20.0, 28.0, 16.0, 8.0];
const IDLE_BARS: [f32; 5] = [4.0; 5];

/// Menu bar icon showing whether a track is being shared on Discord.
pub struct Tray {
    icon: TrayIcon,
    status: MenuItem,
    launch_at_login: CheckMenuItem,
    quit: MenuItem,
    playing: bool,
}

impl Tray {
    pub fn new() -> Self {
        let status = MenuItem::new(t().waiting, false, None);
        let launch_at_login =
            CheckMenuItem::new(t().launch_at_login, true, autostart::is_enabled(), None);
        let quit = MenuItem::new(t().quit, true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &MenuItem::new("Deezer Presence", false, None),
            &status,
            &PredefinedMenuItem::separator(),
            &launch_at_login,
            &quit,
        ])
        .expect("tray menu");

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Deezer Presence")
            .with_icon_templated(bars_icon(&IDLE_BARS))
            .build()
            .expect("tray icon");

        Self {
            icon,
            status,
            launch_at_login,
            quit,
            playing: false,
        }
    }

    /// Handles a menu click; returns true when the app should quit.
    pub fn on_menu(&self, id: &MenuId) -> bool {
        if id == self.launch_at_login.id() {
            let enabled = self.launch_at_login.is_checked();
            if let Err(e) = autostart::set_enabled(enabled) {
                eprintln!("launch at login: {e}");
                self.launch_at_login.set_checked(!enabled);
            }
        }
        id == self.quit.id()
    }

    pub fn update(&mut self, track: Option<&NowPlaying>) {
        let text = match track {
            Some(t) if t.playing => format!("▶ {} — {}", t.title, t.artist),
            Some(t) => format!("⏸ {} — {}", t.title, t.artist),
            None => t().waiting.to_owned(),
        };
        self.status.set_text(&text);
        let _ = self
            .icon
            .set_tooltip(Some(format!("Deezer Presence\n{text}")));

        let playing = track.is_some_and(|t| t.playing);
        if playing != self.playing {
            self.playing = playing;
            let bars = if playing { &PLAYING_BARS } else { &IDLE_BARS };
            let _ = self.icon.set_icon_templated(Some(bars_icon(bars)));
        }
    }
}

/// Rounded, anti-aliased waveform bars drawn as a template image, so macOS tints it for light and dark menu bars.
fn bars_icon(heights: &[f32; 5]) -> Icon {
    const BAR_WIDTH: f32 = 4.0;
    const GAP: f32 = 3.0;
    let radius = BAR_WIDTH / 2.0;
    let total = 5.0 * BAR_WIDTH + 4.0 * GAP;
    let left = (SIZE as f32 - total) / 2.0;
    let center_y = SIZE as f32 / 2.0;

    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    for (i, &height) in heights.iter().enumerate() {
        let center_x = left + i as f32 * (BAR_WIDTH + GAP) + radius;
        let half = (height / 2.0 - radius).max(0.0);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let dx = x as f32 + 0.5 - center_x;
                let dy = ((y as f32 + 0.5 - center_y).abs() - half).max(0.0);
                let coverage = (0.5 - (dx.hypot(dy) - radius)).clamp(0.0, 1.0);
                let alpha = &mut rgba[((y * SIZE + x) * 4 + 3) as usize];
                *alpha = (*alpha).max((coverage * 255.0).round() as u8);
            }
        }
    }
    Icon::from_rgba(rgba, SIZE, SIZE).expect("valid icon buffer")
}

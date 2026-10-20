#![cfg_attr(windows, windows_subsystem = "windows")]

mod autostart;
mod deezer;
mod i18n;
mod presence;
mod source;
mod tray;

use std::env;
use std::fs::{File, TryLockError};
use std::mem;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
#[cfg(target_os = "macos")]
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;

use presence::Presence;
use source::NowPlaying;

/// Public Discord application ID named "Deezer"; override with DISCORD_CLIENT_ID.
const DEFAULT_CLIENT_ID: &str = "1561725299950100590";

/// Re-sent periodically so the presence comes back after a Discord restart.
const REFRESH: Duration = Duration::from_secs(15);

enum UserEvent {
    Track(Option<NowPlaying>),
    Menu(MenuEvent),
}

fn main() {
    if another_instance_running() {
        println!("deezer-presence is already running");
        return;
    }

    let client_id = env::var("DISCORD_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_owned());
    autostart::refresh();

    #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    #[cfg(target_os = "macos")]
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));

    let proxy = event_loop.create_proxy();
    thread::spawn(move || {
        run_presence(&client_id, |track| {
            let _ = proxy.send_event(UserEvent::Track(track));
        })
    });

    let mut tray = None;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => tray = Some(tray::Tray::new()),
            Event::UserEvent(UserEvent::Track(track)) => {
                if let Some(tray) = tray.as_mut() {
                    tray.update(track.as_ref());
                }
            }
            Event::UserEvent(UserEvent::Menu(e))
                if tray.as_ref().is_some_and(|t| t.on_menu(e.id())) =>
            {
                source::shutdown();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

/// Takes an exclusive lock held until the process exits; true if another instance already holds it.
fn another_instance_running() -> bool {
    let file = match File::create(env::temp_dir().join("deezer-presence.lock")) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("single instance lock: {e}");
            return false;
        }
    };
    match file.try_lock() {
        Ok(()) => {
            mem::forget(file);
            false
        }
        Err(TryLockError::WouldBlock) => true,
        Err(TryLockError::Error(e)) => {
            eprintln!("single instance lock: {e}");
            false
        }
    }
}

fn run_presence(client_id: &str, on_change: impl Fn(Option<NowPlaying>)) {
    let (tx, rx) = mpsc::channel();
    source::spawn(tx);

    let mut presence = Presence::new(client_id);
    let mut lookup = deezer::Lookup::new();
    let mut current: Option<NowPlaying> = None;

    loop {
        match rx.recv_timeout(REFRESH) {
            Ok(update) => {
                let unchanged = match (&update, &current) {
                    (Some(a), Some(b)) => a.same_moment(b),
                    (None, None) => true,
                    _ => false,
                };
                if unchanged {
                    continue;
                }
                match &update {
                    Some(t) => println!(
                        "{} {} — {}",
                        if t.playing { "▶" } else { "⏸" },
                        t.title,
                        t.artist
                    ),
                    None => println!("■ Deezer inactif"),
                }
                on_change(update.clone());
                current = update;
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }

        match current.as_ref().filter(|t| t.playing) {
            Some(track) => {
                let info = lookup.get(&track.title, &track.artist, track.album.as_deref());
                presence.set(track, &info);
            }
            None => presence.clear(),
        }
    }
}

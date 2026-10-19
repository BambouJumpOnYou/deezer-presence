<div align="center">

<img src="assets/logo.png" width="112" alt="deezer-presence logo">

# deezer-presence

**Show what you're listening to on Deezer in your Discord status.**

Spotify gets a native Discord integration. Deezer doesn't. This fixes that.

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platform: macOS](https://img.shields.io/badge/platform-macOS-lightgrey?logo=apple)
![Windows: planned](https://img.shields.io/badge/windows-planned-orange?logo=windows)
[![CI](https://github.com/BambouJumpOnYou/deezer-presence/actions/workflows/ci.yml/badge.svg)](https://github.com/BambouJumpOnYou/deezer-presence/actions/workflows/ci.yml)
![Built with Rust](https://img.shields.io/badge/built%20with-Rust-dea584?logo=rust)

</div>

<!-- TODO: add a screenshot of the Discord profile card, e.g. assets/screenshot.png -->

## Features

- 🎧 **"Listening to Deezer"** status with title, artist, album cover and a live progress bar
- 🔗 **"Listen on Deezer"** button so your friends can open the track in one click
- 🌍 **Speaks your language.** English, French, Spanish, German, Italian and Portuguese, picked from your system settings
- 🪶 **Works with the official Deezer desktop app.** No modded client, no login, nothing touches your audio
- 🫥 **Lives in your menu bar.** No Dock icon, just a small waveform that shows what's being shared
- 🚀 **Launch at login** with a single click from the menu
- ⏸️ **Goes quiet on pause**, just like Spotify does
- ⚙️ **Zero setup.** No Discord app to create and no token to paste. Run it and you're done

## Why another Deezer RPC?

Most existing Deezer Discord integrations ship their own wrapper around Deezer's web player. That works until it doesn't:
weird audio glitches, broken logins after a Deezer update, and so on.

deezer-presence takes the opposite approach and **only reads** what your system says is playing.
Your music stays in the official app. If this tool ever crashes, the worst case is that your Discord status disappears.

## Getting started

**Requirements:** macOS 12 or later (Apple Silicon or Intel), plus the Deezer and Discord desktop apps.

1. Download `Deezer-Presence-<version>-macos.zip` from the [latest release](https://github.com/BambouJumpOnYou/deezer-presence/releases/latest).
2. Unzip it and drag **Deezer Presence** into your Applications folder.
3. Open it. The app isn't notarized by Apple yet, so macOS will block it the first time:
   go to **System Settings → Privacy & Security**, scroll down and click **Open Anyway**.
   <details><summary>Prefer the terminal?</summary>

   ```sh
   xattr -dr com.apple.quarantine "/Applications/Deezer Presence.app"
   ```

   </details>

Play something on Deezer and your status should show up within a couple of seconds.
Click the waveform in the menu bar to see what's being shared, turn on **Launch at login**, or quit.

### Building from source

You'll need [Rust](https://rustup.rs) and `cmake` (`brew install cmake`).

```sh
git clone https://github.com/BambouJumpOnYou/deezer-presence.git
cd deezer-presence
./scripts/bundle-macos.sh              # builds dist/Deezer Presence.app
./scripts/bundle-macos.sh --universal  # same, for Apple Silicon and Intel
```

For quick iterations, skip the bundle:

```sh
./scripts/build-adapter.sh   # builds mediaremote-adapter into vendor/
cargo run --release
```

### Configuration

Everything works out of the box, but you can tweak a few things with environment variables:

| Variable | Description |
|---|---|
| `DISCORD_CLIENT_ID` | Use your own Discord application instead of the default one |
| `MEDIAREMOTE_ADAPTER_DIR` | Custom location for the adapter (defaults to the app bundle, then next to the binary, then `vendor/`) |
| `DEEZER_PRESENCE_LANG` | Force a language (`en`, `fr`, `es`, `de`, `it`, `pt`) instead of your system's |

## Translations

The menu bar and the Discord button follow your system language and fall back to English.
Want to see your language? All strings live in [`src/i18n.rs`](src/i18n.rs).
Add a block for your language, map its code in `for_locale`, and open a PR. That's it!

Note that the Discord button is shown in *your* language to everyone who views your profile.

## How it works

1. **Reading what's playing.** macOS keeps track of the current media through its private MediaRemote framework.
   Since macOS 15.4, Apple only lets its own binaries access it.
   [mediaremote-adapter](https://github.com/ungive/mediaremote-adapter) works around this by going through `/usr/bin/perl`.
   deezer-presence listens to that stream and ignores anything that isn't Deezer.
2. **Finding the cover.** The track is looked up through [Deezer's public search API](https://developers.deezer.com/api/search)
   to get the album art and a link to the track. Results are cached.
3. **Updating Discord.** The status is sent to your local Discord client over its IPC socket as a *Listening* activity.

## Known limitations

- **macOS only exposes one player at a time.** If something else takes over (a YouTube tab, another music app…),
  Deezer becomes invisible and your status is cleared until Deezer is the active player again.
- **Not notarized.** macOS Gatekeeper blocks the app the first time. See step 3 of [Getting started](#getting-started).
- **Relies on a workaround.** If Apple closes the gap mediaremote-adapter relies on, the status will stop updating
  until the adapter catches up. Deezer itself will keep working normally.

## Roadmap

- [ ] Windows support through the official media session API (GSMTC)
- [x] Launch at login
- [x] Prebuilt releases

## Acknowledgements

This project stands on the shoulders of some great open-source work:

- **[mediaremote-adapter](https://github.com/ungive/mediaremote-adapter)** by [@ungive](https://github.com/ungive) (BSD-3-Clause).
  It's the piece that makes reading "Now Playing" possible on recent macOS. Go give it a ⭐
- **[discord-rich-presence](https://github.com/vionya/discord-rich-presence)** by [@vionya](https://github.com/vionya) (MIT), for talking to Discord over IPC
- **[tray-icon](https://github.com/tauri-apps/tray-icon)**, **[muda](https://github.com/tauri-apps/muda)** and **[tao](https://github.com/tauri-apps/tao)**
  by the [Tauri](https://tauri.app) team (MIT / Apache-2.0), for the menu bar icon and app event loop
- **[ureq](https://github.com/algesten/ureq)** (MIT / Apache-2.0) and **[serde](https://serde.rs)** (MIT / Apache-2.0), for HTTP and JSON
- **[sys-locale](https://github.com/1Password/sys-locale)** by 1Password (MIT / Apache-2.0), for detecting the system language
- The **[Deezer API](https://developers.deezer.com/api)** for covers and track links, and
  **[Discord Rich Presence](https://discord.com/developers/docs/rich-presence/overview)** for the status itself

## License

[MIT](LICENSE) © 2026 BambouJumpOnYou

Third-party components keep their own licenses. mediaremote-adapter's BSD-3-Clause notice is copied next to it
by `scripts/build-adapter.sh` and must ship with any release.

---

<sub>deezer-presence is a community project and is not affiliated with, endorsed by, or sponsored by Deezer or Discord.</sub>

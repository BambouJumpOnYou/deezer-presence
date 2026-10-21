# Changelog

All notable changes to this project are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project follows [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-10-21

### Added

- **Windows support** (Windows 10 and 11), through the official media session API. Includes a system tray icon,
  launch at login and an app icon.
- Prebuilt Windows zip in releases, next to the macOS one.

### Changed

- **macOS: YouTube, Twitch and other media no longer hide Deezer.** The app now asks macOS for Deezer's player
  specifically instead of whichever player is active.
- macOS: the app is now a single binary with a built-in helper, so there's no extra folder to ship and nothing to install
  with cmake when building from source.

### Fixed

- Wrong album covers: a cover is only shown when the Deezer search finds the exact track.
- Noisy log messages when a browser tab was playing media.

## [0.1.0] - 2026-10-19

### Added

- First release, for macOS: "Listening to Deezer" Discord status with title, artist, cover, progress bar and a
  "Listen on Deezer" button.
- Menu bar icon with the current track, launch at login and quit.
- Translations: English, French, Spanish, German, Italian and Portuguese.

[0.2.0]: https://github.com/BambouJumpOnYou/deezer-presence/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/BambouJumpOnYou/deezer-presence/releases/tag/v0.1.0

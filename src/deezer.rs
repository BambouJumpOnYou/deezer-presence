use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

#[derive(Debug, Clone, Default)]
pub struct TrackInfo {
    pub cover_url: Option<String>,
    pub link: Option<String>,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<Track>,
}

#[derive(Deserialize)]
struct Track {
    title: String,
    #[serde(default)]
    title_short: String,
    link: String,
    artist: Artist,
    album: Album,
}

#[derive(Deserialize)]
struct Artist {
    name: String,
}

#[derive(Deserialize)]
struct Album {
    title: String,
    cover_xl: Option<String>,
}

/// Looks up covers and links via Deezer's public search API, cached per track.
pub struct Lookup {
    agent: Agent,
    cache: HashMap<(String, String), TrackInfo>,
}

impl Lookup {
    pub fn new() -> Self {
        let tls = TlsConfig::builder()
            .provider(TlsProvider::NativeTls)
            .build();
        let agent = Agent::config_builder()
            .tls_config(tls)
            .timeout_global(Some(Duration::from_secs(10)))
            .build()
            .new_agent();
        Self {
            agent,
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, title: &str, artist: &str, album: Option<&str>) -> TrackInfo {
        let key = (title.to_owned(), artist.to_owned());
        if let Some(info) = self.cache.get(&key) {
            return info.clone();
        }
        let info = match search(&self.agent, title, artist, album) {
            Ok(info) => info,
            Err(e) => {
                eprintln!("deezer search failed: {e}");
                return TrackInfo::default();
            }
        };
        self.cache.insert(key, info.clone());
        info
    }
}

fn search(
    agent: &Agent,
    title: &str,
    artist: &str,
    album: Option<&str>,
) -> Result<TrackInfo, ureq::Error> {
    let response: SearchResponse = agent
        .get("https://api.deezer.com/search")
        .query("q", format!("{artist} {title}"))
        .query("limit", "10")
        .call()?
        .body_mut()
        .read_json()?;

    Ok(pick(&response.data, title, artist, album)
        .map(|t| TrackInfo {
            cover_url: t.album.cover_xl.clone(),
            link: Some(t.link.clone()),
        })
        .unwrap_or_default())
}

/// Best exact match, preferring the same album; no match rather than a wrong cover.
fn pick<'a>(
    tracks: &'a [Track],
    title: &str,
    artist: &str,
    album: Option<&str>,
) -> Option<&'a Track> {
    let matches = |t: &&Track| {
        (same(&t.title, title) || same(&t.title_short, title))
            && same_artist(artist, &t.artist.name)
    };
    tracks
        .iter()
        .filter(matches)
        .find(|t| album.is_some_and(|a| same(&t.album.title, a)))
        .or_else(|| tracks.iter().find(matches))
}

fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

/// Players often report every artist ("A, B & C") while the API gives the main one.
fn same_artist(reported: &str, main: &str) -> bool {
    same(reported, main) || reported.split([',', '&']).any(|part| same(part, main))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, artist: &str, album: &str) -> Track {
        Track {
            title: title.to_owned(),
            title_short: String::new(),
            link: format!("https://www.deezer.com/track/{album}"),
            artist: Artist {
                name: artist.to_owned(),
            },
            album: Album {
                title: album.to_owned(),
                cover_xl: None,
            },
        }
    }

    #[test]
    fn prefers_same_album() {
        let tracks = [
            track("Song", "Band", "Single"),
            track("Song", "Band", "Album"),
        ];
        assert_eq!(
            pick(&tracks, "song", "band", Some("Album"))
                .unwrap()
                .album
                .title,
            "Album"
        );
        assert_eq!(
            pick(&tracks, "Song", "Band", None).unwrap().album.title,
            "Single"
        );
    }

    #[test]
    fn matches_featured_artists() {
        let tracks = [track("Song", "Band", "Album")];
        assert!(pick(&tracks, "Song", "Band, Guest & Other", None).is_some());
    }

    #[test]
    #[ignore = "hits the real Deezer API"]
    fn finds_real_track() {
        let info = Lookup::new().get("Never Ending Night", "morgan willis", None);
        assert!(
            info.cover_url
                .is_some_and(|url| url.starts_with("https://"))
        );
    }

    #[test]
    fn rejects_other_tracks() {
        let tracks = [
            track("Other Song", "Band", "Album"),
            track("Song", "Cover Band", "Album"),
        ];
        assert!(pick(&tracks, "Song", "Band", None).is_none());
    }
}

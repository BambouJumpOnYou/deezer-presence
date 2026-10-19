use std::collections::HashMap;

use serde::Deserialize;

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
#[derive(Default)]
pub struct Lookup {
    cache: HashMap<(String, String), TrackInfo>,
}

impl Lookup {
    pub fn get(&mut self, title: &str, artist: &str, album: Option<&str>) -> TrackInfo {
        let key = (title.to_owned(), artist.to_owned());
        if let Some(info) = self.cache.get(&key) {
            return info.clone();
        }
        let info = match search(title, artist, album) {
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

fn search(title: &str, artist: &str, album: Option<&str>) -> Result<TrackInfo, ureq::Error> {
    let response: SearchResponse = ureq::get("https://api.deezer.com/search")
        .query("q", format!("{artist} {title}"))
        .query("limit", "10")
        .call()?
        .body_mut()
        .read_json()?;

    let eq = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
    let best = response
        .data
        .iter()
        .find(|t| {
            eq(&t.title, title)
                && eq(&t.artist.name, artist)
                && album.is_none_or(|a| eq(&t.album.title, a))
        })
        .or_else(|| {
            response
                .data
                .iter()
                .find(|t| eq(&t.title, title) && eq(&t.artist.name, artist))
        })
        .or(response.data.first());

    Ok(best
        .map(|t| TrackInfo {
            cover_url: t.album.cover_xl.clone(),
            link: Some(t.link.clone()),
        })
        .unwrap_or_default())
}

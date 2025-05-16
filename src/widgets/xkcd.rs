use base64::{Engine, engine::general_purpose::STANDARD};
use image::ImageFormat;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

static XKCD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(?:xkcd|comic)(?:\s+#?(?P<number>\d+))?$").unwrap());

#[derive(Debug, Deserialize, Serialize)]
pub struct Xkcd {
    pub num: i32,
    pub title: String,
    pub safe_title: String,
    pub alt: String,
    pub img: String,
    pub year: String,
    pub month: String,
    pub day: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Xkcd {
    pub async fn detect(query: &str, client: &reqwest::Client, db: &sled::Db) -> Option<Self> {
        let caps = XKCD_RE.captures(query.trim())?;
        let comic_number = caps.name("number").map(|m| m.as_str().parse::<i32>().ok());
        Self::fetch_comic(comic_number.flatten(), client, db).await
    }

    async fn fetch_comic(
        number: Option<i32>,
        client: &reqwest::Client,
        db: &sled::Db,
    ) -> Option<Self> {
        let xkcd_cache = db.open_tree("xkcd_comics").ok()?;
        let key = match number {
            Some(n) => n.to_string(),
            None => "latest".to_string(),
        };

        if let Ok(Some(cached)) = xkcd_cache.get(key.as_bytes()) {
            if let Ok(cached_comic) = bincode::deserialize::<Xkcd>(&cached) {
                return Some(cached_comic);
            }
        }

        let url = match number {
            Some(n) => format!("https://xkcd.com/{}/info.0.json", n),
            None => "https://xkcd.com/info.0.json".to_string(),
        };

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(mut comic) = response.json::<Xkcd>().await {
                        comic.error = None;
                        // Process image: fetch, convert to PNG base64
                        if let Ok(img_response) = client.get(&comic.img).send().await {
                            if img_response.status().is_success() {
                                if let Ok(bytes) = img_response.bytes().await {
                                    if let Ok(img) = image::load_from_memory(&bytes) {
                                        let mut png_data = Vec::new();
                                        let _ = img.write_to(
                                            &mut Cursor::new(&mut png_data),
                                            ImageFormat::Png,
                                        );
                                        let base64_img = STANDARD.encode(&png_data);
                                        comic.img = format!("data:image/png;base64,{}", base64_img);
                                    }
                                }
                            }
                        }
                        let _ = xkcd_cache.insert(key.as_bytes(), bincode::serialize(&comic).ok()?);
                        return Some(comic);
                    } else {
                        return Some(Xkcd::error("Failed to parse comic data"));
                    }
                } else {
                    return Some(Xkcd::error("Comic not found"));
                }
            }
            Err(_) => return Some(Xkcd::error("Failed to fetch comic")),
        }
    }

    fn error(message: &str) -> Self {
        Self {
            num: 0,
            title: String::new(),
            safe_title: String::new(),
            alt: String::new(),
            img: String::new(),
            year: String::new(),
            month: String::new(),
            day: String::new(),
            error: Some(message.to_string()),
        }
    }
}

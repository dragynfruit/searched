use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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
    pub async fn detect(query: &str, client: &Client) -> Option<Self> {
        let caps = XKCD_RE.captures(query.trim())?;

        let comic_number = caps.name("number").map(|m| m.as_str().parse::<i32>().ok());

        Self::fetch_comic(comic_number.flatten(), client).await
    }

    async fn fetch_comic(number: Option<i32>, client: &Client) -> Option<Self> {
        let url = match number {
            Some(n) => format!("https://xkcd.com/{}/info.0.json", n),
            None => "https://xkcd.com/info.0.json".to_string(),
        };

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(mut comic) = response.json::<Xkcd>().await {
                        comic.error = None;
                        Some(comic)
                    } else {
                        Some(Xkcd::error("Failed to parse comic data"))
                    }
                } else {
                    Some(Xkcd::error("Comic not found"))
                }
            }
            Err(_) => Some(Xkcd::error("Failed to fetch comic")),
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

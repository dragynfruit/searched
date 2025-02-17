use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

static JOKE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:tell\s+(?:me\s+)?(?:a\s+)?(?:(?P<category>programming|dark|pun|misc|spooky|christmas)\s+)?joke|joke(?:\s+(?P<category2>programming|dark|pun|misc|spooky|christmas))?)$").unwrap()
});

#[derive(Debug, Serialize, Deserialize)]
struct JokeResponse {
    error: bool,
    #[serde(default)]
    category: String,
    #[serde(default)]
    #[serde(rename = "type")]
    joke_type: String,
    #[serde(default)]
    joke: String,
    #[serde(default)]
    setup: String,
    #[serde(default)]
    delivery: String,
}

#[derive(Debug, Serialize)]
pub struct Joke {
    pub category: String,
    pub is_two_part: bool,
    pub first_part: String,
    pub second_part: Option<String>,
    pub error: Option<String>,
}

impl Joke {
    pub async fn detect(query: &str, client: &Client) -> Option<Self> {
        let query = query.trim();
        // Skip if query is shorter than "joke"
        if query.len() < 4 {
            return None;
        }
        let caps = JOKE_RE.captures(query)?;

        let category = caps
            .name("category")
            .or_else(|| caps.name("category2"))
            .map(|m| m.as_str())
            .unwrap_or("Any");

        Self::fetch_joke(category, client).await
    }

    async fn fetch_joke(category: &str, client: &Client) -> Option<Self> {
        let url = format!(
            "https://v2.jokeapi.dev/joke/{}?safe-mode&blacklistFlags=nsfw,religious,political,racist,sexist,explicit",
            category
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if let Ok(joke) = response.json::<JokeResponse>().await {
                    if joke.error {
                        return Some(Joke {
                            category: String::new(),
                            is_two_part: false,
                            first_part: String::new(),
                            second_part: None,
                            error: Some("Could not fetch joke".to_string()),
                        });
                    }

                    let is_two_part = joke.joke_type == "twopart";
                    Some(Joke {
                        category: joke.category,
                        is_two_part,
                        first_part: if is_two_part { joke.setup } else { joke.joke },
                        second_part: if is_two_part {
                            Some(joke.delivery)
                        } else {
                            None
                        },
                        error: None,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

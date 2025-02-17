use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static WIKI_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(?:(?P<topic>.*?)\s+)?wiki(?:pedia)?$").unwrap());

#[derive(Debug, Serialize, Deserialize)]
struct WikiResponse {
    query: WikiQuery,
}

#[derive(Debug, Serialize, Deserialize)]
struct WikiQuery {
    pages: HashMap<String, WikiPage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WikiPage {
    pageid: i64,
    title: String,
    extract: Option<String>,
    thumbnail: Option<WikiThumbnail>,
    pageimage: Option<String>,
    fullurl: Option<String>,
    description: Option<String>, // For disambiguation summaries
    #[serde(default)]
    links: Vec<WikiLink>, // For disambiguation page links
}

#[derive(Debug, Serialize, Deserialize)]
struct WikiThumbnail {
    source: String,
    width: i32,
    height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct WikiLink {
    title: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Wikipedia {
    pub title: String,
    pub extract: String,
    pub image: Option<String>,
    pub url: String,
    pub error: Option<String>,
    pub is_disambiguation: bool,
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug, Serialize)]
pub struct Alternative {
    pub title: String,
    pub description: String,
    pub url: String,
}

impl Wikipedia {
    pub async fn detect(query: &str, client: &Client) -> Option<Self> {
        let query = query.trim();
        let caps = WIKI_RE.captures(query)?;

        let topic = caps
            .name("topic")
            .map(|m| m.as_str().trim())
            .unwrap_or("")
            .to_string();

        if topic.is_empty() {
            return Some(Wikipedia {
                title: String::new(),
                extract: String::new(),
                image: None,
                url: String::new(),
                error: Some("Please specify a topic".to_string()),
                is_disambiguation: false,
                alternatives: vec![],
            });
        }

        Self::fetch_wikipedia(topic, client).await
    }

    async fn fetch_wikipedia(topic: String, client: &Client) -> Option<Self> {
        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&format=json&prop=extracts|pageimages|info|links&inprop=url&pithumbsize=300&exintro=1&explaintext=1&titles={}&plnamespace=0&pllimit=10",
            urlencoding::encode(&topic)
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if let Ok(wiki_data) = response.json::<WikiResponse>().await {
                    let page = wiki_data.query.pages.values().next()?;

                    // Check if this is a disambiguation page
                    let is_disambiguation = page
                        .extract
                        .as_ref()
                        .map(|e| e.contains("may refer to:"))
                        .unwrap_or(false);

                    let alternatives = if is_disambiguation {
                        // Process disambiguation links
                        page.links
                            .iter()
                            .filter_map(|link| {
                                Some(Alternative {
                                    title: link.title.clone(),
                                    description: link.description.clone().unwrap_or_default(),
                                    url: format!(
                                        "https://en.wikipedia.org/wiki/{}",
                                        urlencoding::encode(&link.title)
                                    ),
                                })
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    if page.extract.is_none() && !is_disambiguation {
                        return Some(Wikipedia {
                            title: String::new(),
                            extract: String::new(),
                            image: None,
                            url: String::new(),
                            error: Some("No Wikipedia article found".to_string()),
                            is_disambiguation: false,
                            alternatives: vec![],
                        });
                    }

                    Some(Wikipedia {
                        title: page.title.clone(),
                        extract: page.extract.clone().unwrap_or_default(),
                        image: page.thumbnail.as_ref().map(|t| t.source.clone()),
                        url: page.fullurl.clone().unwrap_or_default(),
                        error: None,
                        is_disambiguation,
                        alternatives,
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

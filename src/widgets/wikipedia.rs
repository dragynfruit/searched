use base64::{Engine, engine::general_purpose::STANDARD};
use image::ImageFormat;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Wikipedia {
    pub title: String,
    pub extract: String,
    pub image: Option<String>,
    pub url: String,
    pub error: Option<String>,
    pub is_disambiguation: bool,
    pub alternatives: Vec<Alternative>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alternative {
    pub title: String,
    pub description: String,
    pub url: String,
}

impl Wikipedia {
    pub async fn detect(query: &str, client: &reqwest::Client, db: &sled::Db) -> Option<Self> {
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

        Self::fetch_wikipedia(topic, client, db).await
    }

    async fn fetch_wikipedia(
        topic: String,
        client: &reqwest::Client,
        db: &sled::Db,
    ) -> Option<Self> {
        // Open cache tree and check cache first
        let wiki_cache = db.open_tree("wikipedia_articles").ok()?;
        let cache_key = format!("wiki_{}", topic.to_lowercase());
        if let Ok(Some(cached)) = wiki_cache.get(cache_key.as_bytes()) {
            if let Ok(article) = bincode::deserialize::<Wikipedia>(&cached) {
                return Some(article);
            }
        }

        let url = format!(
            "https://en.wikipedia.org/w/api.php?action=query&format=json&prop=extracts|pageimages|info|links\
            &inprop=url&pithumbsize=300&exintro=1&explaintext=1&titles={}&plnamespace=0&pllimit=10",
            urlencoding::encode(&topic)
        );

        let article = match client.get(&url).send().await {
            Ok(response) => {
                if let Ok(wiki_data) = response.json::<WikiResponse>().await {
                    let page = wiki_data.query.pages.values().next()?;

                    let is_disambiguation = page
                        .extract
                        .as_ref()
                        .map(|e| e.contains("may refer to:"))
                        .unwrap_or(false);

                    let mut article = if page.extract.is_none() && !is_disambiguation {
                        Wikipedia {
                            title: String::new(),
                            extract: String::new(),
                            image: None,
                            url: String::new(),
                            error: Some("No Wikipedia article found".to_string()),
                            is_disambiguation: false,
                            alternatives: vec![],
                        }
                    } else {
                        Wikipedia {
                            title: page.title.clone(),
                            extract: page.extract.clone().unwrap_or_default(),
                            image: page.thumbnail.as_ref().map(|t| t.source.clone()),
                            url: page.fullurl.clone().unwrap_or_default(),
                            error: None,
                            is_disambiguation,
                            alternatives: if is_disambiguation {
                                page.links
                                    .iter()
                                    .filter_map(|link| {
                                        Some(Alternative {
                                            title: link.title.clone(),
                                            description: link
                                                .description
                                                .clone()
                                                .unwrap_or_default(),
                                            url: format!(
                                                "https://en.wikipedia.org/wiki/{}",
                                                urlencoding::encode(&link.title)
                                            ),
                                        })
                                    })
                                    .collect()
                            } else {
                                vec![]
                            },
                        }
                    };

                    // If not a disambiguation and an image URL exists, fetch and convert it to Base64.
                    if !article.is_disambiguation {
                        if let Some(img_url) = article.image.clone() {
                            if let Ok(response) = client.get(&img_url).send().await {
                                if response.status().is_success() {
                                    if let Ok(bytes) = response.bytes().await {
                                        if let Ok(img) = image::load_from_memory(&bytes) {
                                            let mut png_data = Vec::new();
                                            let _ = img.write_to(
                                                &mut Cursor::new(&mut png_data),
                                                ImageFormat::Png,
                                            );
                                            let base64_img = STANDARD.encode(&png_data);
                                            article.image = Some(format!(
                                                "data:image/png;base64,{}",
                                                base64_img
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    article
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        };

        // Cache the article
        let _ = wiki_cache.insert(cache_key.as_bytes(), bincode::serialize(&article).ok()?);
        Some(article)
    }
}

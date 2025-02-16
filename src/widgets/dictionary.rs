use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phonetic {
    pub text: Option<String>,
    pub audio: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
    pub example: Option<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryEntry {
    pub word: String,
    pub phonetic: Option<String>,
    pub phonetics: Vec<Phonetic>,
    pub origin: Option<String>,
    pub meanings: Vec<Meaning>,
    pub license: Option<License>,
    #[serde(rename = "sourceUrls")]
    pub source_urls: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct Dictionary {
    pub word: String,
    pub entries: Option<Vec<DictionaryEntry>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedDictionary {
    entries: Vec<DictionaryEntry>,
    timestamp: u64,
}

// Update the regex to be more strict
static DICTIONARY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:(?:define|definition(?:\s+of)?)\s+)?(?P<word>\w+)$").unwrap()
});

impl Dictionary {
    pub async fn detect(query: &str, client: &Client, db: &sled::Db) -> Option<Self> {
        let query = query.trim().to_lowercase();
        let caps = DICTIONARY_RE.captures(&query)?;
        let word = caps.name("word")?.as_str();

        // Only trigger for explicit commands or long single words
        let is_explicit = query.starts_with("define") || query.starts_with("definition");
        let is_long_word = !query.contains(' ') && query.len() > 6;

        if !is_explicit && !is_long_word {
            return None;
        }

        let entries = Self::fetch_definition(word, client, db).await;

        // For non-explicit queries, only show if we found a definition
        if !is_explicit && entries.is_none() {
            return None;
        }

        Some(Dictionary {
            word: word.to_string(),
            error: if entries.is_none() {
                Some("Could not find definition".to_string())
            } else {
                None
            },
            entries,
        })
    }

    async fn fetch_definition(
        word: &str,
        client: &Client,
        db: &sled::Db,
    ) -> Option<Vec<DictionaryEntry>> {
        let dictionary_cache = db.open_tree("dictionary").ok()?;
        let cache_key = word.to_lowercase();

        // Check cache first
        if let Ok(Some(cached)) = dictionary_cache.get(cache_key.as_bytes()) {
            if let Ok(cached_dict) = bincode::deserialize::<CachedDictionary>(&cached) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                // Use cache if less than 24 hours old
                if now - cached_dict.timestamp < 24 * 60 * 60 {
                    return Some(cached_dict.entries);
                }
            }
        }

        let encoded_word = urlencoding::encode(&cache_key);
        let url = format!(
            "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
            encoded_word
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(entries) = response.json::<Vec<DictionaryEntry>>().await {
                        // Cache the dictionary response asynchronously
                        let entries_clone = entries.clone();
                        let cache_key = cache_key.clone();
                        let dictionary_cache = dictionary_cache.clone();
                        tokio::spawn(async move {
                            let cached = CachedDictionary {
                                entries: entries_clone,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .ok()?
                                    .as_secs(),
                            };
                            if let Ok(encoded) = bincode::serialize(&cached) {
                                let _ = dictionary_cache.insert(cache_key.as_bytes(), encoded);
                            }
                            Some(())
                        });

                        return Some(entries);
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
}

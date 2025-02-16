use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Phonetic {
    pub text: Option<String>,
    pub audio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Definition {
    pub definition: String,
    pub example: Option<String>,
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meaning {
    #[serde(rename = "partOfSpeech")]
    pub part_of_speech: String,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Dictionary {
    pub async fn detect_with_client(query: &str, client: &Client) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Match different query patterns and extract just the word to look up
        let (word, lookup_type) = if let Some(word) = query.strip_prefix("definition of ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("definition ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("def ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("define ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("synonym of ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("synonym for ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("synonyms of ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("synonyms for ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("antonym of ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("antonym for ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("antonyms of ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if let Some(word) = query.strip_prefix("antonyms for ") {
            (word.trim().split_whitespace().next()?, "explicit")
        } else if query.ends_with(" definition") {
            let word = query.strip_suffix(" definition")?;
            (word, "explicit")
        } else if query.ends_with(" synonym") || query.ends_with(" synonyms") {
            let word = query
                .strip_suffix(" synonym")
                .or_else(|| query.strip_suffix(" synonyms"))?;
            (word, "explicit")
        } else if query.ends_with(" antonym") || query.ends_with(" antonyms") {
            let word = query
                .strip_suffix(" antonym")
                .or_else(|| query.strip_suffix(" antonyms"))?;
            (word, "explicit")
        } else if !query.contains(' ') && query.len() >= 5 {
            // Single word query must be at least 5 characters
            (query.as_str(), "implicit")
        } else {
            return None;
        };

        // Fetch definition from API
        let entries = Self::fetch_definition(word, client).await;

        // For implicit lookups (single word), only return if definition exists
        if lookup_type == "implicit" && entries.is_none() {
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

    async fn fetch_definition(word: &str, client: &Client) -> Option<Vec<DictionaryEntry>> {
        let encoded_word = urlencoding::encode(word);
        let url = format!(
            "https://api.dictionaryapi.dev/api/v2/entries/en/{}",
            encoded_word
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    response.json::<Vec<DictionaryEntry>>().await.ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

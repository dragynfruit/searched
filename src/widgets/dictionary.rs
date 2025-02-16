use reqwest::Client;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use regex::Regex;

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

// New regex to capture a word with optional explicit key phrases.
static DICTIONARY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:(?:definition(?:\s+of)?|def|define|synonym(?:\s+of|s)?|antonym(?:\s+of|s)?)\s+)?(?P<word>\w+)(?:\s+(?:definition|synonym(?:s)?|antonym(?:s)?))?$")
        .unwrap()
});

impl Dictionary {
    pub async fn detect_with_client(query: &str, client: &Client) -> Option<Self> {
        let query = query.trim().to_lowercase();
        
        let caps = DICTIONARY_RE.captures(&query)?;
        let word = caps.name("word")?.as_str();
        let lookup_type = if query.contains("def")
            || query.contains("define")
            || query.contains("synonym")
            || query.contains("antonym")
        {
            "explicit"
        } else if !query.contains(' ') && query.len() >= 5 {
            "implicit"
        } else {
            "explicit"
        };

        let entries = Self::fetch_definition(word, client).await;
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

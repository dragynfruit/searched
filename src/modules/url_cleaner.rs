use log::{info, debug};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use sled::Db;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

static DB: Lazy<Db> = Lazy::new(|| {
    sled::open("data/tracking_rules").expect("Failed to open tracking rules database")
});

#[derive(Debug, Deserialize)]
struct Provider {
    url_pattern: String,
    rules: Vec<String>,
    raw_rules: Vec<String>,
    exceptions: Vec<String>,
    redirections: Vec<String>,
    #[serde(default)]
    force_redirection: bool,
}

#[derive(Debug, Deserialize)]
struct Rules {
    providers: HashMap<String, Provider>,
}

fn is_rules_outdated() -> bool {
    if let Ok(Some(timestamp_bytes)) = DB.get("rules_timestamp") {
        if let Ok(timestamp) = String::from_utf8(timestamp_bytes.to_vec()) {
            if let Ok(stored_time) = timestamp.parse::<u64>() {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return current_time - stored_time > 7 * 24 * 60 * 60; // 1 week
            }
        }
    }
    true
}

pub async fn ensure_rules_exist() {
    if DB.get("rules").unwrap().is_none() || is_rules_outdated() {
        info!("Downloading tracking rules...");
        let rules = reqwest::get("https://gitlab.com/ClearURLs/rules/-/raw/master/data.min.json")
            .await
            .expect("Failed to download rules")
            .text()
            .await
            .expect("Failed to read rules response");

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        DB.insert("rules", rules.as_bytes()).unwrap();
        DB.insert("rules_timestamp", current_time.as_bytes())
            .unwrap();
        info!("Tracking rules downloaded and cached");
    }
}

pub fn clean_url(url: &str) -> String {
    debug!("Cleaning URL: {}", url);
    if let Ok(Some(rules_bytes)) = DB.get("rules") {
        if let Ok(rules_str) = String::from_utf8(rules_bytes.to_vec()) {
            if let Ok(rules) = serde_json::from_str::<Rules>(&rules_str) {
                if let Ok(mut parsed_url) = Url::parse(url) {
                    for provider in rules.providers.values() {
                        if let Ok(pattern) = Regex::new(&provider.url_pattern) {
                            if pattern.is_match(url) {
                                // Skip if URL matches any exception
                                if provider.exceptions.iter().any(|e| {
                                    Regex::new(e).map(|r| r.is_match(url)).unwrap_or(false)
                                }) {
                                    continue;
                                }

                                // Apply tracking parameter removal rules
                                let pairs: Vec<(String, String)> = parsed_url
                                    .query_pairs()
                                    .map(|(k, v)| (k.to_string(), v.to_string()))
                                    .filter(|(k, _)| {
                                        !provider.rules.iter().any(|rule| {
                                            Regex::new(rule).map(|r| r.is_match(k)).unwrap_or(false)
                                        })
                                    })
                                    .collect();

                                if !pairs.is_empty() {
                                    parsed_url.query_pairs_mut().clear();
                                    for (k, v) in pairs {
                                        parsed_url.query_pairs_mut().append_pair(&k, &v);
                                    }
                                } else {
                                    parsed_url.set_query(None);
                                }

                                // Apply raw rules (like path cleanup)
                                for raw_rule in &provider.raw_rules {
                                    if let Ok(regex) = Regex::new(raw_rule) {
                                        let path = parsed_url.path().to_string();
                                        let cleaned = regex.replace_all(&path, "").to_string();
                                        parsed_url.set_path(&cleaned);
                                    }
                                }

                                // Handle redirections if force_redirection is true
                                if provider.force_redirection {
                                    for redirection in &provider.redirections {
                                        if let Ok(regex) = Regex::new(redirection) {
                                            if let Some(caps) = regex.captures(url) {
                                                if let Some(real_url) = caps.get(1) {
                                                    if let Ok(decoded) =
                                                        urlencoding::decode(real_url.as_str())
                                                    {
                                                        return decoded.to_string();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    return parsed_url.to_string();
                }
            }
        }
    }
    url.to_string()
}

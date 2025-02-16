use chrono::{DateTime, Local};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static TIME_QUERY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:time|current\s+time|what(?:\s+is)?\s+(?:time\s+is\s+it|is\s+the\s+current\s+time|the\s+time)|whats\s+the\s+time)$")
        .unwrap()
});

#[derive(Debug, Serialize)]
pub struct Time {
    pub current_time: String,
    pub current_date: String,
    pub timezone: String,
}

impl Time {
    pub fn detect(query: &str) -> Option<Self> {
        if !TIME_QUERY_RE.is_match(query.trim()) {
            return None;
        }
        let now: DateTime<Local> = Local::now();
        Some(Time {
            current_time: now.format("%H:%M:%S").to_string(),
            current_date: now.format("%A, %B %d, %Y").to_string(),
            timezone: now.format("%Z").to_string(),
        })
    }
}

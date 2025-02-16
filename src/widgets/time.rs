use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Time {
    pub current_time: String,
    pub current_date: String,
    pub timezone: String,
}

impl Time {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim().to_lowercase();

        if !["time", "current time", "what time is it", "whats the time"]
            .iter()
            .any(|q| query == *q)
        {
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

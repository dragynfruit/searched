use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Timer {
    pub mode: &'static str,
    pub initial_time: Option<u64>, // milliseconds
}

impl Timer {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Check for {time} timer format
        if query.ends_with("timer") {
            let time_part = query.trim_end_matches("timer").trim();
            if let Some(ms) = parse_combined_time(time_part) {
                return Some(Timer {
                    mode: "timer",
                    initial_time: Some(ms),
                });
            }
        }

        // Only detect if query starts with specific words
        let (mode, remaining) = if let Some(rest) = query.strip_prefix("timer") {
            ("timer", rest)
        } else if let Some(rest) = query.strip_prefix("stopwatch") {
            ("stopwatch", rest)
        } else if let Some(rest) = query.strip_prefix("countdown") {
            ("timer", rest)
        } else if query == "time" {
            return Some(Timer {
                mode: "stopwatch",
                initial_time: None,
            });
        } else {
            return None;
        };

        // If it's a timer, try to parse duration
        if mode == "timer" {
            // First try combined format (e.g., "timer5m")
            if let Some(ms) = parse_combined_time(remaining) {
                return Some(Timer {
                    mode,
                    initial_time: Some(ms),
                });
            }

            // Then try space-separated format
            if let Some(ms) = parse_time_string(&query) {
                return Some(Timer {
                    mode,
                    initial_time: Some(ms),
                });
            }
        }

        // Return with no initial time if no duration specified
        Some(Timer {
            mode,
            initial_time: None,
        })
    }
}

// New regex for combined time format.
static COMBINED_TIME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<number>\d+)(?P<unit>h(?:r(?:s)?)?|hour(?:s)?|m(?:in(?:s)?)?|minute(?:s)?|s(?:ec(?:s)?)?|second(?:s)?|ms|millisecond(?:s)?)$")
        .unwrap()
});

// New regex for tokenized time parts.
static TIME_TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?P<number>\d+)(?P<unit>h(?:r(?:s)?)?|hour(?:s)?|m(?:in(?:s)?)?|minute(?:s)?|s(?:ec(?:s)?)?|second(?:s)?|ms|millisecond(?:s)?)")
        .unwrap()
});

/// Parse combined format like "5m", "10s", etc.
fn parse_combined_time(input: &str) -> Option<u64> {
    let input = input.trim();
    if let Some(caps) = COMBINED_TIME_RE.captures(input) {
        let amount: u64 = caps.name("number")?.as_str().parse().ok()?;
        let unit = caps.name("unit")?.as_str().to_lowercase();
        let ms = match unit.as_str() {
            "h" | "hr" | "hrs" | "hour" | "hours" => amount * 3600000,
            "m" | "min" | "mins" | "minute" | "minutes" => amount * 60000,
            "s" | "sec" | "secs" | "second" | "seconds" => amount * 1000,
            "ms" | "millisecond" | "milliseconds" => amount,
            _ => amount * 1000,
        };
        Some(ms)
    } else {
        None
    }
}

fn parse_time_string(query: &str) -> Option<u64> {
    let mut total_ms = 0u64;
    for cap in TIME_TOKEN_RE.captures_iter(query) {
        let amount: u64 = cap.name("number")?.as_str().parse().ok()?;
        let unit = cap.name("unit")?.as_str().to_lowercase();
        let ms = match unit.as_str() {
            "h" | "hr" | "hrs" | "hour" | "hours" => amount * 3600000,
            "m" | "min" | "mins" | "minute" | "minutes" => amount * 60000,
            "s" | "sec" | "secs" | "second" | "seconds" => amount * 1000,
            "ms" | "millisecond" | "milliseconds" => amount,
            _ => amount * 1000,
        };
        total_ms += ms;
    }
    if total_ms > 0 {
        Some(total_ms)
    } else {
        None
    }
}

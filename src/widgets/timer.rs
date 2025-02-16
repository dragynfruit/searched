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

/// Parse combined format like "5m", "10s", etc.
fn parse_combined_time(input: &str) -> Option<u64> {
    let mut number_end = 0;
    for (i, c) in input.chars().enumerate() {
        if !c.is_ascii_digit() {
            number_end = i;
            break;
        }
    }

    if number_end == 0 {
        return None;
    }

    let (num_str, unit_str) = input.split_at(number_end);
    let amount: u64 = num_str.parse().ok()?;

    // Even with invalid/missing unit, return the value in milliseconds
    let ms = match unit_str {
        "h" | "hr" | "hrs" | "hour" | "hours" => amount * 3600000,
        "m" | "min" | "mins" | "minute" | "minutes" => amount * 60000,
        "s" | "sec" | "secs" | "second" | "seconds" => amount * 1000,
        "ms" | "milliseconds" | "millisecond" => amount,
        _ => amount * 1000, // Default to seconds for invalid/missing units
    };

    Some(ms)
}

fn parse_time_string(query: &str) -> Option<u64> {
    let parts: Vec<&str> = query.split_whitespace().collect();
    let mut total_ms = 0;
    let mut i = 1; // Skip "timer" word

    while i < parts.len() {
        if parts[i] == "for" {
            i += 1;
            continue;
        }

        let part = parts[i];

        // Try to find where the number ends and unit begins
        let mut split_idx = part.len();
        for (idx, ch) in part.char_indices() {
            if !ch.is_ascii_digit() {
                split_idx = idx;
                break;
            }
        }

        // Split the part into number and unit
        let (num_str, unit_str) = part.split_at(split_idx);

        // Parse the number
        let amount: u64 = match num_str.parse() {
            Ok(num) => num,
            Err(_) => break,
        };

        // If there's no unit in this part, look at next part
        let unit = if unit_str.is_empty() && i + 1 < parts.len() {
            i += 1;
            parts[i]
        } else {
            unit_str
        };

        let ms = match unit {
            "h" | "hr" | "hrs" | "hour" | "hours" => amount * 3600000,
            "m" | "min" | "mins" | "minute" | "minutes" => amount * 60000,
            "s" | "sec" | "secs" | "second" | "seconds" => amount * 1000,
            "ms" | "milliseconds" | "millisecond" => amount,
            _ => amount * 1000, // Default to seconds for invalid/missing units
        };

        total_ms += ms;
        i += 1;
    }

    if total_ms > 0 {
        Some(total_ms)
    } else {
        None
    }
}

use log::debug;
use nucleo_matcher::{
    Config, Matcher,
    pattern::{CaseMatching, Normalization, Pattern},
};

pub fn highlight_text(text: &str, query: &str) -> String {
    debug!("Highlighting text for query: {}", query);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Smart, Normalization::Smart);

    // Use a buffer for UTF-32 conversion
    let mut buf = Vec::new();
    let haystack = nucleo_matcher::Utf32Str::new(text, &mut buf);

    let mut indices = Vec::new();
    if let Some(_) = pattern.indices(haystack, &mut matcher, &mut indices) {
        indices.sort_unstable();
        indices.dedup();

        let mut result = String::with_capacity(text.len() * 2);
        let mut last_end = 0;
        let char_indices: Vec<(usize, char)> = text.char_indices().collect();

        for &pos in indices.iter() {
            let pos = pos as usize;
            if pos >= text.len() {
                continue;
            }

            // Find the next valid character boundary
            let start = match char_indices.iter().find(|(i, _)| *i >= pos) {
                Some(&(i, _)) => i,
                None => continue,
            };

            // Only proceed if we have a valid slice
            if start >= last_end && start < text.len() {
                result.push_str(&text[last_end..start]);

                // Find the end of the current character
                let end =
                    if let Some(&(next_start, _)) = char_indices.iter().find(|(i, _)| *i > start) {
                        next_start
                    } else {
                        text.len()
                    };

                if end > start {
                    result.push_str("<b>");
                    result.push_str(&text[start..end]);
                    result.push_str("</b>");
                    last_end = end;
                }
            }
        }

        // Add remaining text
        if last_end < text.len() {
            result.push_str(&text[last_end..]);
        }

        result
    } else {
        text.to_string()
    }
}

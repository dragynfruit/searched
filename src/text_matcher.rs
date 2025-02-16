use nucleo_matcher::{pattern::{CaseMatching, Normalization, Pattern}, Config, Matcher};

pub fn highlight_text(text: &str, query: &str) -> String {
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
        
        for &pos in indices.iter() {
            let pos = pos as usize;
            if pos >= text.len() {
                continue;
            }
            let start = text.char_indices()
                .find(|(i, _)| *i >= pos)
                .map(|(i, _)| i)
                .unwrap_or(text.len());

            result.push_str(&text[last_end..start]);
            
            let end = text[start..].chars().next()
                .map(|c| start + c.len_utf8())
                .unwrap_or(text.len());
            
            result.push_str("<b>");
            result.push_str(&text[start..end]);
            result.push_str("</b>");
            
            last_end = end;
        }
        result.push_str(&text[last_end..]);
        
        result
    } else {
        text.to_string()
    }
}

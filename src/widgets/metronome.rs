use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static METRONOME_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:metronome(?:\s+(?:for|at|@))?\s*(?:(?P<bpm>\d+)\s*(?:bpm)?)?|(?P<bpm2>\d+)\s*bpm\s*metronome)$").unwrap()
});

#[derive(Debug, Serialize)]
pub struct Metronome {
    pub initial_bpm: Option<u32>,
}

impl Metronome {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim();

        if let Some(caps) = METRONOME_RE.captures(query) {
            let bpm = caps
                .name("bpm")
                .or_else(|| caps.name("bpm2"))
                .and_then(|m| m.as_str().parse::<u32>().ok())
                .filter(|&bpm| bpm >= 30 && bpm <= 250);

            return Some(Metronome { initial_bpm: bpm });
        }
        None
    }
}

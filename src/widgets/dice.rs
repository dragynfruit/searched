use serde::Serialize;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Serialize)]
pub struct DiceRoll {
    pub values: Vec<u32>,
    pub sum: u32,
    pub count: u32,
    pub sides: u32,
    pub show_sum: bool,
    pub is_coin: bool,
}

static DICE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:roll\s+)?(?P<count>\d*)d(?P<sides>\d+)$").unwrap()
});

impl DiceRoll {
    fn extract_dice_notation(query: &str) -> Option<(u32, u32)> {
        let trimmed = query.trim();
        if let Some(caps) = DICE_RE.captures(trimmed) {
            let count_str = caps.name("count").map_or("", |m| m.as_str());
            let count = if count_str.is_empty() { 1 } else { count_str.parse().unwrap_or(1) };
            let sides: u32 = caps.name("sides")?.as_str().parse().ok()?;
            Some((count, sides))
        } else {
            None
        }
    }

    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Check for coin flip variations first
        if ["flip coin", "coin flip", "coin", "toss coin", "flip a coin"]
            .iter()
            .any(|&pattern| query == pattern)
        {
            return Some(Self::coin_flip());
        }

        // Specific dice rolling commands
        if ["roll dice", "roll a dice", "throw dice", "dice roll", "roll the dice"]
            .iter()
            .any(|&cmd| query == cmd)
        {
            return Some(Self::default_roll());
        }

        // Allow "roll a {dice notation}" command
        if let Some(stripped) = query.strip_prefix("roll a ") {
            if let Some((count, sides)) = Self::extract_dice_notation(stripped) {
                if count > 0 && count <= 100 && sides > 0 && sides <= 1000 {
                    let values: Vec<u32> = (0..count).map(|_| fastrand::u32(1..=sides)).collect();
                    let sum = values.iter().sum();
                    return Some(DiceRoll {
                        values,
                        sum,
                        count,
                        sides,
                        show_sum: count > 1,
                        is_coin: false,
                    });
                }
            }
        }

        // Use regex to extract dice notation.
        if let Some((count, sides)) = Self::extract_dice_notation(&query) {
            if count > 0 && count <= 100 && sides > 0 && sides <= 1000 {
                let values: Vec<u32> = (0..count).map(|_| fastrand::u32(1..=sides)).collect();
                let sum = values.iter().sum();
                return Some(DiceRoll {
                    values,
                    sum,
                    count,
                    sides,
                    show_sum: count > 1,
                    is_coin: false,
                });
            }
        }
        None
    }

    fn coin_flip() -> Self {
        let value = if fastrand::bool() { 1 } else { 2 };
        DiceRoll {
            values: vec![value],
            sum: value,
            count: 1,
            sides: 2,
            show_sum: false,
            is_coin: true,
        }
    }

    fn default_roll() -> Self {
        let value = fastrand::u32(1..=6);
        DiceRoll {
            values: vec![value],
            sum: value,
            count: 1,
            sides: 6,
            show_sum: false,
            is_coin: false,
        }
    }
}

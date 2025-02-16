use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DiceRoll {
    pub values: Vec<u32>,
    pub sum: u32,
    pub count: u32,
    pub sides: u32,
    pub show_sum: bool,
    pub is_coin: bool, // New field
}

impl DiceRoll {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Check for coin flip variations
        if query.contains("flip") && query.contains("coin")
            || query == "coin"
            || query == "toss coin"
        {
            return Some(Self::coin_flip());
        }

        // Match patterns like "roll dice", "roll 2d6", "dice", "2d20"
        if !query.contains("dice") && !query.contains("roll") && !query.contains('d') {
            return None;
        }

        // Extract dice notation (e.g., 2d6)
        let dice_part = query.split_whitespace().find(|part| part.contains('d'))?;

        // Parse dice notation
        let parts: Vec<&str> = dice_part.split('d').collect();
        if parts.len() != 2 {
            return Some(Self::default_roll()); // Default to 1d6
        }

        let count = if parts[0].is_empty() {
            1 // Handle "d20" as "1d20"
        } else {
            parts[0].parse().unwrap_or(1)
        };

        let sides = parts[1].parse().unwrap_or(6);

        // Validate input
        if count == 0 || count > 100 || sides == 0 || sides > 1000 {
            return Some(Self::default_roll());
        }

        // Generate rolls
        let values: Vec<u32> = (0..count).map(|_| fastrand::u32(1..=sides)).collect();

        let sum = values.iter().sum();
        let show_sum = count > 1;

        Some(DiceRoll {
            values,
            sum,
            count,
            sides,
            show_sum,
            is_coin: false,
        })
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

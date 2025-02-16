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

        // Check for coin flip variations first
        if ["flip coin", "coin flip", "coin", "toss coin", "flip a coin"]
            .iter()
            .any(|&pattern| query == pattern)
        {
            return Some(Self::coin_flip());
        }

        // Specific dice rolling commands
        let is_dice_command = [
            "roll dice",
            "roll a dice",
            "throw dice",
            "dice roll",
            "roll the dice",
        ].iter().any(|&cmd| query == cmd);

        if is_dice_command {
            return Some(Self::default_roll());
        }

        // Match explicit dice notation patterns (e.g., "roll 2d6", "3d20")
        if let Some(notation) = Self::extract_dice_notation(&query) {
            let parts: Vec<&str> = notation.split('d').collect();
            if parts.len() == 2 {
                let count = if parts[0].is_empty() {
                    1 // Handle "d20" as "1d20"
                } else {
                    parts[0].parse().unwrap_or(1)
                };

                let sides = parts[1].parse().unwrap_or(6);

                // Validate input
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

        None
    }

    fn extract_dice_notation(query: &str) -> Option<&str> {
        // Look for patterns like "roll 2d6" or just "2d6"
        let parts: Vec<&str> = query.split_whitespace().collect();
        
        match parts.len() {
            1 => {
                // Single word must be dice notation
                if parts[0].contains('d') {
                    Some(parts[0])
                } else {
                    None
                }
            },
            2 => {
                // Two words: "roll 2d6"
                if parts[0] == "roll" && parts[1].contains('d') {
                    Some(parts[1])
                } else {
                    None
                }
            },
            _ => None,
        }
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

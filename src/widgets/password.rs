use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static PASSWORD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(generate\s+)?pass(?:word)?(?:\s+(?P<length>\d+))?(?:\s+(?P<options>[\w\s]+))?$",
    )
    .unwrap()
});

#[derive(Debug, Serialize)]
pub struct Password {
    pub password: String,
    pub length: usize,
    pub has_uppercase: bool,
    pub has_lowercase: bool,
    pub has_numbers: bool,
    pub has_symbols: bool,
    pub strength: PasswordStrength,
}

#[derive(Debug, Serialize)]
pub enum PasswordStrength {
    Weak,
    Medium,
    Strong,
    VeryStrong,
}

impl Password {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim();
        // Skip if query is shorter than "pass"
        if query.len() < 4 {
            return None;
        }
        let query = query.to_lowercase();
        let caps = PASSWORD_RE.captures(&query)?;

        // Parse length, default to 16
        let length = caps
            .name("length")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(16)
            .clamp(8, 64); // Enforce reasonable limits

        // Parse options
        let options = caps
            .name("options")
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_default();

        // Default to all character types unless specifically configured
        let has_uppercase = !options.contains("no upper") && !options.contains("no uppercase");
        let has_lowercase = !options.contains("no lower") && !options.contains("no lowercase");
        let has_numbers = !options.contains("no num") && !options.contains("no numbers");
        let has_symbols = !options.contains("no sym") && !options.contains("no symbols");

        // Generate password
        let password = Self::generate_password(
            length,
            has_uppercase,
            has_lowercase,
            has_numbers,
            has_symbols,
        );

        // Calculate strength
        let strength = Self::calculate_strength(&password);

        Some(Password {
            password,
            length,
            has_uppercase,
            has_lowercase,
            has_numbers,
            has_symbols,
            strength,
        })
    }

    fn generate_password(
        length: usize,
        upper: bool,
        lower: bool,
        numbers: bool,
        symbols: bool,
    ) -> String {
        let mut chars = String::new();

        if upper {
            chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }
        if lower {
            chars.push_str("abcdefghijklmnopqrstuvwxyz");
        }
        if numbers {
            chars.push_str("0123456789");
        }
        if symbols {
            chars.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        // Fallback to lowercase if no options selected
        if chars.is_empty() {
            chars.push_str("abcdefghijklmnopqrstuvwxyz");
        }

        let chars: Vec<char> = chars.chars().collect();
        let mut rng = fastrand::Rng::new();

        // Generate password
        let mut password: String = (0..length)
            .map(|_| chars[rng.usize(..chars.len())])
            .collect();

        // Ensure at least one character of each required type
        if upper && !password.chars().any(|c| c.is_ascii_uppercase()) {
            let pos = rng.usize(..length);
            password.replace_range(pos..pos + 1, "A");
        }
        if lower && !password.chars().any(|c| c.is_ascii_lowercase()) {
            let pos = rng.usize(..length);
            password.replace_range(pos..pos + 1, "a");
        }
        if numbers && !password.chars().any(|c| c.is_ascii_digit()) {
            let pos = rng.usize(..length);
            password.replace_range(pos..pos + 1, "1");
        }
        if symbols && !password.chars().any(|c| c.is_ascii_punctuation()) {
            let pos = rng.usize(..length);
            password.replace_range(pos..pos + 1, "@");
        }

        password
    }

    fn calculate_strength(password: &str) -> PasswordStrength {
        let length = password.len();
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_number = password.chars().any(|c| c.is_ascii_digit());
        let has_symbol = password.chars().any(|c| c.is_ascii_punctuation());

        let complexity = [has_upper, has_lower, has_number, has_symbol]
            .iter()
            .filter(|&&x| x)
            .count();

        match (length, complexity) {
            (_, 4) if length >= 12 => PasswordStrength::VeryStrong,
            (_, 3) if length >= 12 => PasswordStrength::Strong,
            (_, _) if length >= 12 => PasswordStrength::Medium,
            (_, _) => PasswordStrength::Weak,
        }
    }
}

use csscolorparser::Color as CssColor;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static COLOR_PICKER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:(?:color|hex|rgb|hsl)\s*picker|pick\s*(?:a\s*)?(?:color|hex|rgb|hsl))$")
        .unwrap()
});

#[derive(Debug, Serialize)]
pub struct Color {
    pub original_input: String,
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub rgba: (u8, u8, u8, f32),
    pub hsl: (f64, f64, f64),
    pub hsla: (f64, f64, f64, f32),
    pub is_dark: bool,
}

impl Color {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim();

        // Match any color picker variation using regex
        if COLOR_PICKER_RE.is_match(query) {
            let r = fastrand::u8(..);
            let g = fastrand::u8(..);
            let b = fastrand::u8(..);
            let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
            return Self::from_hex(&hex);
        }

        // Try parsing as hex
        if query.starts_with('#') || query.to_lowercase().starts_with("hex ") {
            let hex = query.trim_start_matches("hex ").trim_start_matches('#');
            return Self::from_hex(&format!("#{}", hex));
        }

        // Try parsing as rgb/rgba
        if query.to_lowercase().starts_with("rgb") {
            return Self::from_css_color(query);
        }

        // Try parsing as hsl/hsla
        if query.to_lowercase().starts_with("hsl") {
            return Self::from_css_color(query);
        }

        // Try parsing as named color
        if query
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c.is_whitespace())
        {
            return Self::from_css_color(query);
        }

        None
    }

    fn from_hex(hex: &str) -> Option<Self> {
        Self::from_css_color(hex)
    }

    fn from_css_color(color_str: &str) -> Option<Self> {
        let color = color_str.parse::<CssColor>().ok()?;

        // Use to_rgba8 for RGB values
        let rgba = color.to_rgba8();
        let r = rgba[0];
        let g = rgba[1];
        let b = rgba[2];
        let a = rgba[3] as f32 / 255.0;

        // Get HSL values (returns array of 4 values: h, s, l, a)
        let hsla = color.to_hsla();
        let h = hsla[0] as f64;
        let s = hsla[1] as f64;
        let l = hsla[2] as f64;
        let a2 = hsla[3] as f32;

        Some(Color {
            original_input: color_str.to_string(),
            hex: color.to_hex_string(),
            rgb: (r, g, b),
            rgba: (r, g, b, a),
            hsl: (h, s * 100.0, l * 100.0),
            hsla: (h, s * 100.0, l * 100.0, a2),
            is_dark: (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) < 128.0,
        })
    }
}

use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct UnitConverter {
    pub value: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub result: Option<f64>,
    pub from_unit_name: String,
    pub to_unit_name: String,
    pub formula: Option<String>,
}

impl UnitConverter {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim().to_lowercase();
        let mut parts: Vec<&str> = query.split_whitespace().collect();

        // If the first token has number and unit combined (e.g. "5km")
        if parts.get(0)?.parse::<f64>().is_err() {
            if let Some((value, extracted_unit)) = split_value_unit(parts[0]) {
                // Replace the first token with the extracted unit and inject the parsed value
                parts.remove(0);
                parts.insert(0, extracted_unit.as_str());
                // value is captured below instead of parsing parts[0]
                return Self::detect_with_value(value, &parts);
            }
        }

        // When the first token is a valid number, parse it and continue
        let value = parts[0].parse::<f64>().ok()?;
        // Remove the numeric token before processing units
        parts.remove(0);
        Self::detect_with_value(value, &parts)
    }

    // New helper to use a provided value and the rest of tokens.
    fn detect_with_value(value: f64, parts: &[&str]) -> Option<Self> {
        // Find separator index (to, in, ->, =)
        let sep_idx = parts
            .iter()
            .position(|&x| x == "to" || x == "in" || x == "->" || x == "=")?;

        if sep_idx < 1 || sep_idx >= parts.len() - 1 {
            return None;
        }

        // Combine unit parts before and after separator
        let from_unit = parts[0..sep_idx].join("");
        let to_unit = parts[sep_idx + 1..].join("");

        let result = convert_unit(value, &from_unit, &to_unit);
        let from_unit_name = get_unit_name(&from_unit).to_string();
        let to_unit_name = get_unit_name(&to_unit).to_string();
        let formula = get_conversion_formula(&from_unit, &to_unit);

        Some(Self {
            value,
            from_unit,
            to_unit,
            result,
            from_unit_name,
            to_unit_name,
            formula,
        })
    }
}

// New helper function to split a token into a numeric value and its appended unit.
// It supports integers and decimals (optionally with a '-' sign).
fn split_value_unit(token: &str) -> Option<(f64, String)> {
    let mut idx = 0;
    for (i, ch) in token.char_indices() {
        if !(ch.is_digit(10) || ch == '.' || (i == 0 && ch == '-')) {
            idx = i;
            break;
        }
    }
    // Fail if no unit part is found.
    if idx == 0 || idx >= token.len() {
        return None;
    }
    let num_part = &token[..idx];
    let unit_part = &token[idx..];
    let value = num_part.parse::<f64>().ok()?;
    Some((value, unit_part.to_string()))
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum UnitType {
    Length,         // base: meters
    Weight,         // base: grams
    Volume,         // base: milliliters
    Temperature,    // special case
    Time,           // base: seconds
    Frequency,      // New
    Electrical,     // New
    Area,           // New
    DataTransfer,   // base: bytes per second (B/s)
    DigitalStorage, // base: bytes (B)
    FuelEconomy,    // base: kilometers per liter (km/L)
    Angle,          // base: radians (rad)
    Pressure,       // base: pascals (Pa)
    Unknown,
}

// Add new helper sets and function for disambiguating "w"
static ELECTRICAL_SET: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "mv", "kv", "µv", "a", "ma", "µa", "ohm", "kohm", "mohm", "kw", "mw", "gw",
    ]
});
static TIME_SET: Lazy<Vec<&'static str>> =
    Lazy::new(|| vec!["y", "mo", "d", "h", "min", "sec", "ms", "µs", "ns"]);

fn disambiguate_w(_unit: &str, other: &str) -> &'static str {
    if ELECTRICAL_SET.contains(&other) {
        "w_e" // electrical: watts
    } else if TIME_SET.contains(&other) {
        "w_t" // time: weeks
    } else {
        // Fallback: if unsure, use the other unit's category to decide
        if get_unit_type(other) == UnitType::Electrical {
            "w_e"
        } else {
            "w_t"
        }
    }
}

pub fn convert_unit(value: f64, from_unit: &str, to_unit: &str) -> Option<f64> {
    // disambiguate "w" if present
    let mut from = normalize_unit(from_unit);
    let mut to = normalize_unit(to_unit);
    if from == "w" {
        from = disambiguate_w("w", to);
    }
    if to == "w" {
        to = disambiguate_w("w", from);
    }

    let unit_type = get_unit_type(from);
    if unit_type != get_unit_type(to) {
        return None;
    }

    match unit_type {
        UnitType::Temperature => convert_temperature(value, from, to),
        UnitType::Unknown => None,
        _ => {
            let base_value = to_base_unit(value, from)?;
            from_base_unit(base_value, to)
        }
    }
}

fn get_unit_type(unit: &str) -> UnitType {
    match unit {
        // Length, Area, Weight, Volume, Temperature remain unchanged
        "km" | "m" | "cm" | "mm" | "µm" | "nm" | "mi" | "nmi" | "in" | "ft" | "yd" => {
            UnitType::Length
        }
        "km²" | "m²" | "ha" | "ac" => UnitType::Area,
        "kg" | "g" | "mg" | "µg" | "lb" | "oz" | "st" | "t" => UnitType::Weight,
        "l" | "ml" | "cl" | "gal" | "qt" | "pt" | "cup" | "floz" | "tbsp" | "tsp" => {
            UnitType::Volume
        }
        "c" | "f" | "k" => UnitType::Temperature,
        // For "w", use markers: "w_t" for weeks (time) and for electrical we use "w_e"
        "w_t" | "y" | "mo" | "d" | "h" | "min" | "sec" | "ms" | "µs" | "ns" => UnitType::Time,
        "w_e" | "hz" | "khz" | "mhz" | "ghz" | "thz" => UnitType::Frequency, // Note: Electrical "w" now provided via "w_e" is not here.
        "v" | "mv" | "kv" | "µv" | "a" | "ma" | "µa" | "ohm" | "kohm" | "mohm" | "kw" | "mw"
        | "gw" => UnitType::Electrical,
        // Data Transfer
        "bps" | "kbps" | "mbps" | "gbps" | "tbps" => UnitType::DataTransfer,
        // Digital Storage
        "b" | "kb" | "mb" | "gb" | "tb" | "pb" => UnitType::DigitalStorage,
        // Fuel Economy
        "mpg" | "kml" | "lp100km" => UnitType::FuelEconomy,
        // Angle
        "rad" | "deg" | "grad" | "arcmin" | "arcsec" => UnitType::Angle,
        // Pressure
        "pa" | "hpa" | "kpa" | "mpa" | "bar" | "mbar" | "psi" | "mmhg" | "atm" => {
            UnitType::Pressure
        }
        _ => UnitType::Unknown,
    }
}

fn to_base_unit(value: f64, unit: &str) -> Option<f64> {
    Some(match unit {
        // Length (to meters)
        "km" => value * 1000.0,
        "m" => value,
        "cm" => value / 100.0,
        "mm" => value / 1000.0,
        "µm" => value / 1_000_000.0,
        "nm" => value / 1_000_000_000.0,
        "mi" => value * 1609.34,
        "nmi" => value * 1852.0,
        "in" => value * 0.0254,
        "ft" => value * 0.3048,
        "yd" => value * 0.9144,

        // Area (to square meters)
        "km²" => value * 1_000_000.0,
        "m²" => value,
        "ha" => value * 10000.0,
        "ac" => value * 4046.86,

        // Weight (to grams)
        "kg" => value * 1000.0,
        "g" => value,
        "mg" => value / 1000.0,
        "µg" => value / 1_000_000.0,
        "lb" => value * 453.592,
        "oz" => value * 28.3495,
        "st" => value * 6350.29,
        "t" => value * 1_000_000.0,

        // Volume (to milliliters)
        "l" => value * 1000.0,
        "ml" => value,
        "cl" => value * 10.0,
        "gal" => value * 3785.41,
        "qt" => value * 946.353,
        "pt" => value * 473.176,
        "cup" => value * 236.588,
        "floz" => value * 29.5735,
        "tbsp" => value * 14.7868,
        "tsp" => value * 4.92892,

        // Time (to seconds)
        "y" => value * 31_536_000.0,
        "mo" => value * 2_592_000.0,
        "w_t" => value * 604_800.0, // weeks conversion as time
        "d" => value * 86400.0,
        "h" => value * 3600.0,
        "min" => value * 60.0,
        "sec" => value,
        "ms" => value / 1000.0,
        "µs" => value / 1_000_000.0,
        "ns" => value / 1_000_000_000.0,

        // Frequency (to Hz)
        "thz" => value * 1_000_000_000_000.0,
        "hz" => value,
        "khz" => value * 1000.0,
        "mhz" => value * 1_000_000.0,
        "ghz" => value * 1_000_000_000.0,

        // Electrical
        // Voltage (to V)
        "v" => value,
        "mv" => value / 1000.0,
        "kv" => value * 1000.0,
        "µv" => value / 1_000_000.0,
        // Current (to A)
        "a" => value,
        "ma" => value / 1000.0,
        "µa" => value / 1_000_000.0,
        // Resistance (to Ω)
        "ohm" => value,
        "kohm" => value * 1000.0,
        "mohm" => value * 1_000_000.0,
        // Power (to W)
        "w_e" => value, // electrical watts base is watts
        "w" => value,   // electrical watts base is watts
        "kw" => value * 1000.0,
        "mw" => value * 1_000_000.0,
        "gw" => value * 1_000_000_000.0,

        // Data Transfer (to B/s)
        "bps" => value,
        "kbps" => value * 1024.0,
        "mbps" => value * 1024.0 * 1024.0,
        "gbps" => value * 1024.0 * 1024.0 * 1024.0,
        "tbps" => value * 1024.0 * 1024.0 * 1024.0 * 1024.0,

        // Digital Storage (to B)
        "b" => value,
        "kb" => value * 1024.0,
        "mb" => value * 1024.0 * 1024.0,
        "gb" => value * 1024.0 * 1024.0 * 1024.0,
        "tb" => value * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "pb" => value * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,

        // Fuel Economy (to km/L)
        "kml" => value,
        "mpg" => value * 0.425144,
        "lp100km" => {
            if value != 0.0 {
                100.0 / value
            } else {
                return None;
            }
        }

        // Angle (to radians)
        "rad" => value,
        "deg" => value * std::f64::consts::PI / 180.0,
        "grad" => value * std::f64::consts::PI / 200.0,
        "arcmin" => value * std::f64::consts::PI / (180.0 * 60.0),
        "arcsec" => value * std::f64::consts::PI / (180.0 * 3600.0),

        // Pressure (to Pa)
        "pa" => value,
        "hpa" => value * 100.0,
        "kpa" => value * 1000.0,
        "mpa" => value * 1_000_000.0,
        "bar" => value * 100_000.0,
        "mbar" => value * 100.0,
        "psi" => value * 6894.76,
        "mmhg" => value * 133.322,
        "atm" => value * 101325.0,

        _ => return None,
    })
}

fn from_base_unit(value: f64, unit: &str) -> Option<f64> {
    Some(match unit {
        // Length (from meters)
        "km" => value / 1000.0,
        "m" => value,
        "cm" => value * 100.0,
        "mm" => value * 1000.0,
        "µm" => value * 1_000_000.0,
        "nm" => value * 1_000_000_000.0,
        "mi" => value / 1609.34,
        "nmi" => value / 1852.0,
        "in" => value / 0.0254,
        "ft" => value / 0.3048,
        "yd" => value / 0.9144,

        // Area (from square meters)
        "km²" => value / 1_000_000.0,
        "m²" => value,
        "ha" => value / 10000.0,
        "ac" => value / 4046.86,

        // Weight (from grams)
        "kg" => value / 1000.0,
        "g" => value,
        "mg" => value * 1000.0,
        "µg" => value * 1_000_000.0,
        "lb" => value / 453.592,
        "oz" => value / 28.3495,
        "st" => value / 6350.29,
        "t" => value / 1_000_000.0,

        // Volume (from milliliters)
        "l" => value / 1000.0,
        "ml" => value,
        "cl" => value / 10.0,
        "gal" => value / 3785.41,
        "qt" => value / 946.353,
        "pt" => value / 473.176,
        "cup" => value / 236.588,
        "floz" => value / 29.5735,
        "tbsp" => value / 14.7868,
        "tsp" => value / 4.92892,

        // Time (from seconds)
        "y" => value / 31_536_000.0,
        "mo" => value / 2_592_000.0,
        "w_t" => value / 604_800.0,
        "d" => value / 86400.0,
        "h" => value / 3600.0,
        "min" => value / 60.0,
        "sec" => value,
        "ms" => value * 1000.0,
        "µs" => value * 1_000_000.0,
        "ns" => value * 1_000_000_000.0,

        // Frequency (from Hz)
        "thz" => value / 1_000_000_000_000.0,
        "hz" => value,
        "khz" => value / 1000.0,
        "mhz" => value / 1_000_000.0,
        "ghz" => value / 1_000_000_000.0,

        // Electrical
        // Voltage (from V)
        "v" => value,
        "mv" => value * 1000.0,
        "kv" => value / 1000.0,
        "µv" => value * 1_000_000.0,
        // Current (from A)
        "a" => value,
        "ma" => value * 1000.0,
        "µa" => value * 1_000_000.0,
        // Resistance (from Ω)
        "ohm" => value,
        "kohm" => value / 1000.0,
        "mohm" => value / 1_000_000.0,
        // Power (from W)
        "w_e" => value, // electrical watts
        "w" => value,   // electrical watts
        "kw" => value / 1000.0,
        "mw" => value / 1_000_000.0,
        "gw" => value / 1_000_000_000.0,

        // Data Transfer (from B/s)
        "bps" => value,
        "kbps" => value / 1024.0,
        "mbps" => value / (1024.0 * 1024.0),
        "gbps" => value / (1024.0 * 1024.0 * 1024.0),
        "tbps" => value / (1024.0 * 1024.0 * 1024.0 * 1024.0),

        // Digital Storage (from B)
        "b" => value,
        "kb" => value / 1024.0,
        "mb" => value / (1024.0 * 1024.0),
        "gb" => value / (1024.0 * 1024.0 * 1024.0),
        "tb" => value / (1024.0 * 1024.0 * 1024.0 * 1024.0),
        "pb" => value / (1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0),

        // Fuel Economy (from km/L)
        "kml" => value,
        "mpg" => value / 0.425144,
        "lp100km" => {
            if value != 0.0 {
                100.0 / value
            } else {
                return None;
            }
        }

        // Angle (from radians)
        "rad" => value,
        "deg" => value * 180.0 / std::f64::consts::PI,
        "grad" => value * 200.0 / std::f64::consts::PI,
        "arcmin" => value * 180.0 * 60.0 / std::f64::consts::PI,
        "arcsec" => value * 180.0 * 3600.0 / std::f64::consts::PI,

        // Pressure (from Pa)
        "pa" => value,
        "hpa" => value / 100.0,
        "kpa" => value / 1000.0,
        "mpa" => value / 1_000_000.0,
        "bar" => value / 100_000.0,
        "mbar" => value / 100.0,
        "psi" => value / 6894.76,
        "mmhg" => value / 133.322,
        "atm" => value / 101325.0,

        _ => return None,
    })
}

fn convert_temperature(value: f64, from: &str, to: &str) -> Option<f64> {
    // Convert to Kelvin first
    let kelvin = match from {
        "c" => value + 273.15,
        "f" => (value - 32.0) * 5.0 / 9.0 + 273.15,
        "k" => value,
        _ => return None,
    };

    // Convert from Kelvin to target
    Some(match to {
        "c" => kelvin - 273.15,
        "f" => (kelvin - 273.15) * 9.0 / 5.0 + 32.0,
        "k" => kelvin,
        _ => return None,
    })
}

fn get_conversion_formula(from: &str, to: &str) -> Option<String> {
    let from = normalize_unit(from);
    let to = normalize_unit(to);

    // Handle temperature conversions first as they're special cases
    match (from, to) {
        ("c", "f") => return Some("multiply by 9/5, then add 32".to_string()),
        ("f", "c") => return Some("subtract 32, then multiply by 5/9".to_string()),
        ("c", "k") => return Some("add 273.15".to_string()),
        ("k", "c") => return Some("subtract 273.15".to_string()),
        ("f", "k") => return Some("subtract 32, multiply by 5/9, then add 273.15".to_string()),
        ("k", "f") => return Some("subtract 273.15, multiply by 9/5, then add 32".to_string()),
        _ => {}
    }

    // Get base unit values for comparison
    let unit1 = to_base_unit(1.0, from)?;
    let unit2 = to_base_unit(1.0, to)?;

    // If we can represent it as a simple multiplication
    if unit1 != 0.0 && unit2 != 0.0 {
        let factor = unit1 / unit2;
        if factor != 1.0 {
            return generate_simple_formula(factor);
        }
    }

    None
}

fn generate_simple_formula(factor: f64) -> Option<String> {
    // Round factor to a reasonable number of decimal places
    let factor = (factor * 100000.0).round() / 100000.0;

    // Handle special cases for common fractions
    match factor {
        60.0 => Some("multiply by 60".to_string()),
        24.0 => Some("multiply by 24".to_string()),
        7.0 => Some("multiply by 7".to_string()),
        12.0 => Some("multiply by 12".to_string()),
        0.5 => Some("divide by 2".to_string()),
        0.25 => Some("divide by 4".to_string()),
        0.1 => Some("divide by 10".to_string()),
        10.0 => Some("multiply by 10".to_string()),
        100.0 => Some("multiply by 100".to_string()),
        1000.0 => Some("multiply by 1000".to_string()),
        0.001 => Some("divide by 1000".to_string()),
        0.01 => Some("divide by 100".to_string()),
        _ => {
            // For other values, show decimal multiplication
            if factor > 1.0 {
                Some(format!("multiply by {}", factor))
            } else {
                Some(format!("multiply by {}", factor))
            }
        }
    }
}

pub fn get_unit_name(unit: &str) -> &'static str {
    match normalize_unit(unit) {
        "km" => "kilometers",
        "m" => "meters",
        "cm" => "centimeters",
        "mm" => "millimeters",
        "µm" => "micrometers",
        "nm" => "nanometers",
        "mi" => "miles",
        "nmi" => "nautical miles",
        "in" => "inches",
        "ft" => "feet",
        "yd" => "yards",
        "km²" => "square kilometers",
        "m²" => "square meters",
        "ha" => "hectares",
        "ac" => "acres",
        "kg" => "kilograms",
        "g" => "grams",
        "mg" => "milligrams",
        "µg" => "micrograms",
        "lb" => "pounds",
        "oz" => "ounces",
        "st" => "stone",
        "t" => "tons",
        "l" => "liters",
        "ml" => "milliliters",
        "cl" => "centiliters",
        "gal" => "gallons",
        "qt" => "quarts",
        "pt" => "pints",
        "cup" => "cups",
        "floz" => "fluid ounces",
        "tbsp" => "tablespoons",
        "tsp" => "teaspoons",
        "c" => "Celsius",
        "f" => "Fahrenheit",
        "k" => "Kelvin",
        "y" => "years",
        "mo" => "months",
        "w_t" => "weeks",
        "d" => "days",
        "h" => "hours",
        "min" => "minutes",
        "sec" => "seconds",
        "ms" => "milliseconds",
        "µs" => "microseconds",
        "ns" => "nanoseconds",
        "hz" => "Hertz",
        "khz" => "Kilohertz",
        "mhz" => "Megahertz",
        "ghz" => "Gigahertz",
        "thz" => "Terahertz",
        "v" => "Volts",
        "mv" => "Millivolts",
        "kv" => "Kilovolts",
        "µv" => "Microvolts",
        "a" => "Amperes",
        "ma" => "Milliamperes",
        "µa" => "Microamps",
        "ohm" => "Ohms",
        "kohm" => "Kilohms",
        "mohm" => "Megohms",
        "w_e" => "Watts",
        "w" => "Watts",
        "kw" => "Kilowatts",
        "mw" => "Megawatts",
        "gw" => "Gigawatts",

        // Data Transfer
        "bps" => "Bytes per second",
        "kbps" => "Kilobytes per second",
        "mbps" => "Megabytes per second",
        "gbps" => "Gigabytes per second",
        "tbps" => "Terabytes per second",

        // Digital Storage
        "b" => "Bytes",
        "kb" => "Kilobytes",
        "mb" => "Megabytes",
        "gb" => "Gigabytes",
        "tb" => "Terabytes",
        "pb" => "Petabytes",

        // Fuel Economy
        "kml" => "Kilometers per liter",
        "mpg" => "Miles per gallon",
        "lp100km" => "Liters per 100 kilometers",

        // Angle
        "rad" => "Radians",
        "deg" => "Degrees",
        "grad" => "Gradians",
        "arcmin" => "Arc minutes",
        "arcsec" => "Arc seconds",

        // Pressure
        "pa" => "Pascals",
        "hpa" => "Hectopascals",
        "kpa" => "Kilopascals",
        "mpa" => "Megapascals",
        "bar" => "Bars",
        "mbar" => "Millibars",
        "psi" => "Pounds per square inch",
        "mmhg" => "Millimeters of mercury",
        "atm" => "Atmospheres",

        _ => "unknown",
    }
}

static UNIT_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // Length
    for &s in &["kilometers", "kilometer", "kms", "km"] {
        m.insert(s, "km");
    }
    for &s in &["meters", "meter", "ms", "m"] {
        m.insert(s, "m");
    }
    for &s in &["centimeters", "centimeter", "cms", "cm"] {
        m.insert(s, "cm");
    }
    for &s in &["millimeters", "millimeter", "mms", "mm"] {
        m.insert(s, "mm");
    }
    for &s in &["micrometers", "micrometer", "µm", "um"] {
        m.insert(s, "µm");
    }
    for &s in &["nanometers", "nanometer", "nm"] {
        m.insert(s, "nm");
    }
    for &s in &["miles", "mile", "mi"] {
        m.insert(s, "mi");
    }
    for &s in &["nautical", "nauticalmiles", "nmi"] {
        m.insert(s, "nmi");
    }
    for &s in &["inches", "inch", "ins", "in"] {
        m.insert(s, "in");
    }
    for &s in &["feet", "foot", "ft"] {
        m.insert(s, "ft");
    }
    for &s in &["yards", "yard", "yd"] {
        m.insert(s, "yd");
    }

    // Area
    for &s in &["squarekilometers", "km2", "km²"] {
        m.insert(s, "km²");
    }
    for &s in &["squaremeters", "m2", "m²"] {
        m.insert(s, "m²");
    }
    for &s in &["hectares", "hectare", "ha"] {
        m.insert(s, "ha");
    }
    for &s in &["acres", "acre", "ac"] {
        m.insert(s, "ac");
    }

    // Weight
    for &s in &["kilograms", "kilogram", "kgs", "kg"] {
        m.insert(s, "kg");
    }
    for &s in &["grams", "gram", "gs", "g"] {
        m.insert(s, "g");
    }
    for &s in &["milligrams", "milligram", "mg"] {
        m.insert(s, "mg");
    }
    for &s in &["micrograms", "microgram", "µg", "ug"] {
        m.insert(s, "µg");
    }
    for &s in &["pounds", "pound", "lbs", "lb"] {
        m.insert(s, "lb");
    }
    for &s in &["ounces", "ounce", "oz"] {
        m.insert(s, "oz");
    }
    for &s in &["stone", "stones", "st"] {
        m.insert(s, "st");
    }
    for &s in &["tons", "ton", "t"] {
        m.insert(s, "t");
    }

    // Volume
    for &s in &["liters", "liter", "ls", "l"] {
        m.insert(s, "l");
    }
    for &s in &["milliliters", "milliliter", "mls", "ml"] {
        m.insert(s, "ml");
    }
    for &s in &["centiliters", "centiliter", "cl"] {
        m.insert(s, "cl");
    }
    for &s in &["gallons", "gallon", "gal"] {
        m.insert(s, "gal");
    }
    for &s in &["quarts", "quart", "qt"] {
        m.insert(s, "qt");
    }
    for &s in &["pints", "pint", "pt"] {
        m.insert(s, "pt");
    }
    for &s in &["cups", "cup"] {
        m.insert(s, "cup");
    }
    for &s in &["fluidounces", "floz", "fl"] {
        m.insert(s, "floz");
    }
    for &s in &["tablespoons", "tablespoon", "tbsp"] {
        m.insert(s, "tbsp");
    }
    for &s in &["teaspoons", "teaspoon", "tsp"] {
        m.insert(s, "tsp");
    }

    // Temperature
    m.insert("celsius", "c");
    m.insert("c", "c");
    m.insert("fahrenheit", "f");
    m.insert("f", "f");
    m.insert("kelvin", "k");
    m.insert("k", "k");

    // Time
    for &s in &["years", "year", "yr", "y"] {
        m.insert(s, "y");
    }
    for &s in &["months", "month", "mo"] {
        m.insert(s, "mo");
    }
    for &s in &["weeks", "week", "wk", "w"] {
        m.insert(s, "w_t");
    }
    for &s in &["days", "day", "d"] {
        m.insert(s, "d");
    }
    for &s in &["hours", "hour", "hrs", "hr", "h"] {
        m.insert(s, "h");
    }
    for &s in &["minutes", "minute", "mins", "min"] {
        m.insert(s, "min");
    }
    for &s in &["seconds", "second", "secs", "sec", "s"] {
        m.insert(s, "sec");
    }
    for &s in &["milliseconds", "millisecond", "ms"] {
        m.insert(s, "ms");
    }
    for &s in &["microseconds", "microsecond", "µs", "us"] {
        m.insert(s, "µs");
    }
    for &s in &["nanoseconds", "nanosecond", "ns"] {
        m.insert(s, "ns");
    }

    // Frequency
    for &s in &["terahertz", "thz"] {
        m.insert(s, "thz");
    }
    for &s in &["hertz", "hz"] {
        m.insert(s, "hz");
    }
    for &s in &["kilohertz", "khz"] {
        m.insert(s, "khz");
    }
    for &s in &["megahertz", "mhz"] {
        m.insert(s, "mhz");
    }
    for &s in &["gigahertz", "ghz"] {
        m.insert(s, "ghz");
    }

    // Electrical
    for &s in &["gigawatts", "gigawatt", "gw"] {
        m.insert(s, "gw");
    }
    for &s in &["microvolts", "microvolt", "µv", "uv"] {
        m.insert(s, "µv");
    }
    for &s in &["microamps", "microamp", "µa", "ua"] {
        m.insert(s, "µa");
    }
    for &s in &["volts", "volt", "v"] {
        m.insert(s, "v");
    }
    for &s in &["millivolts", "millivolt", "mv"] {
        m.insert(s, "mv");
    }
    for &s in &["kilovolts", "kilovolt", "kv"] {
        m.insert(s, "kv");
    }
    for &s in &["amperes", "ampere", "amps", "amp", "a"] {
        m.insert(s, "a");
    }
    for &s in &["milliamperes", "milliamp", "ma"] {
        m.insert(s, "ma");
    }
    for &s in &["ohms", "ohm", "Ω"] {
        m.insert(s, "ohm");
    }
    for &s in &["kilohms", "kilohm", "kΩ", "kohm"] {
        m.insert(s, "kohm");
    }
    for &s in &["megohms", "megohm", "MΩ", "mohm"] {
        m.insert(s, "mohm");
    }
    for &s in &["watts", "watt", "w"] {
        m.insert(s, "w_e");
    }
    for &s in &["kilowatts", "kilowatt", "kw"] {
        m.insert(s, "kw");
    }
    for &s in &["megawatts", "megawatt", "mw"] {
        m.insert(s, "mw");
    }

    // Data Transfer
    for &s in &["bytespersecond", "bps"] {
        m.insert(s, "bps");
    }
    for &s in &["kilobytespersecond", "kbps"] {
        m.insert(s, "kbps");
    }
    for &s in &["megabytespersecond", "mbps"] {
        m.insert(s, "mbps");
    }
    for &s in &["gigabytespersecond", "gbps"] {
        m.insert(s, "gbps");
    }
    for &s in &["terabytespersecond", "tbps"] {
        m.insert(s, "tbps");
    }

    // Digital Storage
    for &s in &["bytes", "byte", "b"] {
        m.insert(s, "b");
    }
    for &s in &["kilobytes", "kilobyte", "kb"] {
        m.insert(s, "kb");
    }
    for &s in &["megabytes", "megabyte", "mb"] {
        m.insert(s, "mb");
    }
    for &s in &["gigabytes", "gigabyte", "gb"] {
        m.insert(s, "gb");
    }
    for &s in &["terabytes", "terabyte", "tb"] {
        m.insert(s, "tb");
    }
    for &s in &["petabytes", "petabyte", "pb"] {
        m.insert(s, "pb");
    }

    // Fuel Economy
    for &s in &["kilometersperliter", "kml"] {
        m.insert(s, "kml");
    }
    for &s in &["milespergallon", "mpg"] {
        m.insert(s, "mpg");
    }
    for &s in &["litresper100km", "lp100km"] {
        m.insert(s, "lp100km");
    }

    // Angle
    for &s in &["radians", "radian", "rad"] {
        m.insert(s, "rad");
    }
    for &s in &["degrees", "degree", "deg", "°"] {
        m.insert(s, "deg");
    }
    for &s in &["gradians", "gradian", "grad"] {
        m.insert(s, "grad");
    }
    for &s in &["arcminutes", "arcminute", "arcmin", "'"] {
        m.insert(s, "arcmin");
    }
    for &s in &["arcseconds", "arcsecond", "arcsec", "\""] {
        m.insert(s, "arcsec");
    }

    // Pressure
    for &s in &["pascals", "pascal", "pa"] {
        m.insert(s, "pa");
    }
    for &s in &["hectopascals", "hectopascal", "hpa"] {
        m.insert(s, "hpa");
    }
    for &s in &["kilopascals", "kilopascal", "kpa"] {
        m.insert(s, "kpa");
    }
    for &s in &["megapascals", "megapascal", "mpa"] {
        m.insert(s, "mpa");
    }
    for &s in &["bars", "bar"] {
        m.insert(s, "bar");
    }
    for &s in &["millibars", "millibar", "mbar"] {
        m.insert(s, "mbar");
    }
    for &s in &["psi", "lbf/in²"] {
        m.insert(s, "psi");
    }
    for &s in &["mmhg", "mmHg", "torr"] {
        m.insert(s, "mmhg");
    }
    for &s in &["atmospheres", "atmosphere", "atm"] {
        m.insert(s, "atm");
    }

    m
});

fn normalize_unit(unit: &str) -> &'static str {
    UNIT_MAP
        .get(unit.trim().to_lowercase().as_str())
        .copied()
        .unwrap_or("unknown")
}

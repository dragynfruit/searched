extern crate fend_core;
use fend_core::Context;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static CALC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:(?P<prefix>calc(?:ulator)?|convert)\s+)?(?P<expr>.+)$").unwrap()
});

#[derive(Debug, Serialize)]
pub struct Calculator {
    pub expression: String,
    pub result: String,
}

impl Calculator {
    pub fn detect(query: &str) -> Option<Self> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }
        let lower = trimmed.to_lowercase();
        // If the query is exactly "calc" or "calculator", open a blank calculator.
        if lower == "calc" || lower == "calculator" {
            return Some(Self {
                expression: "".to_string(),
                result: "".to_string(),
            });
        }
        // Use regex to parse query.
        let caps = CALC_RE.captures(trimmed)?;
        // Get the expression portion.
        let expr = caps.name("expr")?.as_str().trim();
        // If no expression is provided, return a blank calculator.
        if expr.is_empty() {
            return Some(Self {
                expression: "".to_string(),
                result: "".to_string(),
            });
        }
        let explicit_conversion = match caps.name("prefix") {
            Some(m) if m.as_str().to_lowercase() == "convert" => true,
            _ => false,
        };
        // Only accept if expression starts with a digit, math symbol, or contains "to"
        let first = expr.chars().next()?;
        if !first.is_ascii_digit() && !"(-+.".contains(first) && !expr.contains("to") {
            return None;
        }
        let calc = Self::evaluate(expr);
        if !explicit_conversion && calc.result == "Error" {
            None
        } else {
            Some(calc)
        }
    }

    pub fn evaluate(expr: &str) -> Self {
        let mut context = Context::new();
        let result = fend_core::evaluate(expr, &mut context)
            .map(|r| r.get_main_result().to_string())
            .unwrap_or_else(|_| "Error".to_string());
        Calculator {
            expression: expr.to_string(),
            result,
        }
    }
}

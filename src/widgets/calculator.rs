use html_entities::decode_html_entities;
use meval::eval_str;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static CALCULATOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?i)(?:calc(?:ulator)?|=)\s*(?P<expr>.+)?$").unwrap());

#[derive(Debug, Serialize)]
pub struct Calculator {
    pub expression: String,
    pub result: String,
    pub mode: CalcMode,
}

#[derive(Debug, Serialize)]
pub enum CalcMode {
    Expression, // Direct expression evaluation
    Calculator, // Interactive calculator mode
}

impl Calculator {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim();
        // Skip if query is shorter than "="
        if query.is_empty() {
            return None;
        }
        let query = decode_html_entities(query.trim()).ok()?;

        // Check for calculator command patterns
        if let Some(caps) = CALCULATOR_RE.captures(&query) {
            if let Some(expr) = caps.name("expr") {
                // Expression provided - evaluate it
                return Some(Self::evaluate(expr.as_str().trim(), CalcMode::Expression));
            } else {
                // No expression - return empty calculator
                return Some(Calculator {
                    expression: String::new(),
                    result: String::new(),
                    mode: CalcMode::Calculator,
                });
            }
        }

        // Check if the query starts with a number or math symbol
        if query.starts_with(|c: char| c.is_ascii_digit() || "(-+.".contains(c)) {
            return Some(Self::evaluate(&query, CalcMode::Expression));
        }

        None
    }

    fn evaluate(expr: &str, mode: CalcMode) -> Self {
        // Decode any HTML entities in the input expression
        let decoded_expr = decode_html_entities(expr).unwrap_or_else(|_| expr.to_string());
        let expression = decoded_expr.clone();

        // Attempt to evaluate the expression
        match eval_str(&decoded_expr) {
            Ok(result) => {
                let result_str = if result.fract() == 0.0 {
                    result.to_string().trim_end_matches(".0").to_string()
                } else {
                    format!("{:.8}", result)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string()
                };

                Calculator {
                    expression, // Use decoded expression for display
                    result: result_str,
                    mode,
                }
            }
            Err(_) => Calculator {
                expression, // Use decoded expression for display
                result: "Error".to_string(),
                mode,
            },
        }
    }
}

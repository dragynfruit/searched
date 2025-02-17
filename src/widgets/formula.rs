use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct Formula {
    pub category: String,
    pub formulas: Vec<FormulaEntry>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FormulaEntry {
    pub name: String,
    pub formula: String,
    pub variables: Vec<String>,
}

static FORMULA_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:(?:(?P<formula_type>[a-z]+)\s+)?(?:formula|equation)s?\s+(?:for\s+|of\s+)?(?P<category>[a-z\s]+)|(?P<category2>[a-z\s]+)\s+(?:(?P<formula_type2>[a-z]+)\s+)?(?:formula|equation)s?)$").unwrap()
});

static FORMULAS: Lazy<HashMap<&'static str, Vec<FormulaEntry>>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Basic Shapes
    m.insert(
        "square",
        vec![
            FormulaEntry {
                name: "Area".to_string(),
                formula: "A = s²".to_string(),
                variables: vec!["A = area".to_string(), "s = side length".to_string()],
            },
            FormulaEntry {
                name: "Perimeter".to_string(),
                formula: "P = 4s".to_string(),
                variables: vec!["P = perimeter".to_string(), "s = side length".to_string()],
            },
            FormulaEntry {
                name: "Diagonal".to_string(),
                formula: "d = s√2".to_string(),
                variables: vec!["d = diagonal".to_string(), "s = side length".to_string()],
            },
        ],
    );

    m.insert(
        "circle",
        vec![
            FormulaEntry {
                name: "Area".to_string(),
                formula: "A = πr²".to_string(),
                variables: vec!["A = area".to_string(), "r = radius".to_string()],
            },
            FormulaEntry {
                name: "Circumference".to_string(),
                formula: "C = 2πr".to_string(),
                variables: vec!["C = circumference".to_string(), "r = radius".to_string()],
            },
            FormulaEntry {
                name: "Diameter".to_string(),
                formula: "d = 2r".to_string(),
                variables: vec!["d = diameter".to_string(), "r = radius".to_string()],
            },
        ],
    );

    m.insert(
        "triangle",
        vec![
            FormulaEntry {
                name: "Area".to_string(),
                formula: "A = ½bh".to_string(),
                variables: vec![
                    "A = area".to_string(),
                    "b = base".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Perimeter".to_string(),
                formula: "P = a + b + c".to_string(),
                variables: vec![
                    "P = perimeter".to_string(),
                    "a,b,c = side lengths".to_string(),
                ],
            },
            FormulaEntry {
                name: "Pythagorean Theorem".to_string(),
                formula: "a² + b² = c²".to_string(),
                variables: vec![
                    "c = hypotenuse".to_string(),
                    "a,b = other sides".to_string(),
                ],
            },
        ],
    );

    // 3D Shapes
    m.insert(
        "cube",
        vec![
            FormulaEntry {
                name: "Volume".to_string(),
                formula: "V = s³".to_string(),
                variables: vec!["V = volume".to_string(), "s = side length".to_string()],
            },
            FormulaEntry {
                name: "Surface Area".to_string(),
                formula: "SA = 6s²".to_string(),
                variables: vec![
                    "SA = surface area".to_string(),
                    "s = side length".to_string(),
                ],
            },
            FormulaEntry {
                name: "Diagonal".to_string(),
                formula: "d = s√3".to_string(),
                variables: vec!["d = diagonal".to_string(), "s = side length".to_string()],
            },
        ],
    );

    m.insert(
        "sphere",
        vec![
            FormulaEntry {
                name: "Volume".to_string(),
                formula: "V = (4/3)πr³".to_string(),
                variables: vec!["V = volume".to_string(), "r = radius".to_string()],
            },
            FormulaEntry {
                name: "Surface Area".to_string(),
                formula: "SA = 4πr²".to_string(),
                variables: vec!["SA = surface area".to_string(), "r = radius".to_string()],
            },
            FormulaEntry {
                name: "Diameter".to_string(),
                formula: "d = 2r".to_string(),
                variables: vec!["d = diameter".to_string(), "r = radius".to_string()],
            },
        ],
    );

    // Cylinder
    m.insert(
        "cylinder",
        vec![
            FormulaEntry {
                name: "Volume".to_string(),
                formula: "V = πr²h".to_string(),
                variables: vec![
                    "V = volume".to_string(),
                    "r = radius".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Surface Area".to_string(),
                formula: "SA = 2πr² + 2πrh".to_string(),
                variables: vec![
                    "SA = surface area".to_string(),
                    "r = radius".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Lateral Surface Area".to_string(),
                formula: "LSA = 2πrh".to_string(),
                variables: vec![
                    "LSA = lateral surface area".to_string(),
                    "r = radius".to_string(),
                    "h = height".to_string(),
                ],
            },
        ],
    );

    // Prism
    m.insert(
        "prism",
        vec![
            FormulaEntry {
                name: "Volume".to_string(),
                formula: "V = Bh".to_string(),
                variables: vec![
                    "V = volume".to_string(),
                    "B = base area".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Surface Area".to_string(),
                formula: "SA = 2B + Ph".to_string(),
                variables: vec![
                    "SA = surface area".to_string(),
                    "B = base area".to_string(),
                    "P = perimeter of base".to_string(),
                    "h = height".to_string(),
                ],
            },
        ],
    );

    // Electromagnetic
    m.insert(
        "electromagnetic",
        vec![
            FormulaEntry {
                name: "Ohm's Law".to_string(),
                formula: "V = IR".to_string(),
                variables: vec![
                    "V = voltage".to_string(),
                    "I = current".to_string(),
                    "R = resistance".to_string(),
                ],
            },
            FormulaEntry {
                name: "Electrical Power".to_string(),
                formula: "P = VI".to_string(),
                variables: vec![
                    "P = power".to_string(),
                    "V = voltage".to_string(),
                    "I = current".to_string(),
                ],
            },
            FormulaEntry {
                name: "Capacitance".to_string(),
                formula: "C = Q/V".to_string(),
                variables: vec![
                    "C = capacitance".to_string(),
                    "Q = charge".to_string(),
                    "V = voltage".to_string(),
                ],
            },
            FormulaEntry {
                name: "Coulomb's Law".to_string(),
                formula: "F = k(q₁q₂)/r²".to_string(),
                variables: vec![
                    "F = force".to_string(),
                    "k = Coulomb's constant".to_string(),
                    "q₁,q₂ = charges".to_string(),
                    "r = distance".to_string(),
                ],
            },
        ],
    );

    // Fluid Mechanics
    m.insert(
        "fluid",
        vec![
            FormulaEntry {
                name: "Density".to_string(),
                formula: "ρ = m/V".to_string(),
                variables: vec![
                    "ρ = density".to_string(),
                    "m = mass".to_string(),
                    "V = volume".to_string(),
                ],
            },
            FormulaEntry {
                name: "Pressure".to_string(),
                formula: "P = F/A".to_string(),
                variables: vec![
                    "P = pressure".to_string(),
                    "F = force".to_string(),
                    "A = area".to_string(),
                ],
            },
            FormulaEntry {
                name: "Hydrostatic Pressure".to_string(),
                formula: "P = ρgh".to_string(),
                variables: vec![
                    "P = pressure".to_string(),
                    "ρ = density".to_string(),
                    "g = gravity".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Bernoulli's Equation".to_string(),
                formula: "P₁ + ½ρv₁² + ρgh₁ = P₂ + ½ρv₂² + ρgh₂".to_string(),
                variables: vec![
                    "P = pressure".to_string(),
                    "ρ = density".to_string(),
                    "v = velocity".to_string(),
                    "g = gravity".to_string(),
                    "h = height".to_string(),
                ],
            },
        ],
    );

    // Thermodynamics
    m.insert(
        "thermo",
        vec![
            FormulaEntry {
                name: "First Law".to_string(),
                formula: "ΔU = Q - W".to_string(),
                variables: vec![
                    "ΔU = internal energy change".to_string(),
                    "Q = heat".to_string(),
                    "W = work".to_string(),
                ],
            },
            FormulaEntry {
                name: "Ideal Gas Law".to_string(),
                formula: "PV = nRT".to_string(),
                variables: vec![
                    "P = pressure".to_string(),
                    "V = volume".to_string(),
                    "n = moles".to_string(),
                    "R = gas constant".to_string(),
                    "T = temperature".to_string(),
                ],
            },
            FormulaEntry {
                name: "Heat Transfer".to_string(),
                formula: "Q = mcΔT".to_string(),
                variables: vec![
                    "Q = heat".to_string(),
                    "m = mass".to_string(),
                    "c = specific heat".to_string(),
                    "ΔT = temperature change".to_string(),
                ],
            },
        ],
    );

    // Physics
    m.insert(
        "physics",
        vec![
            FormulaEntry {
                name: "Newton's Second Law".to_string(),
                formula: "F = ma".to_string(),
                variables: vec![
                    "F = force".to_string(),
                    "m = mass".to_string(),
                    "a = acceleration".to_string(),
                ],
            },
            FormulaEntry {
                name: "Kinetic Energy".to_string(),
                formula: "KE = ½mv²".to_string(),
                variables: vec![
                    "KE = kinetic energy".to_string(),
                    "m = mass".to_string(),
                    "v = velocity".to_string(),
                ],
            },
            FormulaEntry {
                name: "Potential Energy".to_string(),
                formula: "PE = mgh".to_string(),
                variables: vec![
                    "PE = potential energy".to_string(),
                    "m = mass".to_string(),
                    "g = gravity".to_string(),
                    "h = height".to_string(),
                ],
            },
            FormulaEntry {
                name: "Velocity".to_string(),
                formula: "v = d/t".to_string(),
                variables: vec![
                    "v = velocity".to_string(),
                    "d = distance".to_string(),
                    "t = time".to_string(),
                ],
            },
            FormulaEntry {
                name: "Power".to_string(),
                formula: "P = W/t".to_string(),
                variables: vec![
                    "P = power".to_string(),
                    "W = work".to_string(),
                    "t = time".to_string(),
                ],
            },
        ],
    );

    // Calculus
    m.insert(
        "calculus",
        vec![
            FormulaEntry {
                name: "Power Rule".to_string(),
                formula: "d/dx(xⁿ) = n·xⁿ⁻¹".to_string(),
                variables: vec![
                    "d/dx = derivative".to_string(),
                    "n = power".to_string(),
                    "x = variable".to_string(),
                ],
            },
            FormulaEntry {
                name: "Chain Rule".to_string(),
                formula: "d/dx[f(g(x))] = f'(g(x))·g'(x)".to_string(),
                variables: vec![
                    "f(x) = outer function".to_string(),
                    "g(x) = inner function".to_string(),
                    "f' = derivative of f".to_string(),
                    "g' = derivative of g".to_string(),
                ],
            },
            FormulaEntry {
                name: "Product Rule".to_string(),
                formula: "d/dx[f(x)g(x)] = f'(x)g(x) + f(x)g'(x)".to_string(),
                variables: vec![
                    "f(x), g(x) = functions".to_string(),
                    "f', g' = derivatives".to_string(),
                ],
            },
            FormulaEntry {
                name: "Quotient Rule".to_string(),
                formula: "d/dx[f(x)/g(x)] = [f'(x)g(x) - f(x)g'(x)]/[g(x)]²".to_string(),
                variables: vec![
                    "f(x), g(x) = functions".to_string(),
                    "f', g' = derivatives".to_string(),
                ],
            },
            FormulaEntry {
                name: "Basic Integral".to_string(),
                formula: "∫xⁿ dx = (xⁿ⁺¹)/(n+1) + C".to_string(),
                variables: vec![
                    "∫ = integral".to_string(),
                    "n = power".to_string(),
                    "C = constant of integration".to_string(),
                ],
            },
            FormulaEntry {
                name: "Integration by Parts".to_string(),
                formula: "∫u dv = uv - ∫v du".to_string(),
                variables: vec![
                    "u, v = functions".to_string(),
                    "du, dv = differentials".to_string(),
                ],
            },
            FormulaEntry {
                name: "Fundamental Theorem".to_string(),
                formula: "∫ₐᵇ f(x)dx = F(b) - F(a)".to_string(),
                variables: vec![
                    "f(x) = function".to_string(),
                    "F(x) = antiderivative".to_string(),
                    "a,b = limits".to_string(),
                ],
            },
            FormulaEntry {
                name: "Common Derivatives".to_string(),
                formula: "d/dx(sin x) = cos x\nd/dx(eˣ) = eˣ\nd/dx(ln x) = 1/x".to_string(),
                variables: vec![
                    "e = Euler's number".to_string(),
                    "ln = natural logarithm".to_string(),
                ],
            },
        ],
    );

    m
});

impl Formula {
    pub fn detect(query: &str) -> Option<Self> {
        let query = query.trim();
        // Skip if query is shorter than "formula"
        if query.len() < 7 {
            return None;
        }
        let query = query.to_lowercase();
        let caps = FORMULA_RE.captures(&query)?;

        // Get category and formula type from either capture group
        let category = caps
            .name("category")
            .or_else(|| caps.name("category2"))?
            .as_str()
            .trim();

        let formula_type = caps
            .name("formula_type")
            .or_else(|| caps.name("formula_type2"))
            .map(|m| m.as_str().trim());

        let normalized_category = match category {
            "square" | "squares" | "rectangle" | "rectangles" => "square",
            "circle" | "circles" | "circular" => "circle",
            "triangle" | "triangles" | "triangular" => "triangle",
            "cube" | "cubes" | "cubic" => "cube",
            "sphere" | "spheres" | "spherical" => "sphere",
            "cylinder" | "cylinders" | "cylindrical" => "cylinder",
            "prism" | "prisms" | "prismatic" => "prism",
            "electromagnetic" | "electricity" | "electrical" | "em" => "electromagnetic",
            "fluid" | "fluids" | "liquid" | "liquids" | "hydraulic" => "fluid",
            "thermo" | "thermodynamic" | "thermodynamics" | "heat" => "thermo",
            "physics" | "physical" | "mechanics" | "motion" | "movement" | "force" => "physics",
            "calculus" | "calc" | "derivative" | "derivatives" | "integral" | "integrals" => {
                "calculus"
            }
            _ => return None,
        };

        let mut formulas = FORMULAS.get(normalized_category)?.to_vec();

        // Filter formulas if a specific type was requested
        if let Some(formula_type) = formula_type {
            formulas.retain(|f| {
                let name_lower = f.name.to_lowercase();
                match formula_type {
                    "area" | "surface" => name_lower.contains("area"),
                    "volume" => name_lower.contains("volume"),
                    "perimeter" | "circumference" => {
                        name_lower.contains("perimeter") || name_lower.contains("circumference")
                    }
                    "diagonal" => name_lower.contains("diagonal"),
                    "diameter" => name_lower.contains("diameter"),
                    "velocity" | "speed" => name_lower.contains("velocity"),
                    "energy" => name_lower.contains("energy"),
                    "power" => name_lower.contains("power"),
                    "pressure" => name_lower.contains("pressure"),
                    "heat" | "thermal" => name_lower.contains("heat"),
                    "density" => name_lower.contains("density"),
                    "gas" => name_lower.contains("gas"),
                    "ohm" => name_lower.contains("ohm"),
                    "electrical" => name_lower.contains("electrical"),
                    "derivative" | "derivatives" => {
                        name_lower.contains("derivative") || name_lower.contains("rule")
                    }
                    "integral" | "integrals" => name_lower.contains("integral"),
                    "theorem" => name_lower.contains("theorem"),
                    _ => true,
                }
            });

            // Return None if no formulas match the filter
            if formulas.is_empty() {
                return None;
            }
        }

        Some(Formula {
            category: normalized_category.to_string(),
            formulas,
        })
    }
}

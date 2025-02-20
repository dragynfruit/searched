use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;

static GAMES_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?:play\s+)?(?P<game>snake|asteroid(?:\s*lander)?|lunar(?:\s*lander)?)(?:\s+game)?$",
    )
    .unwrap()
});

#[derive(Debug, Serialize)]
pub enum Game {
    Snake {
        grid_size: u32,
        initial_speed: u32,
        growth_rate: u32,
    },
    AsteroidLander {
        gravity: f32,
        thrust_power: f32,
        max_landing_speed: f32,
        max_landing_angle: f32,
    },
}

impl Game {
    pub fn detect(query: &str) -> Option<Self> {
        let caps = GAMES_RE.captures(query)?;
        let game = caps.name("game")?.as_str().to_lowercase();

        match game.as_str() {
            "snake" => Some(Game::Snake {
                grid_size: 20,
                initial_speed: 200,
                growth_rate: 1,
            }),
            "asteroid" | "asteroid lander" | "lunar" | "lunar lander" => {
                Some(Game::AsteroidLander {
                    gravity: 0.1,
                    thrust_power: 0.2,
                    max_landing_speed: 2.0,
                    max_landing_angle: 15.0,
                })
            }
            _ => None,
        }
    }
}

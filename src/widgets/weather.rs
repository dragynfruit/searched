use reqwest::Client;
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use regex::Regex;

static WEATHER_QUERY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(weather|forecast|temperature)\b").unwrap()
});
static WEATHER_IN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)weather\s+in\s+(?P<place>.+)").unwrap()
});
static PLACE_WEATHER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<place>.+)\s+weather$").unwrap()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CurrentUnits {
    time: String,
    interval: String,
    apparent_temperature: String,
    is_day: String,
    precipitation: String,
    weather_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Current {
    time: String,
    interval: i32,
    apparent_temperature: f64,
    is_day: i32,
    precipitation: f64,
    weather_code: i32,
    wind_speed_10m: f64,
    wind_direction_10m: f64,
    relative_humidity_2m: i32,
    temperature_2m: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HourlyUnits {
    time: String,
    temperature_2m: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    current_units: CurrentUnits,
    current: Current,
    hourly_units: HourlyUnits,
    hourly: HourlyData,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedWeather {
    weather: WeatherResponse,
    timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct HourlyForecast {
    pub time: String,
    pub temperature: f64,
}

#[derive(Debug, Serialize)]
pub struct Weather {
    pub location: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: i32,
    pub precipitation: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub weather_code: i32,
    pub is_day: bool,
    pub hourly: Vec<HourlyForecast>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedCoords {
    lat: f64,
    lon: f64,
    timestamp: u64,
}

impl Weather {
    pub async fn detect(query: &str, client: &Client, db: &sled::Db) -> Option<Self> {
        let query = query.trim();
        // First, check for "weather in {place}"
        if let Some(caps) = WEATHER_IN_RE.captures(query) {
            let place = caps.name("place")?.as_str().trim();
            return Self::fetch_weather(place, client, db).await;
        }
        // Also accept "{place} weather"
        if let Some(caps) = PLACE_WEATHER_RE.captures(query) {
            let place = caps.name("place")?.as_str().trim();
            return Self::fetch_weather(place, client, db).await;
        }
        // Fallback: if a generic weather query is detected, use a default place or return dummy data.
        if WEATHER_QUERY_RE.is_match(query) {
            return Some(Weather {
                location: "Default Location".to_string(),
                temperature: 25.0,
                feels_like: 25.0,
                humidity: 50,
                precipitation: 0.0,
                wind_speed: 5.0,
                wind_direction: 180.0,
                weather_code: 0,
                is_day: true,
                hourly: vec![],
                error: None,
            });
        }
        None
    }

    async fn fetch_weather(location: &str, client: &Client, db: &sled::Db) -> Option<Self> {
        // Get coordinates with caching
        let coords = Self::get_coordinates(location, client, db).await?;
        let weather_cache = db.open_tree("weather").ok()?;
        
        let cache_key = format!("{}_{}", coords.0, coords.1);
        
        // Check weather cache first
        if let Ok(Some(cached)) = weather_cache.get(cache_key.as_bytes()) {
            if let Ok(cached_weather) = bincode::deserialize::<CachedWeather>(&cached) {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                // Use cache if less than 1 hour old
                if now - cached_weather.timestamp < 60 * 60 {
                    return Self::build_weather_response(location, &cached_weather.weather);
                }
            }
        }

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,weather_code,wind_speed_10m,wind_direction_10m&hourly=temperature_2m&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch",
            coords.0, coords.1
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(weather) = response.json::<WeatherResponse>().await {
                        // Cache the weather response asynchronously
                        let weather_clone = weather.clone();
                        let cache_key = cache_key.clone();
                        let weather_cache = weather_cache.clone();
                        tokio::spawn(async move {
                            let cached = CachedWeather {
                                weather: weather_clone,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .ok()?
                                    .as_secs(),
                            };
                            if let Ok(encoded) = bincode::serialize(&cached) {
                                let _ = weather_cache.insert(cache_key.as_bytes(), encoded);
                            }
                            Some(())
                        });
                        
                        return Self::build_weather_response(location, &weather);
                    }
                }
                Some(Weather::error_response(location, "Failed to fetch weather data"))
            }
            Err(_) => Some(Weather::error_response(location, "Failed to connect to weather service")),
        }
    }

    fn build_weather_response(location: &str, weather: &WeatherResponse) -> Option<Self> {
        // Get next 24 hours of forecast
        let hourly: Vec<HourlyForecast> = weather
            .hourly
            .time
            .iter()
            .zip(weather.hourly.temperature_2m.iter())
            .take(24)
            .map(|(time, temp)| {
                let hour = time
                    .split('T')
                    .nth(1)?
                    .split(':')
                    .next()?
                    .parse::<i32>()
                    .ok()?;
                Some(HourlyForecast {
                    time: format!("{}:00", hour),
                    temperature: *temp,
                })
            })
            .flatten()
            .collect();

        Some(Weather {
            location: location.to_string(),
            temperature: weather.current.temperature_2m,
            feels_like: weather.current.apparent_temperature,
            humidity: weather.current.relative_humidity_2m,
            precipitation: weather.current.precipitation,
            wind_speed: weather.current.wind_speed_10m,
            wind_direction: weather.current.wind_direction_10m,
            weather_code: weather.current.weather_code,
            is_day: weather.current.is_day == 1,
            hourly,
            error: None,
        })
    }

    async fn get_coordinates(location: &str, client: &Client, db: &sled::Db) -> Option<(f64, f64)> {
        let locations = db.open_tree("locations").ok()?;
        let location_key = location.to_lowercase();

        // Check cache first
        if let Ok(Some(cached)) = locations.get(&location_key) {
            if let Ok(coords) = bincode::deserialize::<CachedCoords>(&cached) {
                // Check if cache is less than 24 hours old
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                if now - coords.timestamp < 24 * 60 * 60 {
                    return Some((coords.lat, coords.lon));
                }
            }
        }

        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
            urlencoding::encode(&location_key)
        );

        #[derive(Deserialize)]
        struct NominatimResponse {
            lat: String,
            lon: String,
        }

        // Fetch fresh coordinates
        match client
            .get(&url)
            .header("User-Agent", "Searched/1.0")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(loc_results) = response.json::<Vec<NominatimResponse>>().await {
                    if let Some(loc) = loc_results.first() {
                        if let (Ok(lat), Ok(lon)) = (loc.lat.parse(), loc.lon.parse()) {
                            // Cache the coordinates asynchronously
                            let locations = locations.clone();
                            let location_key = location_key.clone();
                            tokio::spawn(async move {
                                let cached = CachedCoords {
                                    lat,
                                    lon,
                                    timestamp: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .ok()?
                                        .as_secs(),
                                };
                                if let Ok(encoded) = bincode::serialize(&cached) {
                                    let _ = locations.insert(&location_key, encoded);
                                }
                                Some(())
                            });
                            
                            return Some((lat, lon));
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    fn error_response(location: &str, error: &str) -> Self {
        Weather {
            location: location.to_string(),
            temperature: 0.0,
            feels_like: 0.0,
            humidity: 0,
            precipitation: 0.0,
            wind_speed: 0.0,
            wind_direction: 0.0,
            weather_code: 0,
            is_day: true,
            hourly: vec![],
            error: Some(error.to_string()),
        }
    }
}

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct CurrentUnits {
    time: String,
    interval: String,
    apparent_temperature: String,
    is_day: String,
    precipitation: String,
    weather_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
struct HourlyUnits {
    time: String,
    temperature_2m: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct HourlyData {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WeatherResponse {
    latitude: f64,
    longitude: f64,
    current_units: CurrentUnits,
    current: Current,
    hourly_units: HourlyUnits,
    hourly: HourlyData,
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

impl Weather {
    pub async fn detect_with_client(query: &str, client: &Client) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Match patterns like "weather in <location>" or "<location> weather"
        let location = if let Some(loc) = query.strip_prefix("weather in ") {
            Some(loc.trim())
        } else if let Some(loc) = query.strip_suffix(" weather") {
            Some(loc.trim())
        } else {
            None
        }?;

        Self::fetch_weather(location, client).await
    }

    async fn fetch_weather(location: &str, client: &Client) -> Option<Self> {
        // First, get coordinates using Nominatim
        let coords = Self::get_coordinates(location, client).await?;

        // Then fetch weather data
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,apparent_temperature,is_day,precipitation,weather_code,wind_speed_10m,wind_direction_10m&hourly=temperature_2m&temperature_unit=fahrenheit&wind_speed_unit=mph&precipitation_unit=inch",
            coords.0, coords.1
        );

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(weather) = response.json::<WeatherResponse>().await {
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

                        return Some(Weather {
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
                        });
                    }
                }
                Some(Weather::error_response(
                    location,
                    "Failed to fetch weather data",
                ))
            }
            Err(_) => Some(Weather::error_response(
                location,
                "Failed to connect to weather service",
            )),
        }
    }

    async fn get_coordinates(location: &str, client: &Client) -> Option<(f64, f64)> {
        let url = format!(
            "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
            urlencoding::encode(location)
        );

        #[derive(Deserialize)]
        struct NominatimResponse {
            lat: String,
            lon: String,
        }

        match client
            .get(&url)
            .header("User-Agent", "Searched/1.0")
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(locations) = response.json::<Vec<NominatimResponse>>().await {
                    if let Some(loc) = locations.first() {
                        if let (Ok(lat), Ok(lon)) = (loc.lat.parse(), loc.lon.parse()) {
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

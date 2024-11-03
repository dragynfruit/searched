use std::collections::HashMap;

use axum::{
    body::Body,
    extract::{Query, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

// THIS NEEDS TO BE CLEANED UP ASAP. ALSO THEME DOESN'T WORK FOR GOD KNOWS WHAT REASON ashuioafshpoafsuhaliouhoufdgsjl;hnbadf;ilohmkla
// aspoihdjoaih98yh089y30ha;dpoijas';pioj

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub favicons: bool,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favicons: false,
            theme: "dark".to_string(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_string(&self) -> String {
        self.clone().into()
    }
}

// convert jar into settings struct
impl From<CookieJar> for Settings {
    fn from(jar: CookieJar) -> Self {
        let defaults = Settings::default();

        let favicons = jar
            .get_string("favicons")
            .map(|c| c == "true")
            .unwrap_or(defaults.favicons);
        let theme = jar.get_string("theme").unwrap_or(defaults.theme);

        Self { favicons, theme }
    }
}

// convert settings into a cookie string that never expires
impl From<Settings> for String {
    fn from(settings: Settings) -> Self {
        format!(
            "favicons={}; theme={}; Max-Age=31536000; SameSite=Strict; Path=/",
            settings.favicons, settings.theme
        )
    }
}

trait BetterCookieJar {
    fn get_string(&self, key: &str) -> Option<String>;
}

impl BetterCookieJar for CookieJar {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).map(|c| c.value().to_string())
    }
}

pub async fn settings_middleware(jar: CookieJar, mut request: Request, next: Next) -> Response {
    let settings = Settings::from(jar);
    request.extensions_mut().insert(settings);
    next.run(request).await
}

pub async fn update_settings(Query(settings): Query<HashMap<String, String>>) -> impl IntoResponse {
    let settings = Settings {
        favicons: settings.get("favicons").map(|c| c == "true").unwrap_or(false),
        theme: settings.get("theme").unwrap_or(&"dark".to_string()).to_string(),
    };

    Response::builder()
        .status(302)
        .header("Location", "/")
        .header("Set-Cookie", settings.to_string())
        .body(Body::empty())
        .unwrap()
}

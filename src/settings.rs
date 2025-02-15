use std::collections::HashMap;
use axum::{
    body::Body,
    extract::{Query, Request, Json, Extension},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub favicons: bool,
    pub theme: String,
    pub show_query_title: bool,
    pub compact_view: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favicons: true,
            theme: "auto".to_string(),
            show_query_title: true,
            compact_view: false,
        }
    }
}

impl Settings {
    pub fn new() -> SettingsBuilder {
        SettingsBuilder::default()
    }

    pub fn to_cookies(&self) -> String {
        // Serialize settings to JSON and base64-encode to make it cookie-safe.
        let json = serde_json::to_string(self).unwrap();
        let encoded = general_purpose::STANDARD.encode(json);
        format!("settings={}; Path=/; Max-Age=31536000; SameSite=Strict; Secure", encoded)
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    favicons: Option<bool>,
    theme: Option<String>,
    show_query_title: Option<bool>,
    compact_view: Option<bool>,
}

impl SettingsBuilder {
    pub fn favicons(mut self, favicons: bool) -> Self {
        self.favicons = Some(favicons);
        self
    }

    pub fn theme<S: Into<String>>(mut self, theme: S) -> Self {
        self.theme = Some(theme.into());
        self
    }

    pub fn show_query_title(mut self, show_query_title: bool) -> Self {
        self.show_query_title = Some(show_query_title);
        self
    }

    pub fn compact_view(mut self, compact_view: bool) -> Self {
        self.compact_view = Some(compact_view);
        self
    }

    pub fn build(self) -> Settings {
        let defaults = Settings::default();
        Settings {
            favicons: self.favicons.unwrap_or(defaults.favicons),
            theme: self.theme.unwrap_or(defaults.theme),
            show_query_title: self.show_query_title.unwrap_or(defaults.show_query_title),
            compact_view: self.compact_view.unwrap_or(defaults.compact_view),
        }
    }
}

impl From<CookieJar> for Settings {
    fn from(jar: CookieJar) -> Self {
        if let Some(cookie) = jar.get("settings") {
            if let Ok(decoded) = general_purpose::STANDARD.decode(cookie.value()) {
                if let Ok(settings) = serde_json::from_slice::<Settings>(&decoded) {
                    return settings;
                }
            }
        }
        Settings::default()
    }
}

pub async fn settings_middleware(jar: CookieJar, mut request: Request, next: Next) -> Response {
    let settings = Settings::from(jar);
    request.extensions_mut().insert(settings);
    next.run(request).await
}

pub async fn update_settings(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let defaults = Settings::default();
    let settings = Settings::new()
        .favicons(params.get("favicons").map(|v| v == "true").unwrap_or(defaults.favicons))
        .theme(params.get("theme")
            .map(|t| t.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or(defaults.theme))
        .show_query_title(params.get("show_query_title").map(|v| v == "true").unwrap_or(defaults.show_query_title))
        .compact_view(params.get("compact_view").map(|v| v == "true").unwrap_or(defaults.compact_view))
        .build();

    let cookie = settings.to_cookies();
    Response::builder()
        .status(302)
        .header("Location", "/settings")
        .header("Set-Cookie", cookie)
        .body(Body::empty())
        .unwrap()
}

// New endpoint: Export settings as JSON.
pub async fn export_settings(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    // Return settings as JSON.
    Json(settings)
}

// New endpoint: Import settings from JSON payload.
pub async fn import_settings(Json(new_settings): Json<Settings>) -> impl IntoResponse {
    let cookie = new_settings.to_cookies();
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/settings")
        .header("Set-Cookie", cookie)
        .body(Body::empty())
        .unwrap()
}

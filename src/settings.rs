use std::collections::HashMap;
use axum::{
    body::Body,
    extract::{Query, Request, Json, Extension, Multipart, Form},
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
    pub no_js: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favicons: true,
            theme: "auto".to_string(),
            show_query_title: true,
            compact_view: false,
            no_js: false,
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
        // Removed the Secure flag for local testing.
        format!("settings={}; Path=/; Max-Age=31536000; SameSite=Strict", encoded)
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    favicons: Option<bool>,
    theme: Option<String>,
    show_query_title: Option<bool>,
    compact_view: Option<bool>,
    no_js: Option<bool>,
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

    pub fn no_js(mut self, no_js: bool) -> Self {
        self.no_js = Some(no_js);
        self
    }

    pub fn build(self) -> Settings {
        let defaults = Settings::default();
        Settings {
            favicons: self.favicons.unwrap_or(defaults.favicons),
            theme: self.theme.unwrap_or(defaults.theme),
            show_query_title: self.show_query_title.unwrap_or(defaults.show_query_title),
            compact_view: self.compact_view.unwrap_or(defaults.compact_view),
            no_js: self.no_js.unwrap_or(defaults.no_js),
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

pub async fn update_settings(Form(params): Form<HashMap<String, String>>) -> impl IntoResponse {
    if params.contains_key("reset") {
        let settings = Settings::default();
        let cookie = settings.to_cookies();
        return Response::builder()
            .status(302)
            .header("Location", "/settings")
            .header("Set-Cookie", cookie)
            .body(Body::empty())
            .unwrap();
    }
    let defaults = Settings::default();
    let settings = Settings::new()
        .favicons(params.get("favicons").map(|v| v == "true").unwrap_or(defaults.favicons))
        .theme(params.get("theme")
            .map(|t| t.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or(defaults.theme))
        .show_query_title(params.get("show_query_title").map(|v| v == "true").unwrap_or(defaults.show_query_title))
        .compact_view(params.get("compact_view").map(|v| v == "true").unwrap_or(defaults.compact_view))
        .no_js(params.get("no_js").map(|v| v == "true").unwrap_or(defaults.no_js))
        .build();

    let cookie = settings.to_cookies();
    Response::builder()
        .status(302)
        .header("Location", "/settings")
        .header("Set-Cookie", cookie)
        .body(Body::empty())
        .unwrap()
}

// Modified export_settings endpoint to allow download via query parameter.
pub async fn export_settings(
    Extension(settings): Extension<Settings>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    if params.contains_key("download") {
        let json = serde_json::to_string_pretty(&settings).unwrap();
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Content-Disposition", "attachment; filename=\"settings.json\"")
            .body(Body::from(json))
            .unwrap()
    } else {
        Json(settings).into_response()
    }
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

// New endpoint: Import settings from multipart form submission (non-JS).
pub async fn import_settings_form(
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("settings_file") {
            let data = field.bytes().await.unwrap();
            if let Ok(new_settings) = serde_json::from_slice::<Settings>(&data) {
                let cookie = new_settings.to_cookies();
                return Response::builder()
                    .status(StatusCode::FOUND)
                    .header("Location", "/settings")
                    .header("Set-Cookie", cookie)
                    .body(Body::empty())
                    .unwrap();
            } else {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::from("Invalid settings file"))
                    .unwrap();
            }
        }
    }
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::from("No file provided"))
        .unwrap()
}

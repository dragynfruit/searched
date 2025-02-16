use axum::{
    body::Body,
    extract::{Extension, Form, Json, Multipart, Query, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use base64::{engine::general_purpose, Engine as _};
use searched::SafeSearch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub favicons: bool,
    pub theme: String,
    pub show_query_title: bool,
    pub compact_view: bool,
    pub no_js: bool,
    pub remove_tracking: bool,
    pub bold_terms: bool,
    pub safesearch: SafeSearch,
    pub enable_widgets: bool,
    pub show_full_path: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favicons: true,
            theme: "auto".to_string(),
            show_query_title: true,
            compact_view: false,
            no_js: false,
            remove_tracking: true,
            bold_terms: true,
            safesearch: SafeSearch::default(),
            enable_widgets: true,
            show_full_path: false,
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
        format!(
            "settings={}; Path=/; Max-Age=31536000; SameSite=Strict; HttpOnly",
            encoded
        )
    }

    pub fn merge_from_json(json_value: serde_json::Value) -> Self {
        let defaults = Settings::default();

        // Try to get each field, fall back to default if missing or wrong type
        Settings {
            favicons: json_value
                .get("favicons")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.favicons),
            theme: json_value
                .get("theme")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or(defaults.theme),
            show_query_title: json_value
                .get("show_query_title")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.show_query_title),
            compact_view: json_value
                .get("compact_view")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.compact_view),
            no_js: json_value
                .get("no_js")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.no_js),
            remove_tracking: json_value
                .get("remove_tracking")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.remove_tracking),
            bold_terms: json_value
                .get("bold_terms")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.bold_terms),
            safesearch: json_value
                .get("safesearch")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(defaults.safesearch),
            enable_widgets: json_value
                .get("enable_widgets")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.enable_widgets),
            show_full_path: json_value
                .get("show_full_path")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.show_full_path),
        }
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    favicons: Option<bool>,
    theme: Option<String>,
    show_query_title: Option<bool>,
    compact_view: Option<bool>,
    no_js: Option<bool>,
    remove_tracking: Option<bool>,
    bold_terms: Option<bool>,
    safesearch: Option<SafeSearch>,
    enable_widgets: Option<bool>,
    show_full_path: Option<bool>,
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

    pub fn remove_tracking(mut self, remove_tracking: bool) -> Self {
        self.remove_tracking = Some(remove_tracking);
        self
    }

    pub fn bold_terms(mut self, bold_terms: bool) -> Self {
        self.bold_terms = Some(bold_terms);
        self
    }

    pub fn safesearch(mut self, safesearch: SafeSearch) -> Self {
        self.safesearch = Some(safesearch);
        self
    }

    pub fn enable_widgets(mut self, enable_widgets: bool) -> Self {
        self.enable_widgets = Some(enable_widgets);
        self
    }

    pub fn show_full_path(mut self, show_full_path: bool) -> Self {
        self.show_full_path = Some(show_full_path);
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
            remove_tracking: self.remove_tracking.unwrap_or(defaults.remove_tracking),
            bold_terms: self.bold_terms.unwrap_or(defaults.bold_terms),
            safesearch: self.safesearch.unwrap_or(defaults.safesearch),
            enable_widgets: self.enable_widgets.unwrap_or(defaults.enable_widgets),
            show_full_path: self.show_full_path.unwrap_or(defaults.show_full_path),
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
        .favicons(
            params
                .get("favicons")
                .map(|v| v == "true")
                .unwrap_or(defaults.favicons),
        )
        .theme(
            params
                .get("theme")
                .map(|t| t.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or(defaults.theme),
        )
        .show_query_title(
            params
                .get("show_query_title")
                .map(|v| v == "true")
                .unwrap_or(defaults.show_query_title),
        )
        .compact_view(
            params
                .get("compact_view")
                .map(|v| v == "true")
                .unwrap_or(defaults.compact_view),
        )
        .no_js(
            params
                .get("no_js")
                .map(|v| v == "true")
                .unwrap_or(defaults.no_js),
        )
        .remove_tracking(
            params
                .get("remove_tracking")
                .map(|v| v == "true")
                .unwrap_or(defaults.remove_tracking),
        )
        .bold_terms(
            params
                .get("bold_terms")
                .map(|v| v == "true")
                .unwrap_or(defaults.bold_terms),
        )
        .safesearch(
            params
                .get("safesearch")
                .and_then(|s| s.parse().ok())
                .unwrap_or(defaults.safesearch),
        )
        .enable_widgets(
            params
                .get("enable_widgets")
                .map(|v| v == "true")
                .unwrap_or(defaults.enable_widgets),
        )
        .show_full_path(
            params
                .get("show_full_path")
                .map(|v| v == "true")
                .unwrap_or(defaults.show_full_path),
        )
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
            .header(
                "Content-Disposition",
                "attachment; filename=\"settings.json\"",
            )
            .body(Body::from(json))
            .unwrap()
    } else {
        Json(settings).into_response()
    }
}

// New endpoint: Import settings from JSON payload.
pub async fn import_settings(Json(json_value): Json<serde_json::Value>) -> impl IntoResponse {
    let new_settings = Settings::merge_from_json(json_value);
    let cookie = new_settings.to_cookies();
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/settings")
        .header("Set-Cookie", cookie)
        .body(Body::empty())
        .unwrap()
}

// New endpoint: Import settings from multipart form submission (non-JS).
pub async fn import_settings_form(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("settings_file") {
            let data = field.bytes().await.unwrap();
            if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&data) {
                let new_settings = Settings::merge_from_json(json_value);
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

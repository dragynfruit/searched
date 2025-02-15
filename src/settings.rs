use std::collections::HashMap;
use axum::{
    body::Body,
    extract::{Query, Request},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

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

    pub fn to_cookies(&self) -> Vec<String> {
        vec![
            format!("favicons={}; Path=/; Max-Age=31536000; SameSite=Strict; Secure", self.favicons),
            format!("theme={}; Path=/; Max-Age=31536000; SameSite=Strict; Secure", self.theme),
            format!("show_query_title={}; Path=/; Max-Age=31536000; SameSite=Strict; Secure", self.show_query_title),
            format!("compact_view={}; Path=/; Max-Age=31536000; SameSite=Strict; Secure", self.compact_view),
        ]
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
        let defaults = Settings::default();
        Settings::new()
            .favicons(jar.get("favicons")
                .map(|c| c.value() == "true")
                .unwrap_or(defaults.favicons))
            .theme(jar.get("theme")
                .map(|c| c.value().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or(defaults.theme))
            .show_query_title(jar.get("show_query_title")
                .map(|c| c.value() == "true")
                .unwrap_or(defaults.show_query_title))
            .compact_view(jar.get("compact_view")
                .map(|c| c.value() == "true")
                .unwrap_or(defaults.compact_view))
            .build()
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

    let mut response = Response::builder()
        .status(302)
        .header("Location", "/settings");

    // Add each cookie as a separate header
    for cookie in settings.to_cookies() {
        response = response.header("Set-Cookie", cookie);
    }

    response.body(Body::empty()).unwrap()
}

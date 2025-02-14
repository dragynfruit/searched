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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            favicons: true,
            theme: "auto".to_string(),
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
        ]
    }
}

#[derive(Default)]
pub struct SettingsBuilder {
    favicons: Option<bool>,
    theme: Option<String>,
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

    pub fn build(self) -> Settings {
        let defaults = Settings::default();
        Settings {
            favicons: self.favicons.unwrap_or(defaults.favicons),
            theme: self.theme.unwrap_or(defaults.theme),
        }
    }
}

impl From<CookieJar> for Settings {
    fn from(jar: CookieJar) -> Self {
        Settings::new()
            .favicons(jar.get("favicons").map(|c| c.value() == "true").unwrap_or_default())
            .theme(jar.get("theme").map(|c| c.value().to_string()).unwrap_or_default())
            .build()
    }
}

pub async fn settings_middleware(jar: CookieJar, mut request: Request, next: Next) -> Response {
    let settings = Settings::from(jar);
    request.extensions_mut().insert(settings);
    next.run(request).await
}

pub async fn update_settings(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let settings = Settings::new()
        .favicons(params.get("favicons").map(|v| v == "true").unwrap_or_default())
        .theme(params.get("theme").map(|t| t.to_string()).unwrap_or_default())
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

#[macro_use]
extern crate log;
extern crate mlua;
extern crate tokio;
#[macro_use]
extern crate serde;
extern crate searched_parser;

pub mod config;
pub mod lua_support;
mod error;

pub use error::Error;

use std::collections::HashMap;
use std::str::FromStr;

use config::ProvidersConfig;
use mlua::{FromLua, IntoLua, LuaSerdeExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

pub static PROVIDER_KINDS: Lazy<HashMap<String, Vec<Kind>>> = Lazy::new(|| {
    HashMap::from_iter(
        ProvidersConfig::load("plugins/providers.toml")
            .0
            .into_iter()
            .map(|(k, v)| (k, v.kinds)),
    )
});

gen_enum! {
    Kind (Kind::General) {
        General        = "sear",
        Images         = "imgs",
        Videos         = "vids",
        News           = "news",
        Maps           = "maps",
        Wiki           = "wiki",
        QuestionAnswer = "qans",
        Documentation  = "docs",
        Papers         = "pprs",
    }

    SafeSearch (SafeSearch::Moderate) {
        Off      = "off",
        Moderate = "moderate",
        Strict   = "strict",
    }
}

impl IntoLua for Kind {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.to_value(&self)
    }
}
impl FromLua for Kind {
    fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl FromStr for SafeSearch {
    type Err = ();
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(SafeSearch::Off),
            "moderate" => Ok(SafeSearch::Moderate),
            "strict" => Ok(SafeSearch::Strict),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for SafeSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafeSearch::Off => write!(f, "off"),
            SafeSearch::Moderate => write!(f, "moderate"),
            SafeSearch::Strict => write!(f, "strict"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename(deserialize = "q"))]
    pub query: String,
    #[serde(rename(deserialize = "k"))]
    pub kind: Kind,
    #[serde(rename(deserialize = "p"))]
    pub page: usize,
    #[serde(rename(deserialize = "s"), default)]
    pub safe: SafeSearch,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub general: Option<GeneralResult>,
    pub forum: Option<ForumResult>,
    pub image: Option<ImageResult>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GeneralResult {
    pub snippet: Option<String>,
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ForumResult {
    pub poster_image: Option<String>,
    pub poster_username: String,
    pub poster_url: Option<String>,
    pub tags: Option<Vec<String>>,
}
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ImageResult {
    pub preview_url: String,
    pub full_size_url: String,
}


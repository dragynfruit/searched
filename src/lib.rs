#[macro_use]
extern crate log;
extern crate mlua;
extern crate tokio;
#[macro_use]
extern crate serde;
extern crate searched_parser;

pub mod config;
pub mod lua_api;

use std::collections::HashMap;

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

    SafeSearch (SafeSearch::On) {
        Off    = "off",
        On     = "on",
        Strict = "strict",
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

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "pr")]
    pub provider: String,
    #[serde(rename = "q")]
    pub query: String,
    #[serde(rename = "k")]
    pub kind: Kind,
    #[serde(rename = "p")]
    pub page: usize,
    #[serde(rename = "s", default)]
    pub safe: SafeSearch,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Result {
    pub url: String,
    pub title: String,
    pub general: Option<GeneralResult>,
    pub forum: Option<ForumResult>,
    pub image: Option<ImageResult>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct GeneralResult {
    pub snippet: String,
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

#[macro_use]
extern crate mlua;

pub mod lua_api;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub enum Kind {
    #[serde(rename = "sear")]
    General,
    #[serde(rename = "imgs")]
    Images,
    #[serde(rename = "vids")]
    Videos,
    #[serde(rename = "news")]
    News,
    #[serde(rename = "maps")]
    Maps,
    #[serde(rename = "docs")]
    Documentation,
    #[serde(rename = "pprs")]
    Papers,
}
impl Default for Kind {
    fn default() -> Self {
        Self::General
    }
}

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize)]
pub struct Query {
    #[serde(rename = "q")]
    pub query: String,
    #[serde(rename = "k")]
    pub kind: Kind,
    #[serde(rename = "p")]
    pub page: usize,
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
pub struct ImageResult {}

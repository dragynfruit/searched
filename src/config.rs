use std::{collections::HashMap, fs::File, io::Read, path::Path};

use crate::Kind;

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Config {
    #[serde(rename = "provider")]
    pub providers: HashMap<String, CfgProvider>,
}
impl Config {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        toml::from_str(&buf).unwrap()
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct CfgProvider {
    /// Underlying engine
    ///
    /// Uses provider name if unset
    pub engine: Option<String>,
    /// Formatting string for URL
    pub url: Option<String>,
    /// Human readable name
    pub name: String,
    pub description: String,
    pub kinds: Vec<Kind>,
}

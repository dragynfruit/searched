use std::{collections::HashMap, fs::File, io::Read, path::Path};

use crate::Kind;

macro_rules! gen_enum {
    ( $(
        $ident:ident $( ( $default:expr ) )? {
            $(
                $var:ident = $string:literal,
            )*
        }
    )* ) => {
        $(
        #[derive(Deserialize, Serialize, Clone, Debug)]
        pub enum $ident {
            $(
            #[serde(rename = $string)]
            $var,
            )*
        }
        $(
        impl Default for $ident {
            fn default() -> Self {
                $default
            }
        }
        )?
        )*
    };
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Config {
    /// Address to listen on
    pub listen_addr: Option<String>,
}
impl Config {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let mut buf = String::new();
        File::open(path).unwrap().read_to_string(&mut buf).unwrap();
        toml::from_str(&buf).unwrap()
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct ProvidersConfig(pub HashMap<String, CfgProvider>);
impl ProvidersConfig {
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
    pub features: Option<CfgProviderFeatures>,
}

gen_enum! {
    CfgSafeSearchSupport ( CfgSafeSearchSupport::No ) {
        No = "no",
        Yes = "yes",
        MultiLevel = "multilevel",
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct CfgProviderFeatures {
    pub safe_search: CfgSafeSearchSupport,
}

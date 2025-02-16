use core::error::Error;
use std::{
    fs::{read_dir, File},
    io::Read,
    time::Instant,
};

use mlua::prelude::*;
use reqwest::Client;

use super::api::*;
use crate::{config::ProvidersConfig, Query};

/// A single-threaded plugin engine
#[derive(Clone)]
pub struct PluginEngine {
    lua: Lua,
    client: Client,
    #[cfg(not(feature = "hot_reload"))]
    providers: ProvidersConfig,
}
impl PluginEngine {
    /// Initialize a new engine for running plugins
    pub async fn new(client: Client) -> Result<Self, Box<dyn Error>> {
        #[cfg(not(feature = "hot_reload"))]
        let providers = ProvidersConfig::load("plugins/providers.toml");

        debug!("initializing plugin engine...");

        let lua = Lua::new();

        // Add Lua global variables we need
        lua.globals()
            .set("__searched_engines__", lua.create_table()?)?;

        // Add Lua interfaces
        lua.globals()
            .set("Url", lua.create_proxy::<UrlWrapper>()?)?;
        lua.globals().set("Query", lua.create_proxy::<Query>()?)?;
        lua.globals()
            .set("Client", lua.create_proxy::<ClientWrapper>()?)?;
        lua.globals()
            .set("Scraper", lua.create_proxy::<Scraper>()?)?;
        lua.globals()
            .set("Element", lua.create_proxy::<ElementWrapper>()?)?;

        // Add standalone Lua functions
        lua.globals()
            .set("add_engine", lua.create_function(add_engine)?)?;
        lua.globals()
            .set("stringify_params", lua.create_function(stringify_params)?)?;
        lua.globals()
            .set("parse_json", lua.create_function(parse_json)?)?;
        lua.globals()
            .set("fend_eval", lua.create_function(fend_eval)?)?;

        debug!("Initialized plugin engine! loading engines...");

        // Load engines
        Self::load_engines(&lua).await;

        debug!("loaded engines!");

        Ok(Self {
            lua,
            client,
            #[cfg(not(feature = "hot_reload"))]
            providers,
        })
    }

    pub async fn load_engines(lua: &Lua) {
        for path in read_dir("plugins/engines").unwrap() {
            if let Ok(path) = path {
                // Do war crime level code
                let name = path
                    .path()
                    .file_stem()
                    .expect("bad file path")
                    .to_str()
                    .expect("filename should be utf-8")
                    .to_string();

                debug!("loading {name}...");
                let load_st = Instant::now();

                // Read the source code into buf
                let mut buf = String::new();
                let mut f = File::open(path.path()).unwrap();
                f.read_to_string(&mut buf).unwrap();

                lua.load(&buf).exec_async().await.unwrap();

                debug!("loaded {name} in {:?}!", load_st.elapsed());
            }
        }
    }

    /// Process the given query
    pub async fn search(&self, query: Query) -> Vec<crate::SearchResult> {
        #[cfg(feature = "hot_reload")]
        Self::load_engines(&self.lua).await;

        #[cfg(feature = "hot_reload")]
        let providers = ProvidersConfig::load("plugins/providers.toml");
        #[cfg(not(feature = "hot_reload"))]
        let providers = &self.providers;

        if let Some(provider) = providers.0.get(&query.provider) {
            let engine = provider
                .engine
                .clone()
                .unwrap_or_else(|| query.provider.clone());
            let target = format!("searched::engine::{engine}");

            // Get engine implementation
            let eng_impl = self
                .lua
                .globals()
                .get::<LuaTable>("__searched_engines__")
                .unwrap()
                .get::<LuaFunction>(engine)
                .unwrap();

            // Run engine for query
            let results = eng_impl
                .call_async::<Vec<LuaTable>>((
                    ClientWrapper(self.client.clone()),
                    query.clone(),
                    self.lua
                        .to_value(&provider.clone().extra.unwrap_or_default()),
                ))
                .await;

            match results {
                Ok(results) => {
                    return results
                        .into_iter()
                        .map(|r| self.lua.from_value(LuaValue::Table(r)).unwrap())
                        .collect();
                }
                Err(err) => {
                    error!(target: &target, "failed to get results from provider {}: {}", query.provider, err);
                }
            }
        }

        Vec::new()
    }
}

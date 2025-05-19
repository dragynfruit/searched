use std::{
    collections::HashMap,
    fs::{File, read_dir},
    io::Read,
    time::Instant,
};

use mlua::prelude::*;
use reqwest::Client;
use tokio::task::JoinSet;

use super::api::*;
use crate::{Error, Query, SearchResult, config::ProvidersConfig, settings::Settings};

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
    pub async fn new(client: Client) -> Result<Self, Box<dyn core::error::Error>> {
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
            .set("HtmlDocument", lua.create_proxy::<Scraper>()?)?;
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

    pub async fn search_multi(
        &self,
        query: Query,
        providers: Vec<String>,
    ) -> Result<Vec<crate::SearchResult>, Error> {
        let mut set = JoinSet::new();

        for provider in providers {
            let eng = self.clone();
            let query = query.clone();
            let provider = provider.clone();

            set.spawn(async move {
                match Self::search(&eng, query, &provider).await {
                    Ok(res) => (res, provider),
                    Err(_) => (Vec::new(), provider),
                }
            });
        }

        let mut results: HashMap<SearchResult, Vec<String>> = HashMap::new();

        let result_batches = set.join_all().await;

        for (batch, provider) in result_batches {
            for res in batch {
                if let Some(providers) = results.get_mut(&res) {
                    providers.push(provider.clone());
                } else {
                    results.insert(res, vec![provider.clone()]);
                }
            }
        }

        let mut results = results.into_iter().collect::<Vec<_>>();
        results.sort_by_key(|r| r.1.len());
        results.reverse();

        let mut results = results.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>();
        results.dedup_by_key(|r| r.url.clone());

        Ok(results)
    }

    /// Process the given query
    pub async fn search(
        &self,
        query: Query,
        provider: impl Into<String>,
    ) -> Result<Vec<crate::SearchResult>, Error> {
        let provider = provider.into();

        #[cfg(feature = "hot_reload")]
        Self::load_engines(&self.lua).await;

        #[cfg(feature = "hot_reload")]
        let providers = ProvidersConfig::load("plugins/providers.toml");
        #[cfg(not(feature = "hot_reload"))]
        let providers = &self.providers;

        if let Some(p) = providers.0.get(&provider) {
            // If the provider has an engine specified, use that engine.
            // Otherwise, the provider is also an engine
            let engine = p.engine.clone().unwrap_or_else(|| provider.clone());
            let target = format!("searched::engine::{engine}");

            // Get engine implementation
            let eng_impl = self
                .lua
                .globals()
                .get::<LuaTable>("__searched_engines__")
                .unwrap()
                .get::<LuaFunction>(engine)
                .map_err(|_| Error::EngineNotLoaded)?;

            // Run engine for query
            let results = eng_impl
                .call_async::<Vec<LuaTable>>((
                    ClientWrapper(self.client.clone()),
                    query.clone(),
                    self.lua.to_value(&p.clone().extra.unwrap_or_default()),
                ))
                .await;

            match results {
                Ok(results) => {
                    return Ok(results
                        .into_iter()
                        .map(|r| self.lua.from_value(LuaValue::Table(r)).unwrap())
                        .collect());
                }
                Err(err) => {
                    error!(target: &target, "failed to get results from provider {provider}: {err}");
                }
            }
        }

        Ok(Vec::new())
    }
}

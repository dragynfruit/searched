use std::{
    collections::HashMap,
    fs::{File, read_dir},
    io::Read,
    time::Instant,
};

use mlua::prelude::*;
use reqwest::Client;
use tokio::task::JoinSet;
use url::Url;

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
        lua.globals()
            .set("__searched_mergers__", lua.create_table()?)?;
        lua.globals()
            .set("__searched_rankers__", lua.create_table()?)?;

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
            .set("add_merger", lua.create_function(add_merger)?)?;
        lua.globals()
            .set("add_ranker", lua.create_function(add_ranker)?)?;
        lua.globals()
            .set("stringify_params", lua.create_function(stringify_params)?)?;
        lua.globals()
            .set("parse_json", lua.create_function(parse_json)?)?;
        lua.globals()
            .set("fend_eval", lua.create_function(fend_eval)?)?;

        debug!("Initialized plugin engine! loading engines...");

        // Load engines
        Self::load_plugins(&lua).await;

        debug!("loaded engines!");

        Ok(Self {
            lua,
            client,
            #[cfg(not(feature = "hot_reload"))]
            providers,
        })
    }

    pub async fn load_plugins(lua: &Lua) {
        for plugin_kind in ["engines", "ranking"] {
        for path in read_dir(&format!("plugins/{plugin_kind}")).unwrap() {
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
    }

    pub async fn search(&self, query: Query, providers: Vec<String>) -> Result<Vec<SearchResult>, Error> {
        let results = self.search_multi(query.clone(), providers).await?;
        let merged = self.merge("multiprovider".to_owned(), query.clone(), results).await?;
        let ranked = self.rank("multiprovider".to_owned(), query, merged).await?;

        Ok(ranked)
    }

    async fn merge(&self, merger: String, query: Query, results: Vec<SearchResult>) -> Result<Vec<SearchResult>, Error> {
        let merger_impl = self.lua.globals().get::<LuaTable>("__searched_mergers__").unwrap().get::<LuaFunction>(merger).unwrap();

        let results = merger_impl
            .call_async::<Vec<LuaTable>>((
                self.lua.to_value(&results).unwrap_or(LuaValue::Nil),
                self.lua.create_table().unwrap(),
            ))
            .await;

        match results {
            Ok(results) => {
                return Ok(results
                    .into_iter()
                    .filter_map(|r| {
                        let res: SearchResult = self.lua.from_value(LuaValue::Table(r)).unwrap();
                        if res.providers.is_empty() {
                            None
                        } else {
                            Some(res)
                        }
                    })
                    .collect());
            }
            Err(_) => {}
        }

        Ok(Vec::new())
    }

    async fn rank(&self, ranker: String, query: Query, results: Vec<SearchResult>) -> Result<Vec<SearchResult>, Error> {
        let ranker_impl = self.lua.globals().get::<LuaTable>("__searched_rankers__").unwrap().get::<LuaFunction>(ranker).unwrap();

        let weights = ranker_impl
            .call_async::<Vec<LuaNumber>>((
                self.lua.to_value(&results).unwrap_or(LuaValue::Nil),
                self.lua.create_table().unwrap(),
            ))
            .await;

        match weights {
            Ok(weights) => {
                let mut res_weights = results
                    .into_iter()
                    .zip(weights)
                    .collect::<Vec<_>>();
                res_weights.sort_by_key(|r| (r.1 * 100.0) as u16);
                res_weights.reverse();
                return Ok(res_weights
                    .into_iter()
                    .unzip::<SearchResult, f64, Vec<_>, Vec<_>>()
                    .0);
            }
            Err(_) => {}
        }

        Ok(Vec::new())
    }

    async fn search_multi(
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
                match Self::search_single(&eng, query, &provider).await {
                    Ok(res) => res,
                    Err(_) => Vec::new(),
                }
            });
        }

        //let mut results: HashMap<Url, Vec<SearchResult>> = HashMap::new();

        let result_batches = set.join_all().await;

        //for (batch, provider) in result_batches {
        //    for res in batch {
        //        if let Some(results) = results.get_mut(&res.url) {
        //            results.push(res);
        //        } else {
        //            results.insert(res.url, vec![res.clone()]);
        //        }
        //    }
        //}

        //let mut results = results.into_iter().collect::<Vec<_>>();
        //results.sort_by_key(|r| r.1.len());
        //results.reverse();

        //let mut results = results.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>();
        //results.dedup_by_key(|r| r.url.clone());

        Ok(result_batches.concat())
    }

    /// Process the given query
    async fn search_single(
        &self,
        query: Query,
        provider: impl Into<String>,
    ) -> Result<Vec<crate::SearchResult>, Error> {
        let provider = provider.into();

        #[cfg(feature = "hot_reload")]
        Self::load_plugins(&self.lua).await;

        #[cfg(feature = "hot_reload")]
        let providers = ProvidersConfig::load("plugins/providers.toml");
        #[cfg(not(feature = "hot_reload"))]
        let providers = &self.providers;

        if let Some(p) = providers.0.get(&provider) {
            let providers = vec![provider.clone()];

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
                        .map(|r| {
                            let mut result: SearchResult = self.lua.from_value(LuaValue::Table(r)).unwrap();
                            result.providers = providers.clone();
                            result
                        })
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

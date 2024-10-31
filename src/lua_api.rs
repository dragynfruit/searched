use core::error;
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fs::{read_dir, File},
    io::Read,
    sync::Arc,
    time::{Duration, Instant},
};

use fend_core::Context;
use mlua::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use scraper::{node::Element, Html, Selector};
use tokio::{
    sync::{oneshot, watch, Mutex},
    task::{spawn_local, LocalSet},
};
use url::Url;

use crate::{config::ProvidersConfig, Query};

impl LuaUserData for Query {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("query", |_, this| Ok(this.query.clone()));
        fields.add_field_method_get("page", |_, this| Ok(this.page));
    }
}

/// Lua wrapper for [url::Url]
struct UrlWrapper(Url);
impl UrlWrapper {
    fn parse(_: &Lua, url: String) -> LuaResult<Self> {
        Url::parse(&url).map(|x| UrlWrapper(x)).into_lua_err()
    }
    fn parse_with_params(_: &Lua, (url, params): (String, LuaTable)) -> LuaResult<Self> {
        Url::parse_with_params(
            &url,
            params
                .pairs::<String, String>()
                .map(|x| x.unwrap())
                .collect::<Vec<(String, String)>>(),
        )
        .map(|x| UrlWrapper(x))
        .into_lua_err()
    }
    fn from_template(_: &Lua, (template, values): (String, LuaTable)) -> LuaResult<Self> {
        let values = HashMap::from_iter(
            values
                .pairs::<String, String>()
                .map(|x| x.unwrap())
                .map(|(k, v)| (k, v)),
        );
        Url::parse(&searched_parser::Url::parse(template.as_bytes()).build(values))
            .map(|x| UrlWrapper(x))
            .into_lua_err()
    }
    fn domain(lua: &Lua, this: &Self, _: ()) -> LuaResult<LuaValue> {
        Ok(if let Some(domain) = this.0.domain() {
            domain.to_string().into_lua(lua)?
        } else {
            LuaValue::Nil
        })
    }
    fn authority(_: &Lua, this: &Self, _: ()) -> LuaResult<String> {
        Ok(this.0.authority().to_string())
    }
    fn path(_: &Lua, this: &Self, _: ()) -> LuaResult<String> {
        Ok(this.0.path().to_string())
    }
    fn path_segments(lua: &Lua, this: &Self, _: ()) -> LuaResult<LuaValue> {
        Ok(if let Some(path_segments) = this.0.path_segments() {
            path_segments.collect::<Vec<&str>>().into_lua(lua)?
        } else {
            LuaValue::Nil
        })
    }
}
impl LuaUserData for UrlWrapper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("from_template", Self::from_template);
        methods.add_function("parse", Self::parse);
        methods.add_function("parse_with_params", Self::parse_with_params);
        methods.add_method("domain", Self::domain);
        methods.add_method("authority", Self::authority);
        methods.add_method("path", Self::path);
        methods.add_method("path_segments", Self::path_segments);
        methods.add_method("string", |_, this, ()| -> LuaResult<String> {
            Ok(this.0.to_string())
        });
    }
}

/// Lua wrapper for [reqwest::Client]
struct ClientWrapper(Client);
impl ClientWrapper {
    async fn get(
        _: Lua,
        this: LuaUserDataRef<Self>,
        (url, headers): (String, LuaTable),
    ) -> LuaResult<String> {
        let mut req = this.0.get(url);

        for ent in headers.pairs::<String, String>() {
            if let Ok((k, v)) = ent {
                req = req.header(k, v);
            }
        }

        Ok(req.send().await.unwrap().text().await.unwrap())
    }

    async fn post(
        _: Lua,
        this: LuaUserDataRef<Self>,
        (url, headers, form): (String, LuaTable, LuaTable),
    ) -> LuaResult<String> {
        let mut form_: HashMap<String, String> = HashMap::new();
        let mut req = this.0.post(url);

        for ent in form.pairs::<String, String>() {
            if let Ok((k, v)) = ent {
                form_.insert(k, v);
            }
        }
        for ent in headers.pairs::<String, String>() {
            if let Ok((k, v)) = ent {
                req = req.header(k, v);
            }
        }

        req = req.form(&form_);

        Ok(req
            .send()
            .await
            .map_err(|err| err.into_lua_err())?
            .text()
            .await
            .map_err(|err| err.into_lua_err())?)
    }
}
impl LuaUserData for ClientWrapper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method("get", Self::get);
        methods.add_async_method("post", Self::post);
    }
}

/// Lua wrapper for [scraper::Html]
struct Scraper(Arc<Mutex<Html>>);
impl LuaUserData for Scraper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |_, raw_html: String| {
            let html = Html::parse_document(&raw_html);
            Ok(Self(Arc::new(Mutex::new(html))))
        });
        async fn select(
            lua: Lua,
            this: LuaUserDataRef<Scraper>,
            selector: String,
        ) -> LuaResult<LuaTable> {
            let sel = Selector::parse(&selector).unwrap();
            lua.create_sequence_from(
                this.0
                    .lock()
                    .await
                    .select(&sel)
                    .map(|x| ElementWrapper(x.inner_html().clone(), x.value().clone())),
            )
        }
        methods.add_async_method("select", select);
    }
}

/// Lua wrapper for [scraper::Element]
#[derive(Clone)]
struct ElementWrapper(String, Element);
unsafe impl Send for ElementWrapper {}
impl LuaUserData for ElementWrapper {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("inner_html", |_, this| Ok(this.0.clone()));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("attr", |_, this, value: String| {
            Ok(this.1.attr(&value).unwrap().to_string())
        });
    }
}

fn add_engine(lua: &Lua, (name, callback): (String, LuaFunction)) -> LuaResult<()> {
    lua.globals()
        .get::<LuaTable>("__searched_engines__")?
        .set(name, callback.clone())?;

    Ok(())
}
fn stringify_params(_: &Lua, params: LuaTable) -> LuaResult<String> {
    Ok(params
        .pairs::<String, String>()
        .filter_map(|ent| ent.ok().map(|(k, v)| [k, v].join("&")))
        .collect::<Vec<_>>()
        .join("&"))
}
fn parse_json(lua: &Lua, raw: String) -> LuaResult<LuaValue> {
    let json: serde_json::Value = serde_json::from_str(&raw).into_lua_err()?;
    lua.to_value(&json)
}
fn fend_eval(_: &Lua, input: String) -> LuaResult<String> {
    Ok(fend_core::evaluate(&input, &mut Context::new())
        .unwrap()
        .get_main_result()
        .to_string())
}

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
                let name = path.path().file_stem().expect("bad file path").to_str().expect("filename should be utf-8").to_string();

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
    pub async fn search(&self, query: Query) -> Vec<crate::Result> {
        #[cfg(feature = "hot_reload")]
        Self::load_engines(&self.lua).await;

        #[cfg(feature = "hot_reload")]
        let providers = ProvidersConfig::load("plugins/providers.toml");
        #[cfg(not(feature = "hot_reload"))]
        let providers = &self.providers;

        if let Some(provider) = providers.0.get(&query.provider) {
            let engine = provider.engine.clone().unwrap_or_else(|| query.provider.clone());
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


use std::{
    cell::UnsafeCell,
    collections::HashMap,
    error::Error,
    fs::{read_dir, File},
    io::Read,
};

use mlua::prelude::*;
use reqwest::Client;
use scraper::{node::Element, Html, Selector};
use tokio::{sync::watch, task::LocalSet};

use crate::{Kind, Query};

impl LuaUserData for Query {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("query", |_, this| Ok(this.query.clone()));
        fields.add_field_method_get("page", |_, this| Ok(this.page));
    }
}

struct Scraper {
    inner: UnsafeCell<Html>,
}
unsafe impl Sync for Scraper {}
unsafe impl Send for Scraper {}
impl LuaUserData for Scraper {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("new", |_, raw_html: String| {
            let html = Html::parse_document(&raw_html);
            Ok(Self { inner: html.into() })
        });
        methods.add_method("select", |lua, this, selector: String| {
            let sel = Selector::parse(&selector).unwrap();
            Ok(lua
                .create_sequence_from(unsafe {
                    this.inner
                        .get()
                        .as_ref()
                        .unwrap()
                        .select(&sel)
                        .map(|x| ElementWrapper(x.inner_html(), x.value().clone()))
                })
                .unwrap())
        });
    }
}

#[derive(Clone)]
struct ElementWrapper(String, Element);
unsafe impl Send for ElementWrapper {}
impl LuaUserData for ElementWrapper {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("inner_html", |_, this| Ok(this.0.clone()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("attr", |_, this, value: String| {
            Ok(this.1.attr(&value).unwrap().to_string())
        });
    }
}

#[derive(Clone)]
pub struct PluginEngine {
    query_tx: watch::Sender<(String, crate::Query)>,
    results_rx: watch::Receiver<Vec<crate::Result>>,
}
impl PluginEngine {
    pub async fn new() -> Result<(Self, LocalSet), Box<dyn Error>> {
        let (query_tx, rx) = watch::channel(Default::default());
        let (tx, results_rx) = watch::channel(Default::default());

        let local = LocalSet::new();

        local.spawn_local(async move {
            Self::inner(rx, tx).await.unwrap();
        });

        Ok((
            Self {
                query_tx,
                results_rx,
            },
            local,
        ))
    }

    /// Actual Lua init and event loop
    async fn inner(
        mut rx: watch::Receiver<(String, crate::Query)>,
        tx: watch::Sender<Vec<crate::Result>>,
    ) -> Result<(), Box<dyn Error>> {
        let lua = Lua::new();

        // Add Lua interfaces

        lua.globals()
            .set("__searched_providers__", lua.create_table()?)?;
        lua.globals().set("Query", lua.create_proxy::<Query>()?)?;
        lua.globals()
            .set("Scraper", lua.create_proxy::<Scraper>()?)?;
        lua.globals()
            .set("Element", lua.create_proxy::<ElementWrapper>()?)?;

        async fn add_search_provider<'eng>(
            lua: &Lua,
            (name, kind, callback): (String, Kind, LuaFunction<'eng>),
        ) -> LuaResult<()> {
            lua.globals()
                .get::<_, LuaTable<'_>>("__searched_providers__")
                .unwrap()
                .set(name, callback.clone())
                .unwrap();

            Ok(())
        }

        lua.globals().set(
            "add_search_provider",
            lua.create_async_function(add_search_provider)?,
        )?;

        async fn get(_: &Lua, (url, headers): (String, LuaTable<'_>)) -> LuaResult<String> {
            let mut req = Client::new().get(url);

            for ent in headers.pairs::<String, String>() {
                if let Ok((k, v)) = ent {
                    req = req.header(k, v);
                }
            }

            Ok(req.send().await.unwrap().text().await.unwrap())
        }
        lua.globals().set("get", lua.create_async_function(get)?)?;

        async fn post(
            _: &Lua,
            (url, headers, form): (String, LuaTable<'_>, LuaTable<'_>),
        ) -> LuaResult<String> {
            let mut form_: HashMap<String, String> = HashMap::new();
            let mut req = Client::new().post(url);

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
        lua.globals()
            .set("post", lua.create_async_function(post)?)?;

        async fn stringify_params(_: &Lua, params: LuaTable<'_>) -> LuaResult<String> {
            let mut form_: HashMap<String, String> = HashMap::new();

            for ent in params.pairs::<String, String>() {
                if let Ok((k, v)) = ent {
                    form_.insert(k, v);
                }
            }

            Ok(form_
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&"))
        }
        lua.globals().set(
            "stringify_params",
            lua.create_async_function(stringify_params)?,
        )?;

        lua.globals().set(
            "parse_json",
            lua.create_function(|lua, raw: String| {
                let json = serde_json::to_value(raw).unwrap();
                Ok(lua.to_value(&json).unwrap())
            })?,
        )?;

        // Load scrapers
        for path in read_dir("plugins/scrapers").unwrap() {
            if let Ok(path) = path {
                let mut buf = String::new();
                let mut f = File::open(path.path()).unwrap();
                f.read_to_string(&mut buf).unwrap();
                lua.load(&buf).exec_async().await.unwrap();
            }
        }

        // Event loop
        while let Ok(()) = rx.changed().await {
            let (provider, query) = rx.borrow_and_update().clone();

            let provider_ = lua
                .globals()
                .get::<_, LuaTable>("__searched_providers__")
                .unwrap()
                .get::<_, LuaFunction>(provider)
                .unwrap();

            let results = provider_
                .call_async::<Query, Vec<HashMap<String, String>>>(query)
                .await;

            match results {
                Ok(results) => {
                    tx.send(
                        results
                            .into_iter()
                            .map(|r| crate::Result {
                                url: r.get("url").unwrap().to_string(),
                                title: r.get("title").unwrap().to_string(),
                                general: Some(crate::GeneralResult {
                                    snippet: r
                                        .get("snippet")
                                        .map(|x| x.to_string())
                                        .unwrap_or_default(),
                                }),
                                ..Default::default()
                            })
                            .collect(),
                    )
                    .unwrap();
                }
                Err(_err) => {
                    tx.send(Vec::new()).unwrap();
                }
            }
        }

        Ok(())
    }

    /// Use the given provider to process the given query
    pub async fn search(&mut self, provider: String, query: Query) -> Vec<crate::Result> {
        self.results_rx.mark_unchanged();
        self.query_tx.send((provider, query)).unwrap();
        self.results_rx.changed().await.unwrap();
        self.results_rx.borrow_and_update().clone()
    }
}

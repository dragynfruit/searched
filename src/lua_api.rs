use std::{
    cell::UnsafeCell,
    collections::{HashMap, VecDeque},
    error::Error,
    fs::{read_dir, File},
    io::Read,
    sync::Arc,
};

use mlua::prelude::*;
use reqwest::Client;
use scraper::{node::Element, Html, Selector};
use tokio::{
    sync::{oneshot, watch, Mutex},
    task::{spawn_local, LocalSet},
};

use crate::{config::ProvidersConfig, Query};

impl LuaUserData for Query {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("query", |_, this| Ok(this.query.clone()));
        fields.add_field_method_get("page", |_, this| Ok(this.page));
    }
}

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

struct Scraper {
    inner: UnsafeCell<Html>,
}
unsafe impl Sync for Scraper {}
unsafe impl Send for Scraper {}
impl LuaUserData for Scraper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
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
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("inner_html", |_, this| Ok(this.0.clone()));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("attr", |_, this, value: String| {
            Ok(this.1.attr(&value).unwrap().to_string())
        });
    }
}

/// A single-threaded plugin engine
#[derive(Clone)]
pub struct PluginEngine {
    query_tx: watch::Sender<crate::Query>,
    results_rx: watch::Receiver<Vec<crate::Result>>,
}
impl PluginEngine {
    pub async fn new(client: Client) -> Result<Self, Box<dyn Error>> {
        let providers = ProvidersConfig::load("plugins/providers.toml");
        let (query_tx, rx) = watch::channel(Default::default());
        let (tx, results_rx) = watch::channel(Default::default());

        spawn_local(async move {
            Self::inner(client, providers, rx, tx).await.unwrap();
        });

        Ok(Self {
            query_tx,
            results_rx,
        })
    }

    /// Actual Lua init and event loop
    async fn inner(
        client: Client,
        providers: ProvidersConfig,
        mut rx: watch::Receiver<crate::Query>,
        tx: watch::Sender<Vec<crate::Result>>,
    ) -> Result<(), Box<dyn Error>> {
        let lua = Lua::new();

        // Add Lua interfaces

        lua.globals()
            .set("__searched_engines__", lua.create_table()?)?;
        lua.globals().set("Query", lua.create_proxy::<Query>()?)?;
        lua.globals()
            .set("Client", lua.create_proxy::<ClientWrapper>()?)?;
        lua.globals()
            .set("Scraper", lua.create_proxy::<Scraper>()?)?;
        lua.globals()
            .set("Element", lua.create_proxy::<ElementWrapper>()?)?;
        lua.globals()
            .set("client", lua.create_userdata(ClientWrapper(Client::new()))?)?;

        lua.globals().set(
            "add_engine",
            //lua.create_async_function(add_engine)?,
            lua.create_function(|lua, (name, callback): (String, LuaFunction)| {
                lua.globals()
                    .get::<LuaTable>("__searched_engines__")
                    .unwrap()
                    .set(name, callback.clone())
                    .unwrap();

                Ok(())
            })?,
        )?;

        lua.globals().set(
            "stringify_params",
            lua.create_function(|_, params: LuaTable| {
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
            })?,
        )?;

        lua.globals().set(
            "parse_json",
            lua.create_function(|lua, raw: String| {
                let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
                Ok(lua.to_value(&json).unwrap())
            })?,
        )?;

        // Load engines
        for path in read_dir("plugins/engines").unwrap() {
            if let Ok(path) = path {
                let mut buf = String::new();
                let mut f = File::open(path.path()).unwrap();
                f.read_to_string(&mut buf).unwrap();
                lua.load(&buf).exec_async().await.unwrap();
            }
        }

        // Event loop
        while let Ok(()) = rx.changed().await {
            let query = rx.borrow_and_update().clone();

            let provider = providers.0.get(&query.provider).unwrap();

            let engine = provider.engine.clone().unwrap_or(query.provider.clone());

            let engine_ = lua
                .globals()
                .get::<LuaTable>("__searched_engines__")
                .unwrap()
                .get::<LuaFunction>(engine)
                .unwrap();

            let url = {
                if let Some(url_fmt) = &provider.url {
                    let query = query.clone();
                    let map = HashMap::from_iter([
                        ("query", query.query),
                        ("page", query.page.to_string()),
                    ]);

                    Some(searched_parser::Url::parse(url_fmt.as_bytes()).build(map))
                } else {
                    None
                }
            };

            let results = engine_
                .call_async::<Vec<HashMap<String, String>>>((
                    ClientWrapper(client.clone()),
                    query,
                    url,
                ))
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
    pub async fn search(&mut self, query: Query) -> Vec<crate::Result> {
        self.results_rx.mark_unchanged();
        self.query_tx.send(query).unwrap();
        self.results_rx.changed().await.unwrap();
        self.results_rx.borrow_and_update().clone()
    }
}

#[derive(Clone)]
pub struct PluginEnginePool {
    queue: Arc<Mutex<VecDeque<(crate::Query, oneshot::Sender<Vec<crate::Result>>)>>>,
}
impl PluginEnginePool {
    pub async fn new(pool_size: usize) -> Self {
        let queue: Arc<Mutex<VecDeque<(crate::Query, oneshot::Sender<Vec<crate::Result>>)>>> =
            Arc::new(Mutex::const_new(VecDeque::new()));

        let client = Client::new();

        for _ in 0..pool_size {
            let queue = queue.clone();
            let client = client.clone();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async move {
                    LocalSet::new()
                        .run_until(async move {
                            let mut eng = PluginEngine::new(client).await.unwrap();
                            loop {
                                let query = queue.lock().await.pop_front();

                                if let Some((query, res_tx)) = query {
                                    res_tx.send(eng.search(query).await).unwrap();
                                }
                            }
                        })
                        .await;
                });
            });
        }

        Self { queue }
    }
    pub async fn search(&self, query: Query) -> Vec<crate::Result> {
        let (res_tx, rx) = oneshot::channel::<Vec<crate::Result>>();
        self.queue.lock().await.push_back((query, res_tx));
        rx.await.unwrap_or_default()
    }
}

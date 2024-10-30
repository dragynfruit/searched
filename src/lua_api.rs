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
        Url::parse_with_params(&url, params.pairs::<String, String>().map(|x| x.unwrap()).collect::<Vec<(String, String)>>()).map(|x| UrlWrapper(x)).into_lua_err()
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
        methods.add_function(
            "from_template",
            Self::from_template,
        );
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
struct Scraper(Html);
impl LuaUserData for Scraper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("new", |_, raw_html: String| {
            let html = Html::parse_document(&raw_html);
            Ok(Self(html))
        });
        methods.add_method("select", |lua, this, selector: String| {
            let sel = Selector::parse(&selector).unwrap();
            Ok(lua
                .create_sequence_from(
                    this.0
                        .select(&sel)
                        .map(|x| ElementWrapper(x.inner_html(), x.value().clone())),
                )
                .unwrap())
        });
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
    Ok(fend_core::evaluate(&input, &mut Context::new()).unwrap().get_main_result().to_string())
}

/// A single-threaded plugin engine
#[derive(Clone)]
pub struct PluginEngine {
    query_tx: watch::Sender<crate::Query>,
    results_rx: watch::Receiver<Vec<crate::Result>>,
}
impl PluginEngine {
    /// Initialize a new engine for running plugins
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

            if let Some(provider) = providers.0.get(&query.provider) {
                let engine = provider.engine.clone().unwrap_or(query.provider.clone());

                // Get engine implementation
                let eng_impl = lua
                    .globals()
                    .get::<LuaTable>("__searched_engines__")
                    .unwrap()
                    .get::<LuaFunction>(engine)
                    .unwrap();

                let results = eng_impl
                    .call_async::<Vec<HashMap<String, LuaValue>>>((
                        ClientWrapper(client.clone()),
                        query,
                        lua.to_value(&provider.clone().extra.unwrap_or_default()),
                    ))
                    .await;

                fn read_to_string(data: &LuaValue) -> String {
                    if let LuaValue::String(s) = data {
                        s.to_str().unwrap().to_string()
                    } else {
                        String::new()
                    }
                }

                match results {
                    Ok(results) => {
                        tx.send(
                            results
                                .into_iter()
                                .map(|r| crate::Result {
                                    url : read_to_string(r.get("url").unwrap()),
                                    title : read_to_string(r.get("title").unwrap()),
                                    general: Some(crate::GeneralResult {
                                        snippet: r
                                            .get("snippet")
                                            .map(|x| read_to_string(x))
                                            .unwrap_or_default(),
                                    }),
                                    ..Default::default()
                                })
                                .collect(),
                        )
                        .unwrap();
                    }
                    Err(err) => {
                        error!("failed to get results from engine: {:?}", err);
                        tx.send(Vec::new()).unwrap();
                    }
                }
            }
        }

        Ok(())
    }

    /// Use the given provider to process the given query
    pub async fn search(&mut self, query: Query) -> Vec<crate::Result> {
        // Clean the last set of results
        self.results_rx.mark_unchanged();

        if self.query_tx.send(query).is_ok() {
            if tokio::time::timeout(Duration::from_secs(3), self.results_rx.changed())
                .await
                .map(|ret| ret.is_ok())
                .unwrap_or(false)
            {
                let results = self.results_rx.borrow_and_update().clone();
                return results;
            }
        }

        Vec::new()
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

        let mut headers = HeaderMap::new();
        for (key, val) in [
            (
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:129.0) Gecko/20100101 Firefox/129.0",
            ),
            (
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
            ),
            ("Accept-Language", "en-US,en;q=0.5"),
            ("Accept-Encoding", "gzip"),
            ("DNT", "1"),
            ("Connection", "keep-alive"),
            ("Upgrade-Insecure-Requests", "1"),
            ("Sec-Fetch-Dest", "document"),
            ("Sec-Fetch-Mode", "navigate"),
            ("Sec-Fetch-Site", "none"),
            ("Sec-Fetch-User", "?1"),
            ("Priority", "u=1"),
            ("TE", "trailers"),
        ] {
            headers.append(key, HeaderValue::from_str(val).unwrap());
        }
        let client = Client::builder().default_headers(headers).build().unwrap();

        info!("starting plugin engine pool with {pool_size} worker threads...");

        for i in 0..pool_size {
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
                            let target = format!("searched::worker{i}");
                            let mut eng = PluginEngine::new(client).await.unwrap();

                            debug!(target: &target, "awaiting a query...");

                            loop {
                                let query = queue.lock().await.pop_front();

                                if let Some((query, res_tx)) = query {
                                    debug!(target: &target, "processing query: {query:?}");

                                    #[cfg(debug_assertions)]
                                    let st = Instant::now();

                                    let result = res_tx.send(eng.search(query).await);

                                    if result.is_err() {
                                        error!(target: &target, "failed to send results back to the main thread!");
                                    }

                                    #[cfg(debug_assertions)]
                                    debug!(target: &target, "done in {:?}! awaiting a query...", st.elapsed());
                                }
                            }
                        })
                        .await;
                });
            });
            info!("started worker thread #{i}");
        }

        Self { queue }
    }
    pub async fn search(&self, query: Query) -> Vec<crate::Result> {
        let (res_tx, rx) = oneshot::channel::<Vec<crate::Result>>();
        self.queue.lock().await.push_back((query, res_tx));
        rx.await.unwrap_or_default()
    }
}

use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    fs::{read_dir, File},
    io::Read,
    marker::PhantomData,
    ops::Deref, time::Duration,
};

use mlua::prelude::*;
use reqwest::Client;
use scraper::{node::Element, ElementRef, Html, Selector};
use tokio::{sync::oneshot, task::{spawn_local, LocalSet}};

use crate::Query;

impl LuaUserData for Query {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("query", |_, this| Ok(this.query.clone()));
        fields.add_field_method_get("page", |_, this| Ok(this.page));
    }
}

//impl LuaUserData for crate::Result {
//    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
//        fields.add_field_method_set("url", |_, this, value: String| {
//        })
//    }
//}

struct Results {
    inner: Vec<crate::Result>,
}
impl LuaUserData for Results {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "add",
            |_, this, (url, title, snippet): (String, String, String)| {
                this.inner.push(crate::Result {
                    url,
                    title,
                    general: Some(crate::GeneralResult { snippet }),
                    ..Default::default()
                });

                Ok(())
            },
        );
    }
}

struct Scraper {
    inner: Html,
}
impl LuaUserData for Scraper {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("new", |_, raw_html: String| {
            Ok(Self {
                inner: Html::parse_document(&raw_html),
            })
        });
        methods.add_method("select", |lua, this, selector: String| {
            let sel = Selector::parse(&selector).unwrap();
            Ok(lua.create_sequence_from(
                this.inner
                    .select(&sel)
                    .map(|x| ElementWrapper(x.inner_html(), x.value().clone())),
            ))
        });
    }
}

#[derive(Clone)]
struct ElementWrapper(String, Element);
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

pub struct PluginEngine {
    lua: Lua,
}
impl PluginEngine {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let lua = Lua::new();

        lua.globals().set("__searched_providers__", lua.create_table()?)?;
        lua.globals().set("Query", lua.create_proxy::<Query>()?)?;
        lua.globals()
            .set("Scraper", lua.create_proxy::<Scraper>()?)?;
        lua.globals()
            .set("Element", lua.create_proxy::<ElementWrapper>()?)?;

        async fn add_search_provider<'eng>(
            lua: &Lua,
            (name, callback): (String, LuaFunction<'eng>),
        ) -> LuaResult<()> {
            lua.globals()
                .get::<_, LuaTable<'_>>("__searched_providers__")
                .unwrap()
                .set(name, callback.clone())
                .unwrap();
            callback
                .call_async::<Query, ()>(Query {
                    query: "rust".to_string(),
                    ..Default::default()
                })
                .await
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

            Ok(req.send().await.unwrap().text().await.unwrap())
        }
        lua.globals()
            .set("post", lua.create_async_function(post)?)?;

        lua.globals().set("parse_json", lua.create_function(|_, raw: String| {
            Ok(serde_json::from_str::<BTreeMap<String, String>>(&raw).unwrap())
        })?)?;

        for path in read_dir("plugins/scrapers").unwrap() {
            if let Ok(path) = path {
                let mut buf = String::new();
                let mut f = File::open(path.path()).unwrap();
                f.read_to_string(&mut buf).unwrap();
                lua.load(&buf).exec_async().await.unwrap();
            }
        }

        Ok(Self { lua })
    }
    //pub async fn search(&mut self, provider: String, query: Query) -> Vec<crate::Result> {
    pub async fn search(&self, provider: String, query: Query) -> Vec<crate::Result> {
        let provider_ = self.lua.globals()
            .get::<_, LuaTable>("__searched_providers__")
            .unwrap()
            .get::<_, LuaFunction>(provider)
            .unwrap();


            let results = provider_.call_async::<Query, Vec<HashMap<String, String>>>(query).await.unwrap();

            results.into_iter()
                .map(|r| {
                    let r = r.clone();

                    crate::Result {
                        url: r.get("url").unwrap().to_string(),
                        title: r.get("title").unwrap().to_string(),
                        general: Some(crate::GeneralResult {
                            snippet: r.get("snippet").map(|x| x.to_string()).unwrap_or_default(),
                        }),
                        ..Default::default()
                    }
                    .clone()
                })
                .collect()
    }
}

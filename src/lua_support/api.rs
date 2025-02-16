use std::{collections::HashMap, sync::Arc};

use fend_core::Context;
use mlua::prelude::*;
use reqwest::Client;
use scraper::{node::Element, Html, Selector};
use tokio::sync::Mutex;
use url::Url;

use crate::Query;

impl LuaUserData for Query {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("query", |_, this| Ok(this.query.clone()));
        fields.add_field_method_get("page", |_, this| Ok(this.page));
        fields.add_field_method_get("safe", |_, this| Ok(this.safe.to_string().to_lowercase()));
    }
}

/// Lua wrapper for [url::Url]
pub struct UrlWrapper(Url);
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
    fn params(lua: &Lua, this: &Self, _: ()) -> LuaResult<LuaValue> {
        Ok(this
            .0
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>()
            .into_lua(lua)?)
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
        methods.add_method("params", Self::params);
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
pub struct ClientWrapper(pub Client);
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
pub struct Scraper(Arc<Mutex<Html>>);
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
pub struct ElementWrapper(String, Element);
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

pub fn add_engine(lua: &Lua, (name, callback): (String, LuaFunction)) -> LuaResult<()> {
    lua.globals()
        .get::<LuaTable>("__searched_engines__")?
        .set(name, callback.clone())?;

    Ok(())
}
pub fn stringify_params(_: &Lua, params: LuaTable) -> LuaResult<String> {
    Ok(params
        .pairs::<String, String>()
        .filter_map(|ent| ent.ok().map(|(k, v)| [k, v].join("&")))
        .collect::<Vec<_>>()
        .join("&"))
}
pub fn parse_json(lua: &Lua, raw: String) -> LuaResult<LuaValue> {
    let json: serde_json::Value = serde_json::from_str(&raw).into_lua_err()?;
    lua.to_value(&json)
}
pub fn fend_eval(_: &Lua, input: String) -> LuaResult<String> {
    Ok(fend_core::evaluate(&input, &mut Context::new())
        .unwrap()
        .get_main_result()
        .to_string())
}

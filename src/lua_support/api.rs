use std::{collections::HashMap, sync::Arc};

use fend_core::Context;
use mlua::prelude::*;
use reqwest::Client;
use scraper::{Html, Selector, node::Element};
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

#[derive(Clone)]
pub struct RequestBuilder {
    client: Client,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    form: Option<HashMap<String, String>>,
    json: Option<serde_json::Value>,
}

impl RequestBuilder {
    pub fn new(client: Client, method: String, url: String) -> Self {
        Self {
            client,
            method,
            url,
            headers: HashMap::new(),
            form: None,
            json: None,
        }
    }
}

impl LuaUserData for RequestBuilder {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("headers", |_, this, headers: LuaTable| {
            let mut map = HashMap::new();
            for pair in headers.pairs::<String, String>() {
                let (k, v) = pair.unwrap();
                map.insert(k, v);
            }
            this.headers = map;
            Ok(this.clone())
        });

        methods.add_method_mut("form", |_, this, form: LuaTable| {
            let mut map = HashMap::new();
            for pair in form.pairs::<String, String>() {
                let (k, v) = pair.unwrap();
                map.insert(k, v);
            }
            this.form = Some(map);
            Ok(this.clone())
        });

        methods.add_method_mut("json", |_lua, this, value: LuaValue| {
            // Convert Lua value to JSON
            let json = match value {
                LuaValue::Table(t) => {
                    // Convert table to HashMap
                    let mut map = HashMap::new();
                    for pair in t.pairs::<String, LuaValue>() {
                        if let Ok((k, v)) = pair {
                            // Convert LuaValue to String or keep as JSON value
                            let value = match v {
                                LuaValue::String(s) => {
                                    serde_json::Value::String(s.to_str()?.to_string())
                                }
                                LuaValue::Number(n) => serde_json::Value::Number(
                                    serde_json::Number::from_f64(n).unwrap(),
                                ),
                                LuaValue::Boolean(b) => serde_json::Value::Bool(b),
                                LuaValue::Nil => serde_json::Value::Null,
                                _ => continue, // Skip other types
                            };
                            map.insert(k, value);
                        }
                    }
                    serde_json::Value::Object(serde_json::Map::from_iter(map))
                }
                LuaValue::String(s) => serde_json::from_str(&s.to_str()?.to_string())
                    .unwrap_or(serde_json::Value::Null),
                _ => serde_json::Value::Null,
            };

            this.json = Some(json);
            Ok(this.clone())
        });

        methods.add_async_method("send", |_, this, _: ()| async move {
            let mut req = this.client.request(
                reqwest::Method::from_bytes(this.method.as_bytes()).unwrap(),
                &this.url,
            );

            // Use references to avoid moves
            for (k, v) in &this.headers {
                req = req.header(k, v);
            }

            if let Some(form) = &this.form {
                req = req.form(form);
            }

            if let Some(json) = &this.json {
                req = req.json(json);
            }

            match req.send().await {
                Ok(res) => match res.text().await {
                    Ok(text) => Ok(text),
                    Err(e) => Err(LuaError::RuntimeError(format!(
                        "Failed to read response: {}",
                        e
                    ))),
                },
                Err(e) => Err(LuaError::RuntimeError(format!("Request failed: {}", e))),
            }
        });

        methods.add_async_method("html", |_, this, _: ()| async move {
            let mut req = this.client.request(
                reqwest::Method::from_bytes(this.method.as_bytes()).unwrap(),
                &this.url,
            );

            // Add headers
            for (k, v) in &this.headers {
                req = req.header(k, v);
            }

            if let Some(form) = &this.form {
                req = req.form(form);
            }

            if let Some(json) = &this.json {
                req = req.json(json);
            }

            match req.send().await {
                Ok(res) => match res.text().await {
                    Ok(text) => {
                        let html = Html::parse_document(&text);
                        Ok(Scraper(Arc::new(Mutex::new(html))))
                    }
                    Err(e) => Err(LuaError::RuntimeError(format!(
                        "Failed to read response: {}",
                        e
                    ))),
                },
                Err(e) => Err(LuaError::RuntimeError(format!("Request failed: {}", e))),
            }
        });
    }
}

impl LuaUserData for ClientWrapper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Only keep the builder method
        methods.add_method("req", |_, this, (method, url): (String, String)| {
            Ok(RequestBuilder::new(this.0.clone(), method, url))
        });
    }
}

/// Lua wrapper for [scraper::Html]
pub struct Scraper(Arc<Mutex<Html>>);
impl LuaUserData for Scraper {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_function("from_string", |_, raw_html: String| {
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

fn strip_html_tags(html: String) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
            }
            _ => {
                if !in_tag {
                    result.push(c);
                }
            }
        }
    }
    result
}

/// Lua wrapper for [scraper::Element]
#[derive(Clone)]
pub struct ElementWrapper(String, Element);
unsafe impl Send for ElementWrapper {}
impl LuaUserData for ElementWrapper {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("inner_html", |_, this| Ok(this.0.clone()));
        fields.add_field_method_get("inner_text", |_, this| Ok(strip_html_tags(this.0.clone())));
    }
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("attr", |_, this, value: String| {
            Ok(this.1.attr(&value).map(|x| x.to_string()))
        });
    }
}

pub fn add_engine(lua: &Lua, (name, callback): (String, LuaFunction)) -> LuaResult<()> {
    lua.globals()
        .get::<LuaTable>("__searched_engines__")?
        .set(name, callback.clone())?;

    Ok(())
}
pub fn add_merger(lua: &Lua, (name, callback): (String, LuaFunction)) -> LuaResult<()> {
    lua.globals()
        .get::<LuaTable>("__searched_mergers__")?
        .set(name, callback.clone())?;

    Ok(())
}
pub fn add_ranker(lua: &Lua, (name, callback): (String, LuaFunction)) -> LuaResult<()> {
    lua.globals()
        .get::<LuaTable>("__searched_rankers__")?
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

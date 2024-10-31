use std::{process, sync::Arc, time::Instant};

use axum::{
    body::Body,
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use once_cell::sync::Lazy;
use searched::{config::ProvidersConfig, lua_api::PluginEngine, Kind, PROVIDER_KINDS};
use tera::{Context, Tera};
use tokio::{sync::RwLock, task::JoinSet};

use crate::AppState;

pub static TEMPLATES: Lazy<Arc<RwLock<Tera>>> = Lazy::new(|| {
    let tera = match Tera::new("views/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    Arc::new(RwLock::new(tera))
});

const MOTD: &'static [&'static str] = &[
    "<i>Blazingly</i> fast",
    "RIIR",
    "\"It's not a cult\"",
    "Search for sigmas",
    "Install Gentoo",
    "Don't be evil",
    "mikerowesoft.com",
    "Check out <a href=\"https://github.com/dragynfruit/pasted\">pasted</a>!",
    "Drink water",
    "The cake is a lie",
    "<a onclick=\"alert('Python bad')\" href=\"#\">Click me!</a>",
    "We <3 <a href=\"https://archive.org\">IA</a>",
    "<i>\"If it is on the Internet then it must be true.\"</i><br>&mdash; Abraham Lincoln",
];

#[derive(Serialize)]
struct SearchCtx {
    motd: &'static str,
}

pub async fn search(Query(params): Query<SearchParams>) -> impl IntoResponse {
    if params.q.is_some() {
        return Redirect::to("/search").into_response();
    }

    #[cfg(debug_assertions)]
    (*TEMPLATES.write().await).full_reload().unwrap();

    Html(
        (*TEMPLATES.read().await)
            .render(
                "index.html",
                &Context::from_serialize(SearchCtx {
                    motd: MOTD[fastrand::usize(..MOTD.len())],
                })
                .unwrap(),
            )
            .unwrap(),
    )
    .into_response()
}

#[derive(Deserialize, Default)]
pub struct SearchParams {
    q: Option<String>,
    k: Option<searched::Kind>,
    s: Option<String>,
    p: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResult {
    url: String,
    title: String,
    body_preview: String,
}

#[derive(Serialize, Default)]
pub struct SearchResults {
    kinds: Vec<Kind>,
    query: searched::Query,
    count: usize,
    page: usize,
    kind: Kind,
    results: Vec<searched::Result>,
    search_time: u128,
}

pub async fn ranked_results(
    engine: PluginEngine,
    ProvidersConfig(provider_cfg): ProvidersConfig,
    query: searched::Query,
) -> Vec<searched::Result> {
    let mut set = JoinSet::new();
    for provider in provider_cfg.keys().cloned() {
        // Clone the query so we can switch the provider
        // and safely pass between threads
        let mut query = query.clone();
        query.provider = provider;

        let engine = engine.clone();

        set.spawn(async move { engine.search(query).await });
    }

    let mut results = set.join_all().await.concat();

    let ranking_tm = Instant::now();

    results.sort_by_key(|x| x.url.clone());

    let scores = results
        .iter()
        .map(|x| results.iter().filter(|y| y.url == x.url).count());

    let mut scored = results.iter().zip(scores).collect::<Vec<_>>();
    scored.dedup_by_key(|x| x.0.url.clone());
    scored.sort_by(|a, b| b.1.cmp(&a.1));

    let ret = scored.iter().map(|x| x.0.clone()).collect();
    debug!("ranking: {:?}", ranking_tm.elapsed());

    ret
}

#[axum_macros::debug_handler]
pub async fn results(
    Query(params): Query<SearchParams>,
    State(st): State<AppState>,
) -> impl IntoResponse {
    if let Some(q) = params.q {
        if q.eq_ignore_ascii_case("rust") {
            Redirect::to("https://rust-lang.org").into_response()
        } else {
        #[cfg(debug_assertions)]
        (*TEMPLATES.write().await).full_reload().unwrap();

        let kind = params.k.unwrap_or_default();

        let query = searched::Query {
            provider: params.s.clone().unwrap_or("duckduckgo".to_string()),
            query: q.clone(),
            kind: kind.clone(),
            page: params.p.unwrap_or(1),
            ..Default::default()
        };

        let search_st = Instant::now();

        let results = if query.provider == String::from("all") {
            ranked_results(
                st.eng,
                ProvidersConfig::load("plugins/providers.toml"),
                query.clone(),
            )
            .await
        } else {
            st.eng.search(query.clone()).await
        };

        let search_tm = search_st.elapsed();
        debug!("results took {search_tm:?}");

        Html(
            (*TEMPLATES.read().await)
                .render(
                    "results.html",
                    &Context::from_serialize(SearchResults {
                        kind,
                        query: query,
                        results: results.to_vec(),
                        search_time: search_tm.as_millis(),
                        ..Default::default()
                    })
                    .unwrap(),
                )
                .unwrap(),
        )
        .into_response()
    } else {
        return Redirect::to("/").into_response();
    }
    }
}

pub async fn settings(State(_st): State<AppState>) -> impl IntoResponse {
    #[cfg(debug_assertions)]
    (*TEMPLATES.write().await).full_reload().unwrap();

    Html(
        (*TEMPLATES.read().await)
            .render("settings.html", &Context::new())
            .unwrap(),
    )
    .into_response()
}

pub async fn logo() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "max-age=31536000")
        .body(Body::from(include_bytes!("../assets/logo.png").to_vec()))
        .unwrap()
        .into_response()
}

pub async fn icon() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/x-icon")
        .header(header::CACHE_CONTROL, "max-age=31536000")
        .body(Body::from(
            include_bytes!("../assets/searched.ico").to_vec(),
        ))
        .unwrap()
        .into_response()
}

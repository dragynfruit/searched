use std::{process, sync::Arc};

use axum::{
    body::Body,
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use once_cell::sync::Lazy;
use searched::Kind;
use tera::{Context, Tera};
use tokio::sync::RwLock;

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
];

#[derive(Serialize)]
struct SearchCtx {
    motd: &'static str,
}

pub async fn search(Query(params): Query<SearchParams>) -> impl IntoResponse {
    if let Some(_q) = params.q {
        return Redirect::to("/search").into_response();
    }

    #[cfg(debug_assertions)]
    (*TEMPLATES.write().await).full_reload().unwrap();

    Html(
        (*TEMPLATES.read().await)
            .render("index.html", &Context::from_serialize(SearchCtx {
                motd: MOTD[fastrand::usize(..MOTD.len())],
            }).unwrap())
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
    query: String,
    count: usize,
    page: usize,
    kind: Kind,
    results: Vec<searched::Result>,
    parse_time: f32,
    search_time: f32,
    gather_time: f32,
}

#[axum_macros::debug_handler]
pub async fn results(
    Query(params): Query<SearchParams>,
    State(st): State<AppState>,
) -> impl IntoResponse {
    if let Some(q) = params.q {
        #[cfg(debug_assertions)]
        (*TEMPLATES.write().await).full_reload().unwrap();
        //let mut results: Vec<SearchResult> = Vec::new();

        //let reader = st.reader.clone();
        //reader.reload().unwrap();
        //let searcher = reader.searcher();

        //let parse_st = Instant::now();
        //let query = st.query_parser.parse_query(&q).unwrap();
        //let parse_time = parse_st.elapsed().as_secs_f32() * 1_000.0;

        ////let count = (*st.count_cache.lock().await).get_or_insert(q.clone(), || {
        ////    searcher.search(&query, &Count).unwrap()
        ////}).clone();
        //let count = searcher.search(&query, &Count).unwrap();

        //let search_st = Instant::now();
        //let resultss: Vec<(Score, DocAddress)> = searcher
        //    .search(
        //        &query,
        //        &TopDocs::with_limit(20).and_offset(params.p.unwrap_or(0)),
        //    )
        //    .unwrap();
        //let search_time = search_st.elapsed().as_secs_f32() * 1_000.0;

        //let gather_st = Instant::now();
        //for (_score, doc_address) in resultss {
        //    // Retrieve the actual content of documents given its `doc_address`.
        //    let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address).unwrap();

        //    let url = retrieved_doc.get_first(st.url).unwrap().as_str().unwrap();
        //    let title = retrieved_doc.get_first(st.title).unwrap().as_str().unwrap();
        //    //let body = retrieved_doc.get_first(st.body).unwrap().as_str().unwrap();

        //    results.push(SearchResult {
        //        url: url.to_string(),
        //        title: title.to_string(),
        //        //body_preview: body.to_string(),
        //        body_preview: String::from("WIP"),
        //    });
        //}
        //let gather_time = gather_st.elapsed().as_secs_f32() * 1_000.0;

        //let search_st = Instant::now();
        //let results = scrapers::search(
        //    params.s.unwrap().as_str(),
        //    st.clone(),
        //    searched::Query {
        //        query: q.clone(),
        //        kind: params.k.unwrap_or_default(),
        //        page: params.p.unwrap_or(1),
        //    },
        //)
        //.await;
        //let search_time = search_st.elapsed().as_secs_f32() * 1_000.0;
        let kind = params.k.unwrap_or_default();

        let results = st
            .pool
            .search(searched::Query {
                provider: params.s.unwrap_or("duckduckgo".to_string()),
                query: q.clone(),
                kind: kind.clone(),
                page: params.p.unwrap_or(1),
            })
            .await;

        //let mut rx = st.result_rx.resubscribe();
        //let send_query = searched::Query {
        //        query: q.clone(),
        //        kind: params.k.unwrap_or_default(),
        //        page: params.p.unwrap_or(1),
        //    };
        //st.query_tx.send(send_query.clone()).await.unwrap();

        //let results = tokio::time::timeout(Duration::from_secs(3), async {
        //    while let Ok((query, results)) = rx.recv().await {
        //        if send_query == query {
        //            return results;
        //        }
        //    }
        //    Vec::new()
        //}).await.unwrap_or_default();

        Html(
            (*TEMPLATES.read().await)
                .render(
                    "results.html",
                    &Context::from_serialize(SearchResults {
                        kind,
                        query: q,
                        //page: send_query.page,
                        //count,
                        results: results.to_vec(),
                        //parse_time,
                        //search_time,
                        //gather_time,
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
        .header(header::CACHE_CONTROL, "max-age=604800")
        .body(Body::from(include_bytes!("../assets/logo.png").to_vec()))
        .unwrap()
        .into_response()
}

pub async fn icon() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/x-icon")
        .header(header::CACHE_CONTROL, "max-age=604800")
        .body(Body::from(
            include_bytes!("../assets/searched.ico").to_vec(),
        ))
        .unwrap()
        .into_response()
}

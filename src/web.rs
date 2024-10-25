use std::{
    io::{stdout, Write},
    ops::Deref,
    process,
    sync::Arc,
    time::Duration,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse, Redirect, Response},
};
use once_cell::sync::Lazy;
use searched::lua_api::PluginEngine;
use tantivy::{
    collector::{Count, TopDocs},
    schema::Value,
    DocAddress, Score, TantivyDocument,
};
use tera::{Context, Tera};
use tokio::{sync::Mutex, task::spawn_local, time::Instant};

use crate::{
    scrapers::{self, duckduckgo::Duckduckgo, Scraper},
    AppState,
};

pub static TEMPLATES: Lazy<Arc<Mutex<Tera>>> = Lazy::new(|| {
    let tera = match Tera::new("views/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };
    Arc::new(Mutex::new(tera))
});

pub async fn search(Query(params): Query<SearchParams>) -> impl IntoResponse {
    if let Some(q) = params.q {
        return Redirect::to("/search").into_response();
    }

    (*TEMPLATES.lock().await).full_reload().unwrap();

    Html(
        (*TEMPLATES.lock().await)
            .render("index.html", &Context::new())
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
        (*TEMPLATES.lock().await).full_reload().unwrap();
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

        let results = {
            let results = st.eng.lock().await.search(
                params.s.unwrap(),
                searched::Query {
                    query: q.clone(),
                    kind: params.k.unwrap_or_default(),
                    page: params.p.unwrap_or(1),
                },
            ).await;
            results
        };

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
            (*TEMPLATES.lock().await)
                .render(
                    "results.html",
                    &Context::from_serialize(SearchResults {
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

pub async fn settings(State(st): State<AppState>) -> impl IntoResponse {
    (*TEMPLATES.lock().await).full_reload().unwrap();

    Html(
        (*TEMPLATES.lock().await)
        .render(
            "settings.html",
            &Context::new(),
        ).unwrap()
    ).into_response()
}

pub async fn dragynfruit_logo() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CACHE_CONTROL, "max-age=604800")
        .body(Body::from(
            include_bytes!("../assets/dragynfruit.png").to_vec(),
        ))
        .unwrap()
        .into_response()
}

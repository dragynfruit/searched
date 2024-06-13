use std::{
    io::{stdout, Write},
    process,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use once_cell::sync::Lazy;
use tantivy::{
    collector::{Count, TopDocs},
    schema::Value,
    DocAddress, Score, Searcher, TantivyDocument,
};
use tera::{Context, Tera};
use tokio::{sync::Mutex, time::Instant};

use crate::AppState;

pub static TEMPLATES: Lazy<Arc<Mutex<Tera>>> = Lazy::new(|| {
    let mut tera = match Tera::new("views/**/*") {
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

#[derive(Deserialize)]
pub struct SearchParams {
    q: Option<String>,
    p: Option<usize>,
}

#[derive(Serialize)]
pub struct SearchResult {
    url: String,
    title: String,
    body_preview: String,
}

#[derive(Serialize)]
pub struct SearchResults {
    query: String,
    count: usize,
    results: Vec<SearchResult>,
    parse_time: f32,
    search_time: f32,
    gather_time: f32,
}

pub async fn results(
    State(st): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    if let Some(q) = params.q {
        (*TEMPLATES.lock().await).full_reload().unwrap();
        let mut results: Vec<SearchResult> = Vec::new();

        let reader = st.reader;
        reader.reload().unwrap();
        let searcher = reader.searcher();

        let parse_st = Instant::now();
        let query = st.query_parser.parse_query(&q).unwrap();
        let parse_time = parse_st.elapsed().as_secs_f32() * 1_000.0;

        //let count = (*st.count_cache.lock().await).get_or_insert(q.clone(), || {
        //    searcher.search(&query, &Count).unwrap()
        //}).clone();
        let count = searcher.search(&query, &Count).unwrap();

        let search_st = Instant::now();
        let resultss: Vec<(Score, DocAddress)> = searcher
            .search(
                &query,
                &TopDocs::with_limit(20).and_offset(params.p.unwrap_or(0)),
            )
            .unwrap();
        let search_time = search_st.elapsed().as_secs_f32() * 1_000.0;

        let gather_st = Instant::now();
        for (_score, doc_address) in resultss {
            // Retrieve the actual content of documents given its `doc_address`.
            let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address).unwrap();

            let url = retrieved_doc.get_first(st.url).unwrap().as_str().unwrap();
            let title = retrieved_doc.get_first(st.title).unwrap().as_str().unwrap();
            //let body = retrieved_doc.get_first(st.body).unwrap().as_str().unwrap();

            results.push(SearchResult {
                url: url.to_string(),
                title: title.to_string(),
                //body_preview: body.to_string(),
                body_preview: String::from("WIP"),
            });
        }
        let gather_time = gather_st.elapsed().as_secs_f32() * 1_000.0;

        return Html(
            (*TEMPLATES.lock().await)
                .render(
                    "results.html",
                    &Context::from_serialize(SearchResults {
                        query: q,
                        count,
                        results,
                        parse_time,
                        search_time,
                        gather_time,
                    })
                    .unwrap(),
                )
                .unwrap(),
        )
        .into_response();
    } else {
        return Redirect::to("/").into_response();
    }
}

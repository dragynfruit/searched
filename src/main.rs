#[macro_use]
extern crate log;
extern crate axum;
extern crate env_logger;
extern crate reqwest;
extern crate tantivy;
extern crate tera;
#[macro_use]
extern crate serde;
extern crate lru;

mod crawler;
mod page;
mod ranking;
mod scrapers;
mod web;

use std::{fs, num::NonZeroUsize, sync::Arc, time::Instant};

use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use log::LevelFilter;
use lru::LruCache;
use page::Page;
use reqwest::Client;
use scraper::Selector;
use sled::Db;
use tantivy::{
    doc,
    query::QueryParser,
    schema::{Field, Schema, FAST, STORED, TEXT},
    store::{Compressor, ZstdCompressor},
    Index, IndexReader, IndexSettings,
};
use tokio::{
    net::TcpListener,
    sync::{mpsc, Mutex},
};

#[derive(Clone)]
pub struct AppState {
    //index: Index,
    count_cache: Arc<Mutex<LruCache<String, usize>>>,
    reader: IndexReader,
    query_parser: QueryParser,
    client: Client,
    db: Db,

    url: Field,
    title: Field,
    body: Field,
}

#[tokio::main(worker_threads = 12)]
async fn main() {
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
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let st = Instant::now();
    //let res = scrapers::stackexchange::StackExchange::search(client.clone(), Query { query: String::from("rust"), page: 2 }).await;
    //println!("{res:?}");
    println!("{:?}", st.elapsed());

    //println!("{res:?}");

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    info!("Starting up...");

    let (tx, mut rx): (mpsc::UnboundedSender<Page>, mpsc::UnboundedReceiver<Page>) =
        mpsc::unbounded_channel();

    let mut schema = Schema::builder();

    let url = schema.add_text_field("url", TEXT | FAST | STORED);
    let title = schema.add_text_field("title", TEXT | FAST | STORED);
    let body = schema.add_text_field("body", TEXT);

    let schema = schema.build();

    let mut index = match Index::open_in_dir("searched-index") {
        Ok(index) => index,
        Err(_) => {
            warn!("no existing index found, creating one");

            fs::create_dir_all("searched-index").unwrap();

            Index::builder()
                .schema(schema.clone())
                .settings(IndexSettings {
                    docstore_compression: Compressor::Zstd(ZstdCompressor {
                        compression_level: Some(10),
                    }),
                    ..Default::default()
                })
                .create_in_dir("searched-index")
                .unwrap()
        }
    };
    index.set_default_multithread_executor().unwrap();

    let mut wr = index.writer(100_000_000).unwrap();

    tokio::spawn(async move {
        let body_sel = Selector::parse("body").unwrap();

        loop {
            if let Some(page) = rx.recv().await {
                wr.add_document(doc! {
                    url => page.url().to_string(),
                    title => page.title(),
                    body => page.dom().select(&body_sel).next().map(|element| element.text().collect()).unwrap_or_else(|| "".to_string()),
                }).unwrap();

                println!("{} ({})", page.title(), page.url());

                wr.commit().unwrap();
            }
        }
    });

    //info!("initializing crawler");
    //let cr = Crawler::new(tx).await;

    //info!("starting crawler");
    //tokio::spawn(async move {
    //    cr.run().await.unwrap();
    //});

    let query_parser = QueryParser::for_index(&index, vec![title, body]);
    //let searcher = index.reader().unwrap().searcher();
    //let res = searcher.search(&query_parser.parse_query("").unwrap(), &TopDocs::with_limit(20)).unwrap();
    //println!("{} {}", searcher.num_docs(), res.len());
    let reader = index.reader().unwrap();

    /*
    loop {
        let mut input = String::new();
        if stdin.read_line(&mut input).unwrap() > 0 {

            let query = query_parser.parse_query(&input).unwrap();
            let results: Vec<(Score, DocAddress)> =
                searcher.search(&query, &TopDocs::with_limit(20)).unwrap();
            for (_score, doc_address) in results {
                // Retrieve the actual content of documents given its `doc_address`.
                let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address).unwrap();
                println!("{}", retrieved_doc.to_json(&schema));
            }
        }
    }
    */

    let count_cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap())));

    let db = sled::open("searched-db").unwrap();

    info!("initializing web");
    let r = Router::new()
        .route("/", get(web::search))
        .route("/search", get(web::results))
        .route("/assets/dragynfruit.png", get(web::dragynfruit_logo))
        .with_state(AppState {
            //index,
            count_cache,
            reader,
            query_parser,
            client,
            db,

            url,
            title,
            body,
        });

    tokio::spawn(async {
        axum::serve(TcpListener::bind("0.0.0.0:6969").await.unwrap(), r)
            .await
            .unwrap();
    });

    tokio::signal::ctrl_c().await.unwrap();
    info!("shutting down");
}

// fn main() {
//     let result = get_rate_page("http://www.varmintal.com/ahunt.htm".to_string()).unwrap();
//     println!("{:?}", result.rating);
// }

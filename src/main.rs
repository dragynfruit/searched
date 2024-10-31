#[macro_use]
extern crate log;
extern crate axum;
extern crate env_logger;
extern crate reqwest;
//extern crate tantivy;
extern crate tera;
#[macro_use]
extern crate serde;
//extern crate lru;
extern crate searched;

mod web;

use axum::{
    http::{HeaderMap, HeaderValue},
    routing::get,
    Router,
};
use log::LevelFilter;
//use reqwest::Client;
use searched::{
    config::Config,
    lua_api::PluginEngine,
};
//use sled::Db;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    //client: Client,
    //db: Db,
    //config: Config,
    eng: PluginEngine,
}

// Need more worker threads if we do our own search index again:
//   #[tokio::main(worker_threads = 12)]
#[tokio::main]
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

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    info!("Starting up...");

    let eng = PluginEngine::new(client).await.unwrap();

    info!("initializing web");
    let r = Router::new()
        .route("/", get(web::search))
        .route("/search", get(web::results))
        .route("/settings", get(web::settings))
        .route("/assets/logo.png", get(web::logo))
        .route("/favicon.ico", get(web::icon))
        .with_state(AppState { eng });

    tokio::spawn(async {
        axum::serve(
            TcpListener::bind("0.0.0.0:6969").await.unwrap(),
            r.into_make_service(),
        )
        .await
        .unwrap();
    });

    tokio::signal::ctrl_c().await.unwrap();
    info!("shutting down");
}

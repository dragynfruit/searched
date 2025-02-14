extern crate log;
extern crate axum;
extern crate env_logger;
extern crate reqwest;
extern crate tera;
extern crate serde;
extern crate searched;
extern crate sled;

mod web;
mod settings;
mod favicon;

use axum::{
    http::{HeaderMap, HeaderValue}, middleware
};
use log::{info, LevelFilter};
use searched::lua_support::PluginEngine;
use tokio::net::TcpListener;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    eng: PluginEngine,
    client: Client,
    db: sled::Db,
}

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

    let db = sled::open("data/db").unwrap();

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    info!("Starting up...");

    let eng = PluginEngine::new(client.clone()).await.unwrap();

    info!("initializing web");
    let app = web::router()
        .with_state(AppState { eng, client: client.clone(), db })
        .layer(middleware::from_fn(settings::settings_middleware));

    tokio::spawn(async {
        axum::serve(
            TcpListener::bind("0.0.0.0:6969").await.unwrap(),
            app.into_make_service(),
        )
        .await
        .unwrap();
    });

    tokio::signal::ctrl_c().await.unwrap();
    info!("shutting down");
}

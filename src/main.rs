extern crate axum;
extern crate env_logger;
extern crate log;
extern crate reqwest;
extern crate searched;
extern crate serde;
extern crate sled;
extern crate tera;

mod settings;
mod web;
mod widgets;

mod modules {
    pub mod favicon;
    pub mod image_proxy;
    pub mod text_matcher;
    pub mod url_cleaner;
}

use axum::{
    http::{HeaderMap, HeaderValue},
    middleware,
};
use log::{debug, error, info, LevelFilter};
use modules::url_cleaner;
use reqwest::Client;
use searched::lua_support::PluginEngine;
use std::process;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    eng: PluginEngine,
    client: Client,
    db: sled::Db,
}

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();

    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    info!("Starting searched...");
    debug!("Configuring HTTP client");

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

    debug!("Opening database");
    let db = sled::open("data/db").unwrap();

    info!("Initializing components...");

    // Initialize tracking rules if enabled
    debug!("Checking URL tracking rules");
    url_cleaner::ensure_rules_exist().await;

    debug!("Initializing plugin engine");
    let eng = PluginEngine::new(client.clone()).await.unwrap();

    info!("Setting up web server");
    let app = web::router()
        .with_state(AppState {
            eng,
            client: client.clone(),
            db,
        })
        .layer(middleware::from_fn(settings::settings_middleware));

    let bind_addr = "0.0.0.0:6969";
    info!("Starting web server on {}", bind_addr);

    tokio::spawn(async move {
        match TcpListener::bind(bind_addr).await {
            Ok(listener) => {
                if let Err(e) = axum::serve(listener, app.into_make_service()).await {
                    error!("Server error: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to bind to {}: {}", bind_addr, e);
                process::exit(1);
            }
        }
    });

    info!("Server started successfully");
    info!("Startup completed in {}ms", start.elapsed().as_millis());

    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("Shutdown signal received"),
        Err(e) => error!("Failed to listen for shutdown signal: {}", e),
    }

    info!("Shutting down gracefully...");
}

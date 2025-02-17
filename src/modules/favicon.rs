use crate::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use image::{
    codecs::png::PngEncoder,
    imageops::{resize, FilterType},
    DynamicImage, ImageFormat,
};
use log::debug;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Deserialize)]
pub struct FaviconParams {
    domain: String,
}

async fn try_load_image(client: &reqwest::Client, url: &Url) -> Option<DynamicImage> {
    if let Ok(response) = client.get(url.clone()).send().await {
        if response.status().is_success() {
            if let Ok(bytes) = response.bytes().await {
                // Try different formats
                if let Ok(image) = image::load_from_memory_with_format(&bytes, ImageFormat::Ico) {
                    return Some(image);
                }
                if let Ok(image) = image::load_from_memory(&bytes) {
                    return Some(image);
                }
            }
        }
    }
    None
}

async fn find_favicon(client: &reqwest::Client, domain_url: &Url) -> Option<DynamicImage> {
    debug!("Finding favicon for domain: {}", domain_url);
    // 1. Try /favicon.ico
    let mut favicon_urls = vec![];

    if let Ok(favicon_url) = domain_url.join("/favicon.ico") {
        favicon_urls.push(favicon_url);
    }

    // 2. Try parsing HTML for link tags
    if let Ok(response) = client.get(domain_url.clone()).send().await {
        if let Ok(html) = response.text().await {
            let document = Html::parse_document(&html);
            let selector =
                Selector::parse(r#"link[rel~="icon"], link[rel~="shortcut icon"]"#).unwrap();

            favicon_urls.extend(
                document
                    .select(&selector)
                    .filter_map(|element| element.value().attr("href"))
                    .filter_map(|href| domain_url.join(href).ok()),
            );
        }
    }

    // Try all collected URLs
    for url in favicon_urls {
        if let Some(image) = try_load_image(client, &url).await {
            return Some(image);
        }
    }

    // 3. Try Google's favicon service
    let google_url = format!(
        "https://www.google.com/s2/favicons?sz=32&domain={}",
        domain_url.host_str().unwrap_or_default()
    );
    if let Ok(url) = Url::parse(&google_url) {
        if let Some(image) = try_load_image(client, &url).await {
            return Some(image);
        }
    }

    // 4. Use local default favicon as last resort
    image::open("static/favicon.ico").ok()
}

fn pack_favicon_data(png_data: &[u8]) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut packed = now.to_be_bytes().to_vec();
    packed.extend_from_slice(png_data);
    packed
}

fn unpack_favicon_data(packed: &[u8]) -> Option<Vec<u8>> {
    if packed.len() < 8 {
        return None;
    }

    let timestamp = u64::from_be_bytes(packed[..8].try_into().unwrap());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if now - timestamp > 604800 {
        return None;
    }

    Some(packed[8..].to_vec())
}

fn build_favicon_response(data: Option<Vec<u8>>) -> Response {
    match data {
        Some(png_data) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "image/png")
            .header(header::CACHE_CONTROL, "public, max-age=604800")
            .header(
                header::EXPIRES,
                chrono::Utc::now()
                    .checked_add_days(chrono::Days::new(7))
                    .unwrap()
                    .format("%a, %d %b %Y %H:%M:%S GMT")
                    .to_string(),
            )
            .body(Body::from(png_data))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

#[axum::debug_handler]
pub async fn favicon(
    Query(params): Query<FaviconParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    debug!("Handling favicon request for domain: {}", params.domain);
    let favicon_db = state.db.open_tree("favicon").unwrap();
    let mut favicon_data = None;

    if let Ok(domain_url) = Url::parse(&params.domain) {
        let host = domain_url.host_str().unwrap_or("").to_string();

        if let Ok(Some(cached_favicon)) = favicon_db.get(host.as_bytes()) {
            if let Some(data) = unpack_favicon_data(&cached_favicon) {
                favicon_data = Some(data);
            } else {
                favicon_db.remove(host.as_bytes()).unwrap();
            }
        }

        if favicon_data.is_none() {
            if let Some(image) = find_favicon(&state.client, &domain_url).await {
                let resized = resize(&image, 32, 32, FilterType::Nearest);
                let mut png_data = Vec::new();

                if resized
                    .write_with_encoder(PngEncoder::new(&mut Cursor::new(&mut png_data)))
                    .is_ok()
                {
                    let save_png_data = png_data.clone();
                    tokio::spawn(async move {
                        let packed_data = pack_favicon_data(&save_png_data);
                        favicon_db.insert(host.as_bytes(), packed_data).unwrap();
                    });

                    favicon_data = Some(png_data);
                }
            }
        }
    }

    build_favicon_response(favicon_data)
}

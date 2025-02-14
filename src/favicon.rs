use axum::{
    response::{IntoResponse, Response},
    extract::{Query, State},
    http::{header, StatusCode},
    body::Body,
};
use image::{imageops::resize, ImageFormat, DynamicImage};
use scraper::{Html, Selector};
use serde::Deserialize;
use url::Url;
use std::io::Cursor;
use crate::AppState;

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
    // 1. Try /favicon.ico
    let mut favicon_urls = vec![];
    
    if let Ok(favicon_url) = domain_url.join("/favicon.ico") {
        favicon_urls.push(favicon_url);
    }

    // 2. Try parsing HTML for link tags
    if let Ok(response) = client.get(domain_url.clone()).send().await {
        if let Ok(html) = response.text().await {
            let document = Html::parse_document(&html);
            let selector = Selector::parse(r#"link[rel~="icon"], link[rel~="shortcut icon"]"#).unwrap();
            
            // Collect all URLs first before async operations
            favicon_urls.extend(
                document.select(&selector)
                    .filter_map(|element| element.value().attr("href"))
                    .filter_map(|href| domain_url.join(href).ok())
            );
        }
    }

    // Try all collected URLs
    for url in favicon_urls {
        if let Some(image) = try_load_image(client, &url).await {
            return Some(image);
        }
    }

    // 3. Try Google's favicon service as last resort
    let google_url = format!("https://www.google.com/s2/favicons?sz=32&domain={}", domain_url.host_str().unwrap_or_default());
    if let Ok(url) = Url::parse(&google_url) {
        if let Some(image) = try_load_image(client, &url).await {
            return Some(image);
        }
    }

    None
}

#[axum::debug_handler]
pub async fn favicon(
    Query(params): Query<FaviconParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let favicon_db = state.db.open_tree("favicon").unwrap();
    
    if let Ok(domain_url) = Url::parse(&params.domain) {
        let host = domain_url.host_str().unwrap_or("").to_string();
        
        // Try to get cached favicon first
        if let Ok(Some(cached_favicon)) = favicon_db.get(host.as_bytes()) {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/png")
                .body(Body::from(cached_favicon.to_vec()))
                .unwrap();
        }

        // Try multiple methods to find favicon
        if let Some(image) = find_favicon(&state.client, &domain_url).await {
            let resized = resize(&image, 32, 32, image::imageops::FilterType::Lanczos3);
            let mut png_data = Vec::new();
            let mut cursor = Cursor::new(&mut png_data);
            
            if resized.write_with_encoder(image::codecs::png::PngEncoder::new(&mut cursor)).is_ok() {
                let png_data = cursor.into_inner();
                favicon_db.insert(host.as_bytes(), png_data.as_slice()).unwrap();
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "image/png")
                    .body(Body::from(png_data.to_vec()))
                    .unwrap();
            }
        }
    }
    
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

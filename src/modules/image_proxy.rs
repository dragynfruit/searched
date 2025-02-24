use crate::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use log::debug;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

static ALLOWED_CONTENT_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/svg+xml",
];

#[derive(Deserialize)]
pub struct ImageParams {
    url: String,
}

fn pack_image_data(image_data: &[u8], content_type: &str) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut packed = now.to_be_bytes().to_vec();

    // Add content type length and content type
    let content_type_bytes = content_type.as_bytes();
    packed.extend_from_slice(&(content_type_bytes.len() as u8).to_be_bytes());
    packed.extend_from_slice(content_type_bytes);

    // Add image data
    packed.extend_from_slice(image_data);
    packed
}

fn unpack_image_data(packed: &[u8]) -> Option<(Vec<u8>, String)> {
    if packed.len() < 9 {
        // 8 bytes timestamp + 1 byte content type length
        return None;
    }

    let timestamp = u64::from_be_bytes(packed[..8].try_into().unwrap());
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Cache for 7 days
    if now - timestamp > 604800 {
        return None;
    }

    let content_type_len = packed[8] as usize;
    let content_type_start = 9;
    let content_type_end = content_type_start + content_type_len;

    if content_type_end >= packed.len() {
        return None;
    }

    let content_type =
        String::from_utf8_lossy(&packed[content_type_start..content_type_end]).to_string();
    let image_data = packed[content_type_end..].to_vec();

    Some((image_data, content_type))
}

fn build_image_response(data: Option<(Vec<u8>, String)>) -> Response {
    match data {
        Some((image_data, content_type)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "public, max-age=604800")
            .header(
                header::EXPIRES,
                chrono::Utc::now()
                    .checked_add_days(chrono::Days::new(7))
                    .unwrap()
                    .format("%a, %d %b %Y %H:%M:%S GMT")
                    .to_string(),
            )
            .body(Body::from(image_data))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}

fn is_allowed_content_type(content_type: &str) -> bool {
    ALLOWED_CONTENT_TYPES.contains(&content_type)
}

#[axum::debug_handler]
pub async fn proxy_image(
    Query(params): Query<ImageParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    debug!("Handling image proxy request for URL: {}", params.url);
    let image_db = state.db.open_tree("image_proxy").unwrap();
    let mut image_data = None;

    let mut hasher = DefaultHasher::new();
    params.url.hash(&mut hasher);
    let url_hash = hasher.finish().to_string();

    if let Ok(Some(cached_image)) = image_db.get(url_hash.as_bytes()) {
        image_data = unpack_image_data(&cached_image);
        if image_data.is_none() {
            image_db.remove(url_hash.as_bytes()).unwrap();
        }
    }

    if image_data.is_none() {
        if let Ok(response) = state.client.get(&params.url).send().await {
            if response.status().is_success() {
                if let Some(content_type) = response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|h| h.to_str().ok())
                    .filter(|ct| ct.starts_with("image/"))
                    .map(|s| s.to_string())
                {
                    if !is_allowed_content_type(&content_type) {
                        return Response::builder()
                            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                            .body(Body::empty())
                            .unwrap();
                    }

                    if let Ok(bytes) = response.bytes().await {
                        let image_bytes = bytes.to_vec();

                        // Save to cache
                        let save_data = image_bytes.clone();
                        let save_content_type = content_type.clone();
                        tokio::spawn(async move {
                            let packed_data = pack_image_data(&save_data, &save_content_type);
                            image_db.insert(url_hash.as_bytes(), packed_data).unwrap();
                        });

                        image_data = Some((image_bytes, content_type));
                    }
                }
            }
        }
    }

    build_image_response(image_data)
}

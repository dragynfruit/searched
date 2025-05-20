use log::{debug, error, info};
use reqwest::Client;
use std::{path::PathBuf, process, sync::Arc};

use axum::http::{StatusCode, header};
use axum::{
    Router,
    extract::{Extension, Query, State},
    middleware,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use once_cell::sync::Lazy;
use searched::{Error, Kind, SearchResult};
use serde::Deserialize;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use tokio::try_join;
use tower_http::services::ServeDir;

use crate::modules::favicon::favicon;
use crate::modules::image_proxy::proxy_image;
use crate::{
    AppState,
    modules::{text_matcher::highlight_text, url_cleaner},
    settings::{
        Settings, export_settings, import_settings, import_settings_form, settings_middleware,
        update_settings,
    },
    widgets,
};

pub static TERA: Lazy<Arc<RwLock<Tera>>> = Lazy::new(|| {
    info!("Initializing Tera templates");
    let tera = create_tera();
    Arc::new(RwLock::new(tera))
});

const MOTD: &'static [&'static str] = &[
    "<i>Blazingly</i> fast",
    "RIIR",
    "\"It's not a cult\"",
    "Search for sigmas",
    "Install Gentoo",
    "Don't be evil",
    "mikerowesoft.com",
    "Check out <a href=\"https://github.com/dragynfruit/pasted\">pasted</a>!",
    "Drink water",
    "The cake is a lie",
    "<a onclick=\"alert('Python bad')\" href=\"#\">Click me!</a>",
    "We <3 <a href=\"https://archive.org\">IA</a>",
    "<i>\"If it is on the Internet then it must be true.\"</i><br>&mdash; Abraham Lincoln",
    "<i>\"skibidi sigma ohio rizzler\"</i><br>&mdash; Drake",
    "<i>\"ok I'll do it tmrw\"</i><br>&mdash; Lincoln",
    "<i>\"Never gonna let you down\"</i><br>&mdash; Rick Astley",
    "<a href=\"https://phoenix4533.org/\">Phoenix 4533</a>",
    "I strongly dislike css",
    "Look ma, no JS!",
    "What's technical debt?",
    "Compiling searched v0.1.0",
    "Clinton the cat >>> Louis",
];

fn get_motd() -> &'static str {
    MOTD[fastrand::usize(..MOTD.len())]
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchParams {
    q: Option<String>,
    k: Option<Kind>,
    s: Option<String>,
    p: Option<usize>,
}

fn create_tera() -> Tera {
    info!("Loading Tera templates from views/**/*");
    let tera = match Tera::new("views/**/*") {
        Ok(t) => t,
        Err(e) => {
            error!("Template parsing error(s): {}", e);
            process::exit(1);
        }
    };

    tera
}

pub async fn index(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    debug!("Handling index request");
    let mut context = Context::new();
    context.insert("settings", &settings);
    context.insert("motd", get_motd());

    let rendered = TERA.read().await.render("index.tera", &context).unwrap();
    Html(rendered).into_response()
}

// Modify helper function to return Result
async fn detect_widget_async(
    query: &str,
    client: &Client,
    db: &sled::Db,
    settings: &Settings,
) -> Result<Option<widgets::Widget>, ()> {
    Ok(if !settings.enable_widgets {
        None
    } else {
        widgets::detect_widget(query, client, db, settings).await
    })
}

pub async fn search_results(
    Extension(settings): Extension<Settings>,
    Query(params): Query<SearchParams>,
    State(st): State<AppState>,
) -> impl IntoResponse {
    debug!("Handling search request with params: {:?}", params);

    let mut context = Context::new();
    context.insert("settings", &settings);

    if let Some(q) = params.q {
        let kind = params.k.unwrap_or_default();
        let query = searched::Query {
            query: q.clone(),
            kind: kind.clone(),
            page: params.p.unwrap_or(1),
            safe: settings.safesearch.clone(),
            ..Default::default()
        };

        let search_start = std::time::Instant::now();

        // Run widget detection and search concurrently with proper Result handling
        let (widget_option, mut search_results) = try_join!(
            detect_widget_async(&q, &st.client, &st.db, &settings),
            //async { Ok(st.eng.search(query.clone(), params.s.clone().unwrap_or("duckduckgo".to_string())).await.unwrap()) as Result<_, ()> }
            async { Ok(st.eng.search(query.clone(), vec!["duckduckgo".to_owned(), "stract".to_owned(), "qwant".to_owned(), "mojeek".to_owned(), "ask".to_owned()]).await.unwrap()) as Result<_, ()> }
        )
        .unwrap_or((None, Vec::<SearchResult>::new()));

        // Process search results
        for result in &mut search_results {
            if settings.bold_terms {
                result.title = highlight_text(&result.title, &q);
            }

            if settings.remove_tracking {
                result.url = url_cleaner::clean_url(result.url.clone());
            }
        }

        let search_time = search_start.elapsed().as_millis();
        debug!("Search completed in {}ms", search_time);

        // Add widget if detected
        if let Some(widget) = widget_option {
            context.insert("widget", &widget);
        }

        // Use the Kind's string value for the template
        match kind {
            Kind::General => context.insert("kind", "sear"),
            Kind::Images => context.insert("kind", "imgs"),
            Kind::Videos => context.insert("kind", "vids"),
            Kind::News => context.insert("kind", "news"),
            Kind::Maps => context.insert("kind", "maps"),
            Kind::Wiki => context.insert("kind", "wiki"),
            Kind::QuestionAnswer => context.insert("kind", "qans"),
            Kind::Documentation => context.insert("kind", "docs"),
            Kind::Papers => context.insert("kind", "pprs"),
        }

        context.insert("query", &query);
        context.insert("results", &search_results);
        context.insert("search_time", &search_time);

        let rendered = TERA.read().await.render("results.tera", &context).unwrap();
        Html(rendered).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

pub async fn settings_page(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);

    let rendered = TERA.read().await.render("settings.tera", &context).unwrap();
    Html(rendered).into_response()
}

pub async fn about_page(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);
    let rendered = TERA.read().await.render("about.tera", &context).unwrap();
    Html(rendered).into_response()
}

pub async fn opensearch() -> impl IntoResponse {
    let xml = include_str!("../static/opensearch.xml");
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/opensearchdescription+xml",
        )
        .body(xml.to_string())
        .unwrap()
}

// Router configuration
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/search", get(search_results))
        .route("/settings", get(settings_page))
        .route("/settings/update", post(update_settings))
        .route("/settings/export", get(export_settings))
        .route("/settings/import", post(import_settings))
        .route("/settings/import_form", post(import_settings_form))
        .route("/about", get(about_page))
        .route("/favicon", get(favicon))
        .route("/image", get(proxy_image))
        .route("/opensearch.xml", get(opensearch))
        .fallback_service(ServeDir::new(PathBuf::from("static")))
        .layer(middleware::from_fn(settings_middleware))
}

use std::{path::PathBuf, process, sync::Arc};

use axum::{
    extract::{Extension, Query, State},
    middleware,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Router,
};
use once_cell::sync::Lazy;
use regex::Regex;
use searched::Kind;
use serde::Deserialize;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

use crate::{
    settings::{
        export_settings, import_settings, import_settings_form, settings_middleware,
        update_settings, Settings,
    },
    AppState,
};
use crate::{text_matcher::highlight_text, widgets};

pub static TERA: Lazy<Arc<RwLock<Tera>>> = Lazy::new(|| {
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
];

fn get_motd() -> &'static str {
    MOTD[fastrand::usize(..MOTD.len())]
}

#[derive(Deserialize, Default)]
pub struct SearchParams {
    q: Option<String>,
    k: Option<Kind>,
    s: Option<String>,
    p: Option<usize>,
}

fn strip_html_tags(text: &str) -> String {
    static TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());
    TAG_RE.replace_all(text, "").into_owned()
}

fn create_tera() -> Tera {
    let tera = match Tera::new("views/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            process::exit(1);
        }
    };

    tera
}

pub async fn index(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);
    context.insert("motd", get_motd());

    let rendered = TERA.read().await.render("index.html", &context).unwrap();
    Html(rendered).into_response()
}

pub async fn search_results(
    Extension(settings): Extension<Settings>,
    Query(params): Query<SearchParams>,
    State(st): State<AppState>,
) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);

    if let Some(q) = params.q {
        if q == "rust" {
            return Redirect::to("https://rust-lang.org").into_response();
        }

        // Check for widgets if enabled
        if settings.enable_widgets {
            if let Some(widget) = widgets::detect_widget(&q, &st.client).await {
                context.insert("widget", &widget);
            }
        }

        let kind = params.k.unwrap_or_default();
        let query = searched::Query {
            provider: params.s.clone().unwrap_or("duckduckgo".to_string()),
            query: q.clone(),
            kind: kind.clone(),
            page: params.p.unwrap_or(1),
            safe: settings.safesearch.clone(),
            ..Default::default()
        };

        let search_start = std::time::Instant::now();
        let mut results = if query.provider == "all" {
            vec![] // TODO: Implement all-provider search
        } else {
            let mut results = st.eng.search(query.clone()).await;

            // Clean and process results
            for result in &mut results {
                // Strip HTML from title and snippet
                result.title = strip_html_tags(&result.title);
                if let Some(general) = &mut result.general {
                    if let Some(snippet) = &general.snippet {
                        general.snippet = Some(strip_html_tags(snippet));
                    }
                }

                // Only highlight titles if enabled
                if settings.bold_terms {
                    result.title = highlight_text(&result.title, &q);
                }
            }

            results
        };

        // Clean tracking from URLs if enabled
        if settings.remove_tracking {
            for result in &mut results {
                result.url = crate::url_cleaner::clean_url(&result.url);
            }
        }

        let search_time = search_start.elapsed().as_millis();

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
        context.insert("results", &results);
        context.insert("search_time", &search_time);

        let rendered = TERA.read().await.render("results.html", &context).unwrap();
        Html(rendered).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}

pub async fn settings_page(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);

    let rendered = TERA.read().await.render("settings.html", &context).unwrap();
    Html(rendered).into_response()
}

pub async fn about_page(Extension(settings): Extension<Settings>) -> impl IntoResponse {
    let mut context = Context::new();
    context.insert("settings", &settings);
    let rendered = TERA.read().await.render("about.html", &context).unwrap();
    Html(rendered).into_response()
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
        .route("/favicon", get(crate::favicon::favicon))
        .fallback_service(ServeDir::new(PathBuf::from("static")))
        .layer(middleware::from_fn(settings_middleware))
}

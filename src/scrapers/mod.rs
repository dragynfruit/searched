//pub mod google;
pub mod duckduckgo;
pub mod stackexchange;
pub mod bandcamp;

use duckduckgo::Duckduckgo;
use searched::Query;
use stackexchange::StackExchange;
use bandcamp::Bandcamp;

use crate::AppState;

pub trait Scraper: Sized {
    const HEADERS: &'static [&'static str];

    async fn search(state: AppState, query: Query) -> Vec<searched::Result>;
}

pub async fn search(scraper: &str, state: AppState, query: Query) -> Vec<searched::Result> {
    match scraper {
        "duckduckgo" => Duckduckgo::search(state, query).await,
        "stackexchange" => StackExchange::search(state, query).await,
        "bandcamp" => Bandcamp::search(state, query).await,
        &_ => unimplemented!(),
    }
}

use scraper::{Html, Selector};
use url::Url;

use super::Scraper;

pub struct Bandcamp;
impl Scraper for Bandcamp {
    const HEADERS: &'static [&'static str] = &[];

    async fn search(state: crate::AppState, query: searched::Query) -> Vec<searched::Result> {
        let res = state.client.get(Url::parse_with_params("https://bandcamp.com/search", &[
                ("q", query.query.clone()),
                ("page", query.page.to_string())
        ]).unwrap()).send().await.unwrap().text().await.unwrap();

        let html = Html::parse_document(&res);
        for result in html.select(&Selector::parse("li[class=searchresult]").unwrap()) {
            println!("{}", result.html());
        }

        vec![]
    }
}

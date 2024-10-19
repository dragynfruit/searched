use url::Url;

use super::Scraper;

#[derive(Deserialize)]
struct Res {
    items: Vec<ResItem>,
}

#[derive(Deserialize)]
struct ResItem {
    link: String,
    title: String,
    tags: Vec<String>,
}

pub struct StackExchange;
impl Scraper for StackExchange {
    const HEADERS: &'static [&'static str] = &[];

    async fn search(state: crate::AppState, query: searched::Query) -> Vec<searched::Result> {
        let data: Res = state
            .client
            .get(
                Url::parse_with_params(
                    "https://api.stackexchange.com/2.3/search/advanced",
                    &[
                        ("q", query.query),
                        ("page", query.page.to_string()),
                        ("site", "stackoverflow".to_string()),
                    ],
                )
                .unwrap(),
            )
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        data.items
            .into_iter()
            .map(|item| searched::Result {
                title: item.title,
                url: item.link,
                general: Some(searched::GeneralResult {
                    snippet: item.tags.join(" "),
                }),
                ..Default::default()
            })
            .collect()
    }
}

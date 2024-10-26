use std::sync::Arc;

use scraper::{Html, Selector};
use url::Url;

use crate::ranking::RankingEngine;

pub struct Page {
    url: Url,
    title: String,
    links: Vec<String>,
    dom: Arc<Html>,
}
impl Page {
    pub fn new(url: Url, html: String) -> Self {
        let dom = Html::parse_document(&html);

        let links = dom
            .select(&Selector::parse("a[href]").unwrap())
            .filter_map(|element| {
                let new_url = element.value().attr("href").unwrap();
                if new_url.contains("#")
                    || new_url.starts_with("mailto:")
                    || new_url.starts_with("tel:")
                {
                    None
                } else if new_url.starts_with("http") {
                    Some(new_url.to_string())
                } else {
                    Some(
                        Url::options()
                            .base_url(Some(&url))
                            .parse(new_url)
                            .unwrap()
                            .to_string(),
                    )
                }
            })
            .collect();

        let title = dom
            .select(&Selector::parse("title").unwrap())
            .next()
            .map(|element| element.text().collect())
            .unwrap_or_else(|| "".to_string());

        //let content = dom
        //    .select(&Selector::parse("body").unwrap())
        //    .next()
        //    .map(|element| element.text().collect())
        //    .unwrap_or_else(|| "".to_string());

        Self {
            url,
            title,
            links,
            dom: Arc::new(dom),
        }
    }

    #[inline(always)]
    pub fn url(&self) -> Url {
        self.url.clone()
    }

    #[inline(always)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[inline(always)]
    pub fn dom(&self) -> Arc<Html> {
        self.dom.clone()
    }

    #[inline(always)]
    pub fn links(&self) -> Vec<String> {
        self.links.clone()
    }

    pub async fn rank(&self) -> i32 {
        RankingEngine::rank(&self.dom).await
    }
}
unsafe impl Send for Page {}
unsafe impl Sync for Page {}

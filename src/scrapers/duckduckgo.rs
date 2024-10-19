use std::collections::HashMap;

use scraper::{Html, Selector};

use super::Scraper;

pub struct Duckduckgo;
impl Scraper for Duckduckgo {
    const HEADERS: &'static [&'static str] = &[];
    async fn search(state: crate::AppState, query: searched::Query) -> Vec<searched::Result> {
        let mut results = Vec::new();

        let mut req = state
            .client
            .post("https://lite.duckduckgo.com/lite/")
            .header("Content-Type", "application/x-www-form-urlencoded");

        let vqds = state.db.open_tree("duckduckgo-vqds").unwrap();

        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("q", query.query.clone());

        let mut offset = 0;
        let mut cookies = String::new();

        if query.page > 1 {
            if query.page == 2 {
                offset = (query.page - 1) * 20;
            } else if query.page > 2 {
                offset = 20 + (query.page - 2) * 50;
            }
            form.insert("s", offset.to_string());
            form.insert("nextParams", String::from(""));
            form.insert("v", String::from("l"));
            form.insert("o", String::from("json"));
            form.insert("dc", (offset + 1).to_string());
            form.insert("api", String::from("d.js"));
            req = req.header("Referer", String::from("https://lite.duckduckgo.com/"));
            form.insert(
                "vqd",
                String::from_utf8(vqds.get(&query.query).unwrap().unwrap().to_vec()).unwrap(),
            );
            form.insert("kl", String::from("wt-wt"));
            cookies.push_str("kl=wt-wt");
        }

        let res = req.form(&form).send().await.unwrap().text().await.unwrap();

        let html = Html::parse_document(&res);

        if !vqds.contains_key(&query.query).unwrap() {
            let vqd = html
                .select(&Selector::parse("input[name=vqd]").unwrap())
                .next()
                .unwrap()
                .attr("value")
                .unwrap();
            vqds.insert(query.query.as_str(), vqd).unwrap();
        }

        let link_sel = Selector::parse("a.result-link").unwrap();
        let link_sels = html.select(&link_sel);

        let snippet_sel = Selector::parse("td.result-snippet").unwrap();
        let snippet_sels = html.select(&snippet_sel);

        let result_sels = link_sels.zip(snippet_sels);

        for (link, snippet) in result_sels {
            let url = link.attr("href").unwrap().to_string();
            let title = link.inner_html();
            let body_preview = snippet.inner_html();

            results.push(searched::Result {
                url,
                title,
                general: Some(searched::GeneralResult {
                    snippet: body_preview,
                }),
                ..Default::default()
            });
        }

        results
    }
}

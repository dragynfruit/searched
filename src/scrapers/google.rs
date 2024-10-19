use reqwest::header::{self, HeaderMap, HeaderValue};
use url::Url;

pub struct Google;
impl Google {
    pub async fn get() {
        let mut headers = HeaderMap::new();
        for (key, val) in [
            ("User-Agent", ""),
            ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
            ("Accept-Language", "en-US,en;q=0.5"),
            ("Accept-Encoding", "gzip"),
            ("DNT", "1"),
            ("Connection", "keep-alive"),
            ("Upgrade-Insecure-Requests", "1"),
            ("Sec-Fetch-Dest", "document"),
            ("Sec-Fetch-Mode", "navigate"),
            ("Sec-Fetch-Site", "none"),
            ("Sec-Fetch-User", "?1"),
            ("Priority", "u=1"),
            ("TE", "trailers"),
        ] {
            headers.append(key, HeaderValue::from_str(val).unwrap());
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let res = client
            .get(Url::parse_with_params("https://google.com/search", [("q", "rust lang")]).unwrap())
            .send()
            .await
            .unwrap();
        //println!("{}", res.text().await.unwrap());
    }
}

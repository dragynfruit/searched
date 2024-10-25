use std::{
    error::Error,
    time::{Duration, Instant},
};

use axum::http::{HeaderMap, HeaderValue};
use reqwest::{
    header::{self, CONTENT_TYPE},
    Client,
};
use sled::Db;
use texting_robots::{get_robots_url, Robot};
use tokio::sync::mpsc;
use url::Url;

use crate::page::Page;

const CRAWL_WORKERS: usize = 5;
const CRAWL_DELAY: Duration = Duration::from_millis(100);

/// A web crawler
#[derive(Clone)]
pub struct Crawler {
    db: Db,
    chan: mpsc::UnboundedSender<Page>,
}
impl Crawler {
    pub async fn new(chan: mpsc::UnboundedSender<Page>) -> Self {
        let db = sled::open("data/db").unwrap();

        let queue = db.open_tree("queue").unwrap();
        if queue.is_empty() {
            queue
                .insert("https://www.apple.com".as_bytes().to_vec(), vec![])
                .unwrap();
        }

        Self { db, chan }
    }
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        for i in 0..CRAWL_WORKERS {
            let cr = self.clone();
            tokio::spawn(async move {
                info!("starting worker {i}");
                cr.crawl().await.unwrap();
            });
        }

        Ok(())
    }
    /// Run a web crawling worker
    pub async fn crawl(&self) -> Result<(), Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_str("Crawled")?);

        let client = Client::builder().default_headers(headers).build()?;

        let links = self.db.open_tree("links").unwrap();
        let queue = self.db.open_tree("queue").unwrap();

        loop {
            let url = queue.pop_min()?.clone();
            if let Some((url, _)) = url {
                // If we've already indexed this url, skip it
                if links.contains_key(url.clone())? {
                    continue;
                }

                let url = Url::parse(core::str::from_utf8(&url)?)?;

                let robots_url = get_robots_url(url.as_str()).unwrap();

                let robots_res = client.get(robots_url).send().await.unwrap();
                let robot = Robot::new("Crawled", &robots_res.bytes().await.unwrap()).unwrap();

                if !robot.allowed(url.as_str()) {
                    debug!("skipped url due to robots.txt: {url}");
                    continue;
                }

                let st = Instant::now();

                match client.get(url.as_str()).send().await {
                    Ok(res) => {
                        if res
                            .headers()
                            .get(CONTENT_TYPE)
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .contains("text/html")
                        {
                            let html = res.text().await?;

                            let page = Page::new(url.clone(), html);

                            for l in page.links() {
                                let link = l.as_bytes().to_vec();
                                queue.insert(link, vec![]).unwrap();
                            }

                            self.chan.send(page).unwrap();
                        }

                        links.insert(url.as_str().as_bytes(), vec![]).unwrap();

                        debug!("page took {:?} to crawl: {url}", st.elapsed());
                    }
                    Err(_) => {
                        continue;
                    }
                }

                tokio::time::sleep(CRAWL_DELAY).await;
            }
        }
    }
}

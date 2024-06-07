use std::{
    collections::HashSet,
    error::Error,
    future::IntoFuture,
    sync::Arc,
    time::{Duration, Instant},
};

use reqwest::{header::CONTENT_TYPE, Client};
use sled::Db;
use tokio::sync::mpsc;
use url::Url;

use crate::page::Page;

const CRAWL_WORKERS: usize = 100;
const CRAWL_DELAY: Duration = Duration::from_millis(20);

/// A web crawler
#[derive(Clone)]
pub struct Crawler {
    db: Db,
    chan: mpsc::UnboundedSender<Page>,
}
impl Crawler {
    pub async fn new(chan: mpsc::UnboundedSender<Page>) -> Self {
        let db = sled::open("searched-db").unwrap();

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
        let client = Client::new();
        let links = self.db.open_tree("links").unwrap();
        let queue = self.db.open_tree("queue").unwrap();

        loop {
            let url = queue.pop_min()?.clone();
            if let Some((url, _)) = url {
                // If we've already indexed this url, skip it
                if links.contains_key(url.clone())? {
                    continue;
                }

                let st = Instant::now();

                let url = Url::parse(core::str::from_utf8(&url)?)?;
                let res = client.get(url.as_str()).send().await?;
                if res.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap().contains("text/html") {
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

            tokio::time::sleep(CRAWL_DELAY).await;
        }
    }
}

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate reqwest;
extern crate tantivy;

mod crawler;
mod page;
mod ranking;

use std::{fs, io, time::Duration};

use crawler::Crawler;
use log::LevelFilter;
use page::Page;
use scraper::Selector;
use tokio::sync::mpsc;

use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Schema, FAST, STORED, TEXT},
    store::{Compressor, ZstdCompressor},
    DocAddress, Document, Index, IndexSettings, Score, TantivyDocument,
};

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).parse_default_env().init();

    info!("Starting up...");

    let (tx, mut rx): (mpsc::UnboundedSender<Page>, mpsc::UnboundedReceiver<Page>) =
        mpsc::unbounded_channel();

    let mut schema = Schema::builder();

    let url = schema.add_text_field("url", TEXT | FAST | STORED);
    let title = schema.add_text_field("title", TEXT | FAST | STORED);
    let body = schema.add_text_field("body", TEXT | STORED);

    let schema = schema.build();

    let index = match Index::open_in_dir("searched-index") {
        Ok(index) => index,
        Err(_) => {
            warn!("no existing index found, creating one");

            fs::create_dir_all("searched-index").unwrap();

            Index::builder()
                .schema(schema.clone())
                .settings(IndexSettings {
                    docstore_compression: Compressor::Zstd(ZstdCompressor {
                        compression_level: Some(10),
                    }),
                    ..Default::default()
                })
                .create_in_dir("searched-index")
                .unwrap()
        }
    };
    let wr = index.writer(100_000_000).unwrap();

    tokio::spawn(async move {
        let body_sel = Selector::parse("body").unwrap();

        loop {
            if let Some(page) = rx.recv().await {
                wr.add_document(doc! {
                    url => page.url().to_string(),
                    title => page.title(),
                    body => page.dom().select(&body_sel).next().map(|element| element.text().collect()).unwrap_or_else(|| "".to_string()),
                }).unwrap();
            }
        }
    });

    info!("initializing crawler");
    let cr = Crawler::new(tx).await;

    tokio::spawn(async move {
        cr.run().await.unwrap();
    });

    let query_parser = QueryParser::for_index(&index, vec![title, body]);
    let searcher = index.reader().unwrap().searcher();

    let stdin = io::stdin();
    /*
    loop {
        let mut input = String::new();
        if stdin.read_line(&mut input).unwrap() > 0 {
            let query = query_parser.parse_query(&input).unwrap();
            let results: Vec<(Score, DocAddress)> =
                searcher.search(&query, &TopDocs::with_limit(20)).unwrap();
            for (_score, doc_address) in results {
                // Retrieve the actual content of documents given its `doc_address`.
                let retrieved_doc = searcher.doc::<TantivyDocument>(doc_address).unwrap();
                println!("{}", retrieved_doc.to_json(&schema));
            }
        }
    }
    */

    tokio::signal::ctrl_c().await.unwrap();
    info!("shutting down");
}

// fn main() {
//     let result = get_rate_page("http://www.varmintal.com/ahunt.htm".to_string()).unwrap();
//     println!("{:?}", result.rating);
// }

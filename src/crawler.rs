use core::time::Duration;
use std::collections::BinaryHeap;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::analyser::Analyser;
use crate::downloader::Downloader;
use crate::value::ScoredValue;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct HtmlUrl {
    pub html: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Crawler {
    analyser: Arc<Analyser>,
    downloader: Arc<Downloader>,
    htmls: Arc<Mutex<BinaryHeap<ScoredValue<HtmlUrl>>>>,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            analyser: Arc::new(Analyser::new()),
            downloader: Arc::new(Downloader::new()),
            htmls: Arc::new(Mutex::new(BinaryHeap::new())),
        }
    }

    pub async fn run(&self) {
        let downloader = self.run_downloader();
        let downloader2 = self.run_downloader();
        let analyser = self.run_analyser();
        downloader.await.await.unwrap();
        downloader2.await.await.unwrap();
        analyser.await.unwrap();
    }

    async fn run_downloader(&self) -> JoinHandle<()> {
        let analyser = Arc::clone(&self.analyser);
        let downloader = Arc::clone(&self.downloader);
        let htmls = Arc::clone(&self.htmls);
        tokio::spawn(async move {
            loop {
                let Some(next_url_to_download) = analyser.next_link().await else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("Downloader has nothing to do");
                    continue;
                };

                let ScoredValue { value: url, score } = next_url_to_download;

                if let Some(html) = downloader.download_html(&url).await {
                    println!("Downloaded {url}");
                    let scored_html = ScoredValue {
                        score,
                        value: HtmlUrl { html, url },
                    };

                    htmls.lock().await.push(scored_html);
                } else {
                    eprintln!("Failed to download {url}")
                }
            }
        })
    }

    fn run_analyser(&self) -> JoinHandle<()> {
        let analyser = Arc::clone(&self.analyser);
        let htmls = Arc::clone(&self.htmls);
        tokio::spawn(async move {
            loop {
                let Some(next_html_to_analyse) = htmls.lock().await.pop() else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("Analyser has nothing to do");
                    continue;
                };

                println!("Analysing {}", next_html_to_analyse.value.url);
                analyser.analyse_html(next_html_to_analyse.value).await;
            }
        })
    }
}

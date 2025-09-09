use core::time::Duration;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::analyser::analyse_html;
use crate::downloader::url_to_html;
use crate::value::ScoredValue;

type ArMx<T> = Arc<Mutex<T>>;
pub type SafePriority<T> = ArMx<BinaryHeap<ScoredValue<T>>>;

#[derive(Default, Debug)]
pub struct Crawler {
    urls_to_download: SafePriority<String>,
    htmls_to_analyse: SafePriority<(String, String)>,
    visited_pages: ArMx<HashMap<String, usize>>,
}

impl Crawler {
    pub fn new() -> Self {
        Self {
            urls_to_download: Arc::new(Mutex::new(
                vec![ScoredValue {
                    value: "https://www.google.com".to_owned(),
                    score: 10,
                }]
                .into_iter()
                .collect(),
            )),
            ..Default::default()
        }
    }

    pub async fn run(&self) {
        let downloader = self.run_downloader();
        let analyser = self.run_analyser();
        downloader.await.unwrap();
        analyser.await.unwrap();
    }

    fn run_downloader(&self) -> JoinHandle<()> {
        let urls = self.urls_to_download.clone();
        let htmls = self.htmls_to_analyse.clone();
        tokio::spawn(async move {
            let mut count = 0;
            loop {
                let new_url = urls.lock().await.pop();

                let Some(next_url_to_download) = new_url else {
                    if count == 60 {
                        break;
                    } else {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        count += 1;
                        continue;
                    }
                };

                let ScoredValue { value: url, score } = next_url_to_download;

                let html = url_to_html(&url).await;

                let scored_html = ScoredValue {
                    value: (url, html),
                    score,
                };

                dbg!(scored_html.score);

                htmls.lock().await.push(scored_html);
            }
        })
    }

    fn run_analyser(&self) -> JoinHandle<()> {
        let urls_to_download = self.urls_to_download.clone();
        let htmls_to_analyse = self.htmls_to_analyse.clone();
        let visited_pages = self.visited_pages.clone();
        tokio::spawn(async move {
            let mut count = 0;
            loop {
                let new_html = htmls_to_analyse.lock().await.pop();

                let Some(next_html_to_analyse) = new_html else {
                    if count == 60 {
                        break;
                    } else {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        count += 1;
                        continue;
                    }
                };
                count = 0;

                let (current_url, current_html) = next_html_to_analyse.value;

                let (new_urls, score) = analyse_html(&current_html).await;
                let mut urls_lock = urls_to_download.lock().await;
                for url in new_urls {
                    urls_lock.push(url);
                }

                visited_pages.lock().await.insert(current_url, score);
            }
        })
    }
}

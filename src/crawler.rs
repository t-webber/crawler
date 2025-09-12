use core::time::Duration;
use std::collections::BinaryHeap;
use std::process::exit;
use std::sync::Arc;

use crossterm::event;
use crossterm::terminal;
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
        let handles = vec![
            self.run_downloader(),
            self.run_downloader(),
            self.run_downloader(),
            self.run_analyser(),
            self.listener(),
        ];
        for handle in handles {
            handle.await.unwrap();
        }
    }

    fn listener(&self) -> JoinHandle<()> {
        let analyser = Arc::clone(&self.analyser);
        terminal::enable_raw_mode().unwrap();
        tokio::spawn(async move {
            loop {
                let event = event::read().unwrap();
                if let Some(key) = event.as_key_event()
                    && key.is_press()
                    && key.code.is_char('q')
                {
                    terminal::disable_raw_mode().unwrap();
                    analyser.create_report().await;
                    exit(0);
                }
            }
        })
    }

    fn run_downloader(&self) -> JoinHandle<()> {
        let analyser = Arc::clone(&self.analyser);
        let downloader = Arc::clone(&self.downloader);
        let htmls = Arc::clone(&self.htmls);
        tokio::spawn(async move {
            loop {
                let Some(next_url_to_download) = analyser.next_link().await else {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    println!("\rDownloader has nothing to do\r");
                    continue;
                };

                let ScoredValue { value: url, score } = next_url_to_download;

                if let Some(html) = downloader.download_html(&url).await {
                    println!("\rDownloaded {url}\r");
                    let scored_html = ScoredValue {
                        score,
                        value: HtmlUrl { html, url },
                    };

                    htmls.lock().await.push(scored_html);
                } else {
                    eprintln!("\rFailed to download {url}\r")
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
                    println!("\rAnalyser has nothing to do\r");
                    continue;
                };

                println!("\rAnalysing {}\r", next_html_to_analyse.value.url);
                analyser.analyse_html(next_html_to_analyse.value).await;
            }
        })
    }
}

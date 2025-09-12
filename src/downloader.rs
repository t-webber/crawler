use reqwest::Client;
use reqwest::redirect::Policy;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; RustCrawler/1.0)")
            .redirect(Policy::limited(20))
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();
        Self { client }
    }

    pub async fn download_html(&self, url: &str) -> Option<String> {
        let mut delay = Duration::from_secs(1);

        for _ in 0..5 {
            match self.client.get(url).send().await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        return Some(text);
                    }
                }
                Err(_) => {
                    println!("\rDownloader delayed\r");
                    sleep(delay).await;
                    delay *= 2;
                }
            }
        }

        None
    }
}

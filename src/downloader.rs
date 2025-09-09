use reqwest::{Client, redirect};
use std::time::Duration;
use tokio::time::sleep;

pub async fn url_to_html(url: &str) -> String {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; RustCrawler/1.0)")
        .redirect(redirect::Policy::limited(20))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let mut delay = Duration::from_secs(1);

    for _ in 0..5 {
        match client.get(url).send().await {
            Ok(resp) => {
                if let Ok(text) = resp.text().await {
                    println!("[DOWNLOAD] url: {url}");
                    return text;
                }
            }
            Err(_) => {
                sleep(delay).await;
                delay *= 2;
            }
        }
    }

    String::new()
}

use crate::crawler::Crawler;

mod analyser;
mod crawler;
mod downloader;
mod value;

#[tokio::main]
async fn main() {
    let crawler = Crawler::new();
    crawler.run().await;
    dbg!(crawler);
}

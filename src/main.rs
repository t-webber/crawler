use core::time::Duration;
use std::collections::{HashMap, HashSet};
use std::io::stdin;
use std::sync::{Arc, LazyLock, Mutex};
use std::{fs, thread};

struct Crawler {
    to_download: HashSet<String>,
    to_analyse: HashMap<String, String>,
    visited: HashMap<String, usize>,
}

impl Crawler {
    const VISITED: &str = "data/visited.txt";
    const TO_DOWNLOAD: &str = "data/to_download.txt";

    fn new() -> Self {
        let mut to_download = HashSet::new();
        let content = fs::read_to_string(Self::TO_DOWNLOAD).unwrap();
        for line in content.lines() {
            to_download.insert(line.to_owned());
        }
        fs::remove_file(Self::TO_DOWNLOAD).unwrap();

        let mut visited = HashMap::new();
        let content = fs::read_to_string(Self::VISITED).unwrap();
        for line in content.lines() {
            let mut split = line.split('\t');
            let url = split.next().unwrap().to_string();
            let score = split.next().unwrap().parse().unwrap();
            visited.insert(url, score);
        }
        fs::remove_file(Self::VISITED).unwrap();

        Self {
            visited,
            to_download,
            to_analyse: HashMap::new(),
        }
    }

    fn analyse_next(&mut self) {}

    fn download_next(&mut self) {}

    fn kill(&mut self) {}
}

static SHUTDOWN_TIMEOUT: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(120));

fn main() {
    let crawler = Arc::new(Mutex::new(Crawler::new()));

    let download_crawler = crawler.clone();
    let downloader = thread::spawn(move || {
        loop {
            download_crawler.lock().unwrap().download_next();
            if download_crawler.lock().unwrap().to_download.is_empty() {
                thread::sleep(*SHUTDOWN_TIMEOUT);
                if download_crawler.lock().unwrap().to_download.is_empty() {
                    break;
                }
            }
        }
    });

    let analysis_crawler = crawler.clone();
    let analyser = thread::spawn(move || {
        loop {
            analysis_crawler.lock().unwrap().analyse_next();
            if analysis_crawler.lock().unwrap().to_analyse.is_empty() {
                thread::sleep(*SHUTDOWN_TIMEOUT);
                if analysis_crawler.lock().unwrap().to_analyse.is_empty() {
                    break;
                }
            }
        }
    });

    let cleanup_crawler = crawler.clone();
    let _ = thread::spawn(move || {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        cleanup_crawler.lock().unwrap().kill();
    });

    downloader.join().unwrap();
    analyser.join().unwrap();

    crawler.lock().unwrap().kill();
}

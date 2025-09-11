use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::Write as _;
use std::panic::catch_unwind;

use html_filter::prelude::{Filter, Html};
use tokio::sync::Mutex;

use crate::crawler::HtmlUrl;
use crate::value::ScoredValue;

#[derive(Debug)]
pub struct Analyser {
    filter: Filter,
    links: Mutex<HashSet<String>>,
    priority_links: Mutex<BinaryHeap<ScoredValue<String>>>,
    discovered_links: Mutex<HashMap<String, usize>>,
}

const INITIAL_LINKS: &[&str] = &[];

impl Analyser {
    pub fn new() -> Self {
        let mut priority_links = BinaryHeap::new();
        for &link in INITIAL_LINKS {
            priority_links.push(ScoredValue {
                value: link.to_owned(),
                score: 100,
            });
        }
        Self {
            filter: Filter::new().tag_name("a"),
            links: Mutex::new(HashSet::new()),
            priority_links: Mutex::new(priority_links),
            discovered_links: Mutex::new(HashMap::new()),
        }
    }

    pub async fn analyse_html(&self, HtmlUrl { html, url }: HtmlUrl) {
        let score = html_to_score(&html);
        if let Err(err) = self.html_to_links(&html, score, &url).await {
            eprintln!("Failed to analyse {url}: {err}");
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open("errors.txt")
                .unwrap();
            writeln!(file, "Failed to analyse {url}: {err}").unwrap();
        }
        self.discovered_links.lock().await.insert(url, score);
    }

    async fn push_link(&self, link: &str, score: usize, parent_link: &str) {
        let resolved_link = if link.starts_with("http") || link.starts_with("www") {
            link.to_owned()
        } else {
            let mut split = parent_link.split('/').collect::<Vec<_>>();
            let len = split.len();
            if len >= 2 {
                split[len - 1] = link;
            } else {
                split.push(link);
            }
            split.join("/")
        };

        let mut links = self.links.lock().await;
        if !links.contains(&resolved_link) {
            links.insert(resolved_link.clone());
            self.priority_links.lock().await.push(ScoredValue {
                score,
                value: resolved_link,
            });
        }
    }

    pub async fn next_link(&self) -> Option<ScoredValue<String>> {
        self.priority_links.lock().await.pop()
    }

    async fn tree_to_links(&self, html: Html, score: usize, parent_link: &str) {
        let mut nodes: Vec<Html> = vec![html];
        let mut count = 0;
        while let Some(node) = nodes.pop() {
            match node {
                Html::Tag { tag, child, .. } => {
                    if let Some(href) = tag.find_attr_value("href") {
                        count += 1;
                        self.push_link(href, score, parent_link).await;
                    }
                    nodes.push(*child);
                }
                Html::Vec(htmls) => {
                    for node in htmls {
                        nodes.push(node)
                    }
                }
                _ => (),
            }
        }
        println!("Added {count} links")
    }

    async fn html_to_links(
        &self,
        html: &str,
        score: usize,
        parent_link: &str,
    ) -> Result<(), String> {
        let ast = catch_unwind(|| Html::parse(html)).map_err(|err| format!("{err:?}"))??;
        let filtered_tree = ast.filter(&self.filter);
        self.tree_to_links(filtered_tree, score, parent_link).await;
        Ok(())
    }
}

const WEIGHTS: &[(&str, usize)] = &[
    ("rust", 5),
    ("linux", 5),
    ("stage", 13),
    ("intern", 15),
    ("stagiaire", 14),
    ("internship", 5),
    ("c++", 4),
    ("assembl", 4),
    ("embedded", 4),
    ("kernel", 4),
    ("operating system", 3),
    ("software", 3),
    ("engineer", 3),
    ("science", 1),
    ("techonology", 1),
    ("architecture", 1),
    ("processor", 5),
    ("distributed system", 5),
];

fn html_to_score(html: &str) -> usize {
    let lowercase_html = html.to_lowercase();
    WEIGHTS
        .iter()
        .map(|(word, weight)| lowercase_html.matches(word).count() * weight)
        .sum()
}

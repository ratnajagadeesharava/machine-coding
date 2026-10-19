#![allow(unused_imports, clippy::needless_range_loop)]
use p81_parallel_crawler::{contract::*, create};
use playground_support::sut;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct Ring {
    n: usize,
    fetches: AtomicUsize,
}

impl Fetcher for Ring {
    fn links(&self, url: &str) -> Result<Vec<String>, FetchError> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        let i: usize = url[1..].parse().map_err(|_| FetchError)?;
        let mut out: Vec<String> = (1..=3).map(|k| format!("p{}", (i * 3 + k) % self.n)).collect();
        out.push(format!("p{}", (i + 1) % self.n));
        Ok(out)
    }
}

#[test]
fn should_crawl_a_hundred_thousand_pages_quickly() {
    let c = sut!(create());
    let n = 100_000;
    let web = Arc::new(Ring { n, fetches: AtomicUsize::new(0) });

    let reached = c.crawl("p0", 16, web.clone()).unwrap();

    assert_eq!(reached.len(), n);
    assert_eq!(web.fetches.load(Ordering::SeqCst), n);
}

struct Deep(usize);

impl Fetcher for Deep {
    fn links(&self, url: &str) -> Result<Vec<String>, FetchError> {
        thread::sleep(Duration::from_millis(2));
        let i: usize = url[1..].parse().map_err(|_| FetchError)?;
        Ok(if i + 1 < self.0 { vec![format!("p{}", i + 1)] } else { vec![] })
    }
}

#[test]
fn should_keep_crawling_when_every_page_is_slow_and_the_graph_is_deep() {
    let c = sut!(create());

    let reached = c.crawl("p0", 32, Arc::new(Deep(300))).unwrap();

    assert_eq!(reached.len(), 300);
}

use p64_iterator::contract::*;
use p64_iterator::{filter_lazy, PagedFeed};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A fake remote API over fixed pages; counts and can fail fetches.
struct FakePages {
    pages: Vec<Vec<&'static str>>,
    fetched: Rc<RefCell<Vec<usize>>>,
    fail_on_page: Rc<Cell<Option<usize>>>,
}

impl PageFetcher for FakePages {
    fn fetch(&self, page: usize) -> Result<Page, FeedError> {
        self.fetched.borrow_mut().push(page);
        if self.fail_on_page.get() == Some(page) {
            return Err(FeedError(format!("remote error on page {page}")));
        }
        Ok(Page { items: self.pages[page].iter().map(|s| s.to_string()).collect(), has_next: page + 1 < self.pages.len() })
    }
}

struct Api {
    feed: PagedFeed,
    fetched: Rc<RefCell<Vec<usize>>>,
    fail_on_page: Rc<Cell<Option<usize>>>,
}

fn api(pages: Vec<Vec<&'static str>>) -> Api {
    let fetched = Rc::new(RefCell::new(Vec::new()));
    let fail_on_page = Rc::new(Cell::new(None));
    let feed = PagedFeed::new(Rc::new(FakePages { pages, fetched: fetched.clone(), fail_on_page: fail_on_page.clone() }));
    Api { feed, fetched, fail_on_page }
}

fn ok(s: &str) -> Option<Result<String, FeedError>> {
    Some(Ok(s.to_string()))
}

#[test]
fn should_fetch_nothing_until_somebody_asks_for_an_item() {
    let a = api(vec![vec!["a", "b"], vec!["c"]]);

    let mut it = a.feed.iter();

    assert!(a.fetched.borrow().is_empty());
    assert_eq!(it.next(), ok("a"));
    assert_eq!(*a.fetched.borrow(), vec![0]);
}

#[test]
fn should_yield_every_item_in_order_across_pages() {
    let a = api(vec![vec!["a", "b"], vec!["c", "d"], vec!["e"]]);

    let all: Vec<String> = a.feed.iter().map(|r| r.unwrap()).collect();

    assert_eq!(all, vec!["a", "b", "c", "d", "e"]);
}

#[test]
fn should_fetch_the_next_page_only_when_the_current_one_is_used_up_and_more_is_requested() {
    let a = api(vec![vec!["a", "b"], vec!["c", "d"], vec!["e"]]);
    let mut it = a.feed.iter();

    assert_eq!(it.next(), ok("a"));
    assert_eq!(it.next(), ok("b"));
    assert_eq!(*a.fetched.borrow(), vec![0]);
    assert_eq!(it.next(), ok("c"));
    assert_eq!(*a.fetched.borrow(), vec![0, 1]);
}

#[test]
fn should_stop_fetching_when_the_consumer_stops_early() {
    let a = api(vec![vec!["a", "b"], vec!["c", "d"], vec!["e", "f"]]);

    let first_three: Vec<String> = a.feed.iter().take(3).map(|r| r.unwrap()).collect();

    assert_eq!(first_three, vec!["a", "b", "c"]);
    assert_eq!(*a.fetched.borrow(), vec![0, 1]);
}

#[test]
fn should_skip_empty_pages() {
    let a = api(vec![vec![], vec!["a"], vec![], vec![], vec!["b"]]);

    let all: Vec<String> = a.feed.iter().map(|r| r.unwrap()).collect();

    assert_eq!(all, vec!["a", "b"]);
    assert_eq!(*a.fetched.borrow(), vec![0, 1, 2, 3, 4]);
}

#[test]
fn should_keep_returning_none_at_the_end_without_fetching_again() {
    let a = api(vec![vec!["a"]]);
    let mut it = a.feed.iter();

    assert_eq!(it.next(), ok("a"));
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);
    assert_eq!(*a.fetched.borrow(), vec![0]);
}

#[test]
fn should_give_every_caller_an_independent_iterator() {
    let a = api(vec![vec!["a", "b"], vec!["c"]]);

    let mut one = a.feed.iter();
    let mut two = a.feed.iter();

    assert_eq!(one.next(), ok("a"));
    assert_eq!(two.next(), ok("a"));
    assert_eq!(one.next(), ok("b"));
    assert_eq!(one.next(), ok("c"));
    assert_eq!(two.next(), ok("b"));
    assert_eq!(two.next(), ok("c"));
    assert_eq!(one.next(), None);
    assert_eq!(two.next(), None);
}

#[test]
fn should_report_fetch_failures_and_retry_the_same_page_next_time() {
    let a = api(vec![vec!["a", "b"], vec!["c"]]);
    a.fail_on_page.set(Some(1));
    let mut it = a.feed.iter();
    assert_eq!(it.next(), ok("a"));
    assert_eq!(it.next(), ok("b"));

    assert_eq!(it.next(), Some(Err(FeedError("remote error on page 1".to_string()))));
    a.fail_on_page.set(None);

    assert_eq!(it.next(), ok("c"));
    assert_eq!(*a.fetched.borrow(), vec![0, 1, 1]);
}

#[test]
fn should_filter_lazily_pulling_no_more_than_needed() {
    let pulled = Rc::new(Cell::new(0));
    let p = pulled.clone();
    let source = (1..=10).inspect(move |_| p.set(p.get() + 1));
    let mut evens = filter_lazy(source, |n| n % 2 == 0);

    assert_eq!(pulled.get(), 0);
    assert_eq!(evens.next(), Some(2));
    assert_eq!(pulled.get(), 2);
    assert_eq!(evens.next(), Some(4));
    assert_eq!(pulled.get(), 4);
}

#[test]
fn should_filter_a_paged_feed_and_end_cleanly() {
    let a = api(vec![vec!["aa", "b", "cc"], vec!["d", "ee"]]);
    let mut two_letters = filter_lazy(a.feed.iter().map(|r| r.unwrap()), |s: &String| s.len() == 2);

    let collected: Vec<String> = two_letters.by_ref().collect();

    assert_eq!(collected, vec!["aa", "cc", "ee"]);
    assert_eq!(two_letters.next(), None);
}

//! Iterator — a lazy, paginated feed and a lazy filter, both as real `Iterator` implementations.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::marker::PhantomData;
use std::rc::Rc;

/// A feed backed by a paginated remote API. `iter()` returns a NEW, independent iterator starting at page 0.
pub struct PagedFeed;

impl PagedFeed {
    /// Nothing is fetched here.
    pub fn new(fetcher: Rc<dyn PageFetcher>) -> PagedFeed {
        todo!()
    }

    pub fn iter(&self) -> FeedIter {
        todo!()
    }
}

/// The lazy iterator over a feed. Add whatever fields you need.
///
/// - creating it fetches nothing; the first `next()` fetches page 0;
/// - the next page is fetched only when the current page's items are used up AND `next()` is called again;
/// - empty pages (even the first) are skipped by fetching the following page;
/// - at the end `next()` keeps returning `None`;
/// - if the fetcher fails, `next()` returns `Some(Err(..))` and the iterator stays usable: the next call retries
///   the SAME page.
pub struct FeedIter;

impl Iterator for FeedIter {
    type Item = Result<String, FeedError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// A lazy filtering iterator: pulls from `inner` only as far as needed (never ahead of the element it returns).
pub struct FilterLazy<I, P>(PhantomData<(I, P)>);

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for FilterLazy<I, P> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// Wraps `inner` so that only elements accepted by `keep` come out.
pub fn filter_lazy<I: Iterator, P: FnMut(&I::Item) -> bool>(inner: I, keep: P) -> FilterLazy<I, P> {
    todo!()
}

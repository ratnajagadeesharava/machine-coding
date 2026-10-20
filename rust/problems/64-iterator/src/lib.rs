//! A paginated feed and a filtering iterator.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::marker::PhantomData;
use std::rc::Rc;

/// A feed backed by a paginated remote API. `iter()` returns a NEW, independent iterator starting at page 0.
pub struct PagedFeed;

impl PagedFeed {
    pub fn new(fetcher: Rc<dyn PageFetcher>) -> PagedFeed {
        todo!()
    }

    pub fn iter(&self) -> FeedIter {
        todo!()
    }
}

/// The iterator over a feed. Add whatever fields you need.
pub struct FeedIter;

impl Iterator for FeedIter {
    type Item = Result<String, FeedError>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// A filtering iterator.
pub struct FilterLazy<I, P>(PhantomData<(I, P)>);

impl<I: Iterator, P: FnMut(&I::Item) -> bool> Iterator for FilterLazy<I, P> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// Only elements accepted by `keep` come out.
pub fn filter_lazy<I: Iterator, P: FnMut(&I::Item) -> bool>(inner: I, keep: P) -> FilterLazy<I, P> {
    todo!()
}

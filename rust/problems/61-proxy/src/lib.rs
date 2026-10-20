//! Proxy.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Caching proxy: the real store is asked once per PRESENT document. Missing documents are not cached.
/// Blank id -> `Err(InvalidId)` (real store not asked).
pub struct CachingStoreProxy;

impl CachingStoreProxy {
    pub fn new(real: Box<dyn DocumentStore>) -> CachingStoreProxy {
        todo!()
    }

    /// Forgets the cached copy of one document (no effect if it is not cached).
    pub fn invalidate(&self, id: &str) {
        todo!()
    }

    /// Number of documents currently cached.
    pub fn cached_count(&self) -> usize {
        todo!()
    }
}

impl DocumentStore for CachingStoreProxy {
    fn fetch(&self, id: &str) -> Result<Option<String>, StoreError> {
        todo!()
    }
}

/// Virtual proxy: the factory is NOT called by `new`; it is called exactly once, on the first `fetch`, and the
/// store it returns is reused.
pub struct LazyStoreProxy;

impl LazyStoreProxy {
    pub fn new(factory: Box<dyn Fn() -> Box<dyn DocumentStore>>) -> LazyStoreProxy {
        todo!()
    }
}

impl DocumentStore for LazyStoreProxy {
    fn fetch(&self, id: &str) -> Result<Option<String>, StoreError> {
        todo!()
    }
}

/// Protection proxy: asks `policy.can_read(user, id)` before every fetch. If not allowed -> `Err(AccessDenied)`
/// and the real store is NEVER asked. Blank id -> `Err(InvalidId)` (policy not asked).
pub struct AccessControlStoreProxy;

impl AccessControlStoreProxy {
    pub fn new(real: Box<dyn DocumentStore>, policy: Box<dyn AccessPolicy>, user: &str) -> AccessControlStoreProxy {
        todo!()
    }
}

impl DocumentStore for AccessControlStoreProxy {
    fn fetch(&self, id: &str) -> Result<Option<String>, StoreError> {
        todo!()
    }
}

use p61_proxy::contract::*;
use p61_proxy::{AccessControlStoreProxy, CachingStoreProxy, LazyStoreProxy};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Shared<T> = Rc<RefCell<T>>;

/// The "real" subject, written in the test: counts every call that reaches it.
#[derive(Clone)]
struct CountingStore {
    documents: Shared<HashMap<String, String>>,
    requests: Shared<Vec<String>>,
}

impl CountingStore {
    fn new(docs: &[(&str, &str)]) -> Self {
        let map = docs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        CountingStore { documents: Rc::new(RefCell::new(map)), requests: Rc::new(RefCell::new(Vec::new())) }
    }
}

impl DocumentStore for CountingStore {
    fn fetch(&self, id: &str) -> Result<Option<String>, StoreError> {
        self.requests.borrow_mut().push(id.to_string());
        Ok(self.documents.borrow().get(id).cloned())
    }
}

struct Policy {
    asked: Shared<Vec<String>>,
    allow: fn(&str) -> bool,
}

impl AccessPolicy for Policy {
    fn can_read(&self, user: &str, document_id: &str) -> bool {
        self.asked.borrow_mut().push(format!("{user}:{document_id}"));
        (self.allow)(document_id)
    }
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_ask_the_real_store_only_once_per_document() {
    let real = CountingStore::new(&[("a", "alpha"), ("b", "beta")]);
    let cache = CachingStoreProxy::new(Box::new(real.clone()));

    assert_eq!(cache.fetch("a"), Ok(Some("alpha".to_string())));
    assert_eq!(cache.fetch("a"), Ok(Some("alpha".to_string())));
    assert_eq!(cache.fetch("b"), Ok(Some("beta".to_string())));
    assert_eq!(cache.fetch("a"), Ok(Some("alpha".to_string())));

    assert_eq!(*real.requests.borrow(), strs(&["a", "b"]));
    assert_eq!(cache.cached_count(), 2);
}

#[test]
fn should_not_cache_misses_because_the_document_may_appear_later() {
    let real = CountingStore::new(&[]);
    let cache = CachingStoreProxy::new(Box::new(real.clone()));

    assert_eq!(cache.fetch("late"), Ok(None));
    assert_eq!(cache.fetch("late"), Ok(None));
    real.documents.borrow_mut().insert("late".into(), "now here".into());

    assert_eq!(cache.fetch("late"), Ok(Some("now here".to_string())));
    assert_eq!(*real.requests.borrow(), strs(&["late", "late", "late"]));
    assert_eq!(cache.cached_count(), 1);
}

#[test]
fn should_forget_a_document_when_invalidated() {
    let real = CountingStore::new(&[("a", "v1")]);
    let cache = CachingStoreProxy::new(Box::new(real.clone()));
    cache.fetch("a").unwrap();
    real.documents.borrow_mut().insert("a".into(), "v2".into());

    assert_eq!(cache.fetch("a"), Ok(Some("v1".to_string())));
    cache.invalidate("a");
    cache.invalidate("never-cached");

    assert_eq!(cache.cached_count(), 0);
    assert_eq!(cache.fetch("a"), Ok(Some("v2".to_string())));
}

#[test]
fn should_build_the_real_store_only_when_first_needed_and_only_once() {
    let built = Rc::new(RefCell::new(0));
    let real = CountingStore::new(&[("a", "alpha")]);
    let (b, r) = (built.clone(), real.clone());
    let lazy = LazyStoreProxy::new(Box::new(move || {
        *b.borrow_mut() += 1;
        Box::new(r.clone())
    }));

    assert_eq!(*built.borrow(), 0);
    assert_eq!(lazy.fetch("a"), Ok(Some("alpha".to_string())));
    assert_eq!(lazy.fetch("zzz"), Ok(None));
    assert_eq!(lazy.fetch("a"), Ok(Some("alpha".to_string())));

    assert_eq!(*built.borrow(), 1);
    assert_eq!(*real.requests.borrow(), strs(&["a", "zzz", "a"]));
}

#[test]
fn should_check_the_policy_before_every_fetch_and_never_touch_the_real_store_when_denied() {
    let real = CountingStore::new(&[("secret", "s3cr3t"), ("public", "hello")]);
    let asked = Rc::new(RefCell::new(Vec::new()));
    let guarded = AccessControlStoreProxy::new(
        Box::new(real.clone()),
        Box::new(Policy { asked: asked.clone(), allow: |id| id.starts_with("public") }),
        "ann",
    );

    assert_eq!(guarded.fetch("public"), Ok(Some("hello".to_string())));
    assert_eq!(guarded.fetch("secret"), Err(StoreError::AccessDenied));

    assert_eq!(*asked.borrow(), strs(&["ann:public", "ann:secret"]));
    assert_eq!(*real.requests.borrow(), strs(&["public"]));
}

#[test]
fn should_reject_blank_ids_before_anything_else_happens() {
    let real = CountingStore::new(&[]);
    let asked = Rc::new(RefCell::new(Vec::new()));
    let guarded = AccessControlStoreProxy::new(
        Box::new(real.clone()),
        Box::new(Policy { asked: asked.clone(), allow: |_| true }),
        "ann",
    );
    let cache = CachingStoreProxy::new(Box::new(real.clone()));

    assert_eq!(guarded.fetch(" "), Err(StoreError::InvalidId));
    assert_eq!(cache.fetch(""), Err(StoreError::InvalidId));

    assert!(asked.borrow().is_empty());
    assert!(real.requests.borrow().is_empty());
}

#[test]
fn should_stack_proxies_because_each_one_is_also_a_document_store() {
    let real = CountingStore::new(&[("secret", "s3cr3t"), ("public", "hello")]);
    let built = Rc::new(RefCell::new(0));
    let (b, r) = (built.clone(), real.clone());
    let lazy = LazyStoreProxy::new(Box::new(move || {
        *b.borrow_mut() += 1;
        Box::new(r.clone())
    }));
    let asked = Rc::new(RefCell::new(Vec::new()));
    let guarded = AccessControlStoreProxy::new(
        Box::new(lazy),
        Box::new(Policy { asked, allow: |id| id == "public" }),
        "bob",
    );
    let stack = CachingStoreProxy::new(Box::new(guarded));

    assert_eq!(*built.borrow(), 0);
    assert_eq!(stack.fetch("secret"), Err(StoreError::AccessDenied));
    assert_eq!(*built.borrow(), 0);
    assert_eq!(stack.fetch("public"), Ok(Some("hello".to_string())));
    assert_eq!(stack.fetch("public"), Ok(Some("hello".to_string())));

    assert_eq!(*built.borrow(), 1);
    assert_eq!(*real.requests.borrow(), strs(&["public"]));
    assert_eq!(stack.cached_count(), 1);
}

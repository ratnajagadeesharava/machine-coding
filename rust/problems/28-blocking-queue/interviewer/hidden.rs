#![allow(unused_imports, clippy::needless_range_loop)]
use p28_blocking_queue::{contract::*, create};
use playground_support::{eventually, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn capacity_one_should_hand_over_every_item_in_order() {
    let b = sut!(create(1));
    let n = 2_000;
    {
        let b = b.clone();
        thread::spawn(move || {
            for i in 0..n {
                b.put(i).unwrap();
            }
            b.close();
        });
    }
    let mut got = Vec::new();

    while let Some(v) = b.poll(10_000) {
        got.push(v);
    }

    assert_eq!(got, (0..n).collect::<Vec<_>>());
}

#[test]
fn put_on_a_full_closed_buffer_should_fail_immediately() {
    let b = sut!(create(1));
    b.put(1).unwrap();
    b.close();
    let failure: Arc<Mutex<Option<Result<(), Closed>>>> = Arc::new(Mutex::new(None));
    {
        let (b, failure) = (b.clone(), failure.clone());
        thread::spawn(move || {
            let r = b.put(2);
            *failure.lock().unwrap() = Some(r);
        });
    }

    assert!(
        eventually(5_000, || failure.lock().unwrap().is_some()),
        "put must not wait for space on a closed buffer"
    );
    assert_eq!(*failure.lock().unwrap(), Some(Err(Closed)));
    assert_eq!(b.offer(3, 30_000), Err(Closed));
    assert_eq!(b.size(), 1);
}

#[test]
fn several_consumers_draining_a_closed_buffer_should_get_every_item_exactly_once() {
    let b = sut!(create(100));
    for i in 0..100 {
        b.put(i).unwrap();
    }
    b.close();
    let got = Arc::new(Mutex::new(Vec::new()));
    let finished = Arc::new(AtomicUsize::new(0));
    for _ in 0..4 {
        let (b, got, finished) = (b.clone(), got.clone(), finished.clone());
        thread::spawn(move || {
            while let Some(v) = b.take() {
                got.lock().unwrap().push(v);
            }
            finished.fetch_add(1, Ordering::SeqCst);
        });
    }

    assert!(
        eventually(10_000, || finished.load(Ordering::SeqCst) == 4),
        "all consumers see the end"
    );

    let got = got.lock().unwrap();
    let unique: HashSet<_> = got.iter().copied().collect();
    assert_eq!(got.len(), 100);
    assert_eq!(unique.len(), 100);
}

#[test]
fn size_should_never_exceed_capacity_while_producers_race() {
    let b = sut!(create(3));
    let mut observed_max = 0;
    for _ in 0..6 {
        let b = b.clone();
        thread::spawn(move || {
            for i in 0..2_000 {
                b.put(i).unwrap();
            }
        });
    }

    let mut consumed = 0;
    while consumed < 12_000 {
        observed_max = observed_max.max(b.size());
        if b.poll(5_000).is_none() {
            break;
        }
        consumed += 1;
    }

    assert_eq!(consumed, 12_000);
    assert!(observed_max <= 3);
    assert_eq!(b.size(), 0);
}

#[test]
fn close_is_idempotent_and_an_empty_closed_buffer_never_blocks() {
    let b = sut!(create(2));

    b.close();
    b.close();

    assert_eq!(b.take(), None);
    assert_eq!(b.poll(30_000), None);
    assert_eq!(b.put(1), Err(Closed));
    assert_eq!(b.size(), 0);
}

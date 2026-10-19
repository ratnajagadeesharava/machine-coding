#![allow(unused_imports, clippy::needless_range_loop)]
use p78_connection_pool::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_sustain_fifty_thousand_leases_on_a_small_pool() {
    let p = sut!(create(5));
    let leases = AtomicUsize::new(0);

    run_concurrently(32, |_| {
        for _ in 0..1_600 {
            let id = p.acquire(30_000).unwrap().unwrap();
            leases.fetch_add(1, Ordering::SeqCst);
            p.release(id).unwrap();
        }
    });

    assert_eq!(leases.load(Ordering::SeqCst), 32 * 1_600);
    assert_eq!(p.available(), 5);
}

#[test]
fn should_keep_counts_consistent_while_observed() {
    let size = 4;
    let p = sut!(create(size));

    run_concurrently(9, |t| {
        if t == 8 {
            for _ in 0..20_000 {
                assert!(p.available() <= size);
                assert!(p.in_use() <= size);
            }
            return;
        }
        for _ in 0..2_000 {
            let id = p.acquire(30_000).unwrap().unwrap();
            p.release(id).unwrap();
        }
    });

    assert_eq!(p.available() + p.in_use(), size);
}

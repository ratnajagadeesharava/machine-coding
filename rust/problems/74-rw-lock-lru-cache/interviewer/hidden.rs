#![allow(unused_imports, clippy::needless_range_loop)]
use p74_rw_lock_lru_cache::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_serve_a_read_heavy_workload_with_a_few_writers() {
    let c = sut!(create(1_000));
    for k in 0..1_000u64 {
        c.put(k, format!("v{k}"));
    }
    let hits = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        let mut state = (t as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for i in 0..200_000u64 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let key = state % 1_500;
            if t == 0 && i % 50 == 0 {
                c.put(key, format!("v{key}"));
            } else if c.peek(key).is_some() {
                hits.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    assert!(hits.load(Ordering::Relaxed) > 0);
    assert!(c.len() <= 1_000);
}

#[test]
fn should_keep_working_when_every_operation_fights_over_one_key() {
    let c = sut!(create(3));

    run_concurrently(16, |t| {
        for i in 0..50_000 {
            c.put(7, format!("v{t}"));
            c.get(7);
            if i % 7 == 0 {
                c.remove(7);
            }
        }
    });

    assert!(c.len() <= 1);
}

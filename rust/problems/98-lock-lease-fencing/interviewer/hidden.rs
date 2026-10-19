#![allow(unused_imports, clippy::needless_range_loop)]
use p98_lock_lease_fencing::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_handle_a_hundred_thousand_independent_locks() {
    let clock = FakeClock::new(0);
    let l = sut!(create(clock.as_clock()));

    run_concurrently(8, |t| {
        for i in 0..12_500 {
            l.acquire(&format!("lock-{}", t * 12_500 + i), "o", 1_000).unwrap();
        }
    });

    assert_eq!(l.latest_token("lock-99999"), 1);
    assert_eq!(l.holder("lock-5"), Some("o".to_string()));
}

#[test]
fn should_keep_tokens_strictly_increasing_when_leases_expire_and_are_stolen_constantly() {
    let clock = FakeClock::new(0);
    let l = sut!(create(clock.as_clock()));
    let regressions = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        if t == 0 {
            for i in 0..1_000u64 {
                clock.advance(10); // the previous 5 ms lease has expired
                let token = l.acquire("hot", &format!("owner-{i}"), 5).unwrap().unwrap();
                if token != i + 1 {
                    regressions.fetch_add(1, Ordering::SeqCst);
                }
            }
            return;
        }
        let mut last = 0;
        for _ in 0..20_000 {
            let now = l.latest_token("hot");
            if now < last {
                regressions.fetch_add(1, Ordering::SeqCst);
            }
            last = now;
            l.holder("hot");
        }
    });

    assert_eq!(regressions.load(Ordering::SeqCst), 0);
    assert_eq!(l.latest_token("hot"), 1_000);
}

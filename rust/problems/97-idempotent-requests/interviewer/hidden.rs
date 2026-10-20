#![allow(unused_imports, clippy::needless_range_loop)]
use p97_idempotent_requests::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_execute_each_key_exactly_once_under_retries() {
    let clock = FakeClock::new(0);
    let p = sut!(create(clock.as_clock()));
    let executions = AtomicUsize::new(0);

    run_concurrently(8, |_| {
        for _ in 0..3 {
            for k in 0..12_500 {
                p.process(&format!("key-{k}"), &format!("body-{k}"), &|_: &str| {
                    Ok(format!("r{}", executions.fetch_add(1, Ordering::SeqCst)))
                })
                .unwrap();
            }
        }
    });

    assert_eq!(executions.load(Ordering::SeqCst), 12_500);
}

#[test]
fn should_not_leak_memory_for_expired_keys() {
    let clock = FakeClock::new(0);
    let p = sut!(create(clock.as_clock()));

    for day in 0..50 {
        for k in 0..2_000 {
            p.process(
                &format!("day{day}-{k}"),
                "b",
                &|_: &str| Ok("r".to_string()),
            )
            .unwrap();
        }
        clock.advance(25 * 3_600_000);
    }

    assert_eq!(
        p.process("fresh", "b", &|_: &str| Ok("ok".to_string())),
        Ok("ok".to_string())
    );
}

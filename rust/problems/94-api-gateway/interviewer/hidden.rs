#![allow(unused_imports, clippy::needless_range_loop)]
use p94_api_gateway::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn should_not_serialize_slow_backend_calls() {
    let clock = FakeClock::new(0);
    let g = sut!(create(clock.as_clock()));
    g.register_backend(
        "slow",
        Arc::new(|_: &str| {
            std::thread::sleep(Duration::from_millis(50));
            Ok("ok".to_string())
        }),
    );
    g.add_route("/slow", &["slow"]).unwrap();
    g.add_client("k", "c", 100_000);

    let start = Instant::now();
    run_concurrently(32, |_| {
        for _ in 0..5 {
            g.handle("k", "/slow/x");
        }
    });

    assert!(start.elapsed() < Duration::from_secs(3), "160 calls of 50 ms: serialised would take 8 s");
}

#[test]
fn should_open_the_circuit_within_a_few_calls_even_when_many_threads_hit_a_failing_backend() {
    let clock = FakeClock::new(0);
    let g = sut!(create(clock.as_clock()));
    let calls = Arc::new(AtomicUsize::new(0));
    let c = calls.clone();
    g.register_backend(
        "bad",
        Arc::new(move |_: &str| {
            c.fetch_add(1, Ordering::SeqCst);
            Err("down".to_string())
        }),
    );
    g.add_route("/bad", &["bad"]).unwrap();
    g.add_client("k", "c", 100_000);

    run_concurrently(16, |_| {
        for _ in 0..100 {
            g.handle("k", "/bad/x");
        }
    });

    assert!(calls.load(Ordering::SeqCst) <= 3 + 16, "at most 3 + the requests already in flight");
}

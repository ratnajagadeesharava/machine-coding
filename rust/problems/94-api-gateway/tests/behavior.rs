#![allow(unused_imports, clippy::needless_range_loop)]
use p94_api_gateway::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn echo(name: &'static str) -> Arc<dyn Backend> {
    Arc::new(move |path: &str| Ok(format!("{name}{path}")))
}

fn broken(down: &Arc<AtomicBool>, name: &'static str) -> Arc<dyn Backend> {
    let down = down.clone();
    Arc::new(move |path: &str| {
        if down.load(Ordering::SeqCst) {
            Err(format!("{name} is down"))
        } else {
            Ok(format!("{name}{path}"))
        }
    })
}

fn gateway() -> Option<(Arc<dyn Gateway>, FakeClock)> {
    let clock = FakeClock::new(1_000_000);
    Some((create(clock.as_clock())?, clock))
}

fn simple() -> Option<(Arc<dyn Gateway>, FakeClock)> {
    let (g, clock) = gateway()?;
    g.register_backend("a", echo("a"));
    g.register_backend("b", echo("b"));
    g.register_backend("c", echo("c"));
    g.add_route("/api", &["a", "b", "c"]).unwrap();
    g.add_client("key-1", "acme", 600);
    Some((g, clock))
}

fn status(g: &Arc<dyn Gateway>, key: &str, path: &str) -> u16 {
    g.handle(key, path).status
}

#[test]
fn should_answer_401_for_unknown_keys_and_not_consume_tokens() {
    let (g, _) = sut!(simple());

    assert_eq!(
        g.handle("nope", "/api/x"),
        Response::new(401, "unauthorized")
    );
    assert_eq!(g.handle("", "/api/x").status, 401);
}

#[test]
fn should_answer_404_without_consuming_a_token_when_nothing_matches() {
    let (g, _) = sut!(gateway());
    g.register_backend("a", echo("a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k", "tiny", 1);

    assert_eq!(g.handle("k", "/other"), Response::new(404, "no route"));
    assert_eq!(status(&g, "k", "/api/x"), 200); // the 404 did not use the only token
}

#[test]
fn should_pick_the_longest_matching_prefix() {
    let (g, _) = sut!(gateway());
    g.register_backend("general", echo("general"));
    g.register_backend("users", echo("users"));
    g.add_route("/api", &["general"]).unwrap();
    g.add_route("/api/users", &["users"]).unwrap();
    g.add_client("k", "c", 100);

    assert_eq!(
        g.handle("k", "/api/users/42"),
        Response::new(200, "users/api/users/42")
    );
    assert_eq!(
        g.handle("k", "/api/orders"),
        Response::new(200, "general/api/orders")
    );
}

#[test]
fn should_reject_bad_routes() {
    let (g, _) = sut!(gateway());
    g.register_backend("a", echo("a"));

    assert_eq!(
        g.add_route("/x", &["ghost"]),
        Err(GatewayError::InvalidRoute)
    );
    assert_eq!(g.add_route("/x", &[]), Err(GatewayError::InvalidRoute));
}

#[test]
fn should_round_robin_over_the_backends() {
    let (g, _) = sut!(simple());

    let bodies: Vec<String> = (0..6).map(|_| g.handle("key-1", "/api/p").body).collect();

    assert_eq!(
        bodies,
        ["a/api/p", "b/api/p", "c/api/p", "a/api/p", "b/api/p", "c/api/p"]
    );
    assert_eq!(g.success_count("acme"), Ok(6));
}

#[test]
fn should_fail_over_to_the_next_backend_when_one_fails() {
    let (g, _) = sut!(gateway());
    let down = Arc::new(AtomicBool::new(true));
    g.register_backend("a", broken(&down, "a"));
    g.register_backend("b", echo("b"));
    g.add_route("/api", &["a", "b"]).unwrap();
    g.add_client("k", "c", 100);

    assert_eq!(g.handle("k", "/api/1"), Response::new(200, "b/api/1"));

    assert_eq!(g.call_count("a"), Ok(1));
    assert_eq!(g.call_count("b"), Ok(1));
}

#[test]
fn should_answer_502_when_every_backend_fails() {
    let (g, _) = sut!(gateway());
    let down = Arc::new(AtomicBool::new(true));
    g.register_backend("a", broken(&down, "a"));
    g.register_backend("b", broken(&down, "b"));
    g.add_route("/api", &["a", "b"]).unwrap();
    g.add_client("k", "c", 100);

    assert_eq!(g.handle("k", "/api/1"), Response::new(502, "bad gateway"));
    assert_eq!(g.success_count("c"), Ok(0));
}

/// Two backends, `a` initially down; returns the gateway, the clock and the `a` switch.
fn with_flaky_a() -> Option<(Arc<dyn Gateway>, FakeClock, Arc<AtomicBool>)> {
    let (g, clock) = gateway()?;
    let a_down = Arc::new(AtomicBool::new(true));
    g.register_backend("a", broken(&a_down, "a"));
    g.register_backend("b", echo("b"));
    g.add_route("/api", &["a", "b"]).unwrap();
    g.add_client("k", "c", 1_000);
    Some((g, clock, a_down))
}

#[test]
fn should_open_the_circuit_after_three_consecutive_failures_and_stop_calling_the_backend() {
    let (g, _, _) = sut!(with_flaky_a());

    for _ in 0..20 {
        assert_eq!(status(&g, "k", "/api/x"), 200);
    }

    assert_eq!(
        g.call_count("a"),
        Ok(3),
        "a is skipped once its circuit is open"
    );
}

#[test]
fn should_let_one_trial_through_after_thirty_seconds_and_close_on_success() {
    let (g, clock, a_down) = sut!(with_flaky_a());
    for _ in 0..10 {
        g.handle("k", "/api/x");
    }
    assert_eq!(g.call_count("a"), Ok(3));

    clock.advance(29_999);
    g.handle("k", "/api/x");
    g.handle("k", "/api/x");
    assert_eq!(g.call_count("a"), Ok(3));

    clock.advance(1);
    a_down.store(false, Ordering::SeqCst);
    g.handle("k", "/api/x"); // round-robin may start at b; two requests make sure a gets its trial
    g.handle("k", "/api/x");
    assert!(g.call_count("a").unwrap() > 3);

    let before = g.call_count("a").unwrap();
    for _ in 0..10 {
        g.handle("k", "/api/x");
    }
    assert!(
        g.call_count("a").unwrap() > before + 3,
        "closed again: a takes its share"
    );
}

#[test]
fn should_reopen_the_circuit_when_the_trial_fails() {
    let (g, clock, _) = sut!(with_flaky_a());
    for _ in 0..10 {
        g.handle("k", "/api/x");
    }

    clock.advance(30_000);
    for _ in 0..4 {
        g.handle("k", "/api/x"); // the trial fails
    }
    let after_trial = g.call_count("a").unwrap();
    for _ in 0..10 {
        g.handle("k", "/api/x");
    }

    assert_eq!(after_trial, 4);
    assert_eq!(g.call_count("a"), Ok(4), "open again for 30 more seconds");
}

#[test]
fn should_answer_503_without_calling_anything_when_every_circuit_is_open() {
    let (g, _) = sut!(gateway());
    let down = Arc::new(AtomicBool::new(true));
    g.register_backend("a", broken(&down, "a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k", "c", 1_000);
    for _ in 0..3 {
        assert_eq!(status(&g, "k", "/api/x"), 502);
    }

    let r = g.handle("k", "/api/x");

    assert_eq!(r, Response::new(503, "service unavailable"));
    assert_eq!(g.call_count("a"), Ok(3));
}

#[test]
fn should_reset_the_failure_count_on_success() {
    let (g, _) = sut!(gateway());
    let down = Arc::new(AtomicBool::new(true));
    g.register_backend("a", broken(&down, "a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k", "c", 1_000);

    for _ in 0..5 {
        down.store(true, Ordering::SeqCst);
        g.handle("k", "/api/1");
        g.handle("k", "/api/2");
        down.store(false, Ordering::SeqCst);
        assert_eq!(status(&g, "k", "/api/3"), 200); // two failures then a success: never three in a row
    }
    assert_eq!(g.call_count("a"), Ok(15));
}

#[test]
fn should_rate_limit_per_client_with_a_token_bucket() {
    let (g, clock) = sut!(gateway());
    g.register_backend("a", echo("a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k1", "slow", 60);
    g.add_client("k2", "other", 60);

    for _ in 0..60 {
        assert_eq!(status(&g, "k1", "/api/x"), 200);
    }
    assert_eq!(g.handle("k1", "/api/x"), Response::new(429, "rate limited"));
    assert_eq!(status(&g, "k2", "/api/x"), 200); // separate bucket

    clock.advance(1_000); // one token per second
    assert_eq!(status(&g, "k1", "/api/x"), 200);
    assert_eq!(status(&g, "k1", "/api/x"), 429);
    clock.advance(500);
    assert_eq!(status(&g, "k1", "/api/x"), 429); // half a token is not enough
    clock.advance(500);
    assert_eq!(status(&g, "k1", "/api/x"), 200);
}

#[test]
fn should_never_hold_more_tokens_than_the_capacity() {
    let (g, clock) = sut!(gateway());
    g.register_backend("a", echo("a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k", "c", 10);

    clock.advance(3_600_000);
    let ok = (0..30).filter(|_| status(&g, "k", "/api/x") == 200).count();

    assert_eq!(ok, 10);
}

#[test]
fn should_admit_exactly_the_capacity_under_concurrent_requests() {
    let (g, _) = sut!(gateway());
    g.register_backend("a", echo("a"));
    g.add_route("/api", &["a"]).unwrap();
    g.add_client("k", "c", 1_000);
    let ok = AtomicUsize::new(0);
    let limited = AtomicUsize::new(0);

    run_concurrently(16, |_| {
        for _ in 0..100 {
            if status(&g, "k", "/api/x") == 200 {
                ok.fetch_add(1, Ordering::SeqCst);
            } else {
                limited.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(ok.load(Ordering::SeqCst), 1_000);
    assert_eq!(limited.load(Ordering::SeqCst), 600);
    assert_eq!(g.call_count("a"), Ok(1_000));
}

#[test]
fn should_spread_concurrent_requests_evenly_over_the_backends() {
    let (g, _) = sut!(simple());
    g.register_backend("d", echo("d"));
    g.add_route("/all", &["a", "b", "c", "d"]).unwrap();
    g.add_client("k", "big", 100_000);

    run_concurrently(16, |_| {
        for _ in 0..100 {
            g.handle("k", "/all/x");
        }
    });

    for id in ["a", "b", "c", "d"] {
        assert_eq!(g.call_count(id), Ok(400), "{id}");
    }
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let (g, _) = sut!(gateway());
        g.register_backend("a", echo("a"));
        g.add_route("/api", &["a"]).unwrap();
        g.add_client("k", "c", 50);
        let ok = AtomicUsize::new(0);

        run_concurrently(8, |_| {
            for _ in 0..20 {
                if status(&g, "k", "/api/x") == 200 {
                    ok.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        assert_eq!(ok.load(Ordering::SeqCst), 50, "run {run}");
        assert_eq!(g.call_count("a"), Ok(50), "run {run}");
    }
}

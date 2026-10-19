#![allow(unused_imports, clippy::needless_range_loop)]
use p98_lock_lease_fencing::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn setup() -> Option<(Arc<dyn LockService>, FakeClock)> {
    let clock = FakeClock::new(0);
    Some((create(clock.as_clock())?, clock))
}

/// What a storage server does with fencing tokens: it refuses writes carrying a token older than one it has seen.
struct FencedStore {
    highest: u64,
    value: String,
}

impl FencedStore {
    fn write(&mut self, token: u64, v: &str) -> bool {
        if token < self.highest {
            return false;
        }
        self.highest = token;
        self.value = v.to_string();
        true
    }
}

#[test]
fn should_grant_a_free_lock_with_the_first_token() {
    let (l, _) = sut!(setup());

    let t = l.acquire("job", "ann", 1_000);

    assert_eq!(t, Ok(Some(1)));
    assert_eq!(l.holder("job"), Some("ann".to_string()));
    assert_eq!(l.latest_token("job"), 1);
}

#[test]
fn should_refuse_another_owner_while_the_lease_is_valid() {
    let (l, clock) = sut!(setup());
    l.acquire("job", "ann", 1_000).unwrap();

    clock.advance(999);

    assert_eq!(l.acquire("job", "bob", 1_000), Ok(None));
    assert_eq!(l.holder("job"), Some("ann".to_string()));
}

#[test]
fn should_keep_locks_independent() {
    let (l, _) = sut!(setup());
    l.acquire("a", "ann", 1_000).unwrap();

    assert_eq!(l.acquire("b", "bob", 1_000), Ok(Some(1)));
    assert_eq!(l.latest_token("c"), 0);
}

#[test]
fn should_let_another_owner_take_over_exactly_when_the_lease_expires_with_a_higher_token() {
    let (l, clock) = sut!(setup());
    l.acquire("job", "ann", 1_000).unwrap();

    clock.advance(1_000);

    assert_eq!(l.holder("job"), None);
    assert_eq!(l.acquire("job", "bob", 1_000), Ok(Some(2)));
    assert_eq!(l.holder("job"), Some("bob".to_string()));
}

#[test]
fn should_refresh_the_lease_and_keep_the_token_when_the_owner_acquires_again() {
    let (l, clock) = sut!(setup());
    let token = l.acquire("job", "ann", 1_000).unwrap().unwrap();

    clock.advance(900);
    let again = l.acquire("job", "ann", 1_000);
    clock.advance(900);

    assert_eq!(again, Ok(Some(token)));
    assert_eq!(l.holder("job"), Some("ann".to_string())); // 1_800 < 900 + 1_000
    assert_eq!(l.latest_token("job"), token);
}

#[test]
fn should_issue_a_new_token_when_the_own_token_has_expired_and_is_reacquired() {
    let (l, clock) = sut!(setup());
    l.acquire("job", "ann", 1_000).unwrap();

    clock.advance(1_000);

    assert_eq!(l.acquire("job", "ann", 1_000), Ok(Some(2)));
}

#[test]
fn should_renew_only_with_the_right_owner_and_token_before_expiry() {
    let (l, clock) = sut!(setup());
    let token = l.acquire("job", "ann", 1_000).unwrap().unwrap();

    assert!(!l.renew("job", "bob", token, 1_000));
    assert!(!l.renew("job", "ann", token + 1, 1_000));
    clock.advance(500);
    assert!(l.renew("job", "ann", token, 1_000));
    clock.advance(999);
    assert_eq!(l.holder("job"), Some("ann".to_string()));
    clock.advance(1);
    assert!(!l.renew("job", "ann", token, 1_000), "too late");
}

#[test]
fn should_release_only_for_the_current_holder() {
    let (l, _) = sut!(setup());
    let token = l.acquire("job", "ann", 1_000).unwrap().unwrap();

    assert!(!l.release("job", "bob", token));
    assert!(!l.release("job", "ann", token + 5));
    assert_eq!(l.holder("job"), Some("ann".to_string()));
    assert!(l.release("job", "ann", token));
    assert!(!l.release("job", "ann", token));
    assert_eq!(l.acquire("job", "bob", 1_000), Ok(Some(token + 1)));
}

#[test]
fn should_not_let_a_stale_owner_release_or_renew_the_new_holders_lock() {
    let (l, clock) = sut!(setup());
    let old = l.acquire("job", "ann", 1_000).unwrap().unwrap();
    clock.advance(1_500);
    let fresh = l.acquire("job", "bob", 1_000).unwrap().unwrap();

    assert!(!l.release("job", "ann", old));
    assert!(!l.renew("job", "ann", old, 1_000));

    assert_eq!(l.holder("job"), Some("bob".to_string()));
    assert!(fresh > old);
}

#[test]
fn should_let_a_fenced_store_reject_a_writer_whose_lease_expired_while_paused() {
    let (l, clock) = sut!(setup());
    let mut store = FencedStore { highest: 0, value: String::new() };
    let ann_token = l.acquire("db", "ann", 1_000).unwrap().unwrap();
    // ann stalls (long GC pause) past her lease; bob takes the lock and writes
    clock.advance(2_000);
    let bob_token = l.acquire("db", "bob", 1_000).unwrap().unwrap();
    assert!(store.write(bob_token, "bob's data"));

    // ann wakes up, still believes she holds the lock and writes
    let ann_wrote = store.write(ann_token, "ann's stale data");

    assert!(!ann_wrote);
    assert_eq!(store.value, "bob's data");
}

#[test]
fn should_reject_zero_leases() {
    let (l, _) = sut!(setup());

    assert_eq!(l.acquire("job", "ann", 0), Err(LockError::InvalidLease));
}

#[test]
fn should_give_mutual_exclusion_and_increasing_tokens_under_contention() {
    let (l, _) = sut!(setup());
    let inside = AtomicUsize::new(0);
    let violations = AtomicUsize::new(0);
    let tokens = Mutex::new(Vec::new());

    run_concurrently(16, |t| {
        let owner = format!("worker-{t}");
        for _ in 0..100 {
            let token = loop {
                if let Some(tok) = l.acquire("shared", &owner, 60_000).unwrap() {
                    break tok;
                }
                thread::yield_now();
            };
            if inside.fetch_add(1, Ordering::SeqCst) + 1 != 1 {
                violations.fetch_add(1, Ordering::SeqCst);
            }
            tokens.lock().unwrap().push(token);
            inside.fetch_sub(1, Ordering::SeqCst);
            if !l.release("shared", &owner, token) {
                violations.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(violations.load(Ordering::SeqCst), 0);
    let tokens = tokens.into_inner().unwrap();
    assert_eq!(tokens.len(), 1_600);
    assert!(tokens.windows(2).all(|w| w[1] > w[0]));
    assert_eq!(l.latest_token("shared"), 1_600);
}

#[test]
fn should_let_exactly_one_thread_win_a_free_lock() {
    let (l, _) = sut!(setup());

    let results = collect_concurrently(16, |t| l.acquire("job", &format!("worker-{t}"), 10_000).unwrap());

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(l.latest_token("job"), 1);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let (l, clock) = sut!(setup());
        let wins = AtomicUsize::new(0);

        run_concurrently(8, |t| {
            if l.acquire("job", &format!("w{t}"), 1_000).unwrap().is_some() {
                wins.fetch_add(1, Ordering::SeqCst);
            }
        });
        clock.advance(1_000);
        run_concurrently(8, |t| {
            if l.acquire("job", &format!("w{t}"), 1_000).unwrap().is_some() {
                wins.fetch_add(1, Ordering::SeqCst);
            }
        });

        assert_eq!(wins.load(Ordering::SeqCst), 2, "run {run}");
        assert_eq!(l.latest_token("job"), 2, "run {run}");
    }
}

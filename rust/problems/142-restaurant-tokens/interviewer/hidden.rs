#![allow(unused_imports, clippy::needless_range_loop)]
use p142_restaurant_tokens::{contract::*, create};
use playground_support::{collect_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const GRACE: u64 = 60_000;
const RECALL: u64 = 120_000;
const D: &str = DINE_IN;
const T: &str = TAKEAWAY;
const V: &str = VIP;

fn system(clock: &FakeClock) -> Option<Arc<dyn TokenSystem>> {
    let s = create(clock.as_clock(), GRACE, RECALL)?;
    s.add_counter("all", &[D, T, V]).unwrap();
    Some(s)
}

fn issue(s: &Arc<dyn TokenSystem>, ty: &str, count: usize) {
    for _ in 0..count {
        s.issue_token("c", ty, 1).unwrap();
    }
}

fn next(s: &Arc<dyn TokenSystem>, counter: &str) -> Option<String> {
    s.call_next(counter).unwrap()
}

fn some(t: &str) -> Option<String> {
    Some(t.to_string())
}

#[test]
fn should_promote_starved_tokens_by_age_not_by_priority() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 2).unwrap(); // T-1
    s.issue_token("b", T, 1).unwrap(); // T-2
    issue(&s, V, 7); // T-3 .. T-9
    for _ in 0..5 {
        next(&s, "all"); // T-3..T-7 pass over both T-1 and T-2
    }

    // both promoted: oldest first even though T-2 has the higher priority; then the remaining VIPs
    assert_eq!(s.queue_snapshot(), vec!["T-1", "T-2", "T-8", "T-9"]);
}

#[test]
fn should_reset_the_passed_over_count_when_a_recalled_token_returns() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 1).unwrap(); // T-1
    s.issue_token("b", D, 1).unwrap(); // T-2
    issue(&s, V, 6); // T-3 .. T-8
    for _ in 0..5 {
        next(&s, "all"); // T-3..T-7 : T-1 and T-2 promoted
    }
    assert_eq!(next(&s, "all"), some("T-1"));
    s.mark_no_show("T-1").unwrap();

    s.recall("T-1").unwrap();

    // T-2 is still promoted; the recalled T-1 starts again from zero at the front of the DINE_IN class
    assert_eq!(s.queue_snapshot(), vec!["T-2", "T-8", "T-1"]);
    assert_eq!(next(&s, "all"), some("T-2"));
    assert_eq!(next(&s, "all"), some("T-8"));
    assert_eq!(next(&s, "all"), some("T-1"));
}

#[test]
fn should_place_a_recalled_token_ahead_of_older_tokens_of_its_class_but_behind_higher_classes() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", T, 1).unwrap(); // T-1
    s.issue_token("b", T, 1).unwrap(); // T-2
    s.issue_token("c", T, 1).unwrap(); // T-3
    s.issue_token("d", D, 1).unwrap(); // T-4
    assert_eq!(next(&s, "all"), some("T-1"));
    assert_eq!(next(&s, "all"), some("T-2"));
    s.mark_no_show("T-2").unwrap();
    s.issue_token("e", V, 1).unwrap(); // T-5

    s.recall("T-2").unwrap();

    assert_eq!(s.queue_snapshot(), vec!["T-5", "T-2", "T-3", "T-4"]);
}

#[test]
fn should_count_a_pass_over_when_a_later_token_is_served_while_a_recalled_token_waits() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 1).unwrap(); // T-1
    assert_eq!(next(&s, "all"), some("T-1"));
    s.mark_no_show("T-1").unwrap();
    issue(&s, V, 6); // T-2 .. T-7
    s.recall("T-1").unwrap();
    for _ in 0..4 {
        next(&s, "all"); // T-2..T-5
    }
    assert_eq!(s.queue_snapshot(), vec!["T-6", "T-7", "T-1"]);

    next(&s, "all"); // T-6: fifth later token since the recall

    assert_eq!(s.queue_snapshot(), vec!["T-1", "T-7"]);
}

#[test]
fn should_expire_only_the_overdue_tokens_and_report_them_in_token_order() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 4);
    next(&s, "all"); // T-1 at 0
    clock.advance(30_000);
    next(&s, "all"); // T-2 at 30000
    next(&s, "all"); // T-3 at 30000
    clock.advance(30_001); // T-1 is 60001 old, the others 30001

    assert_eq!(s.expire_overdue(), vec!["T-1"]);

    clock.advance(30_000); // T-2, T-3 are now 60001 old
    s.mark_served("T-3").unwrap();
    assert_eq!(s.expire_overdue(), vec!["T-2"]);
    assert_eq!(s.stats(), "4:1:2:0:1");
}

#[test]
fn should_measure_the_recall_window_from_the_moment_of_the_no_show() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 1);
    next(&s, "all");
    clock.advance(GRACE + 5_000);
    s.expire_overdue(); // becomes NO_SHOW now, not at the call time
    clock.advance(RECALL);

    s.recall("T-1").unwrap();

    assert_eq!(s.status("T-1"), Ok("WAITING".to_string()));
}

#[test]
fn should_expire_no_shows_only_after_the_recall_window() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);
    next(&s, "all");
    s.mark_no_show("T-1").unwrap();
    clock.advance(RECALL);
    s.expire_overdue();
    assert_eq!(s.status("T-1"), Ok("NO_SHOW".to_string()));

    clock.advance(1);
    assert!(s.expire_overdue().is_empty());
    assert_eq!(s.status("T-1"), Ok("EXPIRED".to_string()));
    assert_eq!(s.stats(), "2:0:1:0:1");
    assert_eq!(s.recall("T-1"), Err(TokenError::InvalidState));
}

#[test]
fn should_floor_the_average_service_time() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 5);
    for (i, took) in [1u64, 1, 2].iter().enumerate() {
        next(&s, "all");
        clock.advance(*took);
        s.mark_served(&format!("T-{}", i + 1)).unwrap();
    }

    // (1 + 1 + 2) / 3 = 1 (floor)
    assert_eq!(s.estimated_wait_ms("T-5"), Ok(1));
}

/// Small deterministic generator so the run is reproducible.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) % bound
    }
}

#[test]
fn should_keep_positions_consistent_with_snapshot_and_next_call_across_a_random_run() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    let mut rnd = Lcg(42);
    let mut recalled: HashSet<String> = HashSet::new();
    for _ in 0..300 {
        let op = rnd.next(10);
        if op < 5 {
            let ty = [D, T, V][rnd.next(3) as usize];
            let size = if ty == D { 1 + rnd.next(12) as u32 } else { 1 };
            s.issue_token("c", ty, size).unwrap();
        } else if op < 8 {
            let before = s.queue_snapshot();
            let got = next(&s, "all");
            assert_eq!(got.is_some(), !before.is_empty());
            if let Some(t) = got {
                assert_eq!(t, before[0]);
                if rnd.next(2) == 0 {
                    clock.advance(rnd.next(1_000));
                    s.mark_served(&t).unwrap();
                } else {
                    s.mark_no_show(&t).unwrap();
                    if rnd.next(2) == 0 && recalled.insert(t.clone()) {
                        s.recall(&t).unwrap();
                    }
                }
            }
        } else {
            let q = s.queue_snapshot();
            if !q.is_empty() {
                s.cancel(&q[rnd.next(q.len() as u64) as usize]).unwrap();
            }
        }
        let snap = s.queue_snapshot();
        for (i, t) in snap.iter().enumerate() {
            assert_eq!(s.position_in_queue(t), Ok(i + 1));
        }
    }
}

#[test]
fn should_keep_statistics_consistent_under_concurrent_work() {
    let clock = FakeClock::new(5_000_000);
    let s = sut!(system(&clock));
    for c in 0..4 {
        s.add_counter(&format!("c{c}"), &[D, T, V]).unwrap();
    }
    let cancelled = AtomicUsize::new(0);
    let served = collect_concurrently(16, |t| {
        let mut mine = vec![];
        for i in 0..100 {
            let tok = s
                .issue_token(&format!("n{t}"), if i % 2 == 0 { D } else { V }, 1)
                .unwrap();
            if i % 10 == 0 {
                // another thread's counter may already have called it
                if s.cancel(&tok).is_ok() {
                    cancelled.fetch_add(1, Ordering::SeqCst);
                }
                continue;
            }
            if let Some(called) = s.call_next(&format!("c{}", t % 4)).unwrap() {
                s.mark_served(&called).unwrap();
                mine.push(called);
            }
        }
        mine
    });

    let all: Vec<String> = served.into_iter().flatten().collect();
    let uniq: HashSet<&String> = all.iter().collect();
    assert_eq!(uniq.len(), all.len());
    let stats = s.stats();
    let parts: Vec<usize> = stats.split(':').map(|p| p.parse().unwrap()).collect();
    assert_eq!(parts[0], 1_600);
    assert_eq!(parts[1], all.len());
    assert_eq!(parts[3], cancelled.load(Ordering::SeqCst));
    assert_eq!(s.queue_snapshot().len(), parts[4]);
    assert_eq!(all.len() + cancelled.load(Ordering::SeqCst) + parts[4], 1_600);
}

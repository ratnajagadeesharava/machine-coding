#![allow(unused_imports, clippy::needless_range_loop)]
use p142_restaurant_tokens::{contract::*, create};
use playground_support::{collect_concurrently, sut, FakeClock};
use std::collections::HashSet;
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

fn status(s: &Arc<dyn TokenSystem>, t: &str) -> String {
    s.status(t).unwrap()
}

#[test]
fn should_number_tokens_from_one_in_issue_order() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    assert_eq!(s.issue_token("Asha", D, 4), Ok("T-1".to_string()));
    assert_eq!(s.issue_token("Ben", T, 1), Ok("T-2".to_string()));
    assert_eq!(s.issue_token("Cy", V, 1), Ok("T-3".to_string()));
    assert_eq!(status(&s, "T-1"), "WAITING");
}

#[test]
fn should_reject_invalid_token_requests() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    let bad = Err(TokenError::InvalidArgument);
    assert_eq!(s.issue_token(" ", D, 2), bad);
    assert_eq!(s.issue_token("a", D, 0), bad);
    assert_eq!(s.issue_token("a", D, 13), bad);
    assert_eq!(s.issue_token("a", T, 2), bad);
    assert_eq!(s.issue_token("a", V, 0), bad);
    assert_eq!(s.issue_token("a", "ROYAL", 1), bad);

    // failed requests do not consume numbers
    assert_eq!(s.issue_token("ok", D, 1), Ok("T-1".to_string()));
}

#[test]
fn should_accept_party_size_boundaries_for_dine_in() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    assert_eq!(s.issue_token("solo", D, 1), Ok("T-1".to_string()));
    assert_eq!(s.issue_token("big", D, 12), Ok("T-2".to_string()));
}

#[test]
fn should_serve_vip_then_takeaway_then_dine_in_oldest_first_within_a_type() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 2).unwrap(); // T-1
    s.issue_token("b", T, 1).unwrap(); // T-2
    s.issue_token("c", V, 1).unwrap(); // T-3
    s.issue_token("d", D, 3).unwrap(); // T-4
    s.issue_token("e", V, 1).unwrap(); // T-5

    assert_eq!(s.queue_snapshot(), vec!["T-3", "T-5", "T-2", "T-1", "T-4"]);
    for t in ["T-3", "T-5", "T-2", "T-1", "T-4"] {
        assert_eq!(next(&s, "all"), some(t));
    }
}

#[test]
fn should_mark_the_token_called_and_remove_it_from_the_queue() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);

    assert_eq!(next(&s, "all"), some("T-1"));

    assert_eq!(status(&s, "T-1"), "CALLED");
    assert_eq!(s.queue_snapshot(), vec!["T-2"]);
}

#[test]
fn should_return_none_when_nothing_is_waiting() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    assert_eq!(next(&s, "all"), None);
    s.issue_token("a", D, 1).unwrap();
    next(&s, "all");
    assert_eq!(next(&s, "all"), None);
}

#[test]
fn should_skip_tokens_a_counter_does_not_support_without_consuming_them() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.add_counter("takeaway-only", &[T]).unwrap();
    s.issue_token("a", D, 2).unwrap(); // T-1
    s.issue_token("b", T, 1).unwrap(); // T-2

    assert_eq!(next(&s, "takeaway-only"), some("T-2"));
    assert_eq!(next(&s, "takeaway-only"), None);
    assert_eq!(status(&s, "T-1"), "WAITING");
    assert_eq!(s.queue_snapshot(), vec!["T-1"]);
    assert_eq!(next(&s, "all"), some("T-1"));
}

#[test]
fn should_let_a_counter_support_several_types() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.add_counter("bar", &[T, V]).unwrap();
    s.issue_token("a", D, 1).unwrap(); // T-1
    s.issue_token("b", T, 1).unwrap(); // T-2
    s.issue_token("c", V, 1).unwrap(); // T-3

    assert_eq!(next(&s, "bar"), some("T-3"));
    assert_eq!(next(&s, "bar"), some("T-2"));
    assert_eq!(next(&s, "bar"), None);
}

#[test]
fn should_reject_bad_counters_and_unknown_counters() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    let bad = Err(TokenError::InvalidArgument);
    assert_eq!(s.add_counter("all", &[D]), bad);
    assert_eq!(s.add_counter(" ", &[D]), bad);
    assert_eq!(s.add_counter("x", &[]), bad);
    assert_eq!(s.add_counter("x", &["ROYAL"]), bad);
    assert_eq!(s.call_next("ghost"), Err(TokenError::InvalidArgument));
    assert_eq!(s.call_next("x"), Err(TokenError::InvalidArgument));
}

#[test]
fn should_mark_served_only_for_called_tokens() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);

    assert_eq!(s.mark_served("T-1"), Err(TokenError::InvalidState));
    assert_eq!(s.mark_served("T-99"), Err(TokenError::UnknownToken));
    next(&s, "all");
    s.mark_served("T-1").unwrap();
    assert_eq!(status(&s, "T-1"), "SERVED");
    assert_eq!(s.mark_served("T-1"), Err(TokenError::InvalidState));
    assert_eq!(s.mark_no_show("T-1"), Err(TokenError::InvalidState));
}

#[test]
fn should_cancel_only_waiting_tokens() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 3);
    next(&s, "all"); // T-1 called

    s.cancel("T-2").unwrap();

    assert_eq!(status(&s, "T-2"), "CANCELLED");
    assert_eq!(s.queue_snapshot(), vec!["T-3"]);
    assert_eq!(s.cancel("T-1"), Err(TokenError::InvalidState));
    assert_eq!(s.cancel("T-2"), Err(TokenError::InvalidState));
    assert_eq!(s.cancel("nope"), Err(TokenError::UnknownToken));
    assert_eq!(next(&s, "all"), some("T-3"));
}

#[test]
fn should_report_positions_in_the_serving_order() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 1).unwrap(); // T-1
    s.issue_token("b", V, 1).unwrap(); // T-2
    s.issue_token("c", T, 1).unwrap(); // T-3

    assert_eq!(s.position_in_queue("T-2"), Ok(1));
    assert_eq!(s.position_in_queue("T-3"), Ok(2));
    assert_eq!(s.position_in_queue("T-1"), Ok(3));
    next(&s, "all");
    assert_eq!(s.position_in_queue("T-1"), Ok(2));
    assert_eq!(s.position_in_queue("T-2"), Err(TokenError::InvalidState));
    assert_eq!(s.position_in_queue("T-42"), Err(TokenError::UnknownToken));
}

#[test]
fn should_estimate_wait_with_five_minutes_before_anything_is_served() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 3);

    assert_eq!(s.estimated_wait_ms("T-1"), Ok(0));
    assert_eq!(s.estimated_wait_ms("T-2"), Ok(300_000));
    assert_eq!(s.estimated_wait_ms("T-3"), Ok(600_000));
    assert_eq!(s.estimated_wait_ms("T-9"), Err(TokenError::UnknownToken));
}

#[test]
fn should_estimate_wait_from_the_integer_mean_of_served_durations() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 5);
    next(&s, "all"); // T-1 at +0
    clock.advance(1_000);
    s.mark_served("T-1").unwrap(); // took 1000
    next(&s, "all"); // T-2 called at +1000
    clock.advance(2_001);
    s.mark_served("T-2").unwrap(); // took 2001

    // mean = (1000 + 2001) / 2 = 1500 (integer division)
    assert_eq!(s.estimated_wait_ms("T-3"), Ok(0));
    assert_eq!(s.estimated_wait_ms("T-4"), Ok(1_500));
    assert_eq!(s.estimated_wait_ms("T-5"), Ok(3_000));
}

#[test]
fn should_not_count_no_shows_or_cancellations_in_the_average() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 5);
    next(&s, "all"); // T-1
    clock.advance(4_000);
    s.mark_served("T-1").unwrap(); // 4000
    next(&s, "all"); // T-2
    clock.advance(90_000);
    s.mark_no_show("T-2").unwrap();
    s.cancel("T-3").unwrap();

    assert_eq!(s.estimated_wait_ms("T-5"), Ok(4_000));
}

#[test]
fn should_promote_a_token_passed_over_by_five_later_tokens() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.issue_token("walk-in", D, 2).unwrap(); // T-1
    issue(&s, V, 6); // T-2 .. T-7

    // each served VIP (T-2..T-5) is later than T-1 and passes it over: count 1, 2, 3, 4
    for t in ["T-2", "T-3", "T-4", "T-5"] {
        assert_eq!(next(&s, "all"), some(t));
    }
    assert_eq!(s.queue_snapshot(), vec!["T-6", "T-7", "T-1"]);

    // T-6 is the 5th: T-1 is promoted ahead of the remaining VIP
    assert_eq!(next(&s, "all"), some("T-6"));
    assert_eq!(s.queue_snapshot(), vec!["T-1", "T-7"]);
    assert_eq!(s.position_in_queue("T-1"), Ok(1));
    assert_eq!(next(&s, "all"), some("T-1"));
    assert_eq!(next(&s, "all"), some("T-7"));
}

#[test]
fn should_not_count_tokens_issued_before_the_waiting_one_as_passing_it_over() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, V, 6); // T-1 .. T-6
    s.issue_token("late", D, 2).unwrap(); // T-7
    for _ in 0..6 {
        next(&s, "all");
    }

    s.issue_token("vip", V, 1).unwrap(); // T-8

    // six earlier tokens were served before T-7 but none was issued after it
    assert_eq!(s.queue_snapshot(), vec!["T-8", "T-7"]);
}

#[test]
fn should_count_pass_overs_made_by_counters_that_cannot_serve_the_waiting_token() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.add_counter("takeaway-only", &[T]).unwrap();
    s.issue_token("a", D, 1).unwrap(); // T-1
    issue(&s, T, 6); // T-2 .. T-7
    for _ in 0..4 {
        next(&s, "takeaway-only"); // T-2..T-5
    }
    assert_eq!(s.queue_snapshot(), vec!["T-6", "T-7", "T-1"]);

    next(&s, "takeaway-only"); // T-6, the fifth

    assert_eq!(s.queue_snapshot(), vec!["T-1", "T-7"]);
    assert_eq!(next(&s, "all"), some("T-1"));
}

#[test]
fn should_promote_several_starved_tokens_oldest_first() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    s.issue_token("a", D, 1).unwrap(); // T-1
    s.issue_token("b", D, 1).unwrap(); // T-2
    issue(&s, V, 6); // T-3 .. T-8
    for _ in 0..5 {
        next(&s, "all"); // T-3..T-7
    }

    assert_eq!(s.queue_snapshot(), vec!["T-1", "T-2", "T-8"]);
}

#[test]
fn should_expire_a_called_token_only_after_the_grace_time() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);
    next(&s, "all"); // T-1 at t0

    clock.advance(GRACE);
    assert!(s.expire_overdue().is_empty());
    assert_eq!(status(&s, "T-1"), "CALLED");

    clock.advance(1);
    assert_eq!(s.expire_overdue(), vec!["T-1"]);
    assert_eq!(status(&s, "T-1"), "NO_SHOW");
    assert_eq!(status(&s, "T-2"), "WAITING");
    assert!(s.expire_overdue().is_empty());
    assert_eq!(s.mark_served("T-1"), Err(TokenError::InvalidState));
}

#[test]
fn should_still_serve_an_overdue_token_until_expiry_is_run() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 1);
    next(&s, "all");
    clock.advance(10 * GRACE);

    s.mark_served("T-1").unwrap();

    assert_eq!(status(&s, "T-1"), "SERVED");
    assert!(s.expire_overdue().is_empty());
}

#[test]
fn should_recall_a_no_show_to_the_front_of_its_priority_class() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 4); // T-1 .. T-4
    next(&s, "all"); // T-1
    next(&s, "all"); // T-2
    s.mark_no_show("T-1").unwrap();
    s.mark_no_show("T-2").unwrap();
    s.issue_token("vip", V, 1).unwrap(); // T-5
    assert_eq!(s.queue_snapshot(), vec!["T-5", "T-3", "T-4"]);

    s.recall("T-1").unwrap();
    assert_eq!(status(&s, "T-1"), "WAITING");
    assert_eq!(s.queue_snapshot(), vec!["T-5", "T-1", "T-3", "T-4"]);

    // a later recall goes in front of an earlier one
    s.recall("T-2").unwrap();
    assert_eq!(s.queue_snapshot(), vec!["T-5", "T-2", "T-1", "T-3", "T-4"]);
}

#[test]
fn should_allow_a_recall_only_once_and_only_for_no_shows() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);
    assert_eq!(s.recall("T-1"), Err(TokenError::InvalidState));
    next(&s, "all");
    assert_eq!(s.recall("T-1"), Err(TokenError::InvalidState));
    s.mark_no_show("T-1").unwrap();

    s.recall("T-1").unwrap();
    assert_eq!(s.recall("T-1"), Err(TokenError::InvalidState));

    assert_eq!(next(&s, "all"), some("T-1"));
    s.mark_no_show("T-1").unwrap();
    assert_eq!(s.recall("T-1"), Err(TokenError::InvalidState));
    assert_eq!(status(&s, "T-1"), "NO_SHOW");
    assert_eq!(s.recall("T-77"), Err(TokenError::UnknownToken));
}

#[test]
fn should_refuse_a_recall_after_the_window_and_expire_the_token() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 2);
    next(&s, "all"); // T-1
    next(&s, "all"); // T-2
    s.mark_no_show("T-1").unwrap();
    s.mark_no_show("T-2").unwrap();
    clock.advance(RECALL);
    s.recall("T-1").unwrap(); // exactly at the window edge is still allowed

    clock.advance(1);
    assert_eq!(s.recall("T-2"), Err(TokenError::InvalidState));
    assert_eq!(status(&s, "T-2"), "NO_SHOW");
    s.expire_overdue();
    assert_eq!(status(&s, "T-2"), "EXPIRED");
    assert_eq!(status(&s, "T-1"), "WAITING");
}

#[test]
fn should_report_statistics() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    assert_eq!(s.stats(), "0:0:0:0:0");
    issue(&s, D, 5);
    assert_eq!(s.stats(), "5:0:0:0:5");
    next(&s, "all"); // T-1
    next(&s, "all"); // T-2
    next(&s, "all"); // T-3
    s.mark_served("T-1").unwrap();
    s.mark_no_show("T-2").unwrap();
    s.cancel("T-5").unwrap();

    // T-3 is CALLED and belongs to no column; T-4 is waiting
    assert_eq!(s.stats(), "5:1:1:1:1");
}

#[test]
fn should_start_a_new_day_without_losing_counters() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    issue(&s, D, 3);
    next(&s, "all");
    clock.advance(9_000);
    s.mark_served("T-1").unwrap();

    s.reset_day();

    assert_eq!(s.stats(), "0:0:0:0:0");
    assert!(s.queue_snapshot().is_empty());
    assert_eq!(s.status("T-1"), Err(TokenError::UnknownToken));
    assert_eq!(s.issue_token("new", D, 1), Ok("T-1".to_string()));
    s.issue_token("new2", D, 1).unwrap();
    assert_eq!(s.estimated_wait_ms("T-2"), Ok(300_000)); // average forgotten
    assert_eq!(next(&s, "all"), some("T-1"));
}

#[test]
fn should_issue_unique_gap_free_numbers_from_sixteen_threads() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));

    let per_thread = collect_concurrently(16, |t| {
        (0..100)
            .map(|i| {
                let ty = [D, T, V][i % 3];
                s.issue_token(&format!("t{t}"), ty, if ty == D { 2 } else { 1 })
                    .unwrap()
            })
            .collect::<Vec<_>>()
    });

    let all: HashSet<String> = per_thread.into_iter().flatten().collect();
    assert_eq!(all.len(), 1_600);
    for n in 1..=1_600 {
        assert!(all.contains(&format!("T-{n}")), "missing T-{n}");
    }
    assert_eq!(s.stats(), "1600:0:0:0:1600");
    assert_eq!(s.queue_snapshot().len(), 1_600);
}

#[test]
fn should_never_call_the_same_token_twice_under_concurrent_counters() {
    let clock = FakeClock::new(1_000_000);
    let s = sut!(system(&clock));
    for c in 0..8 {
        s.add_counter(&format!("c{c}"), &[D, T, V]).unwrap();
    }
    issue(&s, D, 400);
    issue(&s, V, 400);

    let got = collect_concurrently(8, |c| {
        let mut mine = vec![];
        while let Some(t) = s.call_next(&format!("c{c}")).unwrap() {
            mine.push(t);
        }
        mine
    });

    let all: Vec<String> = got.into_iter().flatten().collect();
    let uniq: HashSet<&String> = all.iter().collect();
    assert_eq!(all.len(), 800);
    assert_eq!(uniq.len(), 800);
    assert!(s.queue_snapshot().is_empty());
}

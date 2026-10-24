#![allow(unused_imports, clippy::needless_range_loop)]
use p135_container_orchestrator::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn at(node: &str) -> Result<Option<String>, OrchError> {
    Ok(Some(node.to_string()))
}

fn pending_result() -> Result<Option<String>, OrchError> {
    Ok(None)
}

fn s(v: &str) -> Result<String, OrchError> {
    Ok(v.to_string())
}

fn within_capacity(usage: &str) -> bool {
    let p: Vec<u64> = usage
        .split(['/', ','])
        .map(|x| x.parse().unwrap())
        .collect();
    p[0] <= p[1] && p[2] <= p[3]
}

// ---------------------------------------------------------------- strategies

#[test]
fn should_place_on_the_node_with_the_smallest_id_that_fits_with_first_fit() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("b", 1000, 1000).unwrap();
    o.add_node("a", 1000, 1000).unwrap();

    assert_eq!(o.submit("c1", 100, 100, 1), at("a"));
    assert_eq!(o.submit("c2", 100, 100, 1), at("a"));
    assert_eq!(o.placement("c2"), s("a"));
}

#[test]
fn should_skip_a_node_with_enough_cpu_but_not_enough_memory() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("a", 1000, 100).unwrap();
    o.add_node("b", 500, 500).unwrap();

    assert_eq!(o.submit("c", 200, 200, 1), at("b"));
}

#[test]
fn should_pick_the_tightest_node_with_best_fit_and_the_emptiest_with_worst_fit() {
    let best = sut!(create(Strategy::BestFit));
    let worst = sut!(create(Strategy::WorstFit));
    for o in [&best, &worst] {
        o.add_node("n1", 1000, 1000).unwrap();
        o.add_node("n2", 400, 400).unwrap();
        o.add_node("n3", 2000, 2000).unwrap();
    }

    assert_eq!(best.submit("c", 300, 300, 1), at("n2"));
    assert_eq!(worst.submit("c", 300, 300, 1), at("n3"));
}

#[test]
fn should_measure_leftover_as_the_sum_of_free_cpu_and_free_memory_after_placement() {
    let best = sut!(create(Strategy::BestFit));
    let worst = sut!(create(Strategy::WorstFit));
    for o in [&best, &worst] {
        o.add_node("n1", 1000, 100).unwrap(); // leftover for (100,50): 900 + 50 = 950
        o.add_node("n2", 500, 500).unwrap(); //  leftover for (100,50): 400 + 450 = 850
    }

    assert_eq!(best.submit("c", 100, 50, 1), at("n2"));
    assert_eq!(worst.submit("c", 100, 50, 1), at("n1"));
}

#[test]
fn should_break_strategy_ties_by_node_id_ascending() {
    for strategy in [Strategy::FirstFit, Strategy::BestFit, Strategy::WorstFit] {
        let o = sut!(create(strategy));
        o.add_node("n2", 500, 500).unwrap();
        o.add_node("n1", 500, 500).unwrap();
        o.add_node("n3", 500, 500).unwrap();

        assert_eq!(o.submit("c", 100, 100, 1), at("n1"), "{strategy:?}");
    }
}

#[test]
fn should_apply_a_strategy_change_to_later_submissions_only() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 1000, 1000).unwrap();
    o.add_node("n2", 400, 400).unwrap();
    assert_eq!(o.submit("a", 300, 300, 1), at("n1"));

    o.set_strategy(Strategy::BestFit);

    assert_eq!(o.placement("a"), s("n1")); // nothing moves
    assert_eq!(o.submit("b", 50, 50, 1), at("n2")); // leftover 700 on n2 vs 1300 on n1
    o.set_strategy(Strategy::WorstFit);
    assert_eq!(o.submit("c", 50, 50, 1), at("n1"));
}

#[test]
fn should_accept_a_container_that_fits_exactly() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 500, 500).unwrap();

    assert_eq!(o.submit("exact", 500, 500, 1), at("n"));
    assert_eq!(o.node_usage("n"), s("500/500,500/500"));
    assert_eq!(o.submit("one", 1, 1, 1), pending_result());
}

// ---------------------------------------------------------------- pending and retry

#[test]
fn should_keep_a_container_that_fits_nowhere_as_pending() {
    let o = sut!(create(Strategy::FirstFit));
    assert_eq!(o.submit("early", 10, 10, 1), pending_result()); // no nodes at all
    o.add_node("n", 100, 100).unwrap();
    assert_eq!(o.placement("early"), s("n")); // retried when the node arrived

    assert_eq!(o.submit("a", 60, 60, 5), at("n"));
    assert_eq!(o.submit("b", 60, 60, 5), pending_result());
    assert_eq!(o.placement("b"), s("PENDING"));
    assert_eq!(o.pending(), vec!["b"]);
}

#[test]
fn should_retry_pending_containers_when_capacity_is_freed() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 60, 60, 5).unwrap();
    o.submit("b", 60, 60, 5).unwrap();

    o.remove("a").unwrap();

    assert_eq!(o.placement("b"), s("n"));
    assert!(o.pending().is_empty());
    assert_eq!(o.node_usage("n"), s("60/100,60/100"));
}

#[test]
fn should_retry_pending_containers_when_a_node_is_added() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.submit("a", 100, 100, 1).unwrap();
    o.submit("b", 80, 80, 1).unwrap();

    o.add_node("n2", 100, 100).unwrap();

    assert_eq!(o.placement("b"), s("n2"));
}

#[test]
fn should_retry_highest_priority_first_then_oldest_submission() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("blocker", 100, 100, 1).unwrap();
    o.submit("p1", 100, 100, 5).unwrap();
    o.submit("p2", 100, 100, 9).unwrap();
    o.submit("p3", 100, 100, 5).unwrap();
    o.submit("p4", 100, 100, 9).unwrap();

    assert_eq!(o.pending(), vec!["p2", "p4", "p1", "p3"]);

    o.remove("blocker").unwrap();
    assert_eq!(o.placement("p2"), s("n"));
    assert_eq!(o.pending(), vec!["p4", "p1", "p3"]);

    o.remove("p2").unwrap();
    assert_eq!(o.placement("p4"), s("n"));
    o.remove("p4").unwrap();
    assert_eq!(o.placement("p1"), s("n"));
}

#[test]
fn should_not_let_a_pending_container_that_still_does_not_fit_block_smaller_ones() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 50, 50).unwrap();
    o.submit("x1", 100, 100, 1).unwrap(); // n1
    o.submit("x2", 50, 50, 1).unwrap(); //  n2
    o.submit("big", 100, 100, 50).unwrap();
    o.submit("small", 50, 50, 1).unwrap();
    assert_eq!(o.pending(), vec!["big", "small"]);

    o.remove("x2").unwrap();

    assert_eq!(o.placement("small"), s("n2"));
    assert_eq!(o.pending(), vec!["big"]);
}

#[test]
fn should_place_a_new_container_immediately_even_when_others_wait_with_higher_priority() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 70, 70, 1).unwrap();
    assert_eq!(o.submit("waiting", 50, 50, 90), pending_result());

    assert_eq!(o.submit("small", 30, 30, 1), at("n"));
    assert_eq!(o.pending(), vec!["waiting"]);
}

#[test]
fn should_remove_a_pending_container_without_touching_others() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 100, 100, 1).unwrap();
    o.submit("b", 10, 10, 1).unwrap();
    o.submit("c", 10, 10, 1).unwrap();

    o.remove("b").unwrap();

    assert_eq!(o.pending(), vec!["c"]);
    assert_eq!(o.placement("b"), Err(OrchError::UnknownContainer));
    assert_eq!(o.containers_on("n"), Ok(vec!["a".to_string()]));
}

// ---------------------------------------------------------------- queries

#[test]
fn should_report_usage_and_sorted_containers() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 1000, 2048).unwrap();
    assert_eq!(o.node_usage("n"), s("0/1000,0/2048"));
    assert_eq!(o.containers_on("n"), Ok(vec![]));

    o.submit("b", 300, 100, 1).unwrap();
    o.submit("a", 200, 156, 1).unwrap();

    assert_eq!(o.node_usage("n"), s("500/1000,256/2048"));
    assert_eq!(o.containers_on("n"), Ok(vec!["a".to_string(), "b".to_string()]));
    o.remove("a").unwrap();
    assert_eq!(o.node_usage("n"), s("300/1000,100/2048"));
}

#[test]
fn should_allow_an_id_to_be_reused_after_removal() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 100, 100, 1).unwrap();
    o.remove("a").unwrap();

    assert_eq!(o.submit("a", 20, 20, 1), at("n"));
    assert_eq!(o.node_usage("n"), s("20/100,20/100"));
}

// ---------------------------------------------------------------- node removal

#[test]
fn should_reschedule_the_containers_of_a_removed_node_and_report_them_in_id_order() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 100, 100).unwrap();
    o.submit("b", 40, 40, 1).unwrap(); // n1
    o.submit("a", 60, 60, 1).unwrap(); // n1
    o.submit("c", 50, 50, 1).unwrap(); // n2
    o.submit("d", 10, 10, 1).unwrap(); // n2

    let moved = o.remove_node("n1").unwrap();

    // retry order is b (older) then a; b fits n2's last 40, a no longer fits anywhere
    assert_eq!(moved, vec!["a->PENDING", "b->n2"]);
    assert_eq!(o.pending(), vec!["a"]);
    assert_eq!(o.node_usage("n2"), s("100/100,100/100"));
    assert_eq!(o.node_usage("n1"), Err(OrchError::UnknownNode));
}

#[test]
fn should_give_scarce_space_to_the_higher_priority_container_of_a_removed_node() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 60, 60).unwrap();
    o.submit("lo", 60, 60, 1).unwrap();
    o.submit("hi", 40, 40, 50).unwrap();

    assert_eq!(o.remove_node("n1"), Ok(vec!["hi->n2".to_string(), "lo->PENDING".to_string()]));
    assert_eq!(o.pending(), vec!["lo"]);
}

#[test]
fn should_return_an_empty_list_when_a_removed_node_was_empty_and_keep_everything_pending_without_nodes() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 100, 100).unwrap();
    assert_eq!(o.remove_node("n2"), Ok(vec![]));

    o.submit("a", 10, 10, 1).unwrap();
    assert_eq!(o.remove_node("n1"), Ok(vec!["a->PENDING".to_string()]));
    assert_eq!(o.placement("a"), s("PENDING"));

    o.add_node("n1", 100, 100).unwrap(); // an id may come back after removal
    assert_eq!(o.placement("a"), s("n1"));
}

// ---------------------------------------------------------------- preemption

#[test]
fn should_evict_the_lowest_priority_container_and_only_as_many_as_needed() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 1000, 1000).unwrap();
    o.submit("lowA", 400, 400, 10).unwrap();
    o.submit("lowB", 400, 400, 20).unwrap();

    assert_eq!(o.submit("urgent", 500, 500, 100), at("n"));

    assert_eq!(o.placement("lowA"), s("PENDING"));
    assert_eq!(o.placement("lowB"), s("n"));
    assert_eq!(o.containers_on("n"), Ok(vec!["lowB".to_string(), "urgent".to_string()]));
    assert_eq!(o.pending(), vec!["lowA"]);
}

#[test]
fn should_evict_the_newest_of_equal_priority_victims_first() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 1000, 1000).unwrap();
    o.submit("old", 400, 400, 10).unwrap();
    o.submit("new", 400, 400, 10).unwrap();

    o.submit("urgent", 500, 500, 100).unwrap();

    assert_eq!(o.placement("new"), s("PENDING"));
    assert_eq!(o.placement("old"), s("n"));
}

#[test]
fn should_not_preempt_when_priority_is_below_100() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("low", 100, 100, 1).unwrap();

    assert_eq!(o.submit("important", 100, 100, 99), pending_result());

    assert_eq!(o.placement("low"), s("n"));
    assert_eq!(o.pending(), vec!["important"]);
}

#[test]
fn should_only_evict_containers_of_strictly_lower_priority() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("r", 100, 100, 100).unwrap();

    assert_eq!(o.submit("same", 50, 50, 100), pending_result());
    assert_eq!(o.placement("r"), s("n"));

    assert_eq!(o.submit("above", 50, 50, 101), at("n"));
    assert_eq!(o.placement("r"), s("PENDING"));
}

#[test]
fn should_not_preempt_when_the_container_fits_somewhere_without_it() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 100, 100).unwrap();
    o.add_node("n2", 100, 100).unwrap();
    o.submit("low", 100, 100, 1).unwrap(); // n1

    assert_eq!(o.submit("urgent", 100, 100, 500), at("n2"));
    assert_eq!(o.placement("low"), s("n1"));
}

#[test]
fn should_preempt_on_the_node_needing_the_fewest_evictions_and_break_ties_by_node_id() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 1000, 1000).unwrap();
    o.add_node("n2", 1000, 1000).unwrap();
    o.submit("s1", 300, 300, 1).unwrap(); // n1
    o.submit("s2", 300, 300, 1).unwrap(); // n1
    o.submit("s3", 300, 300, 1).unwrap(); // n1
    o.submit("big", 900, 900, 50).unwrap(); // n2

    // n1 needs two evictions (s3, s2); n2 needs one (big)
    assert_eq!(o.submit("urgent", 600, 600, 100), at("n2"));
    assert_eq!(o.placement("big"), s("PENDING"));
    assert_eq!(
        o.containers_on("n1"),
        Ok(vec!["s1".to_string(), "s2".to_string(), "s3".to_string()])
    );

    let t = sut!(create(Strategy::FirstFit));
    t.add_node("n1", 1000, 1000).unwrap();
    t.add_node("n2", 1000, 1000).unwrap();
    t.submit("x", 900, 900, 1).unwrap(); // n1
    t.submit("y", 900, 900, 1).unwrap(); // n2
    assert_eq!(t.submit("urgent", 500, 500, 100), at("n1"));
    assert_eq!(t.placement("x"), s("PENDING"));
    assert_eq!(t.placement("y"), s("n2"));
}

#[test]
fn should_evict_nothing_when_no_single_node_can_be_freed_enough() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n1", 1000, 1000).unwrap();
    o.add_node("n2", 1000, 1000).unwrap();
    o.submit("guard", 800, 800, 500).unwrap(); // n1, cannot be evicted by priority 100
    o.submit("low", 100, 100, 1).unwrap(); // n1

    // larger than any node: nothing can ever be freed enough
    assert_eq!(o.submit("huge", 1500, 1500, 100), pending_result());
    assert_eq!(o.submit("mid", 900, 900, 100), at("n2")); // fits without eviction
    // n1: evicting "low" frees 200 only (guard outranks the new container); n2: "mid" has equal priority
    assert_eq!(o.submit("mid2", 500, 500, 100), pending_result());

    assert_eq!(o.placement("low"), s("n1"));
    assert_eq!(o.placement("mid"), s("n2"));
    assert_eq!(o.pending(), vec!["huge", "mid2"]);
}

#[test]
fn should_check_both_cpu_and_memory_when_counting_what_eviction_frees() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 1000, 1000).unwrap();
    o.submit("v1", 100, 800, 1).unwrap();
    o.submit("v2", 600, 100, 2).unwrap();

    assert_eq!(o.submit("u1", 300, 500, 100), at("n"));
    assert_eq!(o.placement("v1"), s("PENDING")); // memory was the missing resource
    assert_eq!(o.placement("v2"), s("n"));

    let p = sut!(create(Strategy::FirstFit));
    p.add_node("n", 1000, 1000).unwrap();
    p.submit("v1", 100, 800, 1).unwrap();
    p.submit("v2", 600, 100, 2).unwrap();
    assert_eq!(p.submit("u2", 700, 500, 100), at("n"));
    assert_eq!(p.pending(), vec!["v2", "v1"]); // both evicted; retried highest priority first
}

// ---------------------------------------------------------------- validation

#[test]
fn should_reject_invalid_nodes_and_leave_state_unchanged() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();

    assert_eq!(o.add_node("n", 5, 5), Err(OrchError::AlreadyExists));
    assert_eq!(o.add_node(" ", 5, 5), Err(OrchError::InvalidArgument));
    assert_eq!(o.add_node("z", 0, 5), Err(OrchError::InvalidArgument));
    assert_eq!(o.add_node("z", 5, 0), Err(OrchError::InvalidArgument));
    assert_eq!(o.remove_node("nope"), Err(OrchError::UnknownNode));

    assert_eq!(o.node_usage("n"), s("0/100,0/100"));
    assert_eq!(o.node_usage("z"), Err(OrchError::UnknownNode));
}

#[test]
fn should_reject_invalid_containers_and_leave_state_unchanged() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 50, 50, 1).unwrap();
    o.submit("p", 80, 80, 1).unwrap(); // pending

    assert_eq!(o.submit("a", 1, 1, 1), Err(OrchError::AlreadyExists));
    assert_eq!(o.submit("p", 1, 1, 1), Err(OrchError::AlreadyExists));
    assert_eq!(o.submit(" ", 1, 1, 1), Err(OrchError::InvalidArgument));
    assert_eq!(o.submit("z", 0, 1, 1), Err(OrchError::InvalidArgument));
    assert_eq!(o.submit("z", 1, 0, 1), Err(OrchError::InvalidArgument));
    assert_eq!(o.remove("nope"), Err(OrchError::UnknownContainer));
    assert_eq!(o.placement("nope"), Err(OrchError::UnknownContainer));
    assert_eq!(o.containers_on("nope"), Err(OrchError::UnknownNode));
    assert_eq!(o.placement("z"), Err(OrchError::UnknownContainer));

    assert_eq!(o.node_usage("n"), s("50/100,50/100"));
    assert_eq!(o.pending(), vec!["p"]);
}

#[test]
fn should_accept_negative_priorities() {
    let o = sut!(create(Strategy::FirstFit));
    o.add_node("n", 100, 100).unwrap();
    o.submit("a", 100, 100, 0).unwrap();
    o.submit("neg", 100, 100, -5).unwrap();
    o.submit("zero", 100, 100, 0).unwrap();

    assert_eq!(o.pending(), vec!["zero", "neg"]);
}

// ---------------------------------------------------------------- concurrency

#[test]
fn should_never_over_commit_a_node_under_concurrent_submits() {
    let o = sut!(create(Strategy::FirstFit));
    for i in 0..4 {
        o.add_node(&format!("n{i}"), 1000, 1000).unwrap();
    }
    let done = AtomicBool::new(false);
    let finished = AtomicUsize::new(0);
    let violations = AtomicUsize::new(0);
    let placed = AtomicUsize::new(0);

    run_concurrently(17, |t| {
        if t == 16 {
            while !done.load(Ordering::SeqCst) {
                for n in 0..4 {
                    if !within_capacity(&o.node_usage(&format!("n{n}")).unwrap()) {
                        violations.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
            return;
        }
        for i in 0..100 {
            if o.submit(&format!("t{t}-{i}"), 10, 10, 1).unwrap().is_some() {
                placed.fetch_add(1, Ordering::SeqCst);
            }
        }
        if finished.fetch_add(1, Ordering::SeqCst) + 1 == 16 {
            done.store(true, Ordering::SeqCst);
        }
    });

    assert_eq!(violations.load(Ordering::SeqCst), 0);
    assert_eq!(placed.load(Ordering::SeqCst), 400); // 4 nodes x 100 containers of 10
    for n in 0..4 {
        assert_eq!(o.node_usage(&format!("n{n}")), s("1000/1000,1000/1000"));
        assert_eq!(o.containers_on(&format!("n{n}")).unwrap().len(), 100);
    }
    assert_eq!(o.pending().len(), 1200);
}

#[test]
fn should_stay_consistent_when_submits_and_removals_race() {
    let o = sut!(create(Strategy::BestFit));
    for i in 0..3 {
        o.add_node(&format!("n{i}"), 500, 500).unwrap();
    }

    run_concurrently(16, |t| {
        for i in 0..60 {
            o.submit(&format!("t{t}-{i}"), 10, 10, (t % 4) as i32).unwrap();
        }
        for i in (0..60).step_by(2) {
            o.remove(&format!("t{t}-{i}")).unwrap();
        }
    });

    let mut running = 0;
    for n in 0..3 {
        let on = o.containers_on(&format!("n{n}")).unwrap();
        running += on.len();
        assert_eq!(
            o.node_usage(&format!("n{n}")),
            s(&format!("{0}/500,{0}/500", on.len() * 10))
        );
    }
    // 16 x 30 survivors, 3 x 50 slots
    assert_eq!(running, 150);
    assert_eq!(o.pending().len(), 16 * 30 - 150);
}

#![allow(unused_imports, clippy::needless_range_loop)]
use p106_job_scheduler::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const FEWEST: &str = FEWEST_UNFINISHED;
const MOST: &str = MOST_FINISHED;

fn sub(s: &Arc<dyn JobScheduler>, job: &str, req: &[&str], c: &str) -> Option<String> {
    s.submit_job_with(job, req, c).unwrap()
}

fn some(m: &str) -> Option<String> {
    Some(m.to_string())
}

#[test]
fn should_assign_a_job_to_the_only_eligible_machine() {
    let s = sut!(create());
    s.add_machine("m1", &["gpu", "ssd"]).unwrap();

    assert_eq!(sub(&s, "j1", &["gpu"], FEWEST), some("m1"));
    assert_eq!(s.unfinished_count("m1"), Ok(1));
    assert_eq!(s.finished_count("m1"), Ok(0));
    assert_eq!(s.machine_of("j1"), Ok("m1".to_string()));
}

#[test]
fn should_return_none_when_no_machine_has_all_required_capabilities() {
    let s = sut!(create());
    s.add_machine("m1", &["gpu"]).unwrap();
    s.add_machine("m2", &["ssd"]).unwrap();

    assert_eq!(sub(&s, "j1", &["gpu", "ssd"], FEWEST), None);
    assert_eq!(s.unfinished_count("m1"), Ok(0));
    assert_eq!(s.unfinished_count("m2"), Ok(0));
}

#[test]
fn should_return_none_when_there_are_no_machines() {
    let s = sut!(create());

    assert_eq!(sub(&s, "j1", &[], FEWEST), None);
    assert_eq!(sub(&s, "j2", &["gpu"], MOST), None);
}

#[test]
fn should_fit_a_job_with_no_requirements_on_any_machine() {
    let s = sut!(create());
    s.add_machine("b", &[]).unwrap();
    s.add_machine("a", &["gpu"]).unwrap();

    assert_eq!(sub(&s, "j1", &[], FEWEST), some("a"));
    assert_eq!(sub(&s, "j2", &[], FEWEST), some("b"));
}

#[test]
fn should_match_capabilities_exactly() {
    let s = sut!(create());
    s.add_machine("m1", &["GPU", "gpu-a100"]).unwrap();

    assert_eq!(sub(&s, "j1", &["gpu"], FEWEST), None);
    assert_eq!(sub(&s, "j2", &["gpu-a"], FEWEST), None);
    assert_eq!(sub(&s, "j3", &["GPU"], FEWEST), some("m1"));
}

#[test]
fn should_pick_the_machine_with_fewest_unfinished_jobs() {
    let s = sut!(create());
    for m in ["m1", "m2", "m3"] {
        s.add_machine(m, &["x"]).unwrap();
    }

    assert_eq!(sub(&s, "j1", &["x"], FEWEST), some("m1"));
    assert_eq!(sub(&s, "j2", &["x"], FEWEST), some("m2"));
    assert_eq!(sub(&s, "j3", &["x"], FEWEST), some("m3"));
    assert_eq!(sub(&s, "j4", &["x"], FEWEST), some("m1"));
    s.finish_job("j2").unwrap();
    assert_eq!(sub(&s, "j5", &["x"], FEWEST), some("m2"));
}

#[test]
fn should_break_fewest_unfinished_ties_by_smallest_machine_id_not_insertion_order() {
    let s = sut!(create());
    s.add_machine("m9", &["x"]).unwrap();
    s.add_machine("m10", &["x"]).unwrap();
    s.add_machine("m2", &["x"]).unwrap();

    // lexicographic: "m10" < "m2" < "m9"
    assert_eq!(sub(&s, "j1", &["x"], FEWEST), some("m10"));
    assert_eq!(sub(&s, "j2", &["x"], FEWEST), some("m2"));
    assert_eq!(sub(&s, "j3", &["x"], FEWEST), some("m9"));
}

#[test]
fn should_pick_the_machine_with_most_finished_jobs() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST); // a
    sub(&s, "j2", &["x"], FEWEST); // b
    sub(&s, "j3", &["x"], FEWEST); // a
    sub(&s, "j4", &["x"], FEWEST); // b
    s.finish_job("j2").unwrap();
    s.finish_job("j4").unwrap();
    s.finish_job("j1").unwrap();

    assert_eq!(s.finished_count("a"), Ok(1));
    assert_eq!(s.finished_count("b"), Ok(2));
    assert_eq!(sub(&s, "j5", &["x"], MOST), some("b"));
}

#[test]
fn should_break_most_finished_ties_by_smallest_machine_id() {
    let s = sut!(create());
    s.add_machine("zeta", &["x"]).unwrap();
    s.add_machine("alpha", &["x"]).unwrap();
    s.add_machine("mid", &["x"]).unwrap();

    assert_eq!(sub(&s, "j1", &["x"], MOST), some("alpha"));
    // MOST_FINISHED ignores load: all three still tie at 0 finished
    assert_eq!(sub(&s, "j2", &["x"], MOST), some("alpha"));
    assert_eq!(s.unfinished_count("alpha"), Ok(2));
}

#[test]
fn should_move_a_job_from_unfinished_to_finished_when_it_completes() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST);
    sub(&s, "j2", &["x"], FEWEST);

    s.finish_job("j1").unwrap();

    assert_eq!(s.unfinished_count("m1"), Ok(1));
    assert_eq!(s.finished_count("m1"), Ok(1));
    s.finish_job("j2").unwrap();
    assert_eq!(s.unfinished_count("m1"), Ok(0));
    assert_eq!(s.finished_count("m1"), Ok(2));
}

#[test]
fn should_rank_only_among_eligible_machines() {
    let s = sut!(create());
    s.add_machine("idle", &["cpu"]).unwrap();
    s.add_machine("busy", &["cpu", "gpu"]).unwrap();
    sub(&s, "j1", &["gpu"], FEWEST); // busy (only eligible)
    sub(&s, "j2", &["gpu"], FEWEST); // busy

    assert_eq!(s.unfinished_count("idle"), Ok(0));
    assert_eq!(s.unfinished_count("busy"), Ok(2));
    assert_eq!(sub(&s, "j3", &["gpu"], FEWEST), some("busy"));
    assert_eq!(sub(&s, "j4", &["cpu"], FEWEST), some("idle"));
}

#[test]
fn should_rank_most_finished_only_among_eligible_machines() {
    let s = sut!(create());
    s.add_machine("veteran", &["cpu"]).unwrap();
    s.add_machine("novice", &["cpu", "gpu"]).unwrap();
    sub(&s, "j1", &["cpu"], FEWEST); // tie at 0 -> "novice" < "veteran"
    assert_eq!(s.machine_of("j1"), Ok("novice".to_string()));
    sub(&s, "j2", &["cpu"], FEWEST); // veteran
    s.finish_job("j2").unwrap();

    assert_eq!(sub(&s, "j3", &["cpu"], MOST), some("veteran"));
    assert_eq!(sub(&s, "j4", &["gpu"], MOST), some("novice"));
}

#[test]
fn should_use_fewest_unfinished_as_the_initial_default_criterion() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();

    assert_eq!(s.submit_job("j1", &["x"]), Ok(some("a")));
    assert_eq!(s.submit_job("j2", &["x"]), Ok(some("b")));
    assert_eq!(s.submit_job("j3", &["x"]), Ok(some("a")));
}

#[test]
fn should_honour_a_changed_default_criterion_and_let_a_call_override_it() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST); // a
    sub(&s, "j2", &["x"], FEWEST); // b
    s.finish_job("j2").unwrap(); // b: 1 finished, 0 unfinished; a: 1 unfinished

    s.set_default_criterion(MOST).unwrap();
    assert_eq!(s.submit_job("j3", &["x"]), Ok(some("b")));
    assert_eq!(sub(&s, "j4", &["x"], FEWEST), some("a")); // a:1 b:1 -> tie -> a
    s.set_default_criterion(FEWEST).unwrap();
    assert_eq!(s.submit_job("j5", &["x"]), Ok(some("b"))); // a:2 unfinished, b:1
}

#[test]
fn should_assign_machines_added_after_jobs_were_submitted() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();
    assert_eq!(sub(&s, "j1", &["gpu"], FEWEST), None);

    s.add_machine("m2", &["x", "gpu"]).unwrap();

    assert_eq!(sub(&s, "j2", &["gpu"], FEWEST), some("m2"));
    assert_eq!(sub(&s, "j3", &["x"], FEWEST), some("m1"));
}

#[test]
fn should_not_record_a_job_that_found_no_machine() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();

    assert_eq!(sub(&s, "j1", &["gpu"], FEWEST), None);
    assert_eq!(s.finish_job("j1"), Err(SchedulerError::UnknownJob));
    assert_eq!(s.machine_of("j1"), Err(SchedulerError::UnknownJob));

    s.add_machine("m2", &["gpu"]).unwrap();
    assert_eq!(sub(&s, "j1", &["gpu"], FEWEST), some("m2")); // id reusable after a None result
}

#[test]
fn should_reject_duplicate_and_invalid_machines() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();

    assert_eq!(
        s.add_machine("m1", &["y"]),
        Err(SchedulerError::DuplicateMachine)
    );
    assert_eq!(
        s.add_machine(" ", &["y"]),
        Err(SchedulerError::InvalidArgument)
    );
    assert_eq!(
        s.add_machine("m3", &[" "]),
        Err(SchedulerError::InvalidArgument)
    );
    // the failed duplicate must not have replaced the original capabilities
    assert_eq!(sub(&s, "j1", &["x"], FEWEST), some("m1"));
    assert_eq!(sub(&s, "j2", &["y"], FEWEST), None);
    // failed adds registered nothing
    assert_eq!(
        s.unfinished_count("m3"),
        Err(SchedulerError::UnknownMachine)
    );
}

#[test]
fn should_reject_invalid_submissions_and_change_nothing() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();

    assert_eq!(
        s.submit_job_with(" ", &["x"], FEWEST),
        Err(SchedulerError::InvalidArgument)
    );
    assert_eq!(
        s.submit_job_with("j1", &[" "], FEWEST),
        Err(SchedulerError::InvalidArgument)
    );
    assert_eq!(
        s.submit_job_with("j1", &["x"], "ROUND_ROBIN"),
        Err(SchedulerError::UnknownCriterion)
    );
    assert_eq!(
        s.submit_job_with("j1", &["x"], ""),
        Err(SchedulerError::UnknownCriterion)
    );
    assert_eq!(
        s.submit_job_with("j1", &["x"], "fewest_unfinished"),
        Err(SchedulerError::UnknownCriterion)
    );

    assert_eq!(s.unfinished_count("m1"), Ok(0));
    assert_eq!(s.finish_job("j1"), Err(SchedulerError::UnknownJob));
}

#[test]
fn should_reject_a_job_id_that_is_already_assigned() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();
    s.add_machine("m2", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST); // m1

    assert_eq!(
        s.submit_job_with("j1", &["x"], FEWEST),
        Err(SchedulerError::DuplicateJob)
    );
    s.finish_job("j1").unwrap();
    assert_eq!(
        s.submit_job_with("j1", &["x"], FEWEST),
        Err(SchedulerError::DuplicateJob)
    );

    assert_eq!(s.unfinished_count("m1"), Ok(0));
    assert_eq!(s.unfinished_count("m2"), Ok(0));
    assert_eq!(s.finished_count("m1"), Ok(1));
}

#[test]
fn should_reject_an_unknown_default_criterion_and_keep_the_previous_one() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST); // a

    assert_eq!(
        s.set_default_criterion("bogus"),
        Err(SchedulerError::UnknownCriterion)
    );

    assert_eq!(s.submit_job("j2", &["x"]), Ok(some("b"))); // still FEWEST_UNFINISHED
}

#[test]
fn should_reject_finishing_an_unknown_job_or_finishing_twice() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST);
    sub(&s, "j2", &["x"], FEWEST);

    assert_eq!(s.finish_job("nope"), Err(SchedulerError::UnknownJob));
    s.finish_job("j1").unwrap();
    assert_eq!(s.finish_job("j1"), Err(SchedulerError::AlreadyFinished));

    assert_eq!(s.unfinished_count("m1"), Ok(1));
    assert_eq!(s.finished_count("m1"), Ok(1));
}

#[test]
fn should_reject_counts_for_unknown_machines() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();

    assert_eq!(
        s.unfinished_count("nope"),
        Err(SchedulerError::UnknownMachine)
    );
    assert_eq!(
        s.finished_count("nope"),
        Err(SchedulerError::UnknownMachine)
    );
    assert_eq!(s.machine_of("nope"), Err(SchedulerError::UnknownJob));
}

#[test]
fn should_balance_exactly_when_many_threads_submit_at_once() {
    let s = sut!(create());
    for m in ["m1", "m2", "m3", "m4"] {
        s.add_machine(m, &["x"]).unwrap();
    }

    run_concurrently(16, |t| {
        for i in 0..250 {
            assert!(s
                .submit_job_with(&format!("j-{t}-{i}"), &["x"], FEWEST)
                .unwrap()
                .is_some());
        }
    });

    // 4000 jobs over 4 machines picked atomically by "fewest unfinished": no machine can drift ahead.
    for m in ["m1", "m2", "m3", "m4"] {
        assert_eq!(s.unfinished_count(m), Ok(1000));
    }
}

#[test]
fn should_let_exactly_one_thread_finish_a_job() {
    let s = sut!(create());
    s.add_machine("m1", &["x"]).unwrap();
    for i in 0..200 {
        sub(&s, &format!("j{i}"), &["x"], FEWEST);
    }
    let ok = AtomicUsize::new(0);
    let rejected = AtomicUsize::new(0);

    run_concurrently(8, |_| {
        for i in 0..200 {
            match s.finish_job(&format!("j{i}")) {
                Ok(()) => ok.fetch_add(1, Ordering::SeqCst),
                Err(SchedulerError::AlreadyFinished) => rejected.fetch_add(1, Ordering::SeqCst),
                Err(e) => panic!("unexpected {e:?}"),
            };
        }
    });

    assert_eq!(ok.load(Ordering::SeqCst), 200);
    assert_eq!(rejected.load(Ordering::SeqCst), 1400);
    assert_eq!(s.unfinished_count("m1"), Ok(0));
    assert_eq!(s.finished_count("m1"), Ok(200));
}

#[test]
fn should_keep_counts_consistent_when_submitting_and_finishing_concurrently() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();

    run_concurrently(8, |t| {
        for i in 0..300 {
            let id = format!("j-{t}-{i}");
            let c = if i % 2 == 0 { FEWEST } else { MOST };
            assert!(s.submit_job_with(&id, &["x"], c).unwrap().is_some());
            if i % 3 == 0 {
                s.finish_job(&id).unwrap();
            }
        }
    });

    let fin = s.finished_count("a").unwrap() + s.finished_count("b").unwrap();
    let unf = s.unfinished_count("a").unwrap() + s.unfinished_count("b").unwrap();
    assert_eq!(fin + unf, 8 * 300);
    assert_eq!(fin, 8 * 100);
}

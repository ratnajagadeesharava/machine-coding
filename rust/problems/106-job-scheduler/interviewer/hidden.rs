#![allow(unused_imports, clippy::needless_range_loop)]
use p106_job_scheduler::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const FEWEST: &str = FEWEST_UNFINISHED;
const MOST: &str = MOST_FINISHED;

fn sub(s: &Arc<dyn JobScheduler>, job: &str, req: &[&str], c: &str) -> Option<String> {
    s.submit_job_with(job, req, c).unwrap()
}

fn some(m: &str) -> Option<String> {
    Some(m.to_string())
}

#[test]
fn should_interleave_both_criteria_on_the_same_machines() {
    let s = sut!(create());
    for m in ["a", "b", "c"] {
        s.add_machine(m, &["x"]).unwrap();
    }

    assert_eq!(sub(&s, "j1", &["x"], MOST), some("a")); // all tie at 0 finished
    assert_eq!(sub(&s, "j2", &["x"], FEWEST), some("b")); // a:1 b:0 c:0 -> b
    s.finish_job("j2").unwrap(); // b finished 1
    assert_eq!(sub(&s, "j3", &["x"], MOST), some("b")); // b has the most finished
    assert_eq!(sub(&s, "j4", &["x"], FEWEST), some("c")); // a:1 b:1 c:0
    s.finish_job("j4").unwrap(); // c finished 1
    s.finish_job("j3").unwrap(); // b finished 2
    assert_eq!(sub(&s, "j5", &["x"], MOST), some("b"));
    assert_eq!(sub(&s, "j6", &["x"], FEWEST), some("c")); // a:1 b:1 c:0
    assert_eq!(s.unfinished_count("a"), Ok(1));
    assert_eq!(s.unfinished_count("b"), Ok(1));
    assert_eq!(s.unfinished_count("c"), Ok(1));
    assert_eq!(s.finished_count("b"), Ok(2));
    assert_eq!(s.finished_count("c"), Ok(1));
}

#[test]
fn should_never_pick_a_machine_missing_even_one_capability_across_many_machines() {
    let s = sut!(create());
    let all = ["c0", "c1", "c2", "c3", "c4"];
    let mut offered: HashMap<String, HashSet<&str>> = HashMap::new();
    for m in 0..200usize {
        let caps: Vec<&str> = (0..all.len())
            .filter(|k| (m >> k) & 1 == 1)
            .map(|k| all[k])
            .collect();
        let id = format!("m{m:03}");
        s.add_machine(&id, &caps).unwrap();
        offered.insert(id, caps.into_iter().collect());
    }
    let mut submitted = 0;
    for j in 0..3000usize {
        let need: Vec<&str> = (0..all.len())
            .filter(|k| ((j * 7 + j / 5) >> k) & 1 == 1 && (j % 3 != 0 || *k < 2))
            .map(|k| all[k])
            .collect();
        let c = if j % 2 == 0 { FEWEST } else { MOST };
        if let Some(m) = s.submit_job_with(&format!("job{j}"), &need, c).unwrap() {
            submitted += 1;
            assert!(need.iter().all(|n| offered[&m].contains(n)));
            if j % 4 == 0 {
                s.finish_job(&format!("job{j}")).unwrap();
            }
        }
    }
    let total: usize = offered
        .keys()
        .map(|m| s.unfinished_count(m).unwrap() + s.finished_count(m).unwrap())
        .sum();
    assert_eq!(total, submitted);
}

#[test]
fn should_spread_jobs_evenly_across_eligible_machines_only_and_ignore_others() {
    let s = sut!(create());
    for i in 0..10 {
        s.add_machine(&format!("gpu-{i}"), &["gpu"]).unwrap();
        s.add_machine(&format!("cpu-{i}"), &["cpu"]).unwrap();
    }
    for j in 0..100 {
        assert!(sub(&s, &format!("g{j}"), &["gpu"], FEWEST).is_some());
    }
    for i in 0..10 {
        assert_eq!(s.unfinished_count(&format!("gpu-{i}")), Ok(10));
        assert_eq!(s.unfinished_count(&format!("cpu-{i}")), Ok(0));
    }
}

#[test]
fn should_accumulate_most_finished_on_one_machine_while_load_grows_there() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();
    sub(&s, "seed", &["x"], FEWEST); // a
    sub(&s, "seed2", &["x"], FEWEST); // b
    s.finish_job("seed2").unwrap(); // b leads on finished

    for i in 0..50 {
        assert_eq!(sub(&s, &format!("m{i}"), &["x"], MOST), some("b")); // load on b is ignored
    }
    assert_eq!(s.unfinished_count("b"), Ok(50));
    assert_eq!(s.unfinished_count("a"), Ok(1));
    assert_eq!(sub(&s, "f", &["x"], FEWEST), some("a"));
}

#[test]
fn should_leave_no_trace_of_rejected_operations() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    sub(&s, "j1", &["x"], FEWEST);
    s.finish_job("j1").unwrap();

    assert_eq!(s.finish_job("j1"), Err(SchedulerError::AlreadyFinished));
    assert_eq!(
        s.submit_job_with("j1", &["x"], FEWEST),
        Err(SchedulerError::DuplicateJob)
    );
    assert_eq!(
        s.submit_job_with("j2", &["x"], "BOGUS"),
        Err(SchedulerError::UnknownCriterion)
    );
    assert_eq!(
        s.add_machine("a", &["y"]),
        Err(SchedulerError::DuplicateMachine)
    );

    assert_eq!(s.unfinished_count("a"), Ok(0));
    assert_eq!(s.finished_count("a"), Ok(1));
    assert_eq!(sub(&s, "j2", &["x"], FEWEST), some("a")); // j2 id still free after BOGUS
    assert_eq!(sub(&s, "j3", &["y"], FEWEST), None); // original capabilities kept
}

#[test]
fn should_register_exactly_one_machine_when_many_threads_add_the_same_id() {
    let s = sut!(create());
    let ok = AtomicUsize::new(0);

    run_concurrently(16, |t| match s.add_machine("only", &[&format!("c{t}")]) {
        Ok(()) => {
            ok.fetch_add(1, Ordering::SeqCst);
        }
        Err(SchedulerError::DuplicateMachine) => {}
        Err(e) => panic!("unexpected {e:?}"),
    });

    assert_eq!(ok.load(Ordering::SeqCst), 1);
}

#[test]
fn should_accept_each_job_id_once_when_many_threads_submit_the_same_ids() {
    let s = sut!(create());
    s.add_machine("a", &["x"]).unwrap();
    s.add_machine("b", &["x"]).unwrap();
    let accepted = AtomicUsize::new(0);

    run_concurrently(8, |_| {
        for i in 0..500 {
            match s.submit_job_with(&format!("job-{i}"), &["x"], FEWEST) {
                Ok(_) => {
                    accepted.fetch_add(1, Ordering::SeqCst);
                }
                Err(SchedulerError::DuplicateJob) => {}
                Err(e) => panic!("unexpected {e:?}"),
            }
        }
    });

    let a = s.unfinished_count("a").unwrap();
    let b = s.unfinished_count("b").unwrap();
    assert_eq!(accepted.load(Ordering::SeqCst), 500);
    assert_eq!(a + b, 500);
    assert!(a.abs_diff(b) <= 1);
}

#[test]
fn should_keep_invariants_while_machines_join_and_jobs_flow_concurrently() {
    let s = sut!(create());
    s.add_machine("seed", &["x"]).unwrap();
    let placed = AtomicUsize::new(0);

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..200 {
                s.add_machine(&format!("late-{i}"), &["x"]).unwrap();
            }
            return;
        }
        for i in 0..1_000 {
            let id = format!("j-{t}-{i}");
            let c = if i % 2 == 0 { FEWEST } else { MOST };
            assert!(s.submit_job_with(&id, &["x"], c).unwrap().is_some());
            placed.fetch_add(1, Ordering::SeqCst);
            if i % 5 == 0 {
                s.finish_job(&id).unwrap();
            }
        }
    });

    let mut unfinished = s.unfinished_count("seed").unwrap();
    let mut finished = s.finished_count("seed").unwrap();
    for i in 0..200 {
        unfinished += s.unfinished_count(&format!("late-{i}")).unwrap();
        finished += s.finished_count(&format!("late-{i}")).unwrap();
    }
    assert_eq!(unfinished + finished, 8_000);
    assert_eq!(finished, 8 * 200);
    assert_eq!(placed.load(Ordering::SeqCst), 8_000);
}

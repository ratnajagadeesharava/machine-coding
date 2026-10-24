#![allow(unused_imports, clippy::needless_range_loop)]
use p129_job_candidate_matching::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

#[test]
fn should_not_overflow_the_salary_bonus_for_huge_salaries() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 0, "Pune", u64::MAX).unwrap();
    m.register_candidate("half", &[], 0, "Pune", u64::MAX / 2, false)
        .unwrap();
    m.register_candidate("over_half", &[], 0, "Pune", u64::MAX / 2 + 1, false)
        .unwrap();
    m.register_candidate("max", &[], 0, "Pune", u64::MAX, false)
        .unwrap();

    assert_eq!(m.match_score("J", "half"), Ok(70));
    assert_eq!(m.match_score("J", "over_half"), Ok(60));
    assert_eq!(m.match_score("J", "max"), Ok(60));
}

#[test]
fn should_break_ties_by_years_descending_then_id_as_strings_not_numbers() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 0, "Pune", 100).unwrap();
    for id in ["c9", "c10", "c2", "c1", "Cap"] {
        m.register_candidate(id, &[], 15, "Pune", 10, false).unwrap();
    }
    m.register_candidate("older", &[], 30, "Pune", 10, false)
        .unwrap();

    assert_eq!(
        m.top_candidates("J", 10).unwrap(),
        ["older", "Cap", "c1", "c10", "c2", "c9"]
    );
}

#[test]
fn should_match_a_brute_force_oracle_on_random_data() {
    let m = sut!(create());
    let mut seed = 42u64;
    let mut next = move |n: u64| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) % n
    };
    let pool = ["java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s"];
    let upper = ["JAVA", "SQL", "GO", "RUST", "KAFKA", "REDIS", "DOCKER", "K8S"];
    let cities = ["Pune", "Delhi", "Goa"];
    let mut years = vec![0u32; 300];
    for c in 0..300 {
        let n = 1 + next(6) as usize;
        let sk: Vec<&str> = (0..n).map(|_| pool[next(8) as usize]).collect();
        years[c] = next(15) as u32;
        m.register_candidate(
            &format!("c{c}"),
            &sk,
            years[c],
            cities[next(3) as usize],
            10 + next(90),
            next(4) == 0,
        )
        .unwrap();
    }
    for j in 0..40 {
        let nice: Vec<&str> = (0..4).map(|_| upper[next(8) as usize]).collect();
        m.post_job(
            &format!("j{j}"),
            "t",
            &[pool[next(8) as usize]],
            &nice,
            next(8) as u32,
            cities[next(3) as usize],
            30 + next(80),
        )
        .unwrap();
    }

    for j in 0..40 {
        let job = format!("j{j}");
        let mut expected: Vec<(u32, u32, String)> = (0..300)
            .filter_map(|c| {
                let s = m.match_score(&job, &format!("c{c}")).unwrap();
                (s > 0).then(|| (s, years[c], format!("c{c}")))
            })
            .collect();
        expected.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        let all: Vec<String> = expected.iter().map(|e| e.2.clone()).collect();
        assert_eq!(m.top_candidates(&job, 1_000).unwrap(), all, "{job}");
        let five: Vec<String> = all.iter().take(5).cloned().collect();
        assert_eq!(m.top_candidates(&job, 5).unwrap(), five, "{job}");
    }
    for c in (0..300).step_by(7) {
        let cid = format!("c{c}");
        let mut expected: Vec<(u32, String)> = (0..40)
            .filter_map(|j| {
                let s = m.match_score(&format!("j{j}"), &cid).unwrap();
                (s > 0).then(|| (s, format!("j{j}")))
            })
            .collect();
        expected.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let want: Vec<String> = expected.into_iter().map(|e| e.1).collect();
        assert_eq!(m.top_jobs(&cid, 100).unwrap(), want, "{cid}");
    }
}

#[test]
fn should_never_accept_an_application_after_close_job_returned() {
    for _ in 0..20 {
        let m = sut!(create());
        m.post_job("J", "t", &["java"], &[], 0, "Pune", 100).unwrap();
        for i in 0..200 {
            m.register_candidate(&format!("c{i}"), &["java"], 1, "Pune", 10, false)
                .unwrap();
        }
        let accepted = Mutex::new(Vec::new());
        let closed = AtomicBool::new(false);
        let late = AtomicBool::new(false);

        run_concurrently(9, |t| {
            if t == 8 {
                m.close_job("J").unwrap();
                closed.store(true, Ordering::SeqCst);
                return;
            }
            for i in (t..200).step_by(8) {
                let closed_before = closed.load(Ordering::SeqCst);
                if m.apply(&format!("c{i}"), "J").is_ok() {
                    accepted.lock().unwrap().push(format!("c{i}"));
                    if closed_before {
                        late.store(true, Ordering::SeqCst);
                    }
                }
            }
        });

        assert!(!late.load(Ordering::SeqCst));
        let mut apps = m.applications("J").unwrap();
        let mut acc = accepted.into_inner().unwrap();
        apps.sort();
        acc.sort();
        assert_eq!(apps, acc);
    }
}

#[test]
fn should_stay_consistent_while_candidates_are_updated_and_ranked() {
    let m = sut!(create());
    m.post_job("J", "t", &["java"], &["go"], 1, "Pune", 100)
        .unwrap();
    for i in 0..100 {
        m.register_candidate(&format!("c{i}"), &["java"], 3, "Pune", 10, false)
            .unwrap();
    }

    run_concurrently(8, |t| {
        if t < 4 {
            for i in 0..3_000usize {
                let id = format!("c{}", (i * 4 + t) % 100);
                if i % 2 == 0 {
                    m.update_candidate(&id, &["python"], 3, "Pune", 10, false)
                        .unwrap();
                } else {
                    m.update_candidate(&id, &["java", "go"], 3, "Pune", 10, false)
                        .unwrap();
                }
            }
        } else {
            for i in 0..1_500usize {
                let top = m.top_candidates("J", 10).unwrap();
                assert!(top.len() <= 10);
                assert_eq!(top.iter().collect::<HashSet<_>>().len(), top.len());
                m.top_jobs(&format!("c{}", i % 100), 5).unwrap();
            }
        }
    });

    let top = m.top_candidates("J", 1_000).unwrap();
    for i in 0..100 {
        let id = format!("c{i}");
        assert_eq!(top.contains(&id), m.match_score("J", &id).unwrap() > 0);
    }
    m.update_candidate("c0", &["java", "go"], 3, "Pune", 10, false)
        .unwrap();
    assert_eq!(m.top_candidates("J", 1).unwrap(), ["c0"]);
}

#[test]
fn should_handle_the_scale_scenario_with_concurrent_profile_updates() {
    let m = sut!(create());
    let skills = ["java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s", "aws", "react"];
    let cities = ["Pune", "Delhi", "Mumbai", "Goa"];
    for j in 0..500usize {
        m.post_job(
            &format!("job{j}"),
            "t",
            &[skills[j % 10]],
            &[skills[(j + 2) % 10], skills[(j + 5) % 10]],
            (j % 8) as u32,
            cities[j % 4],
            50 + (j % 50) as u64,
        )
        .unwrap();
    }
    for c in 0..5_000usize {
        m.register_candidate(
            &format!("cand{c}"),
            &[skills[c % 10], skills[(c / 10) % 10], skills[(c * 7) % 10]],
            (c % 12) as u32,
            cities[(c / 3) % 4],
            10 + (c % 60) as u64,
            c % 5 == 0,
        )
        .unwrap();
    }

    let start = Instant::now();
    run_concurrently(5, |t| {
        if t == 4 {
            for i in 0..2_000usize {
                m.update_candidate(
                    &format!("cand{}", i * 2),
                    &[skills[i % 10], "go"],
                    (i % 12) as u32,
                    cities[i % 4],
                    20,
                    true,
                )
                .unwrap();
            }
            return;
        }
        for i in 0..250usize {
            let top = m
                .top_candidates(&format!("job{}", (i * 4 + t) % 500), 10)
                .unwrap();
            assert!(top.len() <= 10);
        }
    });
    for i in 0..1_000usize {
        m.top_candidates(&format!("job{}", i % 500), 10).unwrap();
    }
    let ms = start.elapsed().as_millis();

    assert!(ms < 8_000, "scale run took {ms} ms");
}

#[test]
fn should_count_duplicate_and_mixed_case_skills_once_after_normalisation() {
    let m = sut!(create());
    m.post_job(
        "J",
        "t",
        &["Java", "JAVA", " java "],
        &["Go", "gO", "go ", "RUST", "rust"],
        0,
        "Pune",
        100,
    )
    .unwrap();
    m.register_candidate("c", &["GO", "go", "java"], 0, "Pune", 90, false)
        .unwrap();

    assert_eq!(m.match_score("J", "c"), Ok(70));
}

#[test]
fn should_keep_applications_per_job_independent() {
    let m = sut!(create());
    m.post_job("J", "t", &["java"], &[], 0, "Pune", 100).unwrap();
    m.post_job("K", "t", &["java"], &[], 0, "Pune", 100).unwrap();
    m.register_candidate("a", &["java"], 1, "Pune", 10, false)
        .unwrap();
    m.register_candidate("b", &["java"], 1, "Pune", 10, false)
        .unwrap();
    m.apply("a", "J").unwrap();
    m.apply("b", "J").unwrap();
    m.apply("a", "K").unwrap();

    assert_eq!(m.withdraw("b", "K"), Err(MatchError::NotApplied));
    m.withdraw("a", "K").unwrap();

    assert_eq!(m.applications("J").unwrap(), ["a", "b"]);
    assert!(m.applications("K").unwrap().is_empty());
}

#[test]
fn should_report_a_closed_job_before_eligibility_and_change_nothing() {
    let m = sut!(create());
    m.post_job("J", "t", &["java"], &[], 0, "Pune", 100).unwrap();
    m.register_candidate("a", &["java"], 1, "Pune", 10, false)
        .unwrap();
    m.register_candidate("x", &["go"], 1, "Pune", 10, false)
        .unwrap();
    m.apply("a", "J").unwrap();
    m.close_job("J").unwrap();

    assert_eq!(m.apply("x", "J"), Err(MatchError::JobClosed));
    assert_eq!(m.apply("a", "J"), Err(MatchError::JobClosed));
    assert_eq!(m.applications("J").unwrap(), ["a"]);
}

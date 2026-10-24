#![allow(unused_imports, clippy::needless_range_loop)]
use p129_job_candidate_matching::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

type Svc = Arc<dyn JobMatching>;

fn simple_job(m: &Svc, id: &str) {
    m.post_job(id, "Engineer", &["java"], &[], 2, "Pune", 100)
        .unwrap();
}

fn cand(m: &Svc, id: &str, years: u32, loc: &str, salary: u64, remote: bool, skills: &[&str]) {
    m.register_candidate(id, skills, years, loc, salary, remote)
        .unwrap();
}

fn ids(v: Result<Vec<String>, MatchError>) -> Vec<String> {
    v.unwrap()
}

fn backend_example(m: &Svc) {
    m.post_job(
        "backend",
        "Backend Engineer",
        &["java", "sql"],
        &["kafka", "redis", "docker"],
        3,
        "Pune",
        100,
    )
    .unwrap();
    cand(m, "alice", 6, "Pune", 40, false, &["Java", "SQL", "Kafka", "Redis"]);
    cand(m, "bob", 10, "Delhi", 60, true, &["java", "sql", "docker", "kafka", "redis"]);
    cand(m, "xena", 5, "Delhi", 40, true, &["java", "sql", "kafka", "redis", "docker"]);
    cand(m, "zed", 5, "Delhi", 40, true, &["java", "sql", "kafka", "redis", "docker"]);
    cand(m, "erin", 20, "pune", 50, false, &[" JAVA ", " Sql "]);
    cand(m, "carol", 8, "Pune", 10, false, &["java"]);
    cand(m, "dave", 9, "Mumbai", 10, false, &["java", "sql"]);
    cand(m, "frank", 2, "Pune", 10, false, &["java", "sql"]);
    cand(m, "gina", 4, "Pune", 101, false, &["java", "sql"]);
}

#[test]
fn should_score_the_worked_example_exactly() {
    let m = sut!(create());
    backend_example(&m);

    assert_eq!(m.match_score("backend", "alice"), Ok(96));
    assert_eq!(m.match_score("backend", "bob"), Ok(94));
    assert_eq!(m.match_score("backend", "xena"), Ok(94));
    assert_eq!(m.match_score("backend", "zed"), Ok(94));
    assert_eq!(m.match_score("backend", "erin"), Ok(90));
    for c in ["carol", "dave", "frank", "gina"] {
        assert_eq!(m.match_score("backend", c), Ok(0), "{c}");
    }
}

#[test]
fn should_rank_candidates_by_score_then_years_then_id() {
    let m = sut!(create());
    backend_example(&m);

    assert_eq!(
        ids(m.top_candidates("backend", 10)),
        ["alice", "bob", "xena", "zed", "erin"]
    );
    assert_eq!(ids(m.top_candidates("backend", 3)), ["alice", "bob", "xena"]);
    assert_eq!(ids(m.top_candidates("backend", 1)), ["alice"]);
}

#[test]
fn should_rank_jobs_by_score_then_job_id_and_skip_ineligible_ones() {
    let m = sut!(create());
    cand(&m, "c1", 2, "Pune", 40, true, &["java", "docker"]);
    m.post_job("A", "a", &["java"], &[], 0, "Pune", 100).unwrap();
    m.post_job("B", "b", &["java"], &["docker"], 2, "Delhi", 80)
        .unwrap();
    m.post_job("B2", "b2", &["java"], &["docker"], 2, "Delhi", 80)
        .unwrap();
    m.post_job("D", "d", &["java"], &[], 2, "Delhi", 80).unwrap();
    m.post_job("C", "c", &["java", "go"], &[], 0, "Pune", 100)
        .unwrap();

    assert_eq!(m.match_score("A", "c1"), Ok(74));
    assert_eq!(m.match_score("B", "c1"), Ok(70));
    assert_eq!(m.match_score("D", "c1"), Ok(60));
    assert_eq!(m.match_score("C", "c1"), Ok(0));
    assert_eq!(ids(m.top_jobs("c1", 10)), ["A", "B", "B2", "D"]);
    assert_eq!(ids(m.top_jobs("c1", 2)), ["A", "B"]);

    m.close_job("B").unwrap();

    assert_eq!(ids(m.top_jobs("c1", 10)), ["A", "B2", "D"]);
    assert_eq!(m.match_score("B", "c1"), Ok(0));
}

#[test]
fn should_follow_the_application_lifecycle_of_the_worked_example() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "a", 2, "Pune", 50, false, &["java"]);
    cand(&m, "b", 5, "Pune", 50, false, &["java"]);
    cand(&m, "c", 5, "Pune", 50, false, &["go"]);
    assert_eq!(m.match_score("J", "a"), Ok(70));
    assert_eq!(m.match_score("J", "b"), Ok(76));

    m.apply("a", "J").unwrap();
    m.apply("b", "J").unwrap();
    assert_eq!(ids(m.applications("J")), ["a", "b"]);
    assert_eq!(m.apply("a", "J"), Err(MatchError::AlreadyApplied));
    assert_eq!(m.apply("c", "J"), Err(MatchError::NotEligible));

    m.withdraw("a", "J").unwrap();
    m.apply("a", "J").unwrap();
    assert_eq!(ids(m.applications("J")), ["b", "a"]);

    m.update_candidate("a", &["go"], 2, "Pune", 50, false)
        .unwrap();
    assert_eq!(m.match_score("J", "a"), Ok(0));
    assert_eq!(ids(m.top_candidates("J", 5)), ["b"]);
    assert_eq!(ids(m.applications("J")), ["b", "a"]);

    m.close_job("J").unwrap();
    assert!(ids(m.top_candidates("J", 5)).is_empty());
    assert_eq!(m.match_score("J", "b"), Ok(0));
    assert_eq!(ids(m.applications("J")), ["b", "a"]);
    assert_eq!(m.apply("b", "J"), Err(MatchError::JobClosed));
    assert!(ids(m.top_jobs("b", 5)).is_empty());
}

#[test]
fn should_require_every_required_skill() {
    let m = sut!(create());
    m.post_job("J", "t", &["java", "sql"], &[], 0, "Pune", 100)
        .unwrap();
    cand(&m, "one", 1, "Pune", 10, false, &["java"]);
    cand(&m, "both", 1, "Pune", 10, false, &["java", "sql", "extra"]);

    assert_eq!(m.match_score("J", "one"), Ok(0));
    assert!(m.match_score("J", "both").unwrap() > 0);
}

#[test]
fn should_treat_years_and_salary_boundaries_as_eligible() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "edge", 2, "Pune", 100, false, &["java"]);
    cand(&m, "less_years", 1, "Pune", 10, false, &["java"]);
    cand(&m, "pricey", 5, "Pune", 101, false, &["java"]);

    assert_eq!(m.match_score("J", "edge"), Ok(60));
    assert_eq!(m.match_score("J", "less_years"), Ok(0));
    assert_eq!(m.match_score("J", "pricey"), Ok(0));
}

#[test]
fn should_match_location_case_insensitively_and_let_remote_candidates_in() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "local", 2, "  PUNE ", 90, false, &["java"]);
    cand(&m, "elsewhere", 2, "Delhi", 90, false, &["java"]);
    cand(&m, "remote", 2, "Delhi", 90, true, &["java"]);
    cand(&m, "remote_local", 2, "Pune", 90, true, &["java"]);

    assert_eq!(m.match_score("J", "local"), Ok(60));
    assert_eq!(m.match_score("J", "elsewhere"), Ok(0));
    assert_eq!(m.match_score("J", "remote"), Ok(50));
    assert_eq!(m.match_score("J", "remote_local"), Ok(60));
}

#[test]
fn should_compare_skills_case_insensitively_after_trimming() {
    let m = sut!(create());
    m.post_job("J", "t", &[" JAVA", "Spring Boot"], &["  KaFkA "], 0, "Pune", 100)
        .unwrap();
    cand(
        &m,
        "c",
        0,
        "Pune",
        90,
        false,
        &["java ", "spring boot", " kafka", "KAFKA"],
    );

    assert_eq!(m.match_score("J", "c"), Ok(70));
}

#[test]
fn should_cap_the_nice_to_have_term_at_five_skills() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &["a", "b", "c", "d", "e", "f", "g"], 0, "Pune", 100)
        .unwrap();
    cand(&m, "four", 0, "Pune", 90, false, &["a", "b", "c", "d"]);
    cand(&m, "five", 0, "Pune", 90, false, &["a", "b", "c", "d", "e"]);
    cand(&m, "seven", 0, "Pune", 90, false, &["a", "b", "c", "d", "e", "f", "g"]);

    assert_eq!(m.match_score("J", "four"), Ok(100));
    assert_eq!(m.match_score("J", "five"), Ok(110));
    assert_eq!(m.match_score("J", "seven"), Ok(110));
}

#[test]
fn should_cap_the_experience_term_at_twenty() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 1, "Pune", 100).unwrap();
    for (id, y) in [("y1", 1), ("y4", 4), ("y11", 11), ("y12", 12), ("y40", 40)] {
        cand(&m, id, y, "Pune", 90, false, &[]);
    }

    assert_eq!(m.match_score("J", "y1"), Ok(60));
    assert_eq!(m.match_score("J", "y4"), Ok(66));
    assert_eq!(m.match_score("J", "y11"), Ok(80));
    assert_eq!(m.match_score("J", "y12"), Ok(80));
    assert_eq!(m.match_score("J", "y40"), Ok(80));
}

#[test]
fn should_give_the_salary_bonus_only_when_twice_the_expectation_fits() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 0, "Pune", 100).unwrap();
    cand(&m, "half", 0, "Pune", 50, false, &[]);
    cand(&m, "just_over", 0, "Pune", 51, false, &[]);
    cand(&m, "free", 0, "Pune", 0, false, &[]);
    cand(&m, "all", 0, "Pune", 100, false, &[]);

    assert_eq!(m.match_score("J", "half"), Ok(70));
    assert_eq!(m.match_score("J", "just_over"), Ok(60));
    assert_eq!(m.match_score("J", "free"), Ok(70));
    assert_eq!(m.match_score("J", "all"), Ok(60));
}

#[test]
fn should_count_a_skill_listed_in_both_sets_as_required_only() {
    let m = sut!(create());
    m.post_job("J", "t", &["java"], &["JAVA", "go"], 0, "Pune", 100)
        .unwrap();
    cand(&m, "java_only", 0, "Pune", 90, false, &["java"]);
    cand(&m, "java_and_go", 0, "Pune", 90, false, &["java", "go"]);

    assert_eq!(m.match_score("J", "java_only"), Ok(60));
    assert_eq!(m.match_score("J", "java_and_go"), Ok(70));
}

#[test]
fn should_open_a_job_without_required_skills_to_candidates_without_skills() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 0, "Pune", 100).unwrap();
    cand(&m, "bare", 0, "Pune", 0, false, &[]);

    assert_eq!(m.match_score("J", "bare"), Ok(70));
    assert_eq!(ids(m.top_candidates("J", 5)), ["bare"]);
}

#[test]
fn should_return_fewer_than_k_when_few_are_eligible_and_reject_bad_k() {
    let m = sut!(create());
    backend_example(&m);

    assert_eq!(ids(m.top_candidates("backend", 100)).len(), 5);
    assert_eq!(m.top_candidates("backend", 0), Err(MatchError::InvalidArgument));
    assert_eq!(m.top_jobs("alice", 0), Err(MatchError::InvalidArgument));
    m.post_job("empty", "t", &["cobol"], &[], 0, "Pune", 1)
        .unwrap();
    assert!(ids(m.top_candidates("empty", 3)).is_empty());
}

#[test]
fn should_reject_duplicate_ids_but_keep_separate_id_spaces() {
    let m = sut!(create());
    simple_job(&m, "x");
    cand(&m, "x", 3, "Pune", 10, false, &["java"]);

    assert_eq!(
        m.post_job("x", "t", &[], &[], 0, "Pune", 1),
        Err(MatchError::AlreadyExists)
    );
    assert_eq!(
        m.register_candidate("x", &[], 1, "Pune", 1, false),
        Err(MatchError::AlreadyExists)
    );
    assert!(m.match_score("x", "x").unwrap() > 0);
    cand(&m, "X", 3, "Pune", 10, false, &["java"]);
    assert_eq!(ids(m.top_candidates("x", 5)), ["X", "x"]);
}

#[test]
fn should_reject_unknown_ids_everywhere() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "c", 2, "Pune", 10, false, &["java"]);

    assert_eq!(m.match_score("nope", "c"), Err(MatchError::UnknownJob));
    assert_eq!(m.match_score("J", "nope"), Err(MatchError::UnknownCandidate));
    assert_eq!(m.match_score("nope", "nope"), Err(MatchError::UnknownJob));
    assert_eq!(m.top_candidates("nope", 1), Err(MatchError::UnknownJob));
    assert_eq!(m.top_jobs("nope", 1), Err(MatchError::UnknownCandidate));
    assert_eq!(m.apply("nope", "J"), Err(MatchError::UnknownCandidate));
    assert_eq!(m.apply("c", "nope"), Err(MatchError::UnknownJob));
    assert_eq!(m.withdraw("c", "nope"), Err(MatchError::UnknownJob));
    assert_eq!(m.withdraw("nope", "J"), Err(MatchError::UnknownCandidate));
    assert_eq!(m.applications("nope"), Err(MatchError::UnknownJob));
    assert_eq!(m.close_job("nope"), Err(MatchError::UnknownJob));
    assert_eq!(
        m.update_candidate("nope", &[], 1, "Pune", 1, false),
        Err(MatchError::UnknownCandidate)
    );
}

#[test]
fn should_reject_invalid_input_without_creating_anything() {
    let m = sut!(create());
    let bad = Err(MatchError::InvalidArgument);

    assert_eq!(m.post_job(" ", "t", &[], &[], 0, "Pune", 1), bad);
    assert_eq!(m.post_job("J", " ", &[], &[], 0, "Pune", 1), bad);
    assert_eq!(m.post_job("J", "t", &["java", " "], &[], 0, "Pune", 1), bad);
    assert_eq!(m.post_job("J", "t", &[], &[""], 0, "Pune", 1), bad);
    assert_eq!(m.post_job("J", "t", &[], &[], 0, "", 1), bad);
    assert_eq!(m.register_candidate("", &[], 1, "Pune", 1, false), bad);
    assert_eq!(m.register_candidate("c", &["a", "  "], 1, "Pune", 1, false), bad);
    assert_eq!(m.register_candidate("c", &[], 1, "   ", 1, false), bad);

    simple_job(&m, "J");
    cand(&m, "c", 2, "Pune", 10, false, &["java"]);
    assert!(m.match_score("J", "c").unwrap() > 0);
}

#[test]
fn should_accept_zero_years_and_zero_salary() {
    let m = sut!(create());
    m.post_job("J", "t", &[], &[], 0, "Pune", 0).unwrap();
    cand(&m, "c", 0, "Pune", 0, false, &[]);

    assert_eq!(m.match_score("J", "c"), Ok(70));
}

#[test]
fn should_apply_in_order_and_reject_repeats_and_ineligible_candidates() {
    let m = sut!(create());
    simple_job(&m, "J");
    for c in ["c3", "c1", "c2"] {
        cand(&m, c, 3, "Pune", 10, false, &["java"]);
    }
    cand(&m, "nope", 3, "Pune", 10, false, &["go"]);

    m.apply("c3", "J").unwrap();
    m.apply("c1", "J").unwrap();
    m.apply("c2", "J").unwrap();

    assert_eq!(ids(m.applications("J")), ["c3", "c1", "c2"]);
    assert_eq!(m.apply("c1", "J"), Err(MatchError::AlreadyApplied));
    assert_eq!(m.apply("nope", "J"), Err(MatchError::NotEligible));
    assert_eq!(ids(m.applications("J")), ["c3", "c1", "c2"]);
}

#[test]
fn should_withdraw_and_put_a_reapplication_at_the_end() {
    let m = sut!(create());
    simple_job(&m, "J");
    for c in ["a", "b", "c"] {
        cand(&m, c, 3, "Pune", 10, false, &["java"]);
        m.apply(c, "J").unwrap();
    }

    m.withdraw("a", "J").unwrap();
    assert_eq!(ids(m.applications("J")), ["b", "c"]);
    assert_eq!(m.withdraw("a", "J"), Err(MatchError::NotApplied));
    m.apply("a", "J").unwrap();

    assert_eq!(ids(m.applications("J")), ["b", "c", "a"]);
}

#[test]
fn should_exclude_a_closed_job_from_matching_but_keep_its_applications() {
    let m = sut!(create());
    simple_job(&m, "J");
    simple_job(&m, "K");
    cand(&m, "c", 3, "Pune", 10, false, &["java"]);
    cand(&m, "d", 3, "Pune", 10, false, &["java"]);
    m.apply("c", "J").unwrap();

    m.close_job("J").unwrap();

    assert!(ids(m.top_candidates("J", 5)).is_empty());
    assert_eq!(ids(m.top_jobs("c", 5)), ["K"]);
    assert_eq!(m.match_score("J", "c"), Ok(0));
    assert_eq!(ids(m.applications("J")), ["c"]);
    assert_eq!(m.apply("d", "J"), Err(MatchError::JobClosed));
    assert_eq!(m.close_job("J"), Err(MatchError::AlreadyClosed));
    m.withdraw("c", "J").unwrap();
    assert!(ids(m.applications("J")).is_empty());
}

#[test]
fn should_reflect_candidate_updates_in_the_next_queries() {
    let m = sut!(create());
    m.post_job("J", "t", &["java"], &["go"], 3, "Pune", 100)
        .unwrap();
    cand(&m, "c", 1, "Delhi", 200, false, &["python"]);
    assert!(ids(m.top_candidates("J", 5)).is_empty());
    assert!(ids(m.top_jobs("c", 5)).is_empty());

    m.update_candidate("c", &["Java", "GO"], 5, "PUNE", 40, false)
        .unwrap();

    assert_eq!(ids(m.top_candidates("J", 5)), ["c"]);
    assert_eq!(ids(m.top_jobs("c", 5)), ["J"]);
    assert_eq!(m.match_score("J", "c"), Ok(84));

    m.update_candidate("c", &["java"], 5, "Delhi", 40, true)
        .unwrap();

    assert_eq!(m.match_score("J", "c"), Ok(64));
}

#[test]
fn should_leave_a_candidate_untouched_when_an_update_is_rejected() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "c", 3, "Pune", 10, false, &["java"]);

    assert_eq!(
        m.update_candidate("c", &["go", " "], 9, "Pune", 10, false),
        Err(MatchError::InvalidArgument)
    );
    assert_eq!(
        m.update_candidate("c", &["go"], 9, "", 10, false),
        Err(MatchError::InvalidArgument)
    );

    assert_eq!(m.match_score("J", "c"), Ok(72));
}

#[test]
fn should_let_an_ineligible_candidate_keep_an_earlier_application_but_not_apply_again() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "c", 3, "Pune", 10, false, &["java"]);
    m.apply("c", "J").unwrap();

    m.update_candidate("c", &["java"], 1, "Pune", 10, false)
        .unwrap();

    assert_eq!(ids(m.applications("J")), ["c"]);
    m.withdraw("c", "J").unwrap();
    assert_eq!(m.apply("c", "J"), Err(MatchError::NotEligible));
}

#[test]
fn should_see_new_jobs_and_candidates_in_the_next_ranking() {
    let m = sut!(create());
    cand(&m, "c", 3, "Pune", 10, false, &["java"]);
    assert!(ids(m.top_jobs("c", 5)).is_empty());

    simple_job(&m, "J");
    assert_eq!(ids(m.top_jobs("c", 5)), ["J"]);
    assert_eq!(ids(m.top_candidates("J", 5)), ["c"]);
    cand(&m, "d", 9, "Pune", 10, false, &["java"]);
    assert_eq!(ids(m.top_candidates("J", 5)), ["d", "c"]);
}

#[test]
fn should_let_exactly_one_of_many_identical_applies_succeed() {
    let m = sut!(create());
    simple_job(&m, "J");
    cand(&m, "c", 3, "Pune", 10, false, &["java"]);
    let ok = AtomicUsize::new(0);

    run_concurrently(16, |_| {
        if m.apply("c", "J").is_ok() {
            ok.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(ok.load(Ordering::SeqCst), 1);
    assert_eq!(ids(m.applications("J")), ["c"]);
}

#[test]
fn should_let_exactly_one_of_many_identical_registrations_succeed() {
    let m = sut!(create());
    let ok = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        if m.register_candidate("same", &["java"], t as u32, "Pune", 10, false)
            .is_ok()
        {
            ok.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(ok.load(Ordering::SeqCst), 1);
}

#[test]
fn should_keep_every_application_when_many_candidates_apply_at_once() {
    let m = sut!(create());
    simple_job(&m, "J");
    for i in 0..400 {
        cand(&m, &format!("c{i}"), 3, "Pune", 10, false, &["java"]);
    }

    run_concurrently(8, |t| {
        for i in (t..400).step_by(8) {
            m.apply(&format!("c{i}"), "J").unwrap();
        }
    });

    let apps = ids(m.applications("J"));
    assert_eq!(apps.len(), 400);
    assert_eq!(apps.iter().collect::<HashSet<_>>().len(), 400);
}

#[test]
fn should_rank_five_thousand_candidates_against_five_hundred_jobs_quickly() {
    let m = sut!(create());
    let skills = ["java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s", "aws", "react"];
    let cities = ["Pune", "Delhi", "Mumbai", "Goa"];
    for j in 0..500usize {
        m.post_job(
            &format!("job{j}"),
            &format!("t{j}"),
            &[skills[j % 10], skills[(j * 3 + 1) % 10]],
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
            &[
                skills[c % 10],
                skills[(c / 10) % 10],
                skills[(c / 100) % 10],
                skills[(c * 7) % 10],
            ],
            (c % 12) as u32,
            cities[(c / 3) % 4],
            10 + (c % 60) as u64,
            c % 5 == 0,
        )
        .unwrap();
    }

    let start = Instant::now();
    let mut non_empty = 0;
    for i in 0..1_000usize {
        let top = ids(m.top_candidates(&format!("job{}", i % 500), 10));
        assert!(top.len() <= 10);
        if !top.is_empty() {
            non_empty += 1;
        }
    }
    let ms = start.elapsed().as_millis();

    assert!(non_empty > 900, "non_empty = {non_empty}");
    assert!(ms < 10_000, "1,000 top_candidates calls took {ms} ms");
}

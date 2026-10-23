#![allow(unused_imports, clippy::needless_range_loop)]
use p115_autocomplete_typeahead::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn typeahead(r: Ranking) -> Option<(Arc<dyn Typeahead>, FakeClock)> {
    let clock = FakeClock::new(5_000);
    let t = create(clock.as_clock(), r)?;
    Some((t, clock))
}

fn list(v: &[&str]) -> Result<Vec<String>, TypeaheadError> {
    Ok(v.iter().map(|s| s.to_string()).collect())
}

#[test]
fn should_not_keep_a_stale_top_k_when_a_low_ranked_term_overtakes_the_leaders() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    for i in 0..30u64 {
        let term = format!("ab{}{}", (b'a' + (i % 26) as u8) as char, i);
        t.record_count(&term, 100 - i).unwrap();
    }
    t.record("abzzz-late").unwrap();
    assert!(!t
        .suggest("ab", 3)
        .unwrap()
        .contains(&"abzzz-late".to_string()));

    t.record_count("abzzz-late", 500).unwrap();
    assert_eq!(t.suggest("ab", 3).unwrap()[0], "abzzz-late");
    assert_eq!(t.suggest("abz", 3).unwrap()[0], "abzzz-late");
    assert_eq!(t.suggest("", 1), list(&["abzzz-late"]));
    assert_eq!(t.suggest("abzzz-l", 5), list(&["abzzz-late"]));
}

#[test]
fn should_break_a_thousand_way_tie_alphabetically_for_any_k() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    for i in 0..1_000usize {
        t.record_count(&format!("tie{:04}", (i * 7919) % 1_000), 3)
            .unwrap();
    }

    assert_eq!(
        t.suggest("tie", 4),
        list(&["tie0000", "tie0001", "tie0002", "tie0003"])
    );
    let a = t.suggest("tie09", 100).unwrap();
    assert_eq!(a.len(), 100);
    assert!(a.windows(2).all(|w| w[0] < w[1]));
    let b = t.suggest("", 1_000).unwrap();
    assert_eq!(b.len(), 1_000);
    assert!(b.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn should_re_rank_by_recency_on_bulk_and_personal_records_too() {
    let (t, clock) = sut!(typeahead(Ranking::Recency));
    t.record_count("moon", 1_000).unwrap();
    clock.advance(1);
    t.record("moss").unwrap();
    clock.advance(1);
    t.record_for("ann", "mole").unwrap();
    clock.advance(1);
    t.record_count("moon", 2).unwrap();

    assert_eq!(t.suggest("mo", 10), list(&["moon", "mole", "moss"]));
    assert_eq!(
        t.suggest_for("ann", "mo", 10),
        list(&["mole", "moon", "moss"])
    );
    t.set_ranking(Ranking::Frequency);
    assert_eq!(t.suggest("mo", 10), list(&["moon", "mole", "moss"]));
}

#[test]
fn should_leave_no_trace_of_rejected_calls() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    t.record_count("kept", 2).unwrap();
    assert!(t.record_count("ghost", 0).is_err());
    assert!(t.record_for(" ", "ghost").is_err());
    assert!(t.record_for("ann", "  ").is_err());

    assert_eq!(t.count("ghost"), Ok(0));
    assert_eq!(t.suggest("", 10), list(&["kept"]));
    assert_eq!(t.suggest_for("ann", "", 10), list(&["kept"]));
}

#[test]
fn should_agree_with_a_brute_force_model_on_random_operations() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    let mut model: HashMap<String, u64> = HashMap::new();
    let mut state: u64 = 7;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let alphabet = ['a', 'b', 'c'];
    for step in 0..6_000 {
        let len = 1 + (next() % 5) as usize;
        let term: String = (0..len).map(|_| alphabet[(next() % 3) as usize]).collect();
        let c = 1 + next() % 4;
        if next() % 2 == 0 {
            t.record_count(&term.to_uppercase(), c).unwrap();
        } else {
            t.record_count(&term, c).unwrap();
        }
        *model.entry(term).or_insert(0) += c;

        if step % 10 == 0 {
            let plen = (next() % 4) as usize;
            let prefix: String = (0..plen).map(|_| alphabet[(next() % 3) as usize]).collect();
            let k = 1 + (next() % 8) as usize;
            let mut all: Vec<(&String, &u64)> = model
                .iter()
                .filter(|(s, _)| s.starts_with(&prefix))
                .collect();
            all.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            let expected: Vec<String> = all.into_iter().take(k).map(|(s, _)| s.clone()).collect();
            assert_eq!(
                t.suggest(&prefix, k).unwrap(),
                expected,
                "prefix '{prefix}' k={k} step {step}"
            );
        }
    }
}

#[test]
fn should_keep_a_hot_term_exact_while_readers_ask_for_its_prefix() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    for i in 0..200 {
        t.record_count(&format!("hot{i}"), 10).unwrap();
    }
    let violations = AtomicUsize::new(0);

    run_concurrently(12, |id| {
        for i in 0..20_000usize {
            if id < 8 {
                t.record(&format!("hot{}", i % 200)).unwrap();
                t.record("hot0").unwrap();
            } else {
                let r = t.suggest("hot", 5).unwrap();
                let mut d = r.clone();
                d.sort();
                d.dedup();
                if r.len() != 5 || d.len() != 5 {
                    violations.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    assert_eq!(violations.load(Ordering::SeqCst), 0);
    assert_eq!(t.count("hot0"), Ok(10 + 8 * 100 + 8 * 20_000));
    assert_eq!(t.count("hot199"), Ok(10 + 8 * 100));
    assert_eq!(t.suggest("hot", 1), list(&["hot0"]));
    // all the other 199 terms have exactly the same count: alphabetical
    assert_eq!(
        t.suggest("hot", 4),
        list(&["hot0", "hot1", "hot10", "hot100"])
    );
}

#[test]
fn should_keep_per_user_histories_exact_under_concurrency() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    t.record_count("zeta", 1_000_000).unwrap();
    t.record_count("zeal", 1_000_000).unwrap();

    run_concurrently(8, |id| {
        for _ in 0..2_000 {
            let user = format!("user{}", id % 2);
            t.record_for(&user, if id % 2 == 0 { "zeta" } else { "zeal" })
                .unwrap();
            t.suggest_for(&user, "ze", 2).unwrap();
        }
    });

    assert_eq!(t.count("zeta"), Ok(1_000_000 + 4 * 2_000));
    assert_eq!(t.suggest_for("user0", "ze", 2), list(&["zeta", "zeal"]));
    assert_eq!(t.suggest_for("user1", "ze", 2), list(&["zeal", "zeta"]));
    assert_eq!(t.suggest_for("user2", "ze", 2), list(&["zeal", "zeta"])); // equal globals: alphabetical
}

#[test]
fn should_handle_long_terms_and_a_non_ascii_alphabet() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    let long_term = "x".repeat(2_000);
    let longer = format!("{long_term}y");
    t.record_count(&long_term, 3).unwrap();
    t.record_count(&longer, 2).unwrap();
    t.record_count("\u{c9}COLE", 4).unwrap();
    t.record_count("\u{e9}cole normale", 6).unwrap();

    assert_eq!(
        t.suggest(&"x".repeat(1_500), 5),
        Ok(vec![long_term.clone(), longer.clone()])
    );
    assert_eq!(t.suggest(&format!("{long_term}yy"), 5), list(&[]));
    assert_eq!(
        t.suggest("\u{c9}c", 5),
        list(&["\u{e9}cole normale", "\u{e9}cole"])
    );
}

#[test]
fn should_serve_two_hundred_thousand_terms_and_twenty_thousand_queries_quickly() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    for i in 0..200_000u64 {
        t.record_count(&format!("q{}", to_base36(i * 31)), 1 + (i % 50))
            .unwrap();
    }

    let start = std::time::Instant::now();
    for i in 0..20_000u64 {
        let s = to_base36(i * 31);
        let n = std::cmp::min(s.len(), 1 + (i % 2) as usize);
        t.suggest(&format!("q{}", &s[..n]), 10).unwrap();
    }
    let ms = start.elapsed().as_millis();

    assert!(
        ms < 60_000,
        "20,000 suggest calls over 200,000 terms took {ms} ms"
    );
    assert_eq!(t.suggest("q", 10).unwrap().len(), 10);
}

#[test]
fn should_return_a_stable_order_when_counts_are_raised_to_the_same_value() {
    let (t, _) = sut!(typeahead(Ranking::Frequency));
    t.record_count("beta", 5).unwrap();
    t.record_count("alpha", 3).unwrap();
    t.record_count("alpha", 2).unwrap();

    assert_eq!(t.suggest("", 5), list(&["alpha", "beta"]));
    t.record("beta").unwrap();
    assert_eq!(t.suggest("", 5), list(&["beta", "alpha"]));
}

fn to_base36(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while n > 0 {
        out.push(digits[(n % 36) as usize]);
        n /= 36;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

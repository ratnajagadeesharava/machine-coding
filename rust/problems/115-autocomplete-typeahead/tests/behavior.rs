#![allow(unused_imports, clippy::needless_range_loop)]
use p115_autocomplete_typeahead::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const BAD: Result<(), TypeaheadError> = Err(TypeaheadError::InvalidArgument);

fn typeahead_with(r: Ranking) -> Option<(Arc<dyn Typeahead>, FakeClock)> {
    let clock = FakeClock::new(1_000_000);
    let t = create(clock.as_clock(), r)?;
    Some((t, clock))
}

fn typeahead() -> Option<(Arc<dyn Typeahead>, FakeClock)> {
    typeahead_with(Ranking::Frequency)
}

fn record_n(t: &Arc<dyn Typeahead>, term: &str, times: usize) {
    for _ in 0..times {
        t.record(term).unwrap();
    }
}

fn list(v: &[&str]) -> Result<Vec<String>, TypeaheadError> {
    Ok(v.iter().map(|s| s.to_string()).collect())
}

#[test]
fn should_rank_suggestions_by_frequency_descending() {
    let (t, _) = sut!(typeahead());
    record_n(&t, "apple", 3);
    record_n(&t, "application", 5);
    record_n(&t, "apply", 1);

    assert_eq!(
        t.suggest("app", 10),
        list(&["application", "apple", "apply"])
    );
}

#[test]
fn should_break_frequency_ties_alphabetically() {
    let (t, _) = sut!(typeahead());
    t.record("pear").unwrap();
    t.record("peach").unwrap();
    t.record("pecan").unwrap();
    t.record("pea").unwrap();

    assert_eq!(
        t.suggest("pe", 10),
        list(&["pea", "peach", "pear", "pecan"])
    );
}

#[test]
fn should_return_only_terms_that_start_with_the_prefix() {
    let (t, _) = sut!(typeahead());
    t.record_count("banana", 9).unwrap();
    t.record_count("band", 2).unwrap();
    t.record_count("abandon", 50).unwrap();
    t.record("ban").unwrap();

    assert_eq!(t.suggest("ban", 10), list(&["banana", "band", "ban"]));
    assert_eq!(t.suggest("and", 10), list(&[]));
}

#[test]
fn should_include_a_term_that_equals_the_prefix() {
    let (t, _) = sut!(typeahead());
    t.record_count("go", 5).unwrap();
    t.record_count("golang", 2).unwrap();

    assert_eq!(t.suggest("go", 5), list(&["go", "golang"]));
    assert_eq!(t.suggest("golang", 5), list(&["golang"]));
}

#[test]
fn should_return_at_most_k_suggestions() {
    let (t, _) = sut!(typeahead());
    for i in 0..20u64 {
        let term = format!("item{}", (b'a' + i as u8) as char);
        t.record_count(&term, 20 - i).unwrap();
    }

    assert_eq!(t.suggest("item", 3), list(&["itema", "itemb", "itemc"]));
    assert_eq!(t.suggest("item", 1), list(&["itema"]));
    assert_eq!(t.suggest("item", 1_000).unwrap().len(), 20);
}

#[test]
fn should_return_an_empty_list_when_nothing_matches() {
    let (t, _) = sut!(typeahead());
    t.record("short").unwrap();

    assert_eq!(t.suggest("zzz", 5), list(&[]));
    let (empty, _) = sut!(typeahead());
    assert_eq!(empty.suggest("anything", 5), list(&[]));
}

#[test]
fn should_return_an_empty_list_when_the_prefix_is_longer_than_every_term() {
    let (t, _) = sut!(typeahead());
    t.record_count("cat", 3).unwrap();
    t.record_count("cats", 2).unwrap();

    assert_eq!(t.suggest("catsup-and-more", 5), list(&[]));
    assert_eq!(t.suggest("cats ", 5), list(&["cats"])); // surrounding whitespace is trimmed
}

#[test]
fn should_return_the_top_k_overall_for_an_empty_prefix() {
    let (t, _) = sut!(typeahead());
    t.record_count("zebra", 10).unwrap();
    t.record_count("apple", 4).unwrap();
    t.record_count("mango", 7).unwrap();
    t.record_count("kiwi", 1).unwrap();

    assert_eq!(t.suggest("", 2), list(&["zebra", "mango"]));
    assert_eq!(
        t.suggest("   ", 10),
        list(&["zebra", "mango", "apple", "kiwi"])
    );
}

#[test]
fn should_ignore_case_and_surrounding_whitespace() {
    let (t, _) = sut!(typeahead());
    t.record("  Hello World ").unwrap();
    t.record("HELLO world").unwrap();
    t.record("hello").unwrap();

    assert_eq!(t.count("hello world"), Ok(2));
    assert_eq!(t.count("HELLO"), Ok(1));
    assert_eq!(t.suggest("HeL", 5), list(&["hello world", "hello"]));
    assert_eq!(t.suggest(" hello w", 5), list(&["hello world"]));
}

#[test]
fn should_add_a_bulk_count_and_start_at_zero() {
    let (t, _) = sut!(typeahead());

    assert_eq!(t.count("never"), Ok(0));
    t.record_count("bulk", 40).unwrap();
    t.record("bulk").unwrap();
    t.record_count("bulk", 9).unwrap();

    assert_eq!(t.count("bulk"), Ok(50));
}

#[test]
fn should_make_updates_visible_immediately_and_re_rank() {
    let (t, _) = sut!(typeahead());
    record_n(&t, "red", 5);
    record_n(&t, "reed", 3);
    assert_eq!(t.suggest("re", 5), list(&["red", "reed"]));

    t.record_count("reed", 3).unwrap();
    assert_eq!(t.suggest("re", 5), list(&["reed", "red"]));

    t.record_count("reed", 100).unwrap();
    t.record_count("redwood", 107).unwrap();
    assert_eq!(t.suggest("re", 1), list(&["redwood"]));
}

#[test]
fn should_keep_terms_with_spaces_as_one_term() {
    let (t, _) = sut!(typeahead());
    t.record_count("new york", 4).unwrap();
    t.record_count("new jersey", 6).unwrap();
    t.record_count("newark", 5).unwrap();

    assert_eq!(
        t.suggest("new", 5),
        list(&["new jersey", "newark", "new york"])
    );
    assert_eq!(t.suggest("new y", 5), list(&["new york"]));
}

#[test]
fn should_reject_blank_terms_and_non_positive_counts() {
    let (t, _) = sut!(typeahead());
    t.record("ok").unwrap();

    assert_eq!(t.record(""), BAD);
    assert_eq!(t.record("   "), BAD);
    assert_eq!(t.record_count("ok", 0), BAD);
    assert_eq!(t.record_count("fresh", 0), BAD);

    assert_eq!(t.count("ok"), Ok(1));
    assert_eq!(t.count("fresh"), Ok(0));
    assert_eq!(t.suggest("", 10), list(&["ok"]));
}

#[test]
fn should_reject_zero_k_blank_users_and_blank_terms_elsewhere() {
    let (t, _) = sut!(typeahead());
    t.record("ok").unwrap();
    let bad_list = Err(TypeaheadError::InvalidArgument);

    assert_eq!(t.suggest("o", 0), bad_list);
    assert_eq!(t.suggest_for("u1", "o", 0), bad_list);
    assert_eq!(t.suggest_for(" ", "o", 3), bad_list);
    assert_eq!(t.record_for(" ", "ok"), BAD);
    assert_eq!(t.record_for("u1", " "), BAD);
    assert_eq!(t.count(" "), Err(TypeaheadError::InvalidArgument));
}

#[test]
fn should_rank_by_recency_most_recent_first() {
    let (t, clock) = sut!(typeahead_with(Ranking::Recency));
    t.record_count("alpha", 100).unwrap();
    clock.advance(10);
    t.record("alps").unwrap();
    clock.advance(10);
    t.record_count("alpine", 5).unwrap();

    assert_eq!(t.suggest("al", 10), list(&["alpine", "alps", "alpha"]));

    clock.advance(10);
    t.record("alpha").unwrap(); // recorded again: back to the front even though its count was already the largest
    assert_eq!(t.suggest("al", 10), list(&["alpha", "alpine", "alps"]));
    assert_eq!(t.suggest("al", 2), list(&["alpha", "alpine"]));
}

#[test]
fn should_break_recency_ties_alphabetically() {
    let (t, clock) = sut!(typeahead_with(Ranking::Recency));
    t.record("tea").unwrap();
    t.record("ten").unwrap();
    t.record("tab").unwrap();
    clock.advance(1);
    t.record("tex").unwrap();

    assert_eq!(t.suggest("t", 10), list(&["tex", "tab", "tea", "ten"]));
}

#[test]
fn should_switch_the_ranking_rule_at_runtime_keeping_all_data() {
    let (t, clock) = sut!(typeahead_with(Ranking::Frequency));
    t.record_count("lamp", 9).unwrap();
    clock.advance(5);
    t.record_count("lake", 1).unwrap();
    clock.advance(5);
    t.record_count("lane", 4).unwrap();

    assert_eq!(t.suggest("la", 10), list(&["lamp", "lane", "lake"]));
    t.set_ranking(Ranking::Recency);
    assert_eq!(t.suggest("la", 10), list(&["lane", "lake", "lamp"]));
    t.set_ranking(Ranking::Frequency);
    assert_eq!(t.suggest("la", 10), list(&["lamp", "lane", "lake"]));
    assert_eq!(t.count("lamp"), Ok(9));
}

#[test]
fn should_put_the_users_own_terms_first() {
    let (t, _) = sut!(typeahead());
    t.record_count("java", 50).unwrap();
    t.record_count("javascript", 40).unwrap();
    t.record_count("jazz", 30).unwrap();
    t.record_for("ann", "jazz").unwrap();
    t.record_for("ann", "jazz").unwrap();
    t.record_for("ann", "javascript").unwrap();

    assert_eq!(t.suggest("ja", 10), list(&["java", "javascript", "jazz"]));
    assert_eq!(
        t.suggest_for("ann", "ja", 10),
        list(&["jazz", "javascript", "java"])
    );
    assert_eq!(t.suggest_for("ann", "ja", 2), list(&["jazz", "javascript"]));
}

#[test]
fn should_isolate_user_histories_and_fall_back_to_global_for_unknown_users() {
    let (t, _) = sut!(typeahead());
    t.record_count("sql", 10).unwrap();
    t.record_count("sqlite", 5).unwrap();
    t.record_for("ann", "sqlite").unwrap();
    t.record_for("bob", "sql").unwrap();

    assert_eq!(t.suggest_for("ann", "sql", 5), list(&["sqlite", "sql"]));
    assert_eq!(t.suggest_for("bob", "sql", 5), list(&["sql", "sqlite"]));
    assert_eq!(t.suggest_for("nobody", "sql", 5), t.suggest("sql", 5));
}

#[test]
fn should_count_personal_records_in_the_global_counts_too() {
    let (t, _) = sut!(typeahead());
    t.record_for("ann", "kotlin").unwrap();
    t.record_for("bob", "kotlin").unwrap();
    t.record("kotlin").unwrap();
    t.record_count("kotor", 2).unwrap();

    assert_eq!(t.count("kotlin"), Ok(3));
    assert_eq!(t.suggest("kot", 5), list(&["kotlin", "kotor"]));
}

#[test]
fn should_order_terms_with_equal_user_counts_by_the_active_ranking() {
    let (t, clock) = sut!(typeahead_with(Ranking::Frequency));
    t.record_count("rust", 5).unwrap();
    t.record_count("ruby", 9).unwrap();
    t.record_count("rune", 7).unwrap();
    t.record_for("ann", "rust").unwrap(); // rust now 6 globally, ann: rust=1

    assert_eq!(
        t.suggest_for("ann", "ru", 5),
        list(&["rust", "ruby", "rune"])
    );

    t.set_ranking(Ranking::Recency);
    clock.advance(3);
    t.record("rune").unwrap();
    assert_eq!(
        t.suggest_for("ann", "ru", 5),
        list(&["rust", "rune", "ruby"])
    );
}

#[test]
fn should_have_exact_counts_when_many_threads_record_concurrently() {
    let (t, _) = sut!(typeahead());

    run_concurrently(16, |_| {
        for i in 0..5_000 {
            t.record("shared").unwrap();
            t.record(&format!("term{}", i % 10)).unwrap();
            t.record_count("bulk", 2).unwrap();
        }
    });

    assert_eq!(t.count("shared"), Ok(80_000));
    assert_eq!(t.count("bulk"), Ok(160_000));
    for i in 0..10 {
        assert_eq!(t.count(&format!("term{i}")), Ok(8_000));
    }
    assert_eq!(t.suggest("", 3), list(&["bulk", "shared", "term0"]));
}

#[test]
fn should_serve_readers_while_writers_update() {
    let (t, _) = sut!(typeahead());
    for i in 0..100 {
        t.record(&format!("seed{i}")).unwrap();
    }
    let bad = AtomicUsize::new(0);

    run_concurrently(8, |id| {
        for i in 0..3_000 {
            if id < 4 {
                t.record_for(&format!("u{id}"), &format!("seed{}", i % 100))
                    .unwrap();
                t.record(&format!("w{id}-{}", i % 50)).unwrap();
            } else {
                let r = t.suggest("seed", 5).unwrap();
                let mut d = r.clone();
                d.sort();
                d.dedup();
                if r.len() != 5 || d.len() != 5 || !r.iter().all(|s| s.starts_with("seed")) {
                    bad.fetch_add(1, Ordering::SeqCst);
                }
                t.suggest_for(&format!("u{}", id - 4), "w", 3).unwrap();
            }
        }
    });

    assert_eq!(bad.load(Ordering::SeqCst), 0);
    let total: u64 = (0..100)
        .map(|i| t.count(&format!("seed{i}")).unwrap())
        .sum();
    assert_eq!(total, 100 + 4 * 3_000);
}

#[test]
fn should_handle_one_hundred_thousand_terms_and_ten_thousand_suggest_calls() {
    let (t, _) = sut!(typeahead());
    let mut state: u64 = 42;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut terms: Vec<String> = Vec::with_capacity(100_000);
    for i in 0..100_000usize {
        let len = 4 + (next() % 8) as usize;
        let mut s = String::new();
        for _ in 0..len {
            s.push((b'a' + (next() % 26) as u8) as char);
        }
        s.push_str(&i.to_string());
        terms.push(s);
    }
    for (i, term) in terms.iter().enumerate() {
        t.record_count(term, 1 + (i as u64 % 97)).unwrap();
    }

    let start = std::time::Instant::now();
    let mut non_empty = 0;
    for q in 0..10_000usize {
        let term = &terms[(q * 7) % terms.len()];
        let r = t.suggest(&term[..1 + (q % 3)], 10).unwrap();
        if !r.is_empty() {
            non_empty += 1;
        }
        assert!(r.len() <= 10);
    }
    let elapsed_ms = start.elapsed().as_millis();

    assert_eq!(non_empty, 10_000);
    assert!(
        elapsed_ms < 30_000,
        "10,000 suggest calls over 100,000 terms took {elapsed_ms} ms"
    );
    assert_eq!(t.suggest("", 1).unwrap().len(), 1);
}

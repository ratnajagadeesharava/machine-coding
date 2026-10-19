#![allow(unused_imports, clippy::needless_range_loop)]
use p82_url_shortener::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn url(i: usize) -> String {
    format!("https://example.com/page/{i}")
}

#[test]
fn should_resolve_a_code_back_to_its_url() {
    let s = sut!(create());

    let code = s.shorten("https://example.com/a?b=c#d").unwrap();

    assert!(!code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric()));
    assert_eq!(s.resolve(&code), Some("https://example.com/a?b=c#d".to_string()));
}

#[test]
fn should_return_the_same_code_for_the_same_url_and_different_codes_for_different_urls() {
    let s = sut!(create());

    let a1 = s.shorten(&url(1)).unwrap();
    let a2 = s.shorten(&url(1)).unwrap();
    let b = s.shorten(&url(2)).unwrap();

    assert_eq!(a1, a2);
    assert_ne!(a1, b);
}

#[test]
fn should_reject_invalid_urls() {
    let s = sut!(create());

    assert_eq!(s.shorten(""), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("   "), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("example.com/a"), Err(ShortenError::InvalidUrl));
    assert_eq!(s.shorten("ftp://example.com/a"), Err(ShortenError::InvalidUrl));
}

#[test]
fn should_keep_codes_short() {
    let s = sut!(create());

    for i in 0..10_000 {
        assert!(s.shorten(&url(i)).unwrap().len() <= 6);
    }
}

#[test]
fn should_return_none_for_unknown_codes() {
    let s = sut!(create());

    assert_eq!(s.resolve("nope"), None);
    assert_eq!(s.clicks("nope"), None);
}

#[test]
fn should_support_custom_aliases() {
    let s = sut!(create());

    assert_eq!(s.shorten_with_alias(&url(1), "my-link_1"), Ok("my-link_1".to_string()));
    assert_eq!(s.shorten_with_alias(&url(1), "my-link_1"), Ok("my-link_1".to_string()));
    assert_eq!(s.resolve("my-link_1"), Some(url(1)));
}

#[test]
fn should_reject_bad_or_taken_aliases() {
    let s = sut!(create());
    s.shorten_with_alias(&url(1), "taken").unwrap();
    let generated = s.shorten(&url(2)).unwrap();

    assert_eq!(s.shorten_with_alias(&url(3), "taken"), Err(ShortenError::AliasTaken));
    // a short generated code is rejected either as a bad alias or as a taken one
    assert!(s.shorten_with_alias(&url(3), &generated).is_err());
    assert_eq!(s.shorten_with_alias(&url(3), "ab"), Err(ShortenError::InvalidAlias));
    assert_eq!(s.shorten_with_alias(&url(3), "has space"), Err(ShortenError::InvalidAlias));
    assert_eq!(s.shorten_with_alias(&url(3), &"x".repeat(21)), Err(ShortenError::InvalidAlias));
    assert_eq!(s.resolve("taken"), Some(url(1)));
}

#[test]
fn should_count_only_successful_resolves() {
    let s = sut!(create());
    let code = s.shorten(&url(1)).unwrap();

    for _ in 0..5 {
        s.resolve(&code);
    }
    s.resolve("unknown");

    assert_eq!(s.clicks(&code), Some(5));
}

#[test]
fn should_delete_codes_and_never_reissue_them() {
    let s = sut!(create());
    let code = s.shorten(&url(1)).unwrap();

    assert!(s.delete(&code));
    assert!(!s.delete(&code));

    assert_eq!(s.resolve(&code), None);
    assert_eq!(s.clicks(&code), None);
    let later: HashSet<String> = (10..500).map(|i| s.shorten(&url(i)).unwrap()).collect();
    assert!(!later.contains(&code));
    let again = s.shorten(&url(1)).unwrap();
    assert_eq!(s.resolve(&again), Some(url(1)));
}

#[test]
fn should_give_every_url_its_own_code_under_concurrent_shortening() {
    let s = sut!(create());

    let codes = collect_concurrently(16, |t| {
        (0..2_000).map(|i| s.shorten(&url(t * 2_000 + i)).unwrap()).collect::<Vec<_>>()
    });

    let all: HashSet<&String> = codes.iter().flatten().collect();
    assert_eq!(all.len(), 32_000);
    for t in (0..16).step_by(5) {
        for i in (0..2_000).step_by(97) {
            assert_eq!(s.resolve(&codes[t][i]), Some(url(t * 2_000 + i)));
        }
    }
}

#[test]
fn should_give_one_code_when_many_threads_shorten_the_same_url() {
    let s = sut!(create());

    let codes = collect_concurrently(16, |_| s.shorten("https://example.com/hot").unwrap());

    assert_eq!(codes.iter().collect::<HashSet<_>>().len(), 1);
}

#[test]
fn should_let_exactly_one_thread_win_an_alias() {
    let s = sut!(create());

    let wins = collect_concurrently(16, |t| s.shorten_with_alias(&url(t), "contested").is_ok());

    assert_eq!(wins.iter().filter(|w| **w).count(), 1);
}

#[test]
fn should_count_every_click_under_concurrency() {
    let s = sut!(create());
    let code = s.shorten(&url(1)).unwrap();

    run_concurrently(8, |_| {
        for _ in 0..1_000 {
            s.resolve(&code);
        }
    });

    assert_eq!(s.clicks(&code), Some(8_000));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let s = sut!(create());
        let codes = Mutex::new(HashSet::new());

        run_concurrently(8, |_| {
            for i in 0..50 {
                let code = s.shorten(&url(i)).unwrap();
                codes.lock().unwrap().insert(code);
            }
        });

        assert_eq!(codes.lock().unwrap().len(), 50, "run {run}");
    }
}

#![allow(unused_imports, clippy::needless_range_loop)]
use p110_data_source_layer::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const SOURCES: [&str; 3] = ["DATABASE", "API", "FILE"];

fn total(d: &Arc<dyn DataAccess>) -> usize {
    SOURCES
        .iter()
        .map(|s| {
            d.switch_source(s).unwrap();
            d.size()
        })
        .sum()
}

#[test]
fn should_keep_the_old_value_when_an_api_overwrite_is_refused() {
    let d = sut!(create());
    d.switch_source("API").unwrap();
    d.put("Key", "old").unwrap();

    assert_eq!(d.put("KEY", &"y".repeat(33)), Err(DataError::ValueTooLong));

    assert_eq!(d.get("key"), Ok(Some("old".to_string())));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_treat_a_whitespace_only_key_as_blank_even_on_file() {
    let d = sut!(create());
    d.switch_source("FILE").unwrap();

    assert_eq!(d.put("   ", "v"), Err(DataError::InvalidKey));
    assert_eq!(d.get(" \t "), Err(DataError::InvalidKey));
    assert_eq!(d.remove("\n"), Err(DataError::InvalidKey));
    assert_eq!(d.size(), 0);
}

#[test]
fn should_never_apply_one_source_normalization_to_another() {
    let d = sut!(create());
    d.put(" Mixed ", "db").unwrap();
    d.switch_source("FILE").unwrap();
    d.put(" Mixed ", "file").unwrap();
    d.switch_source("API").unwrap();
    d.put(" Mixed ", "api").unwrap();

    d.switch_source("DATABASE").unwrap();
    assert_eq!(d.get(" Mixed "), Ok(Some("db".to_string())));
    assert_eq!(d.get("Mixed"), Ok(None));
    assert_eq!(d.get(" mixed "), Ok(None));
    d.switch_source("FILE").unwrap();
    assert_eq!(d.get("Mixed"), Ok(Some("file".to_string())));
    assert_eq!(d.get("mixed"), Ok(None));
    d.switch_source("API").unwrap();
    assert_eq!(d.get(" mixed "), Ok(Some("api".to_string())));
    assert_eq!(d.get("Mixed"), Ok(None));
}

#[test]
fn should_keep_long_values_stored_elsewhere_after_a_trip_through_api() {
    let d = sut!(create());
    let long = "z".repeat(40);
    d.put("long", &long).unwrap();
    d.switch_source("API").unwrap();
    assert_eq!(d.get("long"), Ok(None));
    assert_eq!(d.put("long", &long), Err(DataError::ValueTooLong));
    d.switch_source("DATABASE").unwrap();

    assert_eq!(d.get("long"), Ok(Some(long)));
    assert_eq!(d.size(), 1);
}

#[test]
fn should_send_writes_to_the_previous_source_after_a_failed_switch() {
    let d = sut!(create());
    d.switch_source("API").unwrap();

    assert_eq!(d.switch_source("api"), Err(DataError::UnknownSource));
    assert_eq!(d.switch_source(""), Err(DataError::UnknownSource));
    d.put("k", "v").unwrap();

    assert_eq!(d.current_source(), "API");
    assert_eq!(total(&d), 1);
    d.switch_source("API").unwrap();
    assert_eq!(d.get("k"), Ok(Some("v".to_string())));
}

#[test]
fn should_lose_no_record_across_thousands_of_switches() {
    let d = sut!(create());
    for i in 0..300 {
        d.switch_source(SOURCES[i % 3]).unwrap();
        d.put(&format!("k{i}"), &format!("v{i}")).unwrap();
    }
    for i in 0..6_000 {
        d.switch_source(SOURCES[(i * 7) % 3]).unwrap();
    }

    assert_eq!(total(&d), 300);
    d.switch_source("FILE").unwrap();
    assert_eq!(d.get("k2"), Ok(Some("v2".to_string())));
    d.switch_source("API").unwrap();
    assert_eq!(d.get("k1"), Ok(Some("v1".to_string())));
    d.switch_source("DATABASE").unwrap();
    assert_eq!(d.get("k0"), Ok(Some("v0".to_string())));
}

#[test]
fn should_land_every_concurrent_write_in_exactly_one_source() {
    let d = sut!(create());
    let written = AtomicUsize::new(0);

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..6_000 {
                d.switch_source(SOURCES[i % 3]).unwrap();
            }
            return;
        }
        for i in 0..2_000 {
            d.put(&format!("w{t}-{i}"), "v").unwrap();
            written.fetch_add(1, Ordering::Relaxed);
        }
    });

    assert_eq!(written.load(Ordering::Relaxed), 8 * 2_000);
    assert_eq!(total(&d), 8 * 2_000);
}

#[test]
fn should_not_lose_or_duplicate_removes_while_switching() {
    let d = sut!(create());
    for s in SOURCES {
        d.switch_source(s).unwrap();
        for i in 0..1_000 {
            d.put(&format!("k{i}"), "v").unwrap();
        }
    }
    let removed = AtomicUsize::new(0);

    run_concurrently(5, |t| {
        if t == 0 {
            for i in 0..4_000 {
                d.switch_source(SOURCES[i % 3]).unwrap();
            }
            return;
        }
        for _ in 0..3 {
            for i in 0..1_000 {
                if d.remove(&format!("k{i}")).unwrap() {
                    removed.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    assert_eq!(removed.load(Ordering::Relaxed) + total(&d), 3_000);
}

#[test]
fn should_serve_heavy_reads_while_the_source_is_switched_thousands_of_times() {
    let d = sut!(create());
    for s in SOURCES {
        d.switch_source(s).unwrap();
        for i in 0..100 {
            d.put(&format!("k{i}"), &format!("{s}{i}")).unwrap();
        }
    }
    d.switch_source("DATABASE").unwrap();
    let bad = AtomicUsize::new(0);

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..10_000 {
                d.switch_source(SOURCES[i % 3]).unwrap();
            }
            return;
        }
        for i in 0..100_000usize {
            let k = i % 100;
            let ok = match d.get(&format!("k{k}")).unwrap() {
                Some(v) => SOURCES.iter().any(|s| v == format!("{s}{k}")),
                None => false,
            };
            if !ok {
                bad.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    assert_eq!(bad.load(Ordering::Relaxed), 0);
}

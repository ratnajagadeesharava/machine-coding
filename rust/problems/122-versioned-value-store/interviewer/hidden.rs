#![allow(unused_imports, clippy::needless_range_loop)]
use p122_versioned_value_store::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

fn some(v: &str) -> Result<Option<String>, StoreError> {
    Ok(Some(v.to_string()))
}

const NOTHING: Result<Option<String>, StoreError> = Ok(None);

#[test]
fn should_stay_correct_when_hundreds_of_thousands_of_writes_and_reads_interleave() {
    let s = sut!(create());

    let versions = collect_concurrently(8, |t| {
        let mut mine = Vec::new();
        for i in 0..60_000 {
            mine.push(s.put(&format!("key-{}", i % 500), &format!("t{t}-{i}")).unwrap());
            s.get(&format!("key-{}", (i * 7) % 500)).unwrap();
        }
        mine
    });

    let all: HashSet<u64> = versions.into_iter().flatten().collect();
    assert_eq!(all.len(), 480_000);
    assert_eq!(s.current_version(), 480_000);
    assert_eq!(s.history("key-0").unwrap().len(), 8 * 120);
}

#[test]
fn should_answer_get_at_consistently_with_history_after_a_concurrent_mixed_workload() {
    let s = sut!(create());

    run_concurrently(8, |t| {
        for i in 0..2_000 {
            let key = format!("k{}", i % 20);
            match i % 5 {
                0 => {
                    s.delete(&key).unwrap();
                }
                1 => {
                    s.put_if_version(&key, &format!("c{t}"), 0).unwrap();
                }
                _ => {
                    s.put(&key, &format!("t{t}-{i}")).unwrap();
                }
            }
        }
    });

    let end = s.current_version();
    for k in 0..20 {
        let key = format!("k{k}");
        let mut entries: HashMap<u64, String> = HashMap::new();
        for line in s.history(&key).unwrap() {
            let (v, rest) = line.split_once(':').unwrap();
            entries.insert(v.parse().unwrap(), rest.to_string());
        }
        let mut expected: Option<String> = None;
        for v in 1..=end {
            if let Some(at) = entries.get(&v) {
                expected = if at == "<deleted>" { None } else { Some(at.clone()) };
            }
            assert_eq!(s.get_at(&key, v), Ok(expected.clone()), "{key} at {v}");
        }
    }
}

#[test]
fn should_compact_while_writers_and_snapshot_readers_are_running() {
    let s = sut!(create());
    s.put_all(&[("left", "0"), ("right", "0")]).unwrap();
    let pinned = s.snapshot();
    let torn = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        if t < 3 {
            for i in 1..=500 {
                let v = format!("{t}-{i}");
                s.put_all(&[("left", &v), ("right", &v)]).unwrap();
            }
        } else if t < 5 {
            for _ in 0..100 {
                let floor = s.current_version();
                s.compact(floor).unwrap();
            }
        } else {
            for _ in 0..400 {
                let snap = s.snapshot();
                if s.snapshot_get(snap, "left") != s.snapshot_get(snap, "right") {
                    torn.fetch_add(1, Ordering::SeqCst);
                }
                s.release_snapshot(snap).unwrap();
                assert_eq!(s.snapshot_get(pinned, "left"), some("0"));
            }
        }
    });

    assert_eq!(torn.load(Ordering::SeqCst), 0);
    assert_eq!(s.snapshot_get(pinned, "left"), some("0"));
    assert_eq!(s.snapshot_get(pinned, "right"), some("0"));
    assert_eq!(s.get("left"), s.get("right"));
    assert_eq!(s.current_version(), 1 + 3 * 500);
}

#[test]
fn should_release_history_only_when_the_last_snapshot_referencing_it_is_released() {
    let s = sut!(create());
    s.put("a", "v1").unwrap(); // 1
    let first = s.snapshot();
    s.put("a", "v2").unwrap(); // 2
    let second = s.snapshot();
    s.put("a", "v3").unwrap(); // 3
    s.put("a", "v4").unwrap(); // 4

    assert_eq!(s.compact(4), Ok(1));
    assert_eq!(
        s.history("a"),
        Ok(vec!["1:v1".to_string(), "2:v2".to_string(), "4:v4".to_string()])
    );

    s.release_snapshot(second).unwrap();
    assert_eq!(s.compact(4), Ok(1));
    assert_eq!(s.history("a"), Ok(vec!["1:v1".to_string(), "4:v4".to_string()]));
    assert_eq!(s.snapshot_get(first, "a"), some("v1"));

    s.release_snapshot(first).unwrap();
    assert_eq!(s.compact(4), Ok(1));
    assert_eq!(s.history("a"), Ok(vec!["4:v4".to_string()]));
}

#[test]
fn should_keep_a_tombstone_that_hides_older_history_another_snapshot_still_needs() {
    let s = sut!(create());
    s.put("a", "v1").unwrap(); // 1
    let before_delete = s.snapshot();
    s.delete("a").unwrap(); // 2
    let after_delete = s.snapshot();
    s.put("b", "other").unwrap(); // 3
    s.put("b", "other2").unwrap(); // 4

    s.compact(4).unwrap();

    assert_eq!(s.snapshot_get(before_delete, "a"), some("v1"));
    assert_eq!(s.snapshot_get(after_delete, "a"), NOTHING);
    assert_eq!(s.get("a"), NOTHING);
    s.release_snapshot(before_delete).unwrap();
    assert_eq!(s.snapshot_get(after_delete, "a"), NOTHING);
}

#[test]
fn should_use_compare_and_set_correctly_across_deletes_under_contention() {
    let s = sut!(create());
    let wins = AtomicUsize::new(0);
    let deletes = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        for _ in 0..2_000 {
            if s.put_if_version("lock", &format!("owner-{t}"), 0).unwrap().is_some() {
                wins.fetch_add(1, Ordering::SeqCst);
                assert!(s.get("lock").unwrap().is_some());
                if s.delete("lock").unwrap().is_some() {
                    deletes.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    let w = wins.load(Ordering::SeqCst);
    assert!(w > 0);
    assert_eq!(w, deletes.load(Ordering::SeqCst));
    assert_eq!(s.get("lock"), NOTHING);
    assert_eq!(s.current_version(), (2 * w) as u64);
    assert_eq!(s.history("lock").unwrap().len(), 2 * w);
}

#[test]
fn should_keep_batches_atomic_when_batches_overlap_on_different_keys() {
    let s = sut!(create());

    run_concurrently(6, |t| {
        for i in 0..1_000 {
            let v = format!("b{t}-{i}");
            let a = format!("x{}", i % 3);
            let b = format!("x{}", (i + 1) % 3);
            s.put_all(&[(a.as_str(), v.as_str()), (b.as_str(), v.as_str())])
                .unwrap();
        }
    });

    assert_eq!(s.current_version(), 6_000);
    let mut v = 1;
    while v <= 6_000 {
        let prefix = format!("{v}:");
        let mut seen: Vec<String> = Vec::new();
        for k in 0..3 {
            for line in s.history(&format!("x{k}")).unwrap() {
                if let Some(rest) = line.strip_prefix(&prefix) {
                    seen.push(rest.to_string());
                }
            }
        }
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0], seen[1]);
        v += 97;
    }
    assert_eq!(s.get_at("x0", 6_001), Err(StoreError::InvalidArgument));
}

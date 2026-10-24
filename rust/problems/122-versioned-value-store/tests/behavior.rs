#![allow(unused_imports, clippy::needless_range_loop)]
use p122_versioned_value_store::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

fn some(v: &str) -> Result<Option<String>, StoreError> {
    Ok(Some(v.to_string()))
}

const NOTHING: Result<Option<String>, StoreError> = Ok(None);

#[test]
fn should_start_at_version_zero_when_nothing_was_written() {
    let s = sut!(create());

    assert_eq!(s.current_version(), 0);
    assert_eq!(s.get("a"), NOTHING);
    assert_eq!(s.history("a"), Ok(vec![]));
}

#[test]
fn should_number_writes_of_all_keys_with_one_gap_free_counter() {
    let s = sut!(create());

    assert_eq!(s.put("a", "x"), Ok(1));
    assert_eq!(s.put("b", "y"), Ok(2));
    assert_eq!(s.put("a", "z"), Ok(3));
    assert_eq!(s.current_version(), 3);
}

#[test]
fn should_return_the_latest_value() {
    let s = sut!(create());
    s.put("a", "x").unwrap();
    s.put("a", "z").unwrap();

    assert_eq!(s.get("a"), some("z"));
    assert_eq!(s.get("other"), NOTHING);
}

#[test]
fn should_accept_an_empty_string_as_a_value() {
    let s = sut!(create());
    let v = s.put("a", "").unwrap();

    assert_eq!(s.get("a"), some(""));
    assert_eq!(s.get_at("a", v), some(""));
    assert_eq!(s.history("a"), Ok(vec!["1:".to_string()]));
}

#[test]
fn should_record_every_write_even_when_the_value_does_not_change() {
    let s = sut!(create());

    assert_eq!(s.put("a", "same"), Ok(1));
    assert_eq!(s.put("a", "same"), Ok(2));
    assert_eq!(s.history("a"), Ok(vec!["1:same".to_string(), "2:same".to_string()]));
}

#[test]
fn should_delete_with_a_tombstone_that_consumes_a_version() {
    let s = sut!(create());
    s.put("a", "x").unwrap();

    assert_eq!(s.delete("a"), Ok(Some(2)));
    assert_eq!(s.get("a"), NOTHING);
    assert_eq!(s.current_version(), 2);
}

#[test]
fn should_return_none_and_consume_no_version_when_deleting_a_key_without_live_value() {
    let s = sut!(create());

    assert_eq!(s.delete("never"), Ok(None));
    s.put("a", "x").unwrap();
    assert_eq!(s.delete("a"), Ok(Some(2)));
    assert_eq!(s.delete("a"), Ok(None));

    assert_eq!(s.current_version(), 2);
    assert_eq!(s.history("a"), Ok(vec!["1:x".to_string(), "2:<deleted>".to_string()]));
    assert_eq!(s.history("never"), Ok(vec![]));
}

#[test]
fn should_allow_writing_a_key_again_after_it_was_deleted() {
    let s = sut!(create());
    s.put("a", "x").unwrap();
    s.delete("a").unwrap();

    assert_eq!(s.put("a", "y"), Ok(3));
    assert_eq!(s.get("a"), some("y"));
    assert_eq!(
        s.history("a"),
        Ok(vec!["1:x".to_string(), "2:<deleted>".to_string(), "3:y".to_string()])
    );
}

#[test]
fn should_answer_get_at_with_the_newest_write_at_or_before_the_version() {
    let s = sut!(create());
    s.put("a", "x").unwrap(); // 1
    s.put("b", "y").unwrap(); // 2
    s.put("a", "z").unwrap(); // 3
    s.put("c", "w").unwrap(); // 4

    assert_eq!(s.get_at("a", 1), some("x"));
    assert_eq!(s.get_at("a", 2), some("x"));
    assert_eq!(s.get_at("a", 3), some("z"));
    assert_eq!(s.get_at("a", 4), some("z"));
    assert_eq!(s.get_at("b", 4), some("y"));
}

#[test]
fn should_return_none_from_get_at_before_the_key_was_first_written() {
    let s = sut!(create());
    s.put("a", "x").unwrap(); // 1
    s.put("b", "y").unwrap(); // 2

    assert_eq!(s.get_at("b", 1), NOTHING);
    assert_eq!(s.get_at("never", 2), NOTHING);
}

#[test]
fn should_return_none_from_get_at_at_and_after_a_tombstone_but_not_before_it() {
    let s = sut!(create());
    s.put("a", "x").unwrap(); // 1
    s.put("b", "y").unwrap(); // 2
    s.delete("b").unwrap(); // 3
    s.put("c", "w").unwrap(); // 4

    assert_eq!(s.get_at("b", 2), some("y"));
    assert_eq!(s.get_at("b", 3), NOTHING);
    assert_eq!(s.get_at("b", 4), NOTHING);
}

#[test]
fn should_reject_get_at_outside_the_version_range() {
    let s = sut!(create());
    s.put("a", "x").unwrap();
    s.put("a", "y").unwrap();

    assert_eq!(s.get_at("a", 0), Err(StoreError::InvalidArgument));
    assert_eq!(s.get_at("a", 3), Err(StoreError::InvalidArgument));
    assert_eq!(s.get_at("a", 2), some("y"));
}

#[test]
fn should_list_history_oldest_first_with_tombstones() {
    let s = sut!(create());
    s.put("a", "x").unwrap(); // 1
    s.put("b", "y").unwrap(); // 2
    s.put("a", "z").unwrap(); // 3
    s.delete("b").unwrap(); // 4

    assert_eq!(s.history("a"), Ok(vec!["1:x".to_string(), "3:z".to_string()]));
    assert_eq!(s.history("b"), Ok(vec!["2:y".to_string(), "4:<deleted>".to_string()]));
}

#[test]
fn should_write_all_entries_of_a_put_all_as_one_version() {
    let s = sut!(create());
    s.put("a", "old").unwrap(); // 1

    let v = s.put_all(&[("a", "new"), ("b", "new"), ("c", "new")]);

    assert_eq!(v, Ok(2));
    assert_eq!(s.current_version(), 2);
    assert_eq!(s.history("b"), Ok(vec!["2:new".to_string()]));
    assert_eq!(s.get_at("a", 1), some("old"));
    assert_eq!(s.get_at("a", 2), some("new"));
    assert_eq!(s.get_at("b", 1), NOTHING);
    assert_eq!(s.get_at("c", 2), some("new"));
    assert_eq!(s.put("d", "x"), Ok(3));
}

#[test]
fn should_reject_an_invalid_put_all_without_applying_anything_or_consuming_a_version() {
    let s = sut!(create());
    s.put("a", "x").unwrap();

    assert_eq!(s.put_all(&[]), Err(StoreError::InvalidArgument));
    assert_eq!(
        s.put_all(&[("good", "1"), (" ", "2")]),
        Err(StoreError::InvalidArgument)
    );
    assert_eq!(
        s.put_all(&[("good", "1"), ("good", "2")]),
        Err(StoreError::InvalidArgument)
    );

    assert_eq!(s.current_version(), 1);
    assert_eq!(s.get("good"), NOTHING);
    assert_eq!(s.put("b", "y"), Ok(2));
}

#[test]
fn should_reject_blank_keys() {
    let s = sut!(create());
    s.put("a", "x").unwrap();

    assert_eq!(s.put("  ", "v"), Err(StoreError::InvalidArgument));
    assert_eq!(s.put("", "v"), Err(StoreError::InvalidArgument));
    assert_eq!(s.delete(" "), Err(StoreError::InvalidArgument));
    assert_eq!(s.get(""), Err(StoreError::InvalidArgument));
    assert_eq!(s.get_at(" ", 1), Err(StoreError::InvalidArgument));
    assert_eq!(s.history(""), Err(StoreError::InvalidArgument));
    assert_eq!(s.put_if_version(" ", "v", 0), Err(StoreError::InvalidArgument));

    assert_eq!(s.current_version(), 1);
}

#[test]
fn should_put_if_version_zero_only_when_the_key_has_no_live_value() {
    let s = sut!(create());

    assert_eq!(s.put_if_version("k", "a", 0), Ok(Some(1)));
    assert_eq!(s.put_if_version("k", "b", 0), Ok(None));
    assert_eq!(s.get("k"), some("a"));
    assert_eq!(s.current_version(), 1);
}

#[test]
fn should_compare_against_the_version_of_the_keys_last_write_not_the_store_version() {
    let s = sut!(create());
    s.put("k", "a").unwrap(); // 1
    s.put("other", "x").unwrap(); // 2
    s.put("other", "y").unwrap(); // 3

    assert_eq!(s.put_if_version("k", "b", 3), Ok(None));
    assert_eq!(s.put_if_version("k", "b", 1), Ok(Some(4)));
    assert_eq!(s.put_if_version("k", "c", 1), Ok(None));
    assert_eq!(s.put_if_version("k", "c", 4), Ok(Some(5)));
    assert_eq!(s.get("k"), some("c"));
    assert_eq!(s.current_version(), 5);
}

#[test]
fn should_not_consume_a_version_on_a_compare_and_set_mismatch() {
    let s = sut!(create());
    s.put("k", "a").unwrap(); // 1

    assert_eq!(s.put_if_version("k", "b", 7), Ok(None));
    assert_eq!(s.put_if_version("k", "b", 0), Ok(None));

    assert_eq!(s.current_version(), 1);
    assert_eq!(s.history("k"), Ok(vec!["1:a".to_string()]));
    assert_eq!(s.put("x", "y"), Ok(2));
}

#[test]
fn should_treat_a_deleted_key_as_version_zero_for_compare_and_set() {
    let s = sut!(create());
    s.put("k", "a").unwrap(); // 1
    s.delete("k").unwrap(); // 2

    assert_eq!(s.put_if_version("k", "b", 1), Ok(None));
    assert_eq!(s.put_if_version("k", "b", 2), Ok(None));
    assert_eq!(s.put_if_version("k", "b", 0), Ok(Some(3)));
    assert_eq!(s.get("k"), some("b"));
}

#[test]
fn should_freeze_a_snapshot_against_later_writes() {
    let s = sut!(create());
    s.put("a", "1").unwrap(); // 1
    s.put("b", "1").unwrap(); // 2
    let snap = s.snapshot();

    s.put("a", "2").unwrap();
    s.delete("b").unwrap();
    s.put("c", "3").unwrap();

    assert_eq!(s.snapshot_get(snap, "a"), some("1"));
    assert_eq!(s.snapshot_get(snap, "b"), some("1"));
    assert_eq!(s.snapshot_get(snap, "c"), NOTHING);
    assert_eq!(s.get("a"), some("2"));
    assert_eq!(s.get("b"), NOTHING);
}

#[test]
fn should_give_each_snapshot_its_own_view_and_distinct_ids() {
    let s = sut!(create());
    let empty = s.snapshot();
    s.put("a", "1").unwrap();
    let one = s.snapshot();
    s.put("a", "2").unwrap();
    let two = s.snapshot();

    let ids: HashSet<u64> = [empty, one, two].into_iter().collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(s.snapshot_get(empty, "a"), NOTHING);
    assert_eq!(s.snapshot_get(one, "a"), some("1"));
    assert_eq!(s.snapshot_get(two, "a"), some("2"));
    assert_eq!(s.current_version(), 2);
}

#[test]
fn should_reject_unknown_and_released_snapshots() {
    let s = sut!(create());
    s.put("a", "1").unwrap();
    let snap = s.snapshot();
    let other = s.snapshot();

    assert_eq!(s.snapshot_get(9_999, "a"), Err(StoreError::UnknownSnapshot));
    assert_eq!(s.release_snapshot(9_999), Err(StoreError::UnknownSnapshot));
    assert_eq!(s.release_snapshot(snap), Ok(()));
    assert_eq!(s.snapshot_get(snap, "a"), Err(StoreError::UnknownSnapshot));
    assert_eq!(s.release_snapshot(snap), Err(StoreError::UnknownSnapshot));
    assert_eq!(s.snapshot_get(other, "a"), some("1"));
    assert_eq!(s.snapshot_get(other, " "), Err(StoreError::InvalidArgument));
}

#[test]
fn should_compact_old_entries_and_report_how_many_were_removed() {
    let s = sut!(create());
    s.put("a", "v1").unwrap(); // 1
    s.put("a", "v2").unwrap(); // 2
    s.put("b", "x").unwrap(); // 3
    s.put("a", "v3").unwrap(); // 4
    s.delete("b").unwrap(); // 5
    s.put("c", "z").unwrap(); // 6

    assert_eq!(s.compact(4), Ok(2));

    assert_eq!(s.history("a"), Ok(vec!["4:v3".to_string()]));
    assert_eq!(s.history("b"), Ok(vec!["3:x".to_string(), "5:<deleted>".to_string()]));
    assert_eq!(s.get_at("a", 4), some("v3"));
    assert_eq!(s.get_at("b", 4), some("x"));
    assert_eq!(s.get_at("b", 5), NOTHING);
    assert_eq!(s.get_at("c", 6), some("z"));
    assert_eq!(s.get("a"), some("v3"));
    assert_eq!(s.compact(4), Ok(0));
}

#[test]
fn should_reject_get_at_below_the_compaction_floor_but_keep_answering_at_and_above_it() {
    let s = sut!(create());
    for v in ["v1", "v2", "v3", "v4"] {
        s.put("a", v).unwrap();
    }

    s.compact(3).unwrap();

    assert_eq!(s.get_at("a", 1), Err(StoreError::Compacted));
    assert_eq!(s.get_at("a", 2), Err(StoreError::Compacted));
    assert_eq!(s.get_at("never", 2), Err(StoreError::Compacted));
    assert_eq!(s.get_at("a", 3), some("v3"));
    assert_eq!(s.get_at("a", 4), some("v4"));
    assert_eq!(s.get_at("a", 0), Err(StoreError::InvalidArgument));
    assert_eq!(s.get_at("a", 5), Err(StoreError::InvalidArgument));
}

#[test]
fn should_keep_the_entry_that_is_visible_at_the_floor_even_when_it_is_older_than_the_floor() {
    let s = sut!(create());
    s.put("old", "kept").unwrap(); // 1
    s.put("x", "1").unwrap(); // 2
    s.put("x", "2").unwrap(); // 3
    s.put("x", "3").unwrap(); // 4

    assert_eq!(s.compact(4), Ok(2));

    assert_eq!(s.get_at("old", 4), some("kept"));
    assert_eq!(s.history("old"), Ok(vec!["1:kept".to_string()]));
    assert_eq!(s.history("x"), Ok(vec!["4:3".to_string()]));
}

#[test]
fn should_forget_keys_whose_tombstone_is_the_only_thing_left() {
    let s = sut!(create());
    s.put("a", "x").unwrap(); // 1
    s.delete("a").unwrap(); // 2
    s.put("b", "y").unwrap(); // 3

    assert_eq!(s.compact(3), Ok(2));

    assert_eq!(s.history("a"), Ok(vec![]));
    assert_eq!(s.get("a"), NOTHING);
    assert_eq!(s.get_at("a", 3), NOTHING);
    assert_eq!(s.put("a", "again"), Ok(4));
}

#[test]
fn should_keep_entries_an_open_snapshot_still_needs_until_it_is_released() {
    let s = sut!(create());
    s.put("a", "v1").unwrap(); // 1
    s.put("a", "v2").unwrap(); // 2
    let snap = s.snapshot();
    s.put("a", "v3").unwrap(); // 3
    s.put("a", "v4").unwrap(); // 4

    assert_eq!(s.compact(4), Ok(2));
    assert_eq!(s.snapshot_get(snap, "a"), some("v2"));
    assert_eq!(s.history("a"), Ok(vec!["2:v2".to_string(), "4:v4".to_string()]));

    s.release_snapshot(snap).unwrap();
    assert_eq!(s.compact(4), Ok(1));
    assert_eq!(s.history("a"), Ok(vec!["4:v4".to_string()]));
}

#[test]
fn should_keep_snapshots_readable_that_were_taken_before_the_floor_moved_past_them() {
    let s = sut!(create());
    s.put("a", "v1").unwrap(); // 1
    s.put("b", "w1").unwrap(); // 2
    let snap = s.snapshot();
    s.delete("a").unwrap(); // 3
    s.put("b", "w2").unwrap(); // 4
    s.put("c", "z").unwrap(); // 5

    s.compact(5).unwrap();

    assert_eq!(s.snapshot_get(snap, "a"), some("v1"));
    assert_eq!(s.snapshot_get(snap, "b"), some("w1"));
    assert_eq!(s.snapshot_get(snap, "c"), NOTHING);
    assert_eq!(s.get("a"), NOTHING);
    assert_eq!(s.get("b"), some("w2"));
    assert_eq!(s.get_at("a", 2), Err(StoreError::Compacted));
}

#[test]
fn should_reject_compaction_outside_the_version_range() {
    let s = sut!(create());
    assert_eq!(s.compact(1), Err(StoreError::InvalidArgument));
    s.put("a", "x").unwrap();
    s.put("a", "y").unwrap();

    assert_eq!(s.compact(0), Err(StoreError::InvalidArgument));
    assert_eq!(s.compact(3), Err(StoreError::InvalidArgument));
    assert_eq!(s.history("a"), Ok(vec!["1:x".to_string(), "2:y".to_string()]));
}

#[test]
fn should_never_lower_the_floor_and_keep_numbering_versions_after_compaction() {
    let s = sut!(create());
    for i in 1..=6 {
        s.put("a", &format!("v{i}")).unwrap();
    }
    assert_eq!(s.compact(5), Ok(4));

    assert_eq!(s.compact(2), Ok(0));
    assert_eq!(s.get_at("a", 3), Err(StoreError::Compacted));
    assert_eq!(s.put("a", "v7"), Ok(7));
    assert_eq!(s.current_version(), 7);
    assert_eq!(s.get_at("a", 6), some("v6"));
    assert_eq!(
        s.history("a"),
        Ok(vec!["5:v5".to_string(), "6:v6".to_string(), "7:v7".to_string()])
    );
}

#[test]
fn should_hand_out_each_version_exactly_once_when_threads_write_distinct_keys() {
    let s = sut!(create());
    let threads = 8;
    let each = 500;

    let per_thread = collect_concurrently(threads, |t| {
        (0..each)
            .map(|i| s.put(&format!("key-{t}-{i}"), "v").unwrap())
            .collect::<Vec<u64>>()
    });

    let all: HashSet<u64> = per_thread.into_iter().flatten().collect();
    assert_eq!(all.len(), threads * each);
    assert!(all.iter().all(|v| *v >= 1 && *v <= (threads * each) as u64));
    assert_eq!(s.current_version(), (threads * each) as u64);
}

#[test]
fn should_keep_a_keys_history_strictly_increasing_when_threads_write_the_same_key() {
    let s = sut!(create());
    let threads = 8;
    let each = 300;

    let per_thread = collect_concurrently(threads, |t| {
        (0..each)
            .map(|i| s.put("hot", &format!("t{t}-{i}")).unwrap())
            .collect::<Vec<u64>>()
    });

    for list in &per_thread {
        assert!(list.windows(2).all(|w| w[0] < w[1]));
    }
    let history = s.history("hot").unwrap();
    assert_eq!(history.len(), threads * each);
    let mut previous = 0u64;
    for entry in &history {
        let version: u64 = entry.split(':').next().unwrap().parse().unwrap();
        assert!(version > previous);
        previous = version;
    }
    assert_eq!(s.current_version(), (threads * each) as u64);
}

#[test]
fn should_let_exactly_one_thread_win_a_compare_and_set_race() {
    let s = sut!(create());

    let results = collect_concurrently(16, |t| s.put_if_version("k", &format!("from-{t}"), 0).unwrap());

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(s.current_version(), 1);
    assert_eq!(s.history("k").unwrap().len(), 1);

    let second = collect_concurrently(16, |t| s.put_if_version("k", &format!("second-{t}"), 1).unwrap());
    assert_eq!(second.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(s.current_version(), 2);
}

#[test]
fn should_let_exactly_one_thread_delete_a_live_key() {
    let s = sut!(create());
    s.put("k", "x").unwrap();

    let results = collect_concurrently(16, |_| s.delete("k").unwrap());

    assert_eq!(results.iter().filter(|r| r.is_some()).count(), 1);
    assert_eq!(results.iter().filter(|r| r.is_none()).count(), 15);
    assert_eq!(s.current_version(), 2);
}

#[test]
fn should_not_lose_increments_when_threads_retry_compare_and_set() {
    let s = sut!(create());
    s.put("counter", "0").unwrap();
    let threads = 8;
    let each = 200;

    run_concurrently(threads, |_| {
        for _ in 0..each {
            loop {
                let last = s.history("counter").unwrap().pop().unwrap();
                let (version, value) = last.split_once(':').unwrap();
                let next = value.parse::<usize>().unwrap() + 1;
                if s
                    .put_if_version("counter", &next.to_string(), version.parse().unwrap())
                    .unwrap()
                    .is_some()
                {
                    break;
                }
            }
        }
    });

    assert_eq!(s.get("counter"), some(&(threads * each).to_string()));
    assert_eq!(s.current_version(), (1 + threads * each) as u64);
}

#[test]
fn should_show_both_keys_of_a_batch_or_neither_in_every_snapshot_taken_during_writes() {
    let s = sut!(create());
    s.put_all(&[("left", "0"), ("right", "0")]).unwrap();
    let torn = AtomicUsize::new(0);

    run_concurrently(8, |t| {
        if t < 3 {
            for i in 1..=400 {
                let v = format!("{t}-{i}");
                s.put_all(&[("left", &v), ("right", &v)]).unwrap();
            }
            return;
        }
        for _ in 0..400 {
            let snap = s.snapshot();
            let left = s.snapshot_get(snap, "left").unwrap();
            let right = s.snapshot_get(snap, "right").unwrap();
            if left != right {
                torn.fetch_add(1, Ordering::SeqCst);
            }
            s.release_snapshot(snap).unwrap();
        }
    });

    assert_eq!(torn.load(Ordering::SeqCst), 0);
    assert_eq!(s.current_version(), 1 + 3 * 400);
    assert_eq!(s.get("left"), s.get("right"));
}

#[test]
fn should_keep_a_snapshot_stable_while_other_threads_keep_writing() {
    let s = sut!(create());
    for i in 0..50 {
        s.put(&format!("k{i}"), "base").unwrap();
    }
    let snap = s.snapshot();

    run_concurrently(8, |t| {
        for i in 0..300 {
            let key = format!("k{}", i % 50);
            s.put(&key, &format!("t{t}")).unwrap();
            if i % 7 == 0 {
                s.delete(&key).unwrap();
            }
            assert_eq!(s.snapshot_get(snap, &key), some("base"));
            assert_eq!(s.snapshot_get(snap, &format!("new-{t}")), NOTHING);
        }
    });

    for i in 0..50 {
        assert_eq!(s.snapshot_get(snap, &format!("k{i}")), some("base"));
    }
}

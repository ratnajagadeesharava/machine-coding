#![allow(unused_imports, clippy::needless_range_loop)]
use p124_kafka_streaming::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn should_hash_surrogate_pairs_as_two_utf16_code_units() {
    let s = sut!(create());
    s.create_topic("t", 7, None).unwrap();

    // U+1F600 is the code units D83D DE00: 31*55357 + 56832 = 1772899 -> % 7 = 2 (UTF-8 bytes would give 4)
    assert_eq!(s.publish("t", Some("\u{1F600}"), "x").unwrap(), "2:0");
}

#[test]
fn should_not_use_absolute_value_for_the_minimum_hash() {
    let s = sut!(create());
    s.create_topic("t4", 4, None).unwrap();
    s.create_topic("t7", 7, None).unwrap();

    // i32::MIN: rem_euclid 4 = 0, rem_euclid 7 = 5
    assert_eq!(
        s.publish("t4", Some("polygenelubricants"), "x").unwrap(),
        "0:0"
    );
    assert_eq!(
        s.publish("t7", Some("polygenelubricants"), "x").unwrap(),
        "5:0"
    );
}

#[test]
fn should_split_seven_partitions_across_three_consumers() {
    let s = sut!(create());
    s.create_topic("t", 7, None).unwrap();
    for c in ["c3", "c1", "c2"] {
        s.join_group("g", c, "t").unwrap();
    }

    assert_eq!(s.assignment("g", "c1", "t"), Ok(vec![0, 1, 2]));
    assert_eq!(s.assignment("g", "c2", "t"), Ok(vec![3, 4]));
    assert_eq!(s.assignment("g", "c3", "t"), Ok(vec![5, 6]));
}

#[test]
fn should_order_consumer_ids_as_plain_strings() {
    let s = sut!(create());
    s.create_topic("t", 3, None).unwrap();
    for c in ["c10", "c2", "c1"] {
        s.join_group("g", c, "t").unwrap();
    }

    // plain string order: c1 < c10 < c2
    assert_eq!(s.assignment("g", "c1", "t"), Ok(vec![0]));
    assert_eq!(s.assignment("g", "c10", "t"), Ok(vec![1]));
    assert_eq!(s.assignment("g", "c2", "t"), Ok(vec![2]));
}

#[test]
fn should_reject_a_commit_from_a_consumer_that_lost_the_partition_in_a_rebalance() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.publish("t", None, "x").unwrap();
    s.publish("t", None, "y").unwrap();
    s.commit("g", "a", "t", 1, 1).unwrap();

    s.join_group("g", "b", "t").unwrap();

    assert_eq!(s.commit("g", "a", "t", 1, 1), Err(StreamError::NotAssigned));
    s.commit("g", "b", "t", 1, 1).unwrap();
    assert_eq!(s.committed_offset("g", "t", 1), Ok(1));
}

#[test]
fn should_leave_state_untouched_after_failed_operations() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.publish("t", None, "x").unwrap();

    assert_eq!(
        s.commit("g", "a", "t", 0, 9),
        Err(StreamError::InvalidOffset)
    );
    assert_eq!(
        s.publish(" ", Some("k"), "m"),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(s.create_topic("t", 9, None), Err(StreamError::TopicExists));

    assert_eq!(s.committed_offset("g", "t", 0), Ok(0));
    assert_eq!(s.message_count("t"), Ok(1));
    assert_eq!(s.assignment("g", "a", "t"), Ok(vec![0, 1]));
    assert_eq!(s.publish("t", None, "y").unwrap(), "1:0");
}

#[test]
fn should_retain_exactly_the_newest_messages_under_concurrent_publishes() {
    let s = sut!(create());
    s.create_topic("t", 1, Some(100)).unwrap();

    run_concurrently(8, |_| {
        for _ in 0..2_000 {
            s.publish("t", Some("same"), "m").unwrap();
        }
    });

    assert_eq!(s.latest_offset("t", 0), Ok(16_000));
    assert_eq!(s.earliest_offset("t", 0), Ok(15_900));
    assert_eq!(s.message_count("t"), Ok(100));
}

#[test]
fn should_deliver_every_message_exactly_once_to_two_consumers_while_producers_run() {
    let s = sut!(create());
    s.create_topic("t", 4, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.join_group("g", "b", "t").unwrap();
    let (producers, each) = (4usize, 1_000usize);
    let done = AtomicUsize::new(0);

    let seen = collect_concurrently(producers + 2, |t| {
        let mut mine: Vec<String> = vec![];
        if t < producers {
            for i in 0..each {
                let key = if i % 3 == 0 {
                    None
                } else {
                    Some(format!("k{}", i % 5))
                };
                s.publish("t", key.as_deref(), &format!("p{t}-{i}"))
                    .unwrap();
            }
            done.fetch_add(1, Ordering::SeqCst);
            return mine;
        }
        let me = if t == producers { "a" } else { "b" };
        loop {
            let finished = done.load(Ordering::SeqCst) == producers;
            let batch = s.poll("g", me, "t", 50).unwrap();
            for e in &batch {
                let mut it = e.splitn(3, ':');
                let p: usize = it.next().unwrap().parse().unwrap();
                let o: u64 = it.next().unwrap().parse().unwrap();
                mine.push(it.next().unwrap().to_string());
                s.commit("g", me, "t", p, o + 1).unwrap();
            }
            if batch.is_empty() && finished {
                break;
            }
        }
        mine
    });

    let total: usize = seen.iter().map(|v| v.len()).sum();
    let all: HashSet<&String> = seen.iter().flatten().collect();
    assert_eq!(total, producers * each);
    assert_eq!(all.len(), producers * each);
}

#[test]
fn should_keep_assignments_disjoint_and_complete_under_concurrent_joins_and_leaves() {
    let s = sut!(create());
    s.create_topic("t", 16, None).unwrap();

    run_concurrently(8, |t| {
        let me = format!("c{t}");
        for _ in 0..300 {
            s.join_group("g", &me, "t").unwrap();
            s.assignment("g", &me, "t").unwrap();
            s.leave_group("g", &me, "t").unwrap();
        }
        s.join_group("g", &me, "t").unwrap();
    });

    let mut covered = HashSet::new();
    for t in 0..8 {
        let a = s.assignment("g", &format!("c{t}"), "t").unwrap();
        assert_eq!(a.len(), 2);
        for p in a {
            assert!(covered.insert(p));
        }
    }
    assert_eq!(covered.len(), 16);
}

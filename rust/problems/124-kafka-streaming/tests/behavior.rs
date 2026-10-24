#![allow(unused_imports, clippy::needless_range_loop)]
use p124_kafka_streaming::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::Arc;

fn s_vec(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn should_assign_gap_free_offsets_per_partition() {
    let s = sut!(create());
    s.create_topic("t", 3, None).unwrap();

    // keys "a","b","c" go to partitions 1,2,0 of 3 (97 % 3, 98 % 3, 99 % 3)
    assert_eq!(s.publish("t", Some("a"), "m1").unwrap(), "1:0");
    assert_eq!(s.publish("t", Some("b"), "m2").unwrap(), "2:0");
    assert_eq!(s.publish("t", Some("a"), "m3").unwrap(), "1:1");
    assert_eq!(s.publish("t", Some("c"), "m4").unwrap(), "0:0");
    assert_eq!(s.publish("t", Some("a"), "m5").unwrap(), "1:2");
    assert_eq!(s.latest_offset("t", 1), Ok(3));
    assert_eq!(s.latest_offset("t", 2), Ok(1));
    assert_eq!(s.latest_offset("t", 0), Ok(1));
}

#[test]
fn should_round_robin_keyless_messages_starting_at_partition_zero() {
    let s = sut!(create());
    s.create_topic("t", 3, None).unwrap();

    let got: Vec<String> = (0..7)
        .map(|i| s.publish("t", None, &format!("m{i}")).unwrap())
        .collect();

    assert_eq!(
        got,
        s_vec(&["0:0", "1:0", "2:0", "0:1", "1:1", "2:1", "0:2"])
    );
}

#[test]
fn should_not_advance_round_robin_for_keyed_messages() {
    let s = sut!(create());
    s.create_topic("t", 3, None).unwrap();

    assert_eq!(s.publish("t", None, "x").unwrap(), "0:0");
    assert_eq!(s.publish("t", Some("b"), "x").unwrap(), "2:0");
    assert_eq!(s.publish("t", None, "x").unwrap(), "1:0");
    assert_eq!(s.publish("t", Some("b"), "x").unwrap(), "2:1");
    assert_eq!(s.publish("t", None, "x").unwrap(), "2:2");
}

#[test]
fn should_keep_round_robin_counters_per_topic() {
    let s = sut!(create());
    s.create_topic("a", 2, None).unwrap();
    s.create_topic("b", 2, None).unwrap();

    assert_eq!(s.publish("a", None, "x").unwrap(), "0:0");
    assert_eq!(s.publish("a", None, "x").unwrap(), "1:0");
    assert_eq!(s.publish("b", None, "x").unwrap(), "0:0");
}

#[test]
fn should_use_polynomial_hash_over_the_key_characters() {
    let s = sut!(create());
    s.create_topic("t", 4, None).unwrap();

    // "ab" = 97*31 + 98 = 3105 -> 3105 % 4 = 1 ; "" -> 0 ; single char "c" = 99 -> 3
    assert_eq!(s.publish("t", Some("ab"), "x").unwrap(), "1:0");
    assert_eq!(s.publish("t", Some(""), "x").unwrap(), "0:0");
    assert_eq!(s.publish("t", Some("c"), "x").unwrap(), "3:0");
}

#[test]
fn should_map_negative_hashes_with_floor_modulo() {
    let s = sut!(create());
    s.create_topic("t3", 3, None).unwrap();
    s.create_topic("t5", 5, None).unwrap();

    // the 31-polynomial hash of this key is i32::MIN
    let key = "polygenelubricants";
    assert_eq!(s.publish("t3", Some(key), "x").unwrap(), "1:0");
    assert_eq!(s.publish("t5", Some(key), "x").unwrap(), "2:0");
}

#[test]
fn should_reject_bad_topic_definitions() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();

    assert_eq!(s.create_topic("t", 2, None), Err(StreamError::TopicExists));
    assert_eq!(
        s.create_topic("t", 5, Some(10)),
        Err(StreamError::TopicExists)
    );
    assert_eq!(
        s.create_topic(" ", 2, None),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(
        s.create_topic("z", 0, None),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(
        s.create_topic("z", 2, Some(0)),
        Err(StreamError::InvalidArgument)
    );
    s.create_topic("z", 2, None).unwrap();
    assert_eq!(s.message_count("z"), Ok(0));
}

#[test]
fn should_reject_bad_publishes() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();

    assert_eq!(
        s.publish("nope", Some("k"), "m"),
        Err(StreamError::UnknownTopic)
    );
    assert_eq!(
        s.publish(" ", Some("k"), "m"),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(s.message_count("t"), Ok(0));
    assert_eq!(s.publish("t", None, "").unwrap(), "0:0");
}

#[test]
fn should_count_messages_across_all_partitions() {
    let s = sut!(create());
    s.create_topic("t", 3, None).unwrap();
    s.create_topic("other", 1, None).unwrap();

    for i in 0..10 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }
    s.publish("other", None, "x").unwrap();

    assert_eq!(s.message_count("t"), Ok(10));
    assert_eq!(s.message_count("other"), Ok(1));
    assert_eq!(s.earliest_offset("t", 0), Ok(0));
    assert_eq!(s.latest_offset("t", 0), Ok(4));
    assert_eq!(s.latest_offset("t", 1), Ok(3));
    assert_eq!(s.message_count("nope"), Err(StreamError::UnknownTopic));
    assert_eq!(s.latest_offset("t", 3), Err(StreamError::InvalidArgument));
    assert_eq!(s.earliest_offset("t", 3), Err(StreamError::InvalidArgument));
}

#[test]
fn should_split_partitions_into_contiguous_ranges_with_extras_to_the_first_consumers() {
    let s = sut!(create());
    s.create_topic("t", 5, None).unwrap();

    s.join_group("g", "c1", "t").unwrap();
    assert_eq!(s.assignment("g", "c1", "t"), Ok(vec![0, 1, 2, 3, 4]));
    s.join_group("g", "c2", "t").unwrap();
    assert_eq!(s.assignment("g", "c1", "t"), Ok(vec![0, 1, 2]));
    assert_eq!(s.assignment("g", "c2", "t"), Ok(vec![3, 4]));
    s.join_group("g", "c3", "t").unwrap();
    assert_eq!(s.assignment("g", "c1", "t"), Ok(vec![0, 1]));
    assert_eq!(s.assignment("g", "c2", "t"), Ok(vec![2, 3]));
    assert_eq!(s.assignment("g", "c3", "t"), Ok(vec![4]));
}

#[test]
fn should_order_consumers_by_id_not_by_join_order() {
    let s = sut!(create());
    s.create_topic("t", 4, None).unwrap();

    s.join_group("g", "zed", "t").unwrap();
    s.join_group("g", "amy", "t").unwrap();
    s.join_group("g", "mia", "t").unwrap();

    assert_eq!(s.assignment("g", "amy", "t"), Ok(vec![0, 1]));
    assert_eq!(s.assignment("g", "mia", "t"), Ok(vec![2]));
    assert_eq!(s.assignment("g", "zed", "t"), Ok(vec![3]));
}

#[test]
fn should_rebalance_when_a_consumer_leaves() {
    let s = sut!(create());
    s.create_topic("t", 6, None).unwrap();
    for c in ["a", "b", "c"] {
        s.join_group("g", c, "t").unwrap();
    }
    assert_eq!(s.assignment("g", "b", "t"), Ok(vec![2, 3]));

    s.leave_group("g", "a", "t").unwrap();

    assert_eq!(s.assignment("g", "b", "t"), Ok(vec![0, 1, 2]));
    assert_eq!(s.assignment("g", "c", "t"), Ok(vec![3, 4, 5]));
    assert_eq!(s.assignment("g", "a", "t"), Err(StreamError::NotMember));
}

#[test]
fn should_leave_surplus_consumers_idle() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    for c in ["a", "b", "c"] {
        s.join_group("g", c, "t").unwrap();
    }
    s.publish("t", None, "x").unwrap();

    assert_eq!(s.assignment("g", "a", "t"), Ok(vec![0]));
    assert_eq!(s.assignment("g", "b", "t"), Ok(vec![1]));
    assert_eq!(s.assignment("g", "c", "t"), Ok(vec![]));
    assert_eq!(s.poll("g", "c", "t", 10), Ok(vec![]));
}

#[test]
fn should_treat_repeated_join_as_no_op_and_reject_outsiders() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.join_group("g", "b", "t").unwrap();

    assert_eq!(s.assignment("g", "a", "t"), Ok(vec![0]));
    assert_eq!(
        s.leave_group("g", "ghost", "t"),
        Err(StreamError::NotMember)
    );
    assert_eq!(s.poll("g", "ghost", "t", 1), Err(StreamError::NotMember));
    assert_eq!(
        s.commit("g", "ghost", "t", 0, 0),
        Err(StreamError::NotMember)
    );
    assert_eq!(
        s.assignment("other-group", "a", "t"),
        Err(StreamError::NotMember)
    );
    assert_eq!(
        s.join_group("g", "a", "nope"),
        Err(StreamError::UnknownTopic)
    );
    assert_eq!(
        s.join_group(" ", "a", "t"),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(
        s.join_group("g", "", "t"),
        Err(StreamError::InvalidArgument)
    );
}

#[test]
fn should_poll_partitions_in_order_then_offsets_and_honour_max() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "c", "t").unwrap();
    for i in 0..6 {
        s.publish("t", None, &format!("m{i}")).unwrap(); // p0: m0 m2 m4 ; p1: m1 m3 m5
    }

    assert_eq!(
        s.poll("g", "c", "t", 100),
        Ok(s_vec(&[
            "0:0:m0", "0:1:m2", "0:2:m4", "1:0:m1", "1:1:m3", "1:2:m5"
        ]))
    );
    assert_eq!(
        s.poll("g", "c", "t", 4),
        Ok(s_vec(&["0:0:m0", "0:1:m2", "0:2:m4", "1:0:m1"]))
    );
    assert_eq!(s.poll("g", "c", "t", 1), Ok(s_vec(&["0:0:m0"])));
}

#[test]
fn should_keep_colons_in_message_payloads() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "c", "t").unwrap();
    s.publish("t", None, "a:b:c").unwrap();

    assert_eq!(s.poll("g", "c", "t", 5), Ok(s_vec(&["0:0:a:b:c"])));
}

#[test]
fn should_not_advance_the_committed_offset_when_polling() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "c", "t").unwrap();
    for i in 0..5 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }

    assert_eq!(s.poll("g", "c", "t", 2), Ok(s_vec(&["0:0:m0", "0:1:m1"])));
    assert_eq!(s.poll("g", "c", "t", 2), Ok(s_vec(&["0:0:m0", "0:1:m1"])));
    assert_eq!(s.committed_offset("g", "t", 0), Ok(0));

    s.commit("g", "c", "t", 0, 2).unwrap();

    assert_eq!(s.committed_offset("g", "t", 0), Ok(2));
    assert_eq!(
        s.poll("g", "c", "t", 10),
        Ok(s_vec(&["0:2:m2", "0:3:m3", "0:4:m4"]))
    );
    s.commit("g", "c", "t", 0, 5).unwrap();
    assert_eq!(s.poll("g", "c", "t", 10), Ok(vec![]));
}

#[test]
fn should_see_messages_published_after_the_last_poll() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "c", "t").unwrap();
    s.publish("t", None, "m0").unwrap();
    s.commit("g", "c", "t", 0, 1).unwrap();
    assert_eq!(s.poll("g", "c", "t", 10), Ok(vec![]));

    s.publish("t", None, "m1").unwrap();

    assert_eq!(s.poll("g", "c", "t", 10), Ok(s_vec(&["0:1:m1"])));
}

#[test]
fn should_poll_only_the_assigned_partitions() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.join_group("g", "b", "t").unwrap();
    for i in 0..4 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }

    assert_eq!(s.poll("g", "a", "t", 10), Ok(s_vec(&["0:0:m0", "0:1:m2"])));
    assert_eq!(s.poll("g", "b", "t", 10), Ok(s_vec(&["1:0:m1", "1:1:m3"])));
}

#[test]
fn should_let_groups_consume_independently() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g1", "c", "t").unwrap();
    s.join_group("g2", "c", "t").unwrap();
    for i in 0..3 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }

    s.commit("g1", "c", "t", 0, 3).unwrap();

    assert_eq!(s.poll("g1", "c", "t", 10), Ok(vec![]));
    assert_eq!(
        s.poll("g2", "c", "t", 10),
        Ok(s_vec(&["0:0:m0", "0:1:m1", "0:2:m2"]))
    );
    assert_eq!(s.committed_offset("g2", "t", 0), Ok(0));
}

#[test]
fn should_keep_topic_memberships_of_one_group_separate() {
    let s = sut!(create());
    s.create_topic("a", 2, None).unwrap();
    s.create_topic("b", 1, None).unwrap();
    s.join_group("g", "c1", "a").unwrap();
    s.join_group("g", "c2", "a").unwrap();
    s.join_group("g", "c1", "b").unwrap();

    assert_eq!(s.assignment("g", "c1", "a"), Ok(vec![0]));
    assert_eq!(s.assignment("g", "c1", "b"), Ok(vec![0]));
    assert_eq!(s.assignment("g", "c2", "b"), Err(StreamError::NotMember));
}

#[test]
fn should_reject_backwards_and_past_the_end_commits() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "c", "t").unwrap();
    for i in 0..4 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }
    s.commit("g", "c", "t", 0, 3).unwrap();

    assert_eq!(
        s.commit("g", "c", "t", 0, 2),
        Err(StreamError::InvalidOffset)
    );
    assert_eq!(
        s.commit("g", "c", "t", 0, 5),
        Err(StreamError::InvalidOffset)
    );
    assert_eq!(s.committed_offset("g", "t", 0), Ok(3));
    s.commit("g", "c", "t", 0, 3).unwrap(); // same offset is fine
    s.commit("g", "c", "t", 0, 4).unwrap(); // exactly the log end is fine
    assert_eq!(s.committed_offset("g", "t", 0), Ok(4));
}

#[test]
fn should_reject_commits_for_partitions_the_consumer_does_not_own() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.join_group("g", "b", "t").unwrap();
    s.publish("t", None, "x").unwrap();
    s.publish("t", None, "y").unwrap();

    assert_eq!(s.commit("g", "a", "t", 1, 1), Err(StreamError::NotAssigned));
    assert_eq!(
        s.commit("g", "a", "t", 2, 0),
        Err(StreamError::InvalidArgument)
    );
    assert_eq!(
        s.commit("g", "a", "nope", 0, 0),
        Err(StreamError::UnknownTopic)
    );
    s.commit("g", "b", "t", 1, 1).unwrap();
    assert_eq!(s.committed_offset("g", "t", 1), Ok(1));
}

#[test]
fn should_keep_committed_offsets_across_rebalances() {
    let s = sut!(create());
    s.create_topic("t", 2, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    for i in 0..6 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }
    s.commit("g", "a", "t", 0, 2).unwrap();
    s.commit("g", "a", "t", 1, 1).unwrap();

    s.join_group("g", "b", "t").unwrap(); // a keeps 0, b takes 1
    assert_eq!(s.poll("g", "b", "t", 10), Ok(s_vec(&["1:1:m3", "1:2:m5"])));
    assert_eq!(s.commit("g", "a", "t", 1, 3), Err(StreamError::NotAssigned));

    s.leave_group("g", "b", "t").unwrap(); // a owns both again
    assert_eq!(
        s.poll("g", "a", "t", 10),
        Ok(s_vec(&["0:2:m4", "1:1:m3", "1:2:m5"]))
    );
}

#[test]
fn should_keep_group_offsets_when_everyone_leaves_and_comes_back() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "a", "t").unwrap();
    s.publish("t", None, "m0").unwrap();
    s.publish("t", None, "m1").unwrap();
    s.commit("g", "a", "t", 0, 1).unwrap();
    s.leave_group("g", "a", "t").unwrap();

    s.join_group("g", "z", "t").unwrap();

    assert_eq!(s.poll("g", "z", "t", 10), Ok(s_vec(&["0:1:m1"])));
}

#[test]
fn should_reject_invalid_poll_arguments() {
    let s = sut!(create());
    s.create_topic("t", 1, None).unwrap();
    s.join_group("g", "c", "t").unwrap();

    assert_eq!(s.poll("g", "c", "t", 0), Err(StreamError::InvalidArgument));
    assert_eq!(s.poll("g", "c", "nope", 1), Err(StreamError::UnknownTopic));
    assert_eq!(s.poll("g", " ", "t", 1), Err(StreamError::InvalidArgument));
    assert_eq!(s.poll("g", "c", "t", 1), Ok(vec![]));
}

#[test]
fn should_drop_the_oldest_messages_beyond_the_retention_limit() {
    let s = sut!(create());
    s.create_topic("t", 1, Some(3)).unwrap();
    s.join_group("g", "c", "t").unwrap();

    for i in 0..5u64 {
        assert_eq!(
            s.publish("t", None, &format!("m{i}")).unwrap(),
            format!("0:{i}")
        );
    }

    assert_eq!(s.earliest_offset("t", 0), Ok(2));
    assert_eq!(s.latest_offset("t", 0), Ok(5));
    assert_eq!(s.message_count("t"), Ok(3));
    assert_eq!(
        s.poll("g", "c", "t", 10),
        Ok(s_vec(&["0:2:m2", "0:3:m3", "0:4:m4"]))
    );
}

#[test]
fn should_apply_retention_to_each_partition_separately() {
    let s = sut!(create());
    s.create_topic("t", 2, Some(2)).unwrap();

    for i in 0..6 {
        s.publish("t", None, &format!("m{i}")).unwrap(); // 3 per partition
    }
    s.publish("t", Some("b"), "k").unwrap(); // 98 % 2 = 0 -> 4 in p0

    assert_eq!(s.earliest_offset("t", 0), Ok(2));
    assert_eq!(s.earliest_offset("t", 1), Ok(1));
    assert_eq!(s.message_count("t"), Ok(4));
}

#[test]
fn should_jump_to_the_earliest_offset_when_the_committed_one_was_dropped() {
    let s = sut!(create());
    s.create_topic("t", 1, Some(2)).unwrap();
    s.join_group("g", "c", "t").unwrap();
    s.publish("t", None, "m0").unwrap();
    s.commit("g", "c", "t", 0, 1).unwrap();
    for i in 1..6 {
        s.publish("t", None, &format!("m{i}")).unwrap();
    }

    assert_eq!(s.committed_offset("g", "t", 0), Ok(1));
    assert_eq!(s.earliest_offset("t", 0), Ok(4));
    assert_eq!(s.poll("g", "c", "t", 10), Ok(s_vec(&["0:4:m4", "0:5:m5"])));
    s.commit("g", "c", "t", 0, 6).unwrap();
    assert_eq!(s.poll("g", "c", "t", 10), Ok(vec![]));
}

#[test]
fn should_give_concurrent_publishers_unique_gap_free_offsets() {
    let s = sut!(create());
    s.create_topic("t", 4, None).unwrap();
    let (threads, each) = (8usize, 500usize);

    let results = collect_concurrently(threads, |t| {
        (0..each)
            .map(|i| {
                let key = if i % 2 == 0 {
                    None
                } else {
                    Some(format!("key-{}", i % 7))
                };
                s.publish("t", key.as_deref(), &format!("t{t}-{i}"))
                    .unwrap()
            })
            .collect::<Vec<_>>()
    });

    let all: HashSet<String> = results.into_iter().flatten().collect();
    assert_eq!(all.len(), threads * each);
    let mut total = 0;
    for p in 0..4 {
        let latest = s.latest_offset("t", p).unwrap();
        total += latest;
        for o in 0..latest {
            assert!(all.contains(&format!("{p}:{o}")), "missing {p}:{o}");
        }
    }
    assert_eq!(total, (threads * each) as u64);
    assert_eq!(s.message_count("t"), Ok((threads * each) as u64));
}

#[test]
fn should_round_robin_exactly_evenly_under_concurrent_keyless_publishes() {
    let s = sut!(create());
    s.create_topic("t", 4, None).unwrap();

    run_concurrently(8, |_| {
        for _ in 0..250 {
            s.publish("t", None, "m").unwrap();
        }
    });

    for p in 0..4 {
        assert_eq!(s.latest_offset("t", p), Ok(500));
    }
}

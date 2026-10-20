#![allow(unused_imports, clippy::needless_range_loop)]
use p83_feature_flags::{contract::*, create};
use playground_support::{run_concurrently, sut};

#[test]
fn should_evaluate_a_million_times_while_rules_are_being_edited() {
    let f = sut!(create());
    f.define("x", false).unwrap();
    f.set_rollout("x", 50).unwrap();

    run_concurrently(9, |t| {
        if t == 8 {
            for i in 0..2_000 {
                f.add_rule("x", "tenant", &format!("t{i}"), i % 2 == 0)
                    .unwrap();
            }
            return;
        }
        for i in 0..125_000 {
            f.is_enabled(
                "x",
                &format!("user-{i}"),
                &[("tenant", &format!("t{}", i % 2_000))],
            )
            .unwrap();
        }
    });

    assert_eq!(f.is_enabled("x", "u", &[("tenant", "t0")]), Ok(true));
    assert_eq!(f.is_enabled("x", "u", &[("tenant", "t1")]), Ok(false));
}

#[test]
fn should_keep_a_killed_flag_off_under_concurrent_toggling() {
    let f = sut!(create());
    f.define("x", true).unwrap();

    run_concurrently(8, |_| {
        for i in 0..10_000u32 {
            f.set_killed("x", true).unwrap();
            f.set_rollout("x", i % 101).unwrap();
        }
    });

    assert_eq!(f.is_enabled("x", "u", &[]), Ok(false));
}

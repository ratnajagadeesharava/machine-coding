#![allow(unused_imports, clippy::needless_range_loop)]
use p83_feature_flags::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Arc;

const NONE: &[(&str, &str)] = &[];

fn count_on(f: &Arc<dyn FlagService>, flag: &str, users: usize) -> usize {
    (0..users).filter(|u| f.is_enabled(flag, &format!("user-{u}"), NONE).unwrap()).count()
}

#[test]
fn should_return_the_default_value() {
    let f = sut!(create());
    f.define("on", true).unwrap();
    f.define("off", false).unwrap();

    assert_eq!(f.is_enabled("on", "u1", NONE), Ok(true));
    assert_eq!(f.is_enabled("off", "u1", NONE), Ok(false));
}

#[test]
fn should_reject_unknown_flags_duplicates_and_bad_input() {
    let f = sut!(create());
    f.define("x", false).unwrap();

    assert_eq!(f.is_enabled("nope", "u1", NONE), Err(FlagError::UnknownFlag));
    assert_eq!(f.define("x", true), Err(FlagError::AlreadyDefined));
    assert_eq!(f.define(" ", true), Err(FlagError::InvalidArgument));
    assert_eq!(f.is_enabled("x", " ", NONE), Err(FlagError::InvalidArgument));
    assert_eq!(f.add_rule("nope", "a", "b", true), Err(FlagError::UnknownFlag));
    assert_eq!(f.set_rollout("x", 101), Err(FlagError::InvalidArgument));
}

#[test]
fn should_apply_the_first_matching_rule() {
    let f = sut!(create());
    f.define("beta", false).unwrap();
    f.add_rule("beta", "country", "IN", true).unwrap();
    f.add_rule("beta", "country", "IN", false).unwrap();
    f.add_rule("beta", "plan", "free", false).unwrap();
    f.add_rule("beta", "plan", "pro", true).unwrap();

    assert_eq!(f.is_enabled("beta", "u", &[("country", "IN")]), Ok(true));
    assert_eq!(f.is_enabled("beta", "u", &[("plan", "pro")]), Ok(true));
    assert_eq!(f.is_enabled("beta", "u", &[("plan", "free"), ("country", "US")]), Ok(false));
    assert_eq!(f.is_enabled("beta", "u", &[("country", "US")]), Ok(false));
    assert_eq!(f.is_enabled("beta", "u", NONE), Ok(false));
}

#[test]
fn should_let_rules_override_the_rollout() {
    let f = sut!(create());
    f.define("x", false).unwrap();
    f.set_rollout("x", 0).unwrap();
    f.add_rule("x", "vip", "yes", true).unwrap();

    assert_eq!(f.is_enabled("x", "anyone", &[("vip", "yes")]), Ok(true));
    assert_eq!(f.is_enabled("x", "anyone", NONE), Ok(false));
}

#[test]
fn should_let_the_kill_switch_override_everything() {
    let f = sut!(create());
    f.define("x", true).unwrap();
    f.add_rule("x", "vip", "yes", true).unwrap();
    f.set_rollout("x", 100).unwrap();

    f.set_killed("x", true).unwrap();

    assert_eq!(f.is_enabled("x", "u", &[("vip", "yes")]), Ok(false));
    assert_eq!(f.is_enabled("x", "u", NONE), Ok(false));
    f.set_killed("x", false).unwrap();
    assert_eq!(f.is_enabled("x", "u", NONE), Ok(true));
}

#[test]
fn should_honour_the_rollout_extremes() {
    let f = sut!(create());
    f.define("none", true).unwrap();
    f.define("all", false).unwrap();
    f.set_rollout("none", 0).unwrap();
    f.set_rollout("all", 100).unwrap();

    assert_eq!(count_on(&f, "none", 2_000), 0);
    assert_eq!(count_on(&f, "all", 2_000), 2_000);
}

#[test]
fn should_roll_out_to_roughly_the_requested_percentage() {
    let f = sut!(create());
    f.define("x", false).unwrap();
    f.set_rollout("x", 30).unwrap();

    let on = count_on(&f, "x", 20_000);

    assert!((5_400..=6_600).contains(&on), "got {on}");
}

#[test]
fn should_give_the_same_user_the_same_answer_every_time() {
    let f = sut!(create());
    f.define("x", false).unwrap();
    f.set_rollout("x", 50).unwrap();
    let first: Vec<bool> = (0..1_000).map(|u| f.is_enabled("x", &format!("user-{u}"), NONE).unwrap()).collect();

    for _ in 0..3 {
        for u in 0..1_000 {
            assert_eq!(f.is_enabled("x", &format!("user-{u}"), NONE).unwrap(), first[u]);
        }
    }
}

#[test]
fn should_never_turn_anybody_off_when_the_rollout_grows() {
    let f = sut!(create());
    f.define("x", false).unwrap();
    f.set_rollout("x", 30).unwrap();
    let before: Vec<bool> = (0..10_000).map(|u| f.is_enabled("x", &format!("user-{u}"), NONE).unwrap()).collect();

    f.set_rollout("x", 60).unwrap();

    let mut total = 0;
    for u in 0..10_000 {
        let now = f.is_enabled("x", &format!("user-{u}"), NONE).unwrap();
        assert!(!before[u] || now, "user-{u} was on at 30%");
        total += now as usize;
    }
    assert!((5_600..=6_400).contains(&total), "got {total}");
}

#[test]
fn should_bucket_different_flags_independently() {
    let f = sut!(create());
    f.define("a", false).unwrap();
    f.define("b", false).unwrap();
    f.set_rollout("a", 50).unwrap();
    f.set_rollout("b", 50).unwrap();

    let both = (0..20_000)
        .filter(|u| {
            let id = format!("user-{u}");
            f.is_enabled("a", &id, NONE).unwrap() && f.is_enabled("b", &id, NONE).unwrap()
        })
        .count();

    assert!((4_400..=5_600).contains(&both), "correlated buckets would give ~50%, got {both}");
}

#[test]
fn should_lose_no_rule_when_many_threads_add_rules() {
    let f = sut!(create());
    f.define("x", false).unwrap();

    run_concurrently(16, |t| f.add_rule("x", &format!("k{t}"), "v", true).unwrap());

    for t in 0..16 {
        assert_eq!(f.is_enabled("x", "u", &[(&format!("k{t}"), "v")]), Ok(true), "rule {t}");
    }
}

#[test]
fn should_define_many_flags_concurrently_while_evaluating() {
    let f = sut!(create());

    run_concurrently(8, |t| {
        for i in 0..200 {
            let name = format!("flag-{t}-{i}");
            f.define(&name, i % 2 == 0).unwrap();
            f.set_rollout(&name, 100).unwrap();
            assert_eq!(f.is_enabled(&name, "u", NONE), Ok(true));
        }
    });

    assert_eq!(f.is_enabled("flag-3-10", "u", NONE), Ok(true));
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let f = sut!(create());
        f.define("x", false).unwrap();

        run_concurrently(8, |t| {
            f.add_rule("x", &format!("k{t}"), "v", true).unwrap();
            f.set_rollout("x", 100).unwrap();
        });

        for t in 0..8 {
            assert_eq!(f.is_enabled("x", "u", &[(&format!("k{t}"), "v")]), Ok(true), "run {run} rule {t}");
        }
    }
}

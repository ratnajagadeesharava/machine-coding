#![allow(unused_imports, clippy::too_many_arguments)]
use p126_alert_monitoring::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

const T0: u64 = 1_000_000;

/// Records every delivery as "channel|rule|STATE|SEVERITY|value|time"; channels in `failing` return Err.
struct Recorder {
    log: Mutex<Vec<String>>,
    failing: Mutex<HashSet<String>>,
}

impl Recorder {
    fn new() -> Arc<Recorder> {
        Arc::new(Recorder {
            log: Mutex::new(vec![]),
            failing: Mutex::new(HashSet::new()),
        })
    }
    fn take(&self) -> Vec<String> {
        std::mem::take(&mut *self.log.lock().unwrap())
    }
}

fn fmt(v: f64) -> String {
    if v == v.round() {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

impl Notifier for Recorder {
    fn notify(
        &self,
        channel: &str,
        rule: &str,
        state: &str,
        severity: &str,
        value: f64,
        time_ms: u64,
    ) -> Result<(), String> {
        if self.failing.lock().unwrap().contains(channel) {
            return Err(format!("channel down: {channel}"));
        }
        self.log.lock().unwrap().push(format!(
            "{channel}|{rule}|{state}|{severity}|{}|{time_ms}",
            fmt(value)
        ));
        Ok(())
    }
}

type Mon = Arc<dyn AlertMonitor>;

macro_rules! fx {
    () => {{
        let rec = Recorder::new();
        let clock = FakeClock::new(T0);
        let m = sut!(create(rec.clone(), clock.as_clock()));
        (m, clock, rec)
    }};
}

fn rule(
    m: &Mon,
    id: &str,
    metric: &str,
    cmp: &str,
    th: f64,
    window: i64,
    agg: &str,
    sev: &str,
    f: i64,
    cd: i64,
) {
    m.define_rule(id, metric, cmp, th, window, agg, sev, f, cd)
        .unwrap();
}

fn simple(m: &Mon, id: &str, metric: &str, cmp: &str, th: f64, agg: &str) {
    rule(m, id, metric, cmp, th, 60_000, agg, WARN, 0, 0);
}

fn st(m: &Mon, id: &str) -> String {
    m.state(id).unwrap()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_start_in_ok_with_an_empty_history() {
    let (m, _c, _r) = fx!();
    simple(&m, "r1", "cpu", GT, 80.0, AVG);

    assert_eq!(st(&m, "r1"), "OK");
    assert!(m.history("r1").unwrap().is_empty());
    assert!(m.active_alerts().is_empty());
    assert_eq!(m.suppressed_count("r1"), Ok(0));
}

#[test]
fn should_treat_gt_as_strict_and_gte_as_inclusive() {
    let (m, _c, _r) = fx!();
    simple(&m, "gt", "cpu", GT, 80.0, MAX);
    simple(&m, "gte", "cpu", GTE, 80.0, MAX);
    m.record("cpu", 80.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "gt"), "OK");
    assert_eq!(st(&m, "gte"), "FIRING");
}

#[test]
fn should_treat_lt_as_strict_and_lte_as_inclusive() {
    let (m, _c, _r) = fx!();
    simple(&m, "lt", "mem", LT, 10.0, MIN);
    simple(&m, "lte", "mem", LTE, 10.0, MIN);
    m.record("mem", 10.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "lt"), "OK");
    assert_eq!(st(&m, "lte"), "FIRING");
}

#[test]
fn should_compute_every_aggregation_over_the_window() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    for agg in [AVG, MAX, MIN, SUM, COUNT] {
        simple(&m, agg, "lat", GTE, 0.0, agg);
    }
    for v in [10.0, 20.0, 60.0] {
        m.record("lat", v).unwrap();
    }
    m.evaluate();

    assert_eq!(
        r.take(),
        vec![
            format!("c|AVG|FIRING|WARN|30|{T0}"),
            format!("c|COUNT|FIRING|WARN|3|{T0}"),
            format!("c|MAX|FIRING|WARN|60|{T0}"),
            format!("c|MIN|FIRING|WARN|10|{T0}"),
            format!("c|SUM|FIRING|WARN|90|{T0}"),
        ]
    );
}

#[test]
fn should_use_a_half_open_window_that_excludes_a_sample_exactly_window_ms_old() {
    let (m, c, _r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "q", GT, 0.0, 1_000, COUNT, WARN, 0, 0);
    m.record("q", 1.0).unwrap();
    c.advance(999);
    m.evaluate();
    assert_eq!(st(&m, "r"), "FIRING");

    c.advance(1);
    m.evaluate();
    assert_eq!(st(&m, "r"), "RESOLVED");
}

#[test]
fn should_not_fire_without_samples_except_for_count_which_is_zero() {
    let (m, _c, _r) = fx!();
    simple(&m, "gt", "x", GT, -5.0, AVG);
    simple(&m, "lt", "x", LT, 5.0, MIN);
    simple(&m, "sumLt", "x", LT, 5.0, SUM);
    simple(&m, "cntLt", "x", LT, 1.0, COUNT);
    simple(&m, "cntGt", "x", GT, 0.0, COUNT);
    m.evaluate();

    assert_eq!(st(&m, "gt"), "OK");
    assert_eq!(st(&m, "lt"), "OK");
    assert_eq!(st(&m, "sumLt"), "OK");
    assert_eq!(st(&m, "cntLt"), "FIRING");
    assert_eq!(st(&m, "cntGt"), "OK");
}

#[test]
fn should_fire_immediately_when_for_ms_is_zero() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    simple(&m, "r", "cpu", GT, 80.0, AVG);
    m.record("cpu", 95.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "r"), "FIRING");
    assert_eq!(m.history("r").unwrap(), vec![format!("{T0}:FIRING")]);
    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|95|{T0}")]);
    assert_eq!(m.active_alerts(), strs(&["r"]));
}

#[test]
fn should_stay_pending_until_the_condition_held_for_for_ms() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "cpu", GT, 80.0, 60_000, AVG, WARN, 120_000, 0);
    m.record("cpu", 90.0).unwrap();
    m.evaluate();
    assert_eq!(st(&m, "r"), "PENDING");

    c.advance(60_000);
    m.record("cpu", 95.0).unwrap();
    m.evaluate();
    assert_eq!(st(&m, "r"), "PENDING");
    assert!(m.active_alerts().is_empty());
    assert!(r.take().is_empty());

    c.advance(59_999);
    m.record("cpu", 85.0).unwrap();
    m.evaluate();
    assert_eq!(st(&m, "r"), "PENDING");

    c.advance(1);
    m.evaluate();
    assert_eq!(st(&m, "r"), "FIRING");
    assert_eq!(
        m.history("r").unwrap(),
        vec![format!("{T0}:PENDING"), format!("{}:FIRING", T0 + 120_000)]
    );
    assert_eq!(
        r.take(),
        vec![format!("c|r|FIRING|WARN|85|{}", T0 + 120_000)]
    );
}

#[test]
fn should_return_to_ok_without_notifying_when_the_condition_drops_before_for_ms() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "cpu", GT, 80.0, 10_000, MAX, CRITICAL, 30_000, 0);
    m.record("cpu", 99.0).unwrap();
    m.evaluate();
    c.advance(10_000);
    m.evaluate();

    assert_eq!(st(&m, "r"), "OK");
    assert_eq!(
        m.history("r").unwrap(),
        vec![format!("{T0}:PENDING"), format!("{}:OK", T0 + 10_000)]
    );
    assert!(r.take().is_empty());
}

#[test]
fn should_restart_the_pending_period_after_a_drop_back_to_ok() {
    let (m, c, _r) = fx!();
    rule(&m, "r", "cpu", GT, 80.0, 10_000, MAX, WARN, 20_000, 0);
    m.record("cpu", 99.0).unwrap();
    m.evaluate();
    c.advance(10_000);
    m.evaluate();
    c.advance(5_000);
    m.record("cpu", 99.0).unwrap();
    m.evaluate();
    c.advance(10_000);
    m.record("cpu", 99.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "r"), "PENDING");
    c.advance(10_000);
    m.record("cpu", 99.0).unwrap();
    m.evaluate();
    assert_eq!(st(&m, "r"), "FIRING");
}

#[test]
fn should_notify_firing_only_on_the_transition() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    simple(&m, "r", "cpu", GT, 80.0, MAX);
    m.record("cpu", 90.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.record("cpu", 91.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();

    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|90|{T0}")]);
    assert_eq!(m.history("r").unwrap(), vec![format!("{T0}:FIRING")]);
}

#[test]
fn should_resolve_then_return_to_ok_on_the_next_evaluate_that_stays_false() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "cpu", GT, 80.0, 1_000, MAX, WARN, 0, 0);
    m.record("cpu", 90.0).unwrap();
    m.evaluate();
    r.take();

    c.advance(1_000);
    m.evaluate();
    assert_eq!(st(&m, "r"), "RESOLVED");
    assert!(m.active_alerts().is_empty());
    assert_eq!(
        r.take(),
        vec![format!("c|r|RESOLVED|WARN|0|{}", T0 + 1_000)]
    );

    c.advance(1_000);
    m.evaluate();
    assert_eq!(st(&m, "r"), "OK");
    assert!(r.take().is_empty());
    assert_eq!(
        m.history("r").unwrap(),
        vec![
            format!("{T0}:FIRING"),
            format!("{}:RESOLVED", T0 + 1_000),
            format!("{}:OK", T0 + 2_000)
        ]
    );
}

#[test]
fn should_leave_resolved_directly_for_pending_or_firing_when_the_condition_is_true_again() {
    let (m, c, _r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "imm", "a", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    rule(&m, "slow", "b", GT, 0.0, 1_000, MAX, WARN, 5_000, 0);
    m.record("a", 1.0).unwrap();
    m.record("b", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    assert_eq!(st(&m, "imm"), "RESOLVED");
    m.record("a", 1.0).unwrap();
    m.evaluate();
    assert_eq!(st(&m, "imm"), "FIRING");
    assert_eq!(
        m.history("imm").unwrap(),
        vec![
            format!("{T0}:FIRING"),
            format!("{}:RESOLVED", T0 + 1_000),
            format!("{}:FIRING", T0 + 1_000)
        ]
    );
    assert_eq!(
        m.history("slow").unwrap(),
        vec![format!("{T0}:PENDING"), format!("{}:OK", T0 + 1_000)]
    );
}

#[test]
fn should_suppress_refiring_inside_the_cooldown_and_count_it() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "e", GT, 0.0, 1_000, COUNT, WARN, 0, 10_000);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(1_000);
    m.record("e", 1.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "r"), "FIRING");
    assert_eq!(m.suppressed_count("r"), Ok(1));
    assert_eq!(
        r.take(),
        vec![
            format!("c|r|FIRING|WARN|1|{T0}"),
            format!("c|r|RESOLVED|WARN|0|{}", T0 + 1_000)
        ]
    );
}

#[test]
fn should_notify_again_once_the_cooldown_has_passed_exactly() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "e", GT, 0.0, 1_000, COUNT, WARN, 0, 5_000);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(3_999);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    assert_eq!(m.suppressed_count("r"), Ok(1));

    c.advance(1_000);
    m.evaluate();
    c.advance(1);
    m.evaluate();
    m.record("e", 1.0).unwrap();
    m.evaluate();
    assert_eq!(m.suppressed_count("r"), Ok(1));
    assert!(r
        .take()
        .contains(&format!("c|r|FIRING|WARN|1|{}", T0 + 6_000)));
}

#[test]
fn should_still_notify_resolved_inside_the_cooldown() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "e", GT, 0.0, 1_000, COUNT, WARN, 0, 1_000_000);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(1);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();

    assert_eq!(
        r.take(),
        vec![
            format!("c|r|FIRING|WARN|1|{T0}"),
            format!("c|r|RESOLVED|WARN|0|{}", T0 + 1_000),
            format!("c|r|RESOLVED|WARN|0|{}", T0 + 2_001),
        ]
    );
    assert_eq!(m.suppressed_count("r"), Ok(1));
}

#[test]
fn should_route_by_severity_to_each_channel() {
    let (m, _c, r) = fx!();
    m.register_channel("all", INFO).unwrap();
    m.register_channel("pager", CRITICAL).unwrap();
    m.register_channel("chat", WARN).unwrap();
    rule(&m, "i", "x", GT, 0.0, 1_000, MAX, INFO, 0, 0);
    rule(&m, "w", "x", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    rule(&m, "c", "x", GT, 0.0, 1_000, MAX, CRITICAL, 0, 0);
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(
        r.take(),
        vec![
            format!("all|c|FIRING|CRITICAL|1|{T0}"),
            format!("pager|c|FIRING|CRITICAL|1|{T0}"),
            format!("chat|c|FIRING|CRITICAL|1|{T0}"),
            format!("all|i|FIRING|INFO|1|{T0}"),
            format!("all|w|FIRING|WARN|1|{T0}"),
            format!("chat|w|FIRING|WARN|1|{T0}"),
        ]
    );
}

#[test]
fn should_deliver_in_rule_id_order_and_channel_registration_order() {
    let (m, _c, r) = fx!();
    m.register_channel("zeta", INFO).unwrap();
    m.register_channel("alpha", INFO).unwrap();
    simple(&m, "b", "x", GT, 0.0, MAX);
    simple(&m, "a", "x", GT, 0.0, MAX);
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(
        r.take(),
        vec![
            format!("zeta|a|FIRING|WARN|1|{T0}"),
            format!("alpha|a|FIRING|WARN|1|{T0}"),
            format!("zeta|b|FIRING|WARN|1|{T0}"),
            format!("alpha|b|FIRING|WARN|1|{T0}"),
        ]
    );
}

#[test]
fn should_keep_delivering_when_one_notifier_fails() {
    let (m, _c, r) = fx!();
    m.register_channel("a", INFO).unwrap();
    m.register_channel("broken", INFO).unwrap();
    m.register_channel("z", INFO).unwrap();
    r.failing.lock().unwrap().insert("broken".to_string());
    simple(&m, "r1", "x", GT, 0.0, MAX);
    simple(&m, "r2", "x", GT, 0.0, MAX);
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(
        r.take(),
        vec![
            format!("a|r1|FIRING|WARN|1|{T0}"),
            format!("z|r1|FIRING|WARN|1|{T0}"),
            format!("a|r2|FIRING|WARN|1|{T0}"),
            format!("z|r2|FIRING|WARN|1|{T0}"),
        ]
    );
    assert_eq!(st(&m, "r1"), "FIRING");
    assert_eq!(st(&m, "r2"), "FIRING");
}

#[test]
fn should_start_the_cooldown_even_when_no_channel_received_the_firing_notification() {
    let (m, c, _r) = fx!();
    rule(&m, "r", "e", GT, 0.0, 1_000, COUNT, WARN, 0, 10_000);
    m.record("e", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(1);
    m.record("e", 1.0).unwrap();
    m.evaluate();

    assert_eq!(m.suppressed_count("r"), Ok(1));
}

#[test]
fn should_withhold_notifications_while_silenced_but_keep_tracking_state() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    simple(&m, "r", "x", GT, 0.0, MAX);
    m.silence("r", T0 + 5_000).unwrap();
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "r"), "FIRING");
    assert_eq!(m.active_alerts(), strs(&["r"]));
    assert_eq!(m.history("r").unwrap(), vec![format!("{T0}:FIRING")]);
    assert!(r.take().is_empty());
    assert_eq!(m.suppressed_count("r"), Ok(0));
}

#[test]
fn should_end_the_silence_exactly_at_until_ms() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "x", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    m.silence("r", T0 + 1_000).unwrap();
    m.record("x", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    assert_eq!(
        r.take(),
        vec![format!("c|r|RESOLVED|WARN|0|{}", T0 + 1_000)]
    );

    m.silence("r", T0 + 100_000).unwrap();
    c.advance(1_000);
    m.evaluate();
    m.silence("r", 0).unwrap();
    m.record("x", 1.0).unwrap();
    m.evaluate();
    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|1|{}", T0 + 2_000)]);
}

#[test]
fn should_list_active_alerts_by_severity_descending_then_rule_id() {
    let (m, _c, _r) = fx!();
    rule(&m, "b-warn", "x", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    rule(&m, "a-warn", "x", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    rule(&m, "z-crit", "x", GT, 0.0, 1_000, MAX, CRITICAL, 0, 0);
    rule(&m, "m-info", "x", GT, 0.0, 1_000, MAX, INFO, 0, 0);
    rule(&m, "c-crit", "x", GT, 0.0, 1_000, MAX, CRITICAL, 0, 0);
    rule(&m, "pending", "x", GT, 0.0, 1_000, MAX, CRITICAL, 5_000, 0);
    rule(&m, "quiet", "x", GT, 100.0, 1_000, MAX, CRITICAL, 0, 0);
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(
        m.active_alerts(),
        strs(&["c-crit", "z-crit", "a-warn", "b-warn", "m-info"])
    );
}

#[test]
fn should_keep_samples_inside_the_largest_window_of_the_metrics_rules() {
    let (m, c, _r) = fx!();
    rule(&m, "short", "x", GT, 0.0, 1_000, MAX, WARN, 0, 0);
    rule(&m, "long", "x", GT, 0.0, 5_000, MAX, WARN, 0, 0);
    m.record("x", 1.0).unwrap();
    c.advance(2_000);
    m.record("x", 2.0).unwrap();
    c.advance(2_999);
    m.evaluate();
    assert_eq!(m.sample_count("x"), 2);

    c.advance(1);
    m.evaluate();
    assert_eq!(m.sample_count("x"), 1);

    c.advance(10_000);
    m.evaluate();
    assert_eq!(m.sample_count("x"), 0);
}

#[test]
fn should_count_samples_per_metric_and_report_zero_for_unknown_metrics() {
    let (m, _c, _r) = fx!();
    m.record("a", 1.0).unwrap();
    m.record("a", 2.0).unwrap();
    m.record("b", 3.0).unwrap();

    assert_eq!(m.sample_count("a"), 2);
    assert_eq!(m.sample_count("b"), 1);
    assert_eq!(m.sample_count("never"), 0);
}

#[test]
fn should_evaluate_rules_only_against_their_own_metric() {
    let (m, _c, _r) = fx!();
    simple(&m, "cpuRule", "cpu", GT, 0.0, MAX);
    simple(&m, "memRule", "mem", GT, 0.0, MAX);
    m.record("cpu", 5.0).unwrap();
    m.evaluate();

    assert_eq!(st(&m, "cpuRule"), "FIRING");
    assert_eq!(st(&m, "memRule"), "OK");
}

#[test]
fn should_forget_everything_about_a_deleted_rule() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    simple(&m, "r", "x", GT, 0.0, MAX);
    m.record("x", 1.0).unwrap();
    m.evaluate();
    r.take();

    m.delete_rule("r").unwrap();
    assert!(m.active_alerts().is_empty());
    assert_eq!(m.state("r"), Err(MonitorError::UnknownRule));
    assert_eq!(m.history("r"), Err(MonitorError::UnknownRule));
    assert_eq!(m.delete_rule("r"), Err(MonitorError::UnknownRule));

    simple(&m, "r", "x", GT, 0.0, MAX);
    assert_eq!(st(&m, "r"), "OK");
    assert!(m.history("r").unwrap().is_empty());
    m.evaluate();
    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|1|{T0}")]);
}

#[test]
fn should_reject_invalid_rule_definitions_and_change_nothing() {
    let (m, _c, _r) = fx!();
    simple(&m, "r", "x", GT, 1.0, MAX);
    let bad = Err(MonitorError::InvalidArgument);
    let d = |id: &str,
             metric: &str,
             cmp: &str,
             th: f64,
             w: i64,
             agg: &str,
             sev: &str,
             f: i64,
             cd: i64| { m.define_rule(id, metric, cmp, th, w, agg, sev, f, cd) };

    assert_eq!(
        d("r", "y", LT, 9.0, 60_000, MIN, WARN, 0, 0),
        Err(MonitorError::DuplicateRule)
    );
    assert_eq!(d("a", "x", GT, 1.0, 0, MAX, WARN, 0, 0), bad);
    assert_eq!(d("a", "x", GT, 1.0, -5, MAX, WARN, 0, 0), bad);
    assert_eq!(d("a", "x", GT, 1.0, 10, MAX, WARN, -1, 0), bad);
    assert_eq!(d("a", "x", GT, 1.0, 10, MAX, WARN, 0, -1), bad);
    assert_eq!(d("a", "x", "EQ", 1.0, 10, MAX, WARN, 0, 0), bad);
    assert_eq!(d("a", "x", "gt", 1.0, 10, MAX, WARN, 0, 0), bad);
    assert_eq!(d("a", "x", GT, 1.0, 10, "MEDIAN", WARN, 0, 0), bad);
    assert_eq!(d("a", "x", GT, 1.0, 10, MAX, "FATAL", 0, 0), bad);
    assert_eq!(d("a", "x", GT, f64::NAN, 10, MAX, WARN, 0, 0), bad);
    assert_eq!(d(" ", "x", GT, 1.0, 10, MAX, WARN, 0, 0), bad);
    assert_eq!(d("a", "", GT, 1.0, 10, MAX, WARN, 0, 0), bad);

    assert_eq!(m.state("a"), Err(MonitorError::UnknownRule));
    m.record("x", 5.0).unwrap();
    m.evaluate();
    assert_eq!(m.active_alerts(), strs(&["r"]));
}

#[test]
fn should_reject_invalid_records_channels_and_unknown_rules() {
    let (m, _c, _r) = fx!();
    let bad = Err(MonitorError::InvalidArgument);
    m.register_channel("c", WARN).unwrap();

    assert_eq!(m.record("  ", 1.0), bad);
    assert_eq!(m.record("x", f64::NAN), bad);
    assert_eq!(m.record("x", f64::INFINITY), bad);
    assert_eq!(m.sample_count("x"), 0);

    assert_eq!(
        m.register_channel("c", INFO),
        Err(MonitorError::DuplicateChannel)
    );
    assert_eq!(m.register_channel("", INFO), bad);
    assert_eq!(m.register_channel("d", "LOUD"), bad);

    assert_eq!(m.state("nope"), Err(MonitorError::UnknownRule));
    assert_eq!(m.history("nope"), Err(MonitorError::UnknownRule));
    assert_eq!(m.silence("nope", 5), Err(MonitorError::UnknownRule));
    assert_eq!(m.suppressed_count("nope"), Err(MonitorError::UnknownRule));
    assert_eq!(m.delete_rule("nope"), Err(MonitorError::UnknownRule));
}

#[test]
fn should_stamp_samples_with_the_clock_at_record_time() {
    let (m, c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "x", GT, 0.0, 1_000, SUM, WARN, 0, 0);
    m.record("x", 1.0).unwrap();
    c.advance(500);
    m.record("x", 2.0).unwrap();
    c.advance(600);
    m.evaluate();

    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|2|{}", T0 + 1_100)]);
}

#[test]
fn should_not_lose_samples_when_recorded_from_many_threads() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "hits", GTE, 1.0, 60_000, COUNT, WARN, 0, 0);

    run_concurrently(8, |_t| {
        for _ in 0..500 {
            m.record("hits", 1.0).unwrap();
        }
    });

    assert_eq!(m.sample_count("hits"), 4_000);
    m.evaluate();
    assert_eq!(r.take(), vec![format!("c|r|FIRING|WARN|4000|{T0}")]);
}

#[test]
fn should_stay_consistent_when_recording_while_evaluating() {
    let (m, _c, r) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "hits", GTE, 1.0, 60_000, COUNT, WARN, 0, 0);

    run_concurrently(6, |t| {
        for _ in 0..300 {
            if t == 0 {
                m.evaluate();
            } else {
                m.record("hits", 1.0).unwrap();
            }
        }
    });

    m.evaluate();
    assert_eq!(m.sample_count("hits"), 1_500);
    assert_eq!(st(&m, "r"), "FIRING");
    assert_eq!(r.take().len(), 1);
}

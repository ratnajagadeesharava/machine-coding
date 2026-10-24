#![allow(unused_imports, clippy::too_many_arguments)]
use p126_alert_monitoring::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const T0: u64 = 5_000_000;

struct Log(Mutex<Vec<String>>);

impl Notifier for Log {
    fn notify(
        &self,
        ch: &str,
        rule: &str,
        state: &str,
        _sev: &str,
        _value: f64,
        t: u64,
    ) -> Result<(), String> {
        self.0
            .lock()
            .unwrap()
            .push(format!("{ch}|{rule}|{state}|{t}"));
        Ok(())
    }
}

type Mon = Arc<dyn AlertMonitor>;

macro_rules! fx {
    () => {{
        let log = Arc::new(Log(Mutex::new(vec![])));
        let clock = FakeClock::new(T0);
        let m = sut!(create(log.clone(), clock.as_clock()));
        (m, clock, log)
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

fn lines(l: &Log) -> Vec<String> {
    l.0.lock().unwrap().clone()
}

#[test]
fn should_count_suppressed_refires_across_a_flapping_rule() {
    let (m, c, l) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "flap", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 10_000);
    for _ in 0..10 {
        m.record("x", 1.0).unwrap();
        m.evaluate();
        c.advance(1_000);
        m.evaluate();
        c.advance(1_000);
        m.evaluate();
        c.advance(1_000);
    }
    // The rule enters FIRING at 0, 3, 6, ... 27 s. With a 10 s cooldown the notified ones are 0, 12 and 24 s.
    let all = lines(&l);
    assert_eq!(all.iter().filter(|s| s.contains("|FIRING|")).count(), 3);
    assert_eq!(all.iter().filter(|s| s.contains("|RESOLVED|")).count(), 10);
    assert_eq!(m.suppressed_count("flap"), Ok(7));
    assert_eq!(all[0], format!("c|flap|FIRING|{T0}"));
}

#[test]
fn should_not_spend_the_cooldown_on_a_silenced_firing() {
    let (m, c, l) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 60_000);
    m.silence("r", T0 + 1_000).unwrap();
    m.record("x", 1.0).unwrap();
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    c.advance(1_000);
    m.evaluate();
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(
        lines(&l),
        vec![
            format!("c|r|RESOLVED|{}", T0 + 1_000),
            format!("c|r|FIRING|{}", T0 + 2_000)
        ]
    );
    assert_eq!(m.suppressed_count("r"), Ok(0));
}

#[test]
fn should_never_send_the_firing_notification_late_when_a_silence_ends_while_still_firing() {
    let (m, c, l) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "x", GT, 0.0, 100_000, COUNT, CRITICAL, 0, 0);
    m.silence("r", T0 + 10_000).unwrap();
    m.record("x", 1.0).unwrap();
    m.evaluate();
    c.advance(20_000);
    m.evaluate();
    assert!(lines(&l).is_empty());

    c.advance(100_000);
    m.evaluate();
    assert_eq!(lines(&l), vec![format!("c|r|RESOLVED|{}", T0 + 120_000)]);
}

#[test]
fn should_shrink_retention_when_the_largest_window_rule_is_deleted() {
    let (m, c, _l) = fx!();
    rule(&m, "short", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 0);
    rule(&m, "long", "x", GT, 0.0, 60_000, COUNT, WARN, 0, 0);
    for _ in 0..10 {
        m.record("x", 1.0).unwrap();
        c.advance(1_000);
    }
    m.evaluate();
    assert_eq!(m.sample_count("x"), 10);

    m.delete_rule("long").unwrap();
    m.evaluate();
    assert_eq!(m.sample_count("x"), 0);
}

#[test]
fn should_keep_samples_of_metrics_that_no_rule_watches_and_keep_retention_per_metric() {
    let (m, c, _l) = fx!();
    rule(&m, "r", "watched", GT, 0.0, 1_000, COUNT, WARN, 0, 0);
    m.record("watched", 1.0).unwrap();
    m.record("unwatched", 1.0).unwrap();
    c.advance(5_000);
    m.evaluate();

    assert_eq!(m.sample_count("watched"), 0);
    assert_eq!(m.sample_count("unwatched"), 1);
}

#[test]
fn should_evaluate_a_new_rule_against_samples_recorded_before_it_existed() {
    let (m, c, l) = fx!();
    m.record("x", 10.0).unwrap();
    c.advance(500);
    m.record("x", 30.0).unwrap();
    rule(&m, "late", "x", GTE, 20.0, 1_000, AVG, WARN, 0, 0);
    m.register_channel("c", INFO).unwrap();
    m.evaluate();

    assert_eq!(m.state("late").unwrap(), "FIRING");
    assert_eq!(lines(&l), vec![format!("c|late|FIRING|{}", T0 + 500)]);
}

#[test]
fn should_count_a_sample_recorded_at_the_same_millisecond_as_the_evaluation() {
    let (m, c, _l) = fx!();
    rule(&m, "r", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 0);
    c.advance(7);
    m.record("x", 1.0).unwrap();
    m.evaluate();

    assert_eq!(m.state("r").unwrap(), "FIRING");
}

#[test]
fn should_reset_the_pending_period_when_the_condition_briefly_drops_and_comes_back() {
    let (m, c, _l) = fx!();
    rule(&m, "r", "x", GT, 10.0, 2_000, MAX, WARN, 5_000, 0);
    m.record("x", 50.0).unwrap();
    m.evaluate();
    c.advance(2_000);
    m.evaluate();
    c.advance(1_000);
    m.record("x", 50.0).unwrap();
    m.evaluate();
    c.advance(2_000);
    m.record("x", 50.0).unwrap();
    m.evaluate();
    c.advance(2_000);
    m.record("x", 50.0).unwrap();
    m.evaluate();
    assert_eq!(m.state("r").unwrap(), "PENDING");
    c.advance(1_000);
    m.record("x", 50.0).unwrap();
    m.evaluate();

    assert_eq!(m.state("r").unwrap(), "FIRING");
    assert_eq!(
        m.history("r").unwrap(),
        vec![
            format!("{T0}:PENDING"),
            format!("{}:OK", T0 + 2_000),
            format!("{}:PENDING", T0 + 3_000),
            format!("{}:FIRING", T0 + 8_000)
        ]
    );
}

#[test]
fn should_handle_negative_values_and_fractions() {
    let (m, c, _l) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "cold", "temp", LT, -4.5, 10_000, AVG, INFO, 0, 0);
    m.record("temp", -3.0).unwrap();
    m.record("temp", -6.5).unwrap();
    m.evaluate();
    assert_eq!(m.state("cold").unwrap(), "FIRING");

    m.record("temp", 10.0).unwrap();
    c.advance(1);
    m.evaluate();
    assert_eq!(m.state("cold").unwrap(), "RESOLVED");
}

#[test]
fn should_not_notify_for_a_rule_deleted_while_firing_and_start_fresh_when_redefined() {
    let (m, _c, l) = fx!();
    m.register_channel("c", INFO).unwrap();
    rule(&m, "r", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 600_000);
    m.record("x", 1.0).unwrap();
    m.evaluate();
    l.0.lock().unwrap().clear();

    m.delete_rule("r").unwrap();
    assert!(lines(&l).is_empty());
    rule(&m, "r", "x", GT, 0.0, 1_000, COUNT, WARN, 0, 600_000);
    m.evaluate();

    assert_eq!(lines(&l), vec![format!("c|r|FIRING|{T0}")]);
    assert_eq!(m.suppressed_count("r"), Ok(0));
}

#[test]
fn should_handle_many_rules_and_channels_with_exact_delivery_counts() {
    let (m, _c, l) = fx!();
    m.register_channel("info", INFO).unwrap();
    m.register_channel("warn", WARN).unwrap();
    m.register_channel("crit", CRITICAL).unwrap();
    let sev = [INFO, WARN, CRITICAL];
    for i in 0..300usize {
        rule(
            &m,
            &format!("rule-{i:03}"),
            &format!("m{}", i % 5),
            GT,
            (i % 7) as f64,
            10_000,
            MAX,
            sev[i % 3],
            0,
            0,
        );
    }
    for k in 0..5 {
        m.record(&format!("m{k}"), 3.0).unwrap();
    }
    m.evaluate();

    let (mut firing, mut deliveries) = (0, 0);
    for i in 0..300usize {
        if 3 > i % 7 {
            firing += 1;
            deliveries += 1 + (i % 3);
        }
    }
    let active = m.active_alerts();
    assert_eq!(active.len(), firing);
    assert_eq!(lines(&l).len(), deliveries);
    let sev_of = |id: &String| id[5..].parse::<usize>().unwrap() % 3;
    assert_eq!(sev_of(&active[0]), 2);
    assert_eq!(sev_of(active.last().unwrap()), 0);
    assert!(active
        .windows(2)
        .all(|w| sev_of(&w[0]) > sev_of(&w[1]) || (sev_of(&w[0]) == sev_of(&w[1]) && w[0] < w[1])));
}

#[test]
fn should_retain_every_sample_recorded_concurrently_and_prune_afterwards() {
    let (m, c, _l) = fx!();
    rule(&m, "r", "x", GT, 0.0, 5_000, COUNT, WARN, 0, 0);
    let evals = AtomicUsize::new(0);
    run_concurrently(8, |t| {
        for i in 0..1_000 {
            m.record("x", i as f64).unwrap();
            if t == 7 && i % 100 == 0 {
                m.evaluate();
                evals.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(m.sample_count("x"), 8_000);
    c.advance(5_000);
    m.evaluate();
    assert_eq!(m.sample_count("x"), 0);
    assert_eq!(evals.load(Ordering::SeqCst), 10);
}

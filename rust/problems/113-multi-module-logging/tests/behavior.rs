#![allow(unused_imports, clippy::needless_range_loop)]
use p113_multi_module_logging::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// Recording sink whose outcome the test controls.
#[derive(Default)]
struct FakeSink {
    lines: Mutex<Vec<String>>,
    failing: AtomicBool,
}

impl FakeSink {
    fn lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
}

impl Sink for FakeSink {
    fn write(&self, line: &str) -> bool {
        if self.failing.load(Ordering::SeqCst) {
            return false;
        }
        self.lines.lock().unwrap().push(line.to_string());
        true
    }
}

struct Flaky {
    writes: AtomicUsize,
    inner: Arc<FakeSink>,
}

impl Sink for Flaky {
    fn write(&self, line: &str) -> bool {
        self.writes.fetch_add(1, Ordering::SeqCst) % 3 != 2 && self.inner.write(line)
    }
}

fn setup() -> (Arc<FakeSink>, Arc<FakeSink>) {
    (Arc::new(FakeSink::default()), Arc::new(FakeSink::default()))
}

fn seq_of(line: &str) -> u64 {
    line.splitn(5, '|').next().unwrap().parse().unwrap()
}

fn seqs(lines: &[String]) -> Vec<u64> {
    lines.iter().map(|l| seq_of(l)).collect()
}

fn all(s: &Arc<dyn LogService>) -> Vec<String> {
    s.query(None, None, None, None, None).unwrap()
}

#[test]
fn should_start_at_one_and_increase_by_one() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));

    assert_eq!(s.log("AUTH", "FILE", "INFO", "login"), Ok(1));
    assert_eq!(s.log("PAYMENTS", "TOOL", "WARN", "retry"), Ok(2));
    assert_eq!(s.log("SEARCH", "FILE", "DEBUG", "query"), Ok(3));
    s.close();
}

#[test]
fn should_have_written_the_line_to_the_sink_before_acknowledging() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t, 8));

    assert_eq!(s.log("AUTH", "FILE", "WARN", "bad password"), Ok(1));
    assert_eq!(f.lines(), vec!["1|AUTH|FILE|WARN|bad password"]);
    s.close();
}

#[test]
fn should_route_each_log_to_its_own_destination_only() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t.clone(), 8));
    s.log("AUTH", "FILE", "INFO", "a").unwrap();
    s.log("AUTH", "TOOL", "INFO", "b").unwrap();
    s.log("SEARCH", "FILE", "INFO", "c").unwrap();

    assert_eq!(f.lines(), vec!["1|AUTH|FILE|INFO|a", "3|SEARCH|FILE|INFO|c"]);
    assert_eq!(t.lines(), vec!["2|AUTH|TOOL|INFO|b"]);
    s.close();
}

#[test]
fn should_accept_every_fixed_module_and_level() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    let mut n = 0;
    for m in ["AUTH", "PAYMENTS", "SEARCH"] {
        for l in ["DEBUG", "INFO", "WARN", "ERROR"] {
            n += 1;
            assert_eq!(s.log(m, "FILE", l, "x"), Ok(n));
        }
    }
    s.close();
}

#[test]
fn should_keep_the_activity_text_verbatim() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    s.log("AUTH", "FILE", "INFO", "  a|b | c  ").unwrap();

    assert_eq!(all(&s), vec!["1|AUTH|FILE|INFO|  a|b | c  "]);
    s.close();
}

#[test]
fn should_reject_an_unknown_module() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));

    assert_eq!(
        s.log("BILLING", "FILE", "INFO", "x"),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.log("auth", "FILE", "INFO", "x"),
        Err(LogError::InvalidArgument)
    );
    s.close();
}

#[test]
fn should_reject_unknown_destination_level_and_blank_activity() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));

    assert_eq!(
        s.log("AUTH", "DISK", "INFO", "x"),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.log("AUTH", "FILE", "TRACE", "x"),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.log("AUTH", "FILE", "INFO", " "),
        Err(LogError::InvalidArgument)
    );
    s.close();
}

#[test]
fn should_not_consume_a_sequence_number_or_write_anything_for_invalid_input() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t.clone(), 8));
    assert_eq!(
        s.log("BILLING", "FILE", "INFO", "x"),
        Err(LogError::InvalidArgument)
    );

    assert_eq!(s.log("AUTH", "FILE", "INFO", "first"), Ok(1));
    assert_eq!(f.lines().len(), 1);
    assert!(t.lines().is_empty());
    s.close();
}

#[test]
fn should_report_failure_when_the_sink_write_returns_false() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t, 8));
    f.failing.store(true, Ordering::SeqCst);

    assert_eq!(
        s.log("AUTH", "FILE", "ERROR", "boom"),
        Err(LogError::WriteFailed)
    );
    s.close();
}

#[test]
fn should_not_consume_a_sequence_number_when_the_write_fails() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t, 8));
    assert_eq!(s.log("AUTH", "FILE", "INFO", "one"), Ok(1));
    f.failing.store(true, Ordering::SeqCst);
    assert_eq!(
        s.log("AUTH", "FILE", "INFO", "lost"),
        Err(LogError::WriteFailed)
    );
    assert_eq!(
        s.log("AUTH", "FILE", "INFO", "lost too"),
        Err(LogError::WriteFailed)
    );
    f.failing.store(false, Ordering::SeqCst);

    assert_eq!(s.log("AUTH", "FILE", "INFO", "two"), Ok(2));
    assert_eq!(seqs(&all(&s)), vec![1, 2]);
    s.close();
}

#[test]
fn should_not_store_a_log_whose_write_failed() {
    let (f, t) = setup();
    let s = sut!(create(f, t.clone(), 8));
    t.failing.store(true, Ordering::SeqCst);
    let _ = s.log("PAYMENTS", "TOOL", "ERROR", "never stored");
    s.log("PAYMENTS", "FILE", "ERROR", "stored").unwrap();

    assert_eq!(all(&s), vec!["1|PAYMENTS|FILE|ERROR|stored"]);
    assert!(s
        .query(Some("TOOL"), None, None, None, None)
        .unwrap()
        .is_empty());
    s.close();
}

#[test]
fn should_fail_one_destination_without_affecting_the_other() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t, 8));
    f.failing.store(true, Ordering::SeqCst);

    assert_eq!(s.log("AUTH", "TOOL", "INFO", "ok"), Ok(1));
    assert_eq!(
        s.log("AUTH", "FILE", "INFO", "bad"),
        Err(LogError::WriteFailed)
    );
    assert_eq!(s.log("AUTH", "TOOL", "INFO", "ok again"), Ok(2));
    s.close();
}

#[test]
fn should_return_everything_in_sequence_order_without_filters() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    s.log("AUTH", "FILE", "INFO", "a").unwrap();
    s.log("SEARCH", "TOOL", "WARN", "b").unwrap();
    s.log("PAYMENTS", "FILE", "ERROR", "c").unwrap();

    assert_eq!(
        all(&s),
        vec![
            "1|AUTH|FILE|INFO|a",
            "2|SEARCH|TOOL|WARN|b",
            "3|PAYMENTS|FILE|ERROR|c"
        ]
    );
    s.close();
}

#[test]
fn should_return_an_empty_list_when_nothing_matches() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    assert!(all(&s).is_empty());
    s.log("AUTH", "FILE", "INFO", "a").unwrap();

    assert!(s
        .query(Some("TOOL"), None, None, None, None)
        .unwrap()
        .is_empty());
    assert!(s
        .query(None, None, None, Some(5), Some(9))
        .unwrap()
        .is_empty());
    s.close();
}

#[test]
fn should_filter_by_destination_and_by_module() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    s.log("AUTH", "FILE", "INFO", "a").unwrap();
    s.log("AUTH", "TOOL", "INFO", "b").unwrap();
    s.log("SEARCH", "FILE", "INFO", "c").unwrap();

    let q = |d, m| seqs(&s.query(d, m, None, None, None).unwrap());
    assert_eq!(q(Some("FILE"), None), vec![1, 3]);
    assert_eq!(q(None, Some("AUTH")), vec![1, 2]);
    assert_eq!(q(Some("FILE"), Some("SEARCH")), vec![3]);
    s.close();
}

#[test]
fn should_treat_the_level_filter_as_a_minimum_level() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    for (l, m) in [("DEBUG", "d"), ("INFO", "i"), ("WARN", "w"), ("ERROR", "e")] {
        s.log("AUTH", "FILE", l, m).unwrap();
    }

    let q = |l| seqs(&s.query(None, None, Some(l), None, None).unwrap());
    assert_eq!(q("DEBUG"), vec![1, 2, 3, 4]);
    assert_eq!(q("WARN"), vec![3, 4]);
    assert_eq!(q("ERROR"), vec![4]);
    s.close();
}

#[test]
fn should_apply_an_inclusive_sequence_range() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    for i in 0..6 {
        s.log("AUTH", "FILE", "INFO", &format!("m{i}")).unwrap();
    }

    let q = |a, b| seqs(&s.query(None, None, None, a, b).unwrap());
    assert_eq!(q(Some(2), Some(4)), vec![2, 3, 4]);
    assert_eq!(q(Some(5), None), vec![5, 6]);
    assert_eq!(q(None, Some(2)), vec![1, 2]);
    assert_eq!(q(Some(3), Some(3)), vec![3]);
    s.close();
}

#[test]
fn should_combine_all_filters() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    s.log("AUTH", "FILE", "ERROR", "1").unwrap();
    s.log("PAYMENTS", "FILE", "ERROR", "2").unwrap();
    s.log("PAYMENTS", "TOOL", "ERROR", "3").unwrap();
    s.log("PAYMENTS", "FILE", "INFO", "4").unwrap();
    s.log("PAYMENTS", "FILE", "WARN", "5").unwrap();
    s.log("PAYMENTS", "FILE", "ERROR", "6").unwrap();

    let r = s
        .query(Some("FILE"), Some("PAYMENTS"), Some("WARN"), Some(2), Some(5))
        .unwrap();
    assert_eq!(seqs(&r), vec![2, 5]);
    s.close();
}

#[test]
fn should_reject_invalid_filters_and_a_reversed_range() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));

    assert_eq!(
        s.query(Some("DISK"), None, None, None, None),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.query(None, Some("BILLING"), None, None, None),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.query(None, None, Some("TRACE"), None, None),
        Err(LogError::InvalidArgument)
    );
    assert_eq!(
        s.query(None, None, None, Some(5), Some(4)),
        Err(LogError::InvalidArgument)
    );
    s.close();
}

#[test]
fn should_return_a_snapshot_the_caller_owns() {
    let (f, t) = setup();
    let s = sut!(create(f, t, 8));
    s.log("AUTH", "FILE", "INFO", "a").unwrap();
    let mut first = all(&s);
    s.log("AUTH", "FILE", "INFO", "b").unwrap();

    assert_eq!(first.len(), 1);
    first.clear();
    assert_eq!(all(&s).len(), 2);
    s.close();
}

#[test]
fn should_reject_logs_after_close_but_keep_answering_queries() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t, 8));
    s.log("AUTH", "FILE", "INFO", "a").unwrap();

    s.close();
    s.close();

    assert_eq!(
        s.log("AUTH", "FILE", "INFO", "late"),
        Err(LogError::Closed)
    );
    assert_eq!(all(&s).len(), 1);
    assert_eq!(f.lines().len(), 1);
}

#[test]
fn should_give_every_concurrent_log_a_unique_gap_free_number_and_lose_nothing() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t.clone(), 8));
    let modules = ["AUTH", "PAYMENTS", "SEARCH"];
    let dests = ["FILE", "TOOL"];

    let per_thread = collect_concurrently(16, |th| {
        (0..500)
            .map(|i| {
                s.log(
                    modules[(th + i) % 3],
                    dests[i % 2],
                    "INFO",
                    &format!("t{th}-{i}"),
                )
                .unwrap()
            })
            .collect::<Vec<u64>>()
    });

    let got: HashSet<u64> = per_thread.into_iter().flatten().collect();
    assert_eq!(got.len(), 8_000);
    assert!(got.iter().all(|n| (1..=8_000).contains(n)));
    assert_eq!(f.lines().len() + t.lines().len(), 8_000);
    let q = seqs(&all(&s));
    assert_eq!(q.len(), 8_000);
    assert!(q.windows(2).all(|w| w[0] < w[1]));
    s.close();
}

#[test]
fn should_deliver_lines_to_each_sink_in_increasing_sequence_order_under_concurrency() {
    let (f, t) = setup();
    let s = sut!(create(f.clone(), t.clone(), 8));

    run_concurrently(8, |_| {
        for i in 0..300 {
            s.log("SEARCH", if i % 2 == 0 { "FILE" } else { "TOOL" }, "DEBUG", "x")
                .unwrap();
        }
    });

    for sink in [&f, &t] {
        let q = seqs(&sink.lines());
        assert!(q.windows(2).all(|w| w[0] < w[1]));
    }
    s.close();
}

#[test]
fn should_stay_gap_free_over_acknowledged_logs_when_writes_fail_under_concurrency() {
    let (f, t) = setup();
    let flaky = Arc::new(Flaky {
        writes: AtomicUsize::new(0),
        inner: f.clone(),
    });
    let s = sut!(create(flaky, t, 4));

    let results = collect_concurrently(8, |_| {
        (0..200)
            .map(|_| s.log("AUTH", "FILE", "INFO", "x"))
            .collect::<Vec<_>>()
    });

    let mut acked: Vec<u64> = results
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();
    acked.sort_unstable();
    assert!(!acked.is_empty());
    for (i, n) in acked.iter().enumerate() {
        assert_eq!(*n, i as u64 + 1);
    }
    assert_eq!(all(&s).len(), acked.len());
    assert_eq!(f.lines().len(), acked.len());
    s.close();
}

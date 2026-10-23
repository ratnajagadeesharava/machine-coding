#![allow(unused_imports, clippy::needless_range_loop)]
use p113_multi_module_logging::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

const MODULES: [&str; 3] = ["AUTH", "PAYMENTS", "SEARCH"];
const LEVELS: [&str; 4] = ["DEBUG", "INFO", "WARN", "ERROR"];

#[derive(Default)]
struct Rec(Mutex<Vec<String>>);

impl Rec {
    fn count(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

impl Sink for Rec {
    fn write(&self, line: &str) -> bool {
        self.0.lock().unwrap().push(line.to_string());
        true
    }
}

/// Fails according to a call-count predicate, otherwise records.
struct Scripted {
    calls: AtomicUsize,
    fail: fn(usize) -> bool,
    rec: Rec,
}

impl Scripted {
    fn new(fail: fn(usize) -> bool) -> Arc<Self> {
        Arc::new(Scripted {
            calls: AtomicUsize::new(0),
            fail,
            rec: Rec::default(),
        })
    }
}

impl Sink for Scripted {
    fn write(&self, line: &str) -> bool {
        let c = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        !(self.fail)(c) && self.rec.write(line)
    }
}

/// Blocks every write until `open` is called.
struct Gate {
    open: Mutex<bool>,
    cv: Condvar,
    rec: Rec,
}

impl Sink for Gate {
    fn write(&self, line: &str) -> bool {
        let mut g = self.open.lock().unwrap();
        while !*g {
            g = self.cv.wait(g).unwrap();
        }
        drop(g);
        self.rec.write(line)
    }
}

fn seq_of(line: &str) -> u64 {
    line.splitn(5, '|').next().unwrap().parse().unwrap()
}

fn rank(l: &str) -> usize {
    LEVELS.iter().position(|x| *x == l).unwrap()
}

#[test]
fn should_lose_nothing_with_32_threads_through_a_tiny_buffer() {
    let (f, t) = (Arc::new(Rec::default()), Arc::new(Rec::default()));
    let s = sut!(create(f.clone(), t.clone(), 1));

    let res = collect_concurrently(32, |th| {
        (0..1_000)
            .map(|i| {
                s.log(
                    MODULES[i % 3],
                    if i % 2 == 0 { "FILE" } else { "TOOL" },
                    LEVELS[i % 4],
                    &format!("t{th}"),
                )
                .unwrap()
            })
            .collect::<Vec<_>>()
    });
    s.close();

    let all: HashSet<u64> = res.into_iter().flatten().collect();
    assert_eq!(all.len(), 32_000);
    assert_eq!(*all.iter().min().unwrap(), 1);
    assert_eq!(*all.iter().max().unwrap(), 32_000);
    assert_eq!(f.count() + t.count(), 32_000);
    assert_eq!(s.query(None, None, None, None, None).unwrap().len(), 32_000);
}

#[test]
fn should_make_an_acknowledged_log_immediately_visible_to_its_own_producer() {
    let s = sut!(create(Arc::new(Rec::default()), Arc::new(Rec::default()), 4));

    run_concurrently(8, |t| {
        for i in 0..500 {
            let m = MODULES[t % 3];
            let seq = s.log(m, "FILE", "WARN", &format!("t{t}-{i}")).unwrap();
            let one = s.query(None, None, None, Some(seq), Some(seq)).unwrap();
            assert_eq!(one, vec![format!("{seq}|{m}|FILE|WARN|t{t}-{i}")]);
        }
    });
    s.close();
}

#[test]
fn should_return_ordered_duplicate_free_snapshots_while_producers_are_running() {
    let s = sut!(create(Arc::new(Rec::default()), Arc::new(Rec::default()), 8));

    run_concurrently(9, |t| {
        if t == 8 {
            for _ in 0..300 {
                let snap = s.query(None, None, None, None, None).unwrap();
                let mut prev = 0;
                for line in &snap {
                    let q = seq_of(line);
                    assert!(q > prev);
                    prev = q;
                }
                assert_eq!(prev as usize, snap.len());
            }
            return;
        }
        for i in 0..400 {
            s.log("AUTH", if i % 2 == 0 { "FILE" } else { "TOOL" }, "INFO", "x")
                .unwrap();
        }
    });
    s.close();
}

#[test]
fn should_keep_acknowledged_numbers_gap_free_when_one_destination_fails_intermittently() {
    let file = Arc::new(Rec::default());
    let tool = Scripted::new(|c| c % 4 == 0);
    let s = sut!(create(file.clone(), tool.clone(), 2));

    let res = collect_concurrently(12, |_| {
        (0..400)
            .map(|i| {
                s.log(
                    "PAYMENTS",
                    if i % 2 == 0 { "FILE" } else { "TOOL" },
                    "ERROR",
                    "x",
                )
            })
            .collect::<Vec<_>>()
    });
    s.close();

    let mut acked = Vec::new();
    let mut failures = 0;
    for r in res.into_iter().flatten() {
        match r {
            Ok(n) => acked.push(n),
            Err(LogError::WriteFailed) => failures += 1,
            Err(e) => panic!("unexpected {e:?}"),
        }
    }
    acked.sort_unstable();
    for (i, n) in acked.iter().enumerate() {
        assert_eq!(*n, i as u64 + 1);
    }
    assert_eq!(failures, tool.calls.load(Ordering::SeqCst) / 4);
    assert_eq!(file.count() + tool.rec.count(), acked.len());
    assert_eq!(
        s.query(None, None, None, None, None).unwrap().len(),
        acked.len()
    );
}

#[test]
fn should_apply_back_pressure_with_a_slow_sink_and_still_finish_everything() {
    let gate = Arc::new(Gate {
        open: Mutex::new(false),
        cv: Condvar::new(),
        rec: Rec::default(),
    });
    let s = sut!(create(gate.clone(), Arc::new(Rec::default()), 1));

    let res = collect_concurrently(11, |t| {
        if t == 10 {
            *gate.open.lock().unwrap() = true;
            gate.cv.notify_all();
            return Vec::new();
        }
        (0..20)
            .map(|_| s.log("SEARCH", "FILE", "INFO", "x").unwrap())
            .collect()
    });
    s.close();

    let all: HashSet<u64> = res.into_iter().flatten().collect();
    assert_eq!(all.len(), 200);
    assert_eq!(gate.rec.count(), 200);
}

#[test]
fn should_account_for_every_log_when_closed_while_producers_are_running() {
    let (f, t) = (Arc::new(Rec::default()), Arc::new(Rec::default()));
    let s = sut!(create(f.clone(), t.clone(), 4));
    let acked = AtomicUsize::new(0);

    run_concurrently(9, |th| {
        if th == 8 {
            while acked.load(Ordering::SeqCst) < 200 {
                std::hint::spin_loop();
            }
            s.close();
            return;
        }
        for i in 0..2_000 {
            match s.log("AUTH", if i % 2 == 0 { "FILE" } else { "TOOL" }, "INFO", "x") {
                Ok(n) => {
                    assert!(n > 0);
                    acked.fetch_add(1, Ordering::SeqCst);
                }
                Err(LogError::Closed) => return,
                Err(e) => panic!("unexpected {e:?}"),
            }
        }
    });

    let n = acked.load(Ordering::SeqCst);
    assert_eq!(f.count() + t.count(), n);
    assert_eq!(s.query(None, None, None, None, None).unwrap().len(), n);
    assert_eq!(s.log("AUTH", "FILE", "INFO", "late"), Err(LogError::Closed));
}

#[test]
fn should_match_a_brute_force_model_for_every_combination_of_filters() {
    let s = sut!(create(Arc::new(Rec::default()), Arc::new(Rec::default()), 3));
    let mut model = Vec::new();
    for i in 0..240usize {
        let (m, d, l) = (
            MODULES[i % 3],
            if (i / 3) % 2 == 0 { "FILE" } else { "TOOL" },
            LEVELS[(i / 6) % 4],
        );
        let seq = s.log(m, d, l, &format!("n{i}")).unwrap();
        model.push(format!("{seq}|{m}|{d}|{l}|n{i}"));
    }

    let dests = [None, Some("FILE"), Some("TOOL")];
    let mods = [None, Some("AUTH"), Some("PAYMENTS"), Some("SEARCH")];
    let lvls = [None, Some("DEBUG"), Some("INFO"), Some("WARN"), Some("ERROR")];
    for d in dests {
        for m in mods {
            for l in lvls {
                let expected: Vec<String> = model
                    .iter()
                    .filter(|line| {
                        let p: Vec<&str> = line.splitn(5, '|').collect();
                        let seq: u64 = p[0].parse().unwrap();
                        (17..=201).contains(&seq)
                            && d.map_or(true, |x| x == p[2])
                            && m.map_or(true, |x| x == p[1])
                            && l.map_or(true, |x| rank(p[3]) >= rank(x))
                    })
                    .cloned()
                    .collect();
                assert_eq!(
                    s.query(d, m, l, Some(17), Some(201)).unwrap(),
                    expected,
                    "{d:?} {m:?} {l:?}"
                );
            }
        }
    }
    s.close();
}

#[test]
fn should_not_reuse_or_skip_numbers_across_long_runs_of_consecutive_failures() {
    let burst = Scripted::new(|c| (5..=50).contains(&c));
    let s = sut!(create(burst.clone(), Arc::new(Rec::default()), 2));

    let out: Vec<Result<u64, LogError>> = (0..60)
        .map(|_| s.log("AUTH", "FILE", "INFO", "x"))
        .collect();
    s.close();

    let mut expected: Vec<Result<u64, LogError>> = (1..=4).map(Ok).collect();
    expected.extend((0..46).map(|_| Err(LogError::WriteFailed)));
    expected.extend((5..=14).map(Ok));
    assert_eq!(out, expected);
    assert_eq!(burst.rec.count(), 14);
}

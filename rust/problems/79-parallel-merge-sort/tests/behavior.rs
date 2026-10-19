#![allow(unused_imports, clippy::needless_range_loop)]
use p79_parallel_merge_sort::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

/// Tests that count live threads must not overlap with other tests that spawn threads, so every test takes this.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (seed + 1).wrapping_mul(0xD1B5_4A32_D192_ED03))
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn random(n: usize, seed: u64) -> Vec<i32> {
    let mut r = Rng::new(seed);
    (0..n).map(|_| r.next() as i32).collect()
}

fn assert_sorts(s: &Arc<dyn ParallelSorter>, input: &[i32], parallelism: usize) {
    let mut expected = input.to_vec();
    expected.sort();
    let mut actual = input.to_vec();

    s.sort(&mut actual, parallelism).unwrap();

    assert!(actual == expected, "n={} parallelism={parallelism}", input.len());
}

/// Live threads of this process (Linux only).
fn thread_count() -> Option<usize> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|l| l.strip_prefix("Threads:"))
        .and_then(|v| v.trim().parse().ok())
}

/// Peak number of live threads while a big sort runs, minus the baseline. `None` if it cannot be measured.
fn max_extra_threads_while_sorting(s: &Arc<dyn ParallelSorter>, parallelism: usize) -> Option<usize> {
    thread_count()?;
    let mut big = random(1_500_000, 5);
    let stop = Arc::new(AtomicBool::new(false));
    let peak = Arc::new(AtomicUsize::new(0));
    let monitor = {
        let (stop, peak) = (stop.clone(), peak.clone());
        thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                if let Some(n) = thread_count() {
                    peak.fetch_max(n, Ordering::SeqCst);
                }
                thread::sleep(Duration::from_micros(200));
            }
        })
    };
    thread::sleep(Duration::from_millis(50));
    let baseline = thread_count()?;
    peak.store(baseline, Ordering::SeqCst);

    s.sort(&mut big, parallelism).unwrap();

    stop.store(true, Ordering::SeqCst);
    monitor.join().unwrap();
    Some(peak.load(Ordering::SeqCst).saturating_sub(baseline))
}

#[test]
fn should_handle_tiny_inputs() {
    let _g = serial();
    let s = sut!(create());

    for p in [1, 2, 8] {
        assert_sorts(&s, &[], p);
        assert_sorts(&s, &[42], p);
        assert_sorts(&s, &[2, 1], p);
        assert_sorts(&s, &[3, 1, 2], p);
    }
}

#[test]
fn should_sort_special_shapes() {
    let _g = serial();
    let s = sut!(create());
    let n = 50_000;
    let sorted: Vec<i32> = (0..n as i32).collect();
    let reversed: Vec<i32> = (0..n as i32).rev().collect();
    let equal = vec![5; n];
    let mut r = Rng::new(7);
    let few_distinct: Vec<i32> = (0..n).map(|_| (r.next() % 4) as i32).collect();

    for shape in [&sorted, &reversed, &equal, &few_distinct] {
        assert_sorts(&s, shape, 4);
    }
}

#[test]
fn should_handle_extreme_values_without_overflow() {
    let _g = serial();
    let s = sut!(create());
    let a = [i32::MAX, i32::MIN, 0, -1, 1, i32::MAX, i32::MIN];

    assert_sorts(&s, &a, 1);
    assert_sorts(&s, &a, 4);
}

#[test]
fn should_sort_random_arrays_of_many_sizes_and_parallelisms() {
    let _g = serial();
    let s = sut!(create());

    for n in [10usize, 1_000, 100_003, 400_000] {
        for p in [1, 2, 4, 8] {
            assert_sorts(&s, &random(n, n as u64 * 31 + p as u64), p);
        }
    }
}

#[test]
fn should_accept_more_threads_than_elements() {
    let _g = serial();
    let s = sut!(create());

    assert_sorts(&s, &random(5, 1), 64);
    assert_sorts(&s, &random(100, 2), 1_000);
}

#[test]
fn should_reject_zero_parallelism() {
    let _g = serial();
    let s = sut!(create());

    assert_eq!(s.sort(&mut [2, 1], 0), Err(InvalidParallelism));
}

#[test]
fn should_be_usable_from_several_threads_at_once() {
    let _g = serial();
    let s = sut!(create());

    run_concurrently(6, |t| assert_sorts(&s, &random(200_000, 100 + t as u64), 4));
}

#[test]
fn should_not_spawn_more_threads_than_allowed() {
    let _g = serial();
    let s = sut!(create());
    s.sort(&mut random(10_000, 1), 2).unwrap();

    let Some(one) = max_extra_threads_while_sorting(&s, 1) else {
        eprintln!("SKIPPED: cannot count threads on this platform");
        return;
    };
    assert!(one <= 1, "parallelism 1 needs no (or one) helper, saw {one} extra threads");
    let four = max_extra_threads_while_sorting(&s, 4).unwrap();
    assert!(four <= 4 + 1, "a thread per split would explode here, saw {four} extra threads");
}

#[test]
fn should_really_run_in_parallel() {
    let _g = serial();
    let s = sut!(create());
    s.sort(&mut random(10_000, 1), 2).unwrap();

    let Some(extra) = max_extra_threads_while_sorting(&s, 4) else {
        eprintln!("SKIPPED: cannot count threads on this platform");
        return;
    };

    assert!(extra >= 1, "no helper thread was ever seen: the sort looks sequential");
}

#[test]
fn should_finish_a_large_sort_quickly() {
    let _g = serial();
    let s = sut!(create());
    let mut big = random(1_000_000, 99);

    let start = Instant::now();
    s.sort(&mut big, 4).unwrap();

    assert!(start.elapsed() < Duration::from_secs(60), "too slow");
    assert!(big.windows(2).all(|w| w[0] <= w[1]), "not sorted");
}

#[test]
fn should_stay_correct_over_100_runs() {
    let _g = serial();
    let s = sut!(create());
    let mut r = Rng::new(2024);
    for run in 0..100u64 {
        let n = (r.next() % 20_000) as usize;
        let p = 1 + (r.next() % 8) as usize;
        assert_sorts(&s, &random(n, run), p);
    }
}

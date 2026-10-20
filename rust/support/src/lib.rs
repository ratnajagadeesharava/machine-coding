//! Shared infrastructure for the playground problems. Not part of any design.

pub mod sim;
pub use sim::{MsgFn, Rng, SendFn, SimNet, TickFn};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Time source. Problems that depend on time take an `Arc<dyn Clock>`.
pub trait Clock: Send + Sync {
    fn now_millis(&self) -> u64;
}

/// Wall-clock time.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Manually advanced clock. Clones share the same time.
#[derive(Clone, Default)]
pub struct FakeClock(Arc<AtomicU64>);

impl FakeClock {
    pub fn new(start_millis: u64) -> Self {
        FakeClock(Arc::new(AtomicU64::new(start_millis)))
    }

    pub fn advance(&self, millis: u64) {
        self.0.fetch_add(millis, Ordering::SeqCst);
    }

    pub fn set(&self, millis: u64) {
        self.0.store(millis, Ordering::SeqCst);
    }

    /// A shareable `Arc<dyn Clock>` view of this clock (shares the same time).
    pub fn as_clock(&self) -> Arc<dyn Clock> {
        Arc::new(self.clone())
    }
}

impl Clock for FakeClock {
    fn now_millis(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// `STRICT=1` turns "not implemented" into a failure.
pub fn strict() -> bool {
    std::env::var("STRICT").map(|v| v == "1").unwrap_or(false)
}

/// Called by `sut!` when `create` still returns `None`.
pub fn skipped() {
    if strict() {
        panic!("Not implemented (STRICT=1): wire your design in `create` (src/lib.rs)");
    }
    eprintln!("SKIPPED: not implemented yet - wire your design in `create` (src/lib.rs)");
}

/// `let mut api = sut!(create(..));` - returns early (skipped) while `create` returns `None`.
#[macro_export]
macro_rules! sut {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => {
                $crate::skipped();
                return;
            }
        }
    };
}

/// Aborts the whole test process with a clear message if it is still alive after `timeout_ms`.
/// Use it around code that could deadlock, so a hang becomes a failure instead of a frozen `cargo test`.
/// Dropping the guard disarms it.
pub struct Watchdog(Arc<std::sync::atomic::AtomicBool>);

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

pub fn watchdog(timeout_ms: u64) -> Watchdog {
    let done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = done.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        while Instant::now() < deadline {
            if flag.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        if !flag.load(Ordering::SeqCst) {
            eprintln!(
                "\nTIMEOUT after {timeout_ms}ms: deadlock or livelock? Aborting the test run."
            );
            std::process::exit(101);
        }
    });
    Watchdog(done)
}

/// Runs `body(i)` on `threads` threads that all start at the same instant. Panics propagate.
/// A deadlock is turned into a failure after 60 seconds (see [`watchdog`]).
pub fn run_concurrently<F>(threads: usize, body: F)
where
    F: Fn(usize) + Sync,
{
    collect_concurrently(threads, body);
}

/// Like `run_concurrently` but collects each thread's return value (in thread-index order).
pub fn collect_concurrently<T, F>(threads: usize, body: F) -> Vec<T>
where
    T: Send,
    F: Fn(usize) -> T + Sync,
{
    let _guard = watchdog(60_000);
    let barrier = Barrier::new(threads);
    std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|i| {
                let barrier = &barrier;
                let body = &body;
                s.spawn(move || {
                    barrier.wait();
                    body(i)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("worker thread panicked"))
            .collect()
    })
}

/// Polls `cond` until it is true or the timeout elapses.
pub fn eventually<F: Fn() -> bool>(timeout_ms: u64, cond: F) -> bool {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    cond()
}

/// True if `cond` holds for the whole window (e.g. "this thread is still blocked").
pub fn stays<F: Fn() -> bool>(window_ms: u64, cond: F) -> bool {
    let deadline = Instant::now() + Duration::from_millis(window_ms);
    while Instant::now() < deadline {
        if !cond() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    true
}

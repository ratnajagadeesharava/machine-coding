#![allow(unused_imports, clippy::needless_range_loop)]
use p75_bank_transfer::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn acc(i: usize) -> String {
    format!("acc-{i}")
}

fn bank_with(b: Arc<dyn Bank>, accounts: usize, each: u64) -> Arc<dyn Bank> {
    for i in 0..accounts {
        b.open(&acc(i), each).unwrap();
    }
    b
}

/// Counts a mover as finished even if it panics, so the auditor never spins forever.
struct Finished<'a>(&'a AtomicUsize);
impl Drop for Finished<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Tiny xorshift so the tests need no external crates.
struct Rng(u64);
impl Rng {
    fn new(seed: usize) -> Self {
        Rng(0x9E37_79B9_7F4A_7C15 ^ (seed as u64 + 1).wrapping_mul(0xD1B5_4A32_D192_ED03))
    }
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

#[test]
fn should_move_money_between_accounts() {
    let b = bank_with(sut!(create()), 2, 100);

    let ok = b.transfer(&acc(0), &acc(1), 30);

    assert_eq!(ok, Ok(true));
    assert_eq!(b.balance(&acc(0)), Ok(70));
    assert_eq!(b.balance(&acc(1)), Ok(130));
    assert_eq!(b.total(), 200);
}

#[test]
fn should_refuse_when_funds_are_insufficient_and_change_nothing() {
    let b = bank_with(sut!(create()), 2, 100);

    assert_eq!(b.transfer(&acc(0), &acc(1), 101), Ok(false));

    assert_eq!(b.balance(&acc(0)), Ok(100));
    assert_eq!(b.balance(&acc(1)), Ok(100));
}

#[test]
fn should_allow_transfer_of_the_whole_balance() {
    let b = bank_with(sut!(create()), 2, 100);

    assert_eq!(b.transfer(&acc(0), &acc(1), 100), Ok(true));

    assert_eq!(b.balance(&acc(0)), Ok(0));
    assert_eq!(b.balance(&acc(1)), Ok(200));
}

#[test]
fn should_reject_invalid_requests() {
    let b = bank_with(sut!(create()), 2, 100);

    assert_eq!(
        b.transfer(&acc(0), "ghost", 1),
        Err(BankError::UnknownAccount)
    );
    assert_eq!(
        b.transfer("ghost", &acc(0), 1),
        Err(BankError::UnknownAccount)
    );
    assert_eq!(
        b.transfer(&acc(0), &acc(0), 1),
        Err(BankError::InvalidRequest)
    );
    assert_eq!(
        b.transfer(&acc(0), &acc(1), 0),
        Err(BankError::InvalidRequest)
    );
    assert_eq!(b.balance("ghost"), Err(BankError::UnknownAccount));
    assert_eq!(b.open(&acc(0), 5), Err(BankError::AccountExists));
    assert_eq!(b.total(), 200);
}

#[test]
fn should_not_deadlock_when_two_accounts_transfer_to_each_other() {
    let b = bank_with(sut!(create()), 2, 1_000_000);

    run_concurrently(4, |i| {
        let (from, to) = (acc(i % 2), acc((i + 1) % 2));
        for _ in 0..20_000 {
            b.transfer(&from, &to, 1).unwrap();
        }
    });

    assert_eq!(b.balance(&acc(0)), Ok(1_000_000));
    assert_eq!(b.balance(&acc(1)), Ok(1_000_000));
}

#[test]
fn should_not_deadlock_when_transfers_form_a_cycle() {
    let n = 8;
    let b = bank_with(sut!(create()), n, 100_000);

    run_concurrently(n, |i| {
        for _ in 0..5_000 {
            b.transfer(&acc(i), &acc((i + 1) % n), 1).unwrap();
        }
    });

    assert_eq!(b.total(), n as u64 * 100_000);
    for i in 0..n {
        assert_eq!(b.balance(&acc(i)), Ok(100_000));
    }
}

#[test]
fn should_accept_exactly_as_many_withdrawals_as_the_balance_allows() {
    let b = sut!(create());
    b.open("source", 500).unwrap();
    b.open("sink", 0).unwrap();

    let results = collect_concurrently(100, |_| b.transfer("source", "sink", 10).unwrap());

    assert_eq!(results.iter().filter(|r| **r).count(), 50);
    assert_eq!(b.balance("source"), Ok(0));
    assert_eq!(b.balance("sink"), Ok(500));
}

#[test]
fn should_conserve_money_under_random_load() {
    let accounts = 20;
    let b = bank_with(sut!(create()), accounts, 1_000);
    let succeeded = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        let mut rnd = Rng::new(t);
        for _ in 0..10_000 {
            let from = rnd.next(accounts);
            let to = (from + 1 + rnd.next(accounts - 1)) % accounts;
            if b.transfer(&acc(from), &acc(to), 1 + rnd.next(50) as u64)
                .unwrap()
            {
                succeeded.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(b.total(), accounts as u64 * 1_000);
    assert!(succeeded.load(Ordering::SeqCst) > 0);
}

#[test]
fn should_never_show_money_in_flight_to_an_auditor() {
    let accounts = 10;
    let expected = accounts as u64 * 1_000;
    let b = bank_with(sut!(create()), accounts, 1_000);
    let movers_left = AtomicUsize::new(6);
    let wrong_totals = AtomicU64::new(0);
    let audits = AtomicU64::new(0);

    run_concurrently(7, |t| {
        if t == 6 {
            while movers_left.load(Ordering::SeqCst) > 0 {
                if b.total() != expected {
                    wrong_totals.fetch_add(1, Ordering::SeqCst);
                }
                audits.fetch_add(1, Ordering::SeqCst);
            }
            return;
        }
        let _finished = Finished(&movers_left);
        let mut rnd = Rng::new(t);
        for _ in 0..20_000 {
            let from = rnd.next(accounts);
            let to = (from + 1 + rnd.next(accounts - 1)) % accounts;
            b.transfer(&acc(from), &acc(to), 1 + rnd.next(100) as u64)
                .unwrap();
        }
    });

    assert!(
        audits.load(Ordering::SeqCst) > 0,
        "auditor must make progress while transfers run"
    );
    assert_eq!(
        wrong_totals.load(Ordering::SeqCst),
        0,
        "total() observed a half-applied transfer"
    );
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let n = 4;
        let b = bank_with(sut!(create()), n, 1_000);

        run_concurrently(n * 2, |i| {
            for _ in 0..300 {
                b.transfer(&acc(i % n), &acc((i + 1) % n), 1).unwrap();
                b.transfer(&acc((i + 1) % n), &acc(i % n), 1).unwrap();
            }
        });

        assert_eq!(b.total(), n as u64 * 1_000, "run {run}");
    }
}

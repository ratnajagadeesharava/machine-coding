#![allow(unused_imports, clippy::needless_range_loop)]
use p75_bank_transfer::{contract::*, create};
use playground_support::{run_concurrently, sut};

#[test]
fn should_handle_one_hot_account_receiving_from_everyone() {
    let b = sut!(create());
    let senders = 32;
    b.open("hot", 0).unwrap();
    for i in 0..senders {
        b.open(&format!("s{i}"), 10_000).unwrap();
    }

    run_concurrently(senders, |i| {
        let name = format!("s{i}");
        for _ in 0..2_000 {
            b.transfer(&name, "hot", 1).unwrap();
        }
    });

    assert_eq!(b.balance("hot"), Ok(senders as u64 * 2_000));
    assert_eq!(b.total(), senders as u64 * 10_000);
}

#[test]
fn should_keep_total_with_transfers_both_ways_between_many_pairs() {
    let b = sut!(create());
    let n = 50usize;
    for i in 0..n {
        b.open(&format!("a{i}"), 5_000).unwrap();
    }

    run_concurrently(24, |t| {
        let mut state = (t as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        for _ in 0..20_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let x = (state % n as u64) as usize;
            let y = (x + 1 + (state >> 20) as usize % (n - 1)) % n;
            b.transfer(&format!("a{x}"), &format!("a{y}"), 1 + (state >> 40) % 20).unwrap();
        }
    });

    assert_eq!(b.total(), n as u64 * 5_000);
}

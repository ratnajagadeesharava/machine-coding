#![allow(unused_imports, clippy::needless_range_loop)]
use p79_parallel_merge_sort::{contract::*, create};
use playground_support::sut;
use std::time::Instant;

fn random(n: usize, seed: u64) -> Vec<i32> {
    let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    (0..n)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s as i32
        })
        .collect()
}

#[test]
fn should_sort_five_million_elements() {
    let s = sut!(create());
    let mut a = random(5_000_000, 1);
    let mut expected = a.clone();
    expected.sort();

    s.sort(&mut a, 8).unwrap();

    assert!(a == expected);
}

#[test]
fn should_not_be_much_slower_than_the_sequential_sort() {
    let s = sut!(create());
    let base = random(2_000_000, 2);
    let mut seq = base.clone();
    let mut par = base.clone();

    let t0 = Instant::now();
    s.sort(&mut seq, 1).unwrap();
    let sequential = t0.elapsed();
    let t1 = Instant::now();
    s.sort(&mut par, 4).unwrap();
    let parallel = t1.elapsed();

    assert!(par == seq);
    assert!(parallel < sequential * 3, "parallel {parallel:?} vs sequential {sequential:?}");
}

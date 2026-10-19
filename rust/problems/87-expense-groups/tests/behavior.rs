#![allow(unused_imports, clippy::needless_range_loop)]
use p87_expense_groups::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

fn bal(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// Applies "from>to:amount" payments and returns the resulting balances (zero ones removed).
fn apply(before: &BTreeMap<String, i64>, payments: &[String]) -> BTreeMap<String, i64> {
    let mut b = before.clone();
    for p in payments {
        let (from, rest) = p.split_once('>').unwrap();
        let (to, amount) = rest.split_once(':').unwrap();
        let amount: i64 = amount.parse().unwrap();
        assert!(amount > 0, "payment {p}");
        *b.entry(from.to_string()).or_insert(0) += amount;
        *b.entry(to.to_string()).or_insert(0) -= amount;
    }
    b.retain(|_, v| *v != 0);
    b
}

/// Fewest payments needed: (people with a balance) - (max number of disjoint zero-sum subsets).
fn optimum(balances: &BTreeMap<String, i64>) -> usize {
    let v: Vec<i64> = balances.values().copied().collect();
    let k = v.len();
    if k == 0 {
        return 0;
    }
    let mut sum = vec![0i64; 1 << k];
    let mut best = vec![0usize; 1 << k];
    for m in 1..(1usize << k) {
        sum[m] = sum[m & (m - 1)] + v[m.trailing_zeros() as usize];
        let b = (0..k).filter(|i| m >> i & 1 == 1).map(|i| best[m ^ (1 << i)]).max().unwrap_or(0);
        best[m] = b + usize::from(sum[m] == 0);
    }
    k - best[(1 << k) - 1]
}

/// Balances exactly {a:+2, b:+3, c:-4, d:-5, e:+4}. The optimum is 3 payments, greedy matching needs 4.
fn greedy_trap() -> Option<Arc<dyn ExpenseGroups>> {
    let g = create()?;
    g.create_group("trip", &["a", "b", "c", "d", "e"]).unwrap();
    g.add_exact_expense("trip", "e", &[("c", 4)]).unwrap();
    g.add_exact_expense("trip", "a", &[("d", 2)]).unwrap();
    g.add_exact_expense("trip", "b", &[("d", 3)]).unwrap();
    Some(g)
}

#[test]
fn should_split_an_expense_equally_and_give_the_remainder_to_the_first_names() {
    let g = sut!(create());
    g.create_group("flat", &["ann", "bob", "cy"]).unwrap();

    g.add_expense("flat", "ann", 100, &["cy", "ann", "bob"]).unwrap(); // 34, 33, 33 in name order

    assert_eq!(g.balances("flat"), Ok(bal(&[("ann", 66), ("bob", -33), ("cy", -33)])));
    assert_eq!(g.total_spent("flat"), Ok(100));
}

#[test]
fn should_support_exact_shares_and_a_payer_outside_the_split() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob", "cy"]).unwrap();

    g.add_exact_expense("g", "ann", &[("bob", 30), ("cy", 70)]).unwrap();

    assert_eq!(g.balances("g"), Ok(bal(&[("ann", 100), ("bob", -30), ("cy", -70)])));
}

#[test]
fn should_reject_invalid_input_and_change_nothing() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob"]).unwrap();

    assert_eq!(g.create_group("g", &["x"]), Err(GroupError::AlreadyExists));
    assert_eq!(g.create_group("h", &[]), Err(GroupError::InvalidInput));
    assert_eq!(g.create_group("h", &["x", "x"]), Err(GroupError::InvalidInput));
    assert_eq!(g.add_member("g", "ann"), Err(GroupError::AlreadyExists));
    assert_eq!(g.add_expense("nope", "ann", 10, &["ann"]), Err(GroupError::UnknownGroup));
    assert_eq!(g.add_expense("g", "zed", 10, &["ann"]), Err(GroupError::UnknownMember));
    assert_eq!(g.add_expense("g", "ann", 10, &["ann", "zed"]), Err(GroupError::UnknownMember));
    assert_eq!(g.add_expense("g", "ann", 0, &["ann"]), Err(GroupError::InvalidInput));
    assert_eq!(g.add_expense("g", "ann", 10, &[]), Err(GroupError::InvalidInput));
    assert_eq!(g.add_expense("g", "ann", 10, &["bob", "bob"]), Err(GroupError::InvalidInput));
    assert_eq!(g.add_exact_expense("g", "ann", &[("bob", 0)]), Err(GroupError::InvalidInput));
    assert_eq!(g.settle("g", "ann", "bob", -5), Err(GroupError::InvalidInput));
    assert_eq!(g.settle("g", "ann", "ann", 5), Err(GroupError::InvalidInput));
    assert_eq!(g.balances("g"), Ok(bal(&[])));
    assert_eq!(g.total_spent("g"), Ok(0));
}

#[test]
fn should_keep_balances_of_different_groups_apart() {
    let g = sut!(create());
    g.create_group("a", &["ann", "bob"]).unwrap();
    g.create_group("b", &["ann", "bob"]).unwrap();

    g.add_expense("a", "ann", 10, &["ann", "bob"]).unwrap();

    assert_eq!(g.balances("a"), Ok(bal(&[("ann", 5), ("bob", -5)])));
    assert_eq!(g.balances("b"), Ok(bal(&[])));
}

#[test]
fn should_let_a_settlement_reduce_the_debt() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob"]).unwrap();
    g.add_expense("g", "ann", 100, &["ann", "bob"]).unwrap();

    g.settle("g", "bob", "ann", 20).unwrap();
    assert_eq!(g.balances("g"), Ok(bal(&[("ann", 30), ("bob", -30)])));
    g.settle("g", "bob", "ann", 30).unwrap();

    assert_eq!(g.balances("g"), Ok(bal(&[])));
    assert_eq!(g.total_spent("g"), Ok(100));
}

#[test]
fn should_only_let_members_leave_when_they_are_square() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob", "cy"]).unwrap();
    g.add_expense("g", "ann", 90, &["ann", "bob"]).unwrap();

    assert_eq!(g.remove_member("g", "bob"), Ok(false));
    assert_eq!(g.remove_member("g", "cy"), Ok(true));
    assert_eq!(g.add_expense("g", "ann", 10, &["cy"]), Err(GroupError::UnknownMember));
    g.settle("g", "bob", "ann", 45).unwrap();
    assert_eq!(g.remove_member("g", "bob"), Ok(true));
    assert_eq!(g.remove_member("g", "ann"), Ok(true));
}

#[test]
fn should_return_no_payments_when_everybody_is_square() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob"]).unwrap();
    g.add_expense("g", "ann", 10, &["ann", "bob"]).unwrap();
    g.add_expense("g", "bob", 10, &["ann", "bob"]).unwrap();

    assert_eq!(g.settle_up("g"), Ok(vec![]));
}

#[test]
fn should_settle_up_a_pair_with_one_payment() {
    let g = sut!(create());
    g.create_group("g", &["ann", "bob"]).unwrap();
    g.add_expense("g", "ann", 100, &["ann", "bob"]).unwrap();

    assert_eq!(g.settle_up("g"), Ok(vec!["bob>ann:50".to_string()]));
}

#[test]
fn should_settle_up_with_the_fewest_possible_payments() {
    let g = sut!(greedy_trap());
    let before = g.balances("trip").unwrap();
    assert_eq!(before, bal(&[("a", 2), ("b", 3), ("c", -4), ("d", -5), ("e", 4)]));

    let payments = g.settle_up("trip").unwrap();

    assert_eq!(payments.len(), 3, "greedy matching needs 4, the optimum is 3");
    assert!(apply(&before, &payments).is_empty());
    let mut sorted = payments.clone();
    sorted.sort();
    assert_eq!(payments, sorted);
}

#[test]
fn should_find_the_optimum_on_larger_random_groups() {
    let mut s = 5u64;
    let mut next = move |n: usize| {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % n as u64) as usize
    };
    for round in 0..20 {
        let g = sut!(create());
        let members: Vec<String> = (0..12).map(|i| format!("m{i}")).collect();
        let refs: Vec<&str> = members.iter().map(|s| s.as_str()).collect();
        g.create_group("g", &refs).unwrap();
        // two independent sub-groups of 6 that each sum to zero
        for half in 0..2 {
            for _ in 0..5 {
                let payer = half * 6 + next(6);
                let debtor = half * 6 + next(6);
                if payer != debtor {
                    g.add_exact_expense("g", &format!("m{payer}"), &[(&format!("m{debtor}"), 1 + next(50) as i64)]).unwrap();
                }
            }
        }
        let before = g.balances("g").unwrap();

        let payments = g.settle_up("g").unwrap();

        assert!(apply(&before, &payments).is_empty(), "round {round}");
        assert_eq!(payments.len(), optimum(&before), "round {round}: fewer payments exist");
    }
}

#[test]
fn should_split_across_threads_without_losing_an_expense() {
    let g = sut!(create());
    let all = ["a", "b", "c", "d"];
    g.create_group("g", &all).unwrap();

    run_concurrently(8, |t| {
        for _ in 0..500 {
            g.add_expense("g", all[t % 4], 10, &all).unwrap();
        }
    });

    assert_eq!(g.total_spent("g"), Ok(8 * 500 * 10));
    // every expense of 10 split 4 ways is charged 3,3,2,2 to a,b,c,d; each payer paid 2 * 500 * 10; 4_000 expenses
    assert_eq!(
        g.balances("g"),
        Ok(bal(&[("a", 10_000 - 12_000), ("b", 10_000 - 12_000), ("c", 10_000 - 8_000), ("d", 10_000 - 8_000)]))
    );
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let g = sut!(create());
        g.create_group("g", &["a", "b", "c"]).unwrap();

        run_concurrently(6, |_| {
            for _ in 0..20 {
                g.add_expense("g", "a", 7, &["a", "b", "c"]).unwrap();
                g.settle("g", "b", "a", 1).unwrap();
            }
        });

        // 120 expenses of 7: a is charged 3,2,2 and paid 840; b settled 120 in total
        assert_eq!(g.total_spent("g"), Ok(840), "run {run}");
        assert_eq!(g.balances("g"), Ok(bal(&[("a", 360), ("b", -120), ("c", -240)])), "run {run}");
    }
}

#![allow(unused_imports)]
use p139_shopping_cart::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_allocate_the_coupon_by_largest_remainder_before_any_lowest_sku_tie_break() {
    let s = sut!(create());
    s.add_product("a", "A", 1000, "x", 5).unwrap();
    s.add_product("b", "B", 1000, "x", 5).unwrap();
    s.add_product("c", "C", 1100, "x", 5).unwrap();
    s.set_tax_rate("x", 1000).unwrap();
    s.define_coupon("OFF1000", CouponKind::Fixed, 1000, 0, 1).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "a", 1).unwrap();
    s.add_item("c1", "b", 1).unwrap();
    s.add_item("c1", "c", 1).unwrap();
    s.apply_coupon("c1", "OFF1000").unwrap();

    // floors 322, 322, 354 (sum 998); remainders 1800, 1800, 2600: "c" gets one cent, then "a" (tie, lowest sku)
    // shares 323, 322, 355 -> taxable 677, 678, 745 -> tax 67 + 67 + 74
    assert_eq!(s.total("c1").unwrap(), "3100|0|1000|208|2308");
}

#[test]
fn should_allocate_a_percent_coupon_and_keep_the_total_consistent_with_its_parts() {
    let s = sut!(create());
    s.add_product("a", "A", 333, "x", 50).unwrap();
    s.add_product("b", "B", 777, "y", 50).unwrap();
    s.set_tax_rate("x", 1800).unwrap();
    s.set_tax_rate("y", 575).unwrap();
    s.define_coupon("P7", CouponKind::Percent, 7, 0, 5).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "a", 3).unwrap(); // 999
    s.add_item("c1", "b", 2).unwrap(); // 1554
    s.apply_coupon("c1", "P7").unwrap();

    // subtotal 2553, coupon floor(178.71) = 178; shares a 69.65 -> 69, b 108.35 -> 108, the spare cent goes to a: 70, 108
    // taxable 929 and 1446: tax floor(167.22) = 167 and floor(83.145) = 83
    assert_eq!(s.total("c1").unwrap(), "2553|0|178|250|2625");
}

#[test]
fn should_combine_promotion_coupon_and_tax_on_several_lines() {
    let s = sut!(create());
    s.add_product("A", "A", 999, "x", 100).unwrap();
    s.add_product("B", "B", 1, "y", 100).unwrap();
    s.set_buy_x_get_y("A", 3, 2).unwrap();
    s.set_percent_off("B", 90).unwrap();
    s.set_tax_rate("x", 2000).unwrap();
    s.define_coupon("FIX", CouponKind::Fixed, 123, 1000, 2).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "A", 11).unwrap(); // 10989, 2 groups of 5 -> 4 free = 3996 -> 6993
    s.add_item("c1", "B", 5).unwrap(); // 5, 90% -> floor(4.5) = 4 -> 1
    s.apply_coupon("c1", "FIX").unwrap();

    // S = 6994, coupon 123; shares a 122.98 -> 122, b 0; the spare cent goes to a: 123, 0
    // taxable a 6870 -> 1374, b 1 at 0 bp -> 0
    assert_eq!(s.total("c1").unwrap(), "10994|4000|123|1374|8245");
}

#[test]
fn should_use_wide_arithmetic_for_large_amounts() {
    let s = sut!(create());
    s.add_product("gold", "Gold bar", 999_999, "lux", 5_000).unwrap();
    s.set_tax_rate("lux", 10_000).unwrap();
    s.define_coupon("HALF", CouponKind::Percent, 50, 0, 1).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "gold", 1_000).unwrap();
    s.apply_coupon("c1", "HALF").unwrap();

    assert_eq!(s.total("c1").unwrap(), "999999000|0|499999500|499999500|999999000");
}

#[test]
fn should_give_each_concurrent_checkout_a_distinct_sequential_order_id() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 1_000).unwrap();
    let carts = 40;
    for i in 0..carts {
        s.create_cart(&format!("c{i}"), &format!("u{i}")).unwrap();
        s.add_item(&format!("c{i}"), "pen", 2).unwrap();
    }

    let ids = collect_concurrently(carts, |i| s.checkout(&format!("c{i}")).unwrap());

    let expected: HashSet<String> = (1..=carts).map(|i| format!("ORD-{i}")).collect();
    assert_eq!(ids.into_iter().collect::<HashSet<_>>(), expected);
    assert_eq!(s.stock("pen"), Ok(1_000 - 2 * carts as u32));
}

#[test]
fn should_never_deadlock_when_carts_hold_the_same_skus_in_opposite_orders() {
    let s = sut!(create());
    s.add_product("a", "A", 100, "x", 10_000).unwrap();
    s.add_product("b", "B", 100, "x", 10_000).unwrap();
    let carts = 16;
    for i in 0..carts {
        s.create_cart(&format!("c{i}"), &format!("u{i}")).unwrap();
    }

    run_concurrently(carts, |t| {
        let cid = format!("c{t}");
        for _ in 0..300 {
            if t % 2 == 0 {
                s.add_item(&cid, "a", 1).unwrap();
                s.add_item(&cid, "b", 1).unwrap();
            } else {
                s.add_item(&cid, "b", 1).unwrap();
                s.add_item(&cid, "a", 1).unwrap();
            }
            s.checkout(&cid).unwrap();
        }
    });

    assert_eq!(s.stock("a"), Ok(10_000 - carts as u32 * 300));
    assert_eq!(s.stock("b"), Ok(10_000 - carts as u32 * 300));
    let orders: usize = (0..carts).map(|i| s.orders(&format!("u{i}")).unwrap().len()).sum();
    assert_eq!(orders, carts * 300);
}

#[test]
fn should_not_let_add_item_overshoot_stock_when_many_threads_add_to_the_same_cart() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 500).unwrap();
    s.create_cart("c1", "u1").unwrap();
    let accepted = AtomicUsize::new(0);

    run_concurrently(8, |_| {
        for _ in 0..100 {
            if s.add_item("c1", "pen", 1).is_ok() {
                accepted.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    assert_eq!(accepted.load(Ordering::SeqCst), 500);
    assert_eq!(s.quantity("c1", "pen"), Ok(500));
    assert_eq!(s.total("c1").unwrap(), "125000|0|0|0|125000");
}

#[test]
fn should_keep_a_failed_checkout_atomic_while_others_check_out_the_same_skus() {
    let s = sut!(create());
    s.add_product("a", "A", 100, "x", 100).unwrap();
    s.add_product("b", "B", 100, "x", 5).unwrap();
    s.define_coupon("C", CouponKind::Fixed, 10, 0, 1_000).unwrap();
    let carts = 24;
    for i in 0..carts {
        s.create_cart(&format!("c{i}"), &format!("u{i}")).unwrap();
        s.add_item(&format!("c{i}"), "a", 1).unwrap();
        s.add_item(&format!("c{i}"), "b", 1).unwrap();
        s.apply_coupon(&format!("c{i}"), "C").unwrap();
    }

    let ok = collect_concurrently(carts, |i| match s.checkout(&format!("c{i}")) {
        Ok(_) => true,
        Err(e) => {
            assert_eq!(e, CartError::InsufficientStock("b".into()));
            false
        }
    });

    assert_eq!(ok.iter().filter(|b| **b).count(), 5);
    assert_eq!(s.stock("b"), Ok(0));
    assert_eq!(s.stock("a"), Ok(95)); // failed checkouts took nothing from "a"
    assert_eq!(s.coupon_uses("C"), Ok(5));
    for i in 0..carts {
        if !ok[i] {
            assert_eq!(s.quantity(&format!("c{i}"), "a"), Ok(1));
            assert!(s.orders(&format!("u{i}")).unwrap().is_empty());
        }
    }
}

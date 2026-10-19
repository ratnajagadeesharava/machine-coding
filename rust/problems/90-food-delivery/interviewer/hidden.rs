#![allow(unused_imports, clippy::needless_range_loop)]
use p90_food_delivery::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

#[test]
fn should_restore_the_stock_exactly_when_many_orders_are_cancelled_concurrently() {
    let a = sut!(create());
    a.add_restaurant("r", "x", 0, 0);
    a.add_item("r", "dish", 10, 1_000).unwrap();

    run_concurrently(10, |t| {
        for i in 0..100 {
            let user = format!("u{t}");
            let id = a.place_order(&user, "r", &[("dish", 1)]).unwrap();
            if i % 2 == 0 {
                a.cancel(&id, &user).unwrap();
            }
        }
    });

    assert_eq!(a.stock_of("r", "dish"), Ok(1_000 - 500));
}

#[test]
fn should_handle_a_burst_of_ready_orders_with_few_partners() {
    let a = sut!(create());
    a.add_restaurant("r", "x", 0, 0);
    a.add_item("r", "dish", 10, 10_000).unwrap();
    for i in 0..3 {
        a.add_partner(&format!("p{i}"), i, 0);
    }
    let delivered = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        for _ in 0..100 {
            let id = a
                .place_order(&format!("u{t}"), "r", &[("dish", 1)])
                .unwrap();
            a.accept(&id).unwrap();
            a.mark_ready(&id).unwrap();
            while a.partner_of(&id).unwrap().is_none() {
                thread::yield_now();
            }
            a.pick_up(&id).unwrap();
            a.deliver(&id).unwrap();
            delivered.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(delivered.load(Ordering::SeqCst), 1_600);
}

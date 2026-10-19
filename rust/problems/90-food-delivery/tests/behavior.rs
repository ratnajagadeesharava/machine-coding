#![allow(unused_imports, clippy::needless_range_loop)]
use p90_food_delivery::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn app() -> Option<Arc<dyn FoodDelivery>> {
    let a = create()?;
    a.add_restaurant("r1", "pizza", 0, 0);
    a.add_restaurant("r2", "pizza", 5, 5);
    a.add_restaurant("r3", "sushi", 1, 1);
    a.add_item("r1", "margherita", 250, 10).unwrap();
    a.add_item("r1", "cola", 50, 100).unwrap();
    Some(a)
}

fn ready(a: &Arc<dyn FoodDelivery>, customer: &str, item: &str) -> String {
    let id = a.place_order(customer, "r1", &[(item, 1)]).unwrap();
    a.accept(&id).unwrap();
    a.mark_ready(&id).unwrap();
    id
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn partner(a: &Arc<dyn FoodDelivery>, id: &str) -> Option<String> {
    a.partner_of(id).unwrap()
}

#[test]
fn should_search_restaurants_by_cuisine_and_distance() {
    let a = sut!(app());

    assert_eq!(
        a.search_restaurants("pizza", 1, 1, 100),
        strs(&["r1", "r2"])
    );
    assert_eq!(
        a.search_restaurants("pizza", 5, 4, 100),
        strs(&["r2", "r1"])
    );
    assert_eq!(a.search_restaurants("pizza", 1, 1, 3), strs(&["r1"]));
    assert!(a.search_restaurants("sushi", 0, 0, 1).is_empty());
    assert!(a.search_restaurants("thai", 0, 0, 100).is_empty());
}

#[test]
fn should_place_an_order_reserve_stock_and_compute_the_total() {
    let a = sut!(app());

    let id = a
        .place_order("ann", "r1", &[("margherita", 2), ("cola", 3)])
        .unwrap();

    assert_eq!(a.total_of(&id), Ok(2 * 250 + 3 * 50));
    assert_eq!(a.status_of(&id), Ok("PLACED".to_string()));
    assert_eq!(a.stock_of("r1", "margherita"), Ok(8));
    assert_eq!(a.stock_of("r1", "cola"), Ok(97));
}

#[test]
fn should_refuse_an_order_without_enough_stock_and_reserve_nothing() {
    let a = sut!(app());

    assert_eq!(
        a.place_order("ann", "r1", &[("cola", 5), ("margherita", 11)]),
        Err(FoodError::OutOfStock)
    );

    assert_eq!(a.stock_of("r1", "cola"), Ok(100));
    assert_eq!(a.stock_of("r1", "margherita"), Ok(10));
}

#[test]
fn should_reject_invalid_orders() {
    let a = sut!(app());

    assert_eq!(
        a.place_order("ann", "nope", &[("cola", 1)]),
        Err(FoodError::UnknownId)
    );
    assert_eq!(
        a.place_order("ann", "r1", &[("sushi roll", 1)]),
        Err(FoodError::UnknownId)
    );
    assert_eq!(
        a.place_order("ann", "r1", &[("cola", 0)]),
        Err(FoodError::InvalidInput)
    );
    assert_eq!(
        a.place_order("ann", "r1", &[]),
        Err(FoodError::InvalidInput)
    );
    assert_eq!(a.status_of("nope"), Err(FoodError::UnknownId));
}

#[test]
fn should_walk_through_the_happy_path() {
    let a = sut!(app());
    a.add_partner("p1", 1, 1);
    let id = a.place_order("ann", "r1", &[("cola", 1)]).unwrap();

    a.accept(&id).unwrap();
    assert_eq!(a.status_of(&id), Ok("ACCEPTED".to_string()));
    a.mark_ready(&id).unwrap();
    assert_eq!(a.status_of(&id), Ok("READY".to_string()));
    assert_eq!(partner(&a, &id), Some("p1".to_string()));
    a.pick_up(&id).unwrap();
    assert_eq!(a.status_of(&id), Ok("PICKED_UP".to_string()));
    a.deliver(&id).unwrap();
    assert_eq!(a.status_of(&id), Ok("DELIVERED".to_string()));
}

#[test]
fn should_reject_illegal_transitions() {
    let a = sut!(app());
    a.add_partner("p1", 0, 0);
    let id = a.place_order("ann", "r1", &[("cola", 1)]).unwrap();

    assert_eq!(a.mark_ready(&id), Err(FoodError::InvalidState));
    assert_eq!(a.pick_up(&id), Err(FoodError::InvalidState));
    a.accept(&id).unwrap();
    assert_eq!(a.accept(&id), Err(FoodError::InvalidState));
    assert_eq!(a.deliver(&id), Err(FoodError::InvalidState));
    a.mark_ready(&id).unwrap();
    a.pick_up(&id).unwrap();
    assert_eq!(a.cancel(&id, "ann"), Err(FoodError::InvalidState));
    a.deliver(&id).unwrap();
    assert_eq!(a.deliver(&id), Err(FoodError::InvalidState));
}

#[test]
fn should_assign_the_nearest_free_partner_with_ties_by_id() {
    let a = sut!(app());
    a.add_partner("far", 9, 9);
    a.add_partner("pb", 1, 0);
    a.add_partner("pa", 0, 1);

    let first = ready(&a, "ann", "cola");
    let second = ready(&a, "bob", "cola");
    let third = ready(&a, "cy", "cola");

    assert_eq!(partner(&a, &first), Some("pa".to_string())); // tie with pb at distance 1: lower id wins
    assert_eq!(partner(&a, &second), Some("pb".to_string()));
    assert_eq!(partner(&a, &third), Some("far".to_string()));
}

#[test]
fn should_queue_ready_orders_when_no_partner_is_free_and_assign_them_in_order() {
    let a = sut!(app());
    a.add_partner("p1", 0, 0);
    let o1 = ready(&a, "ann", "cola");
    let o2 = ready(&a, "bob", "cola");
    let o3 = ready(&a, "cy", "cola");

    assert_eq!(partner(&a, &o1), Some("p1".to_string()));
    assert_eq!(partner(&a, &o2), None);
    assert_eq!(a.pick_up(&o2), Err(FoodError::InvalidState));

    a.pick_up(&o1).unwrap();
    a.deliver(&o1).unwrap();
    assert_eq!(partner(&a, &o2), Some("p1".to_string()));
    assert_eq!(partner(&a, &o3), None);
    a.pick_up(&o2).unwrap();
    a.deliver(&o2).unwrap();
    assert_eq!(partner(&a, &o3), Some("p1".to_string()));
}

#[test]
fn should_restore_stock_and_free_the_partner_on_cancel() {
    let a = sut!(app());
    a.add_partner("p1", 0, 0);
    let o1 = ready(&a, "ann", "margherita");
    let o2 = ready(&a, "bob", "cola");
    assert_eq!(a.stock_of("r1", "margherita"), Ok(9));

    a.cancel(&o1, "ann").unwrap();

    assert_eq!(a.status_of(&o1), Ok("CANCELLED".to_string()));
    assert_eq!(a.stock_of("r1", "margherita"), Ok(10));
    assert_eq!(partner(&a, &o2), Some("p1".to_string())); // the waiting order got the freed partner
    assert_eq!(a.cancel(&o1, "ann"), Err(FoodError::InvalidState));
}

#[test]
fn should_remove_a_cancelled_waiting_order_from_the_queue() {
    let a = sut!(app());
    a.add_partner("p1", 0, 0);
    let o1 = ready(&a, "ann", "cola");
    let o2 = ready(&a, "bob", "cola");
    let o3 = ready(&a, "cy", "cola");

    a.cancel(&o2, "bob").unwrap();
    a.pick_up(&o1).unwrap();
    a.deliver(&o1).unwrap();

    assert_eq!(partner(&a, &o3), Some("p1".to_string()));
    assert_eq!(partner(&a, &o2), None);
}

#[test]
fn should_only_let_the_customer_cancel() {
    let a = sut!(app());
    let id = a.place_order("ann", "r1", &[("cola", 1)]).unwrap();

    assert_eq!(a.cancel(&id, "bob"), Err(FoodError::NotAllowed));
}

#[test]
fn should_never_sell_more_than_the_stock_under_contention() {
    let a = sut!(app());

    let results = collect_concurrently(50, |t| {
        a.place_order(&format!("user{t}"), "r1", &[("margherita", 1)])
            .is_ok()
    });

    assert_eq!(results.iter().filter(|r| **r).count(), 10);
    assert_eq!(a.stock_of("r1", "margherita"), Ok(0));
}

#[test]
fn should_give_every_partner_at_most_one_order_and_deliver_everything() {
    let a = sut!(app());
    a.add_item("r1", "cola", 50, 1_000).unwrap(); // plenty of stock for 300 orders
    for i in 0..4 {
        a.add_partner(&format!("p{i}"), i, i);
    }
    let load: Mutex<HashMap<String, usize>> = Mutex::new(HashMap::new());
    let violations = AtomicUsize::new(0);
    let delivered = AtomicUsize::new(0);

    run_concurrently(12, |t| {
        for _ in 0..25 {
            let id = a
                .place_order(&format!("user{t}"), "r1", &[("cola", 1)])
                .unwrap();
            a.accept(&id).unwrap();
            a.mark_ready(&id).unwrap();
            let p = loop {
                if let Some(p) = partner(&a, &id) {
                    break p;
                }
                thread::yield_now();
            };
            {
                let mut l = load.lock().unwrap();
                let n = l.entry(p.clone()).or_insert(0);
                *n += 1;
                if *n > 1 {
                    violations.fetch_add(1, Ordering::SeqCst);
                }
            }
            a.pick_up(&id).unwrap();
            *load.lock().unwrap().get_mut(&p).unwrap() -= 1;
            a.deliver(&id).unwrap();
            delivered.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(violations.load(Ordering::SeqCst), 0);
    assert_eq!(delivered.load(Ordering::SeqCst), 300);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let a = sut!(app());
        a.add_partner("p1", 0, 0);
        a.add_partner("p2", 1, 1);

        let assigned = collect_concurrently(6, |t| {
            let id = ready(&a, &format!("u{t}"), "cola");
            partner(&a, &id)
        });

        let mut got: Vec<String> = assigned.into_iter().flatten().collect();
        assert_eq!(got.len(), 2, "run {run}");
        got.sort();
        got.dedup();
        assert_eq!(
            got.len(),
            2,
            "run {run}: both partners busy with one order each"
        );
    }
}

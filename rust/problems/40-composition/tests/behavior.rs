use p40_composition::contract::{LineItem, OrderError};
use p40_composition::Order;
use std::sync::{Mutex, MutexGuard};

// `LineItem::live_count()` is process-wide and tests run in parallel: take turns.
static TURN: Mutex<()> = Mutex::new(());

fn exclusive() -> MutexGuard<'static, ()> {
    TURN.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn should_create_lines_and_compute_the_total() {
    let _turn = exclusive();
    let mut order = Order::new("o-1");

    order.add_line("A", 2, 500).unwrap();
    order.add_line("B", 1, 250).unwrap();

    assert_eq!(order.id(), "o-1");
    assert_eq!(order.lines().len(), 2);
    assert_eq!(order.total(), 1250);
    assert_eq!(order.lines()[0].sku(), "A");
    assert_eq!(order.lines()[0].subtotal(), 1000);
}

#[test]
fn should_merge_the_same_sku_into_one_line() {
    let _turn = exclusive();
    let mut order = Order::new("o");
    order.add_line("A", 2, 500).unwrap();

    order.add_line("A", 3, 500).unwrap();

    assert_eq!(order.lines().len(), 1);
    assert_eq!(order.lines()[0].quantity(), 5);
    assert_eq!(order.add_line("A", 1, 999), Err(OrderError::PriceMismatch));
    assert_eq!(order.lines()[0].quantity(), 5);
}

#[test]
fn should_destroy_the_lines_when_the_order_is_dropped() {
    let _turn = exclusive();
    let before = LineItem::live_count();
    let mut order = Order::new("o");
    order.add_line("A", 1, 100).unwrap();
    order.add_line("B", 1, 100).unwrap();
    order.add_line("C", 1, 100).unwrap();
    assert_eq!(LineItem::live_count(), before + 3);

    drop(order);

    assert_eq!(LineItem::live_count(), before);
}

#[test]
fn should_destroy_a_line_when_it_is_removed() {
    let _turn = exclusive();
    let before = LineItem::live_count();
    let mut order = Order::new("o");
    order.add_line("A", 1, 100).unwrap();
    order.add_line("B", 1, 100).unwrap();

    assert!(order.remove_line("A"));
    assert!(!order.remove_line("A"));

    assert_eq!(LineItem::live_count(), before + 1);
    assert_eq!(order.total(), 100);
}

#[test]
fn should_destroy_all_lines_on_cancel_and_refuse_new_ones() {
    let _turn = exclusive();
    let before = LineItem::live_count();
    let mut order = Order::new("o");
    order.add_line("A", 1, 100).unwrap();
    order.add_line("B", 2, 100).unwrap();

    order.cancel();

    assert!(order.is_cancelled());
    assert_eq!(order.lines().len(), 0);
    assert_eq!(order.total(), 0);
    assert_eq!(LineItem::live_count(), before);
    assert_eq!(order.add_line("C", 1, 1), Err(OrderError::Cancelled));
    assert_eq!(LineItem::live_count(), before);
}

#[test]
fn should_not_share_parts_between_orders() {
    let _turn = exclusive();
    let before = LineItem::live_count();
    let mut one = Order::new("one");
    let mut two = Order::new("two");
    one.add_line("A", 1, 100).unwrap();
    two.add_line("A", 1, 100).unwrap();
    assert_eq!(LineItem::live_count(), before + 2);

    drop(one);

    assert_eq!(LineItem::live_count(), before + 1);
    assert_eq!(two.lines().len(), 1);
}

#[test]
fn should_reject_invalid_lines() {
    let _turn = exclusive();
    let before = LineItem::live_count();
    let mut order = Order::new("o");

    assert_eq!(order.add_line("", 1, 100), Err(OrderError::InvalidLine));
    assert_eq!(order.add_line("A", 0, 100), Err(OrderError::InvalidLine));
    assert_eq!(order.add_line("A", 1, 0), Err(OrderError::InvalidLine));
    assert_eq!(order.lines().len(), 0);
    assert_eq!(LineItem::live_count(), before);
}

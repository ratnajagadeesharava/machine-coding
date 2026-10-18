use p38_encapsulation::contract::StockError;
use p38_encapsulation::StockItem;

fn item(sku: &str, cap: u32) -> StockItem {
    StockItem::new(sku, cap).unwrap()
}

#[test]
fn should_start_empty_and_track_adds_and_removes() {
    let mut s = item("SKU-1", 10);

    s.add(7).unwrap();
    s.remove(2).unwrap();

    assert_eq!(s.sku(), "SKU-1");
    assert_eq!(s.capacity(), 10);
    assert_eq!(s.quantity(), 5);
}

#[test]
fn should_never_allow_quantity_above_capacity_or_below_zero() {
    let mut s = item("S", 10);
    s.add(8).unwrap();

    assert_eq!(s.add(3), Err(StockError::InvariantViolation));
    assert_eq!(s.quantity(), 8);
    assert_eq!(s.remove(9), Err(StockError::InvariantViolation));
    assert_eq!(s.quantity(), 8);
    s.add(2).unwrap();
    assert_eq!(s.quantity(), 10);
}

#[test]
fn should_reject_zero_amounts_and_bad_construction() {
    let mut s = item("S", 10);

    assert_eq!(s.add(0), Err(StockError::InvalidInput));
    assert_eq!(s.remove(0), Err(StockError::InvalidInput));
    assert!(matches!(StockItem::new(" ", 5), Err(StockError::InvalidInput)));
    assert!(matches!(StockItem::new("S", 0), Err(StockError::InvalidInput)));
    assert_eq!(s.quantity(), 0);
}

#[test]
fn should_transfer_atomically_or_not_at_all() {
    let mut from = item("A", 10);
    let mut to = item("B", 5);
    from.add(8).unwrap();
    to.add(3).unwrap();

    assert_eq!(from.transfer_to(&mut to, 4), Err(StockError::InvariantViolation));
    assert_eq!((from.quantity(), to.quantity()), (8, 3));
    assert_eq!(from.transfer_to(&mut to, 9), Err(StockError::InvariantViolation));
    assert_eq!((from.quantity(), to.quantity()), (8, 3));

    from.transfer_to(&mut to, 2).unwrap();

    assert_eq!((from.quantity(), to.quantity()), (6, 5));
}

#[test]
fn should_normalise_and_deduplicate_tags() {
    let mut s = item("S", 10);
    s.add_tag("Fragile");
    s.add_tag("  fragile ");
    s.add_tag("");
    s.add_tag("Heavy");

    assert_eq!(s.tags(), &["fragile".to_string(), "heavy".to_string()]);
}

#[test]
fn should_keep_a_history_of_successful_changes_only() {
    let mut a = item("A", 10);
    let mut b = item("B", 10);
    a.add(5).unwrap();
    a.remove(2).unwrap();
    assert!(a.remove(50).is_err());
    a.transfer_to(&mut b, 1).unwrap();

    assert_eq!(a.history(), &["+5".to_string(), "-2".to_string(), "-1 (transfer)".to_string()]);
    assert_eq!(b.history(), &["+1 (transfer)".to_string()]);
}

#[test]
fn should_not_change_state_when_a_transfer_to_a_full_item_fails() {
    let mut from = item("A", 10);
    let mut full = item("B", 2);
    from.add(5).unwrap();
    full.add(2).unwrap();

    assert!(from.transfer_to(&mut full, 1).is_err());

    assert_eq!(from.quantity(), 5);
    assert_eq!(full.quantity(), 2);
    assert_eq!(from.history(), &["+5".to_string()]);
    assert_eq!(full.history(), &["+2".to_string()]);
}

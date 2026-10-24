#![allow(unused_imports)]
use p139_shopping_cart::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::Arc;

fn cart(s: &Arc<dyn CartService>, cart_id: &str, user_id: &str, sku: &str, qty: u32) {
    s.create_cart(cart_id, user_id).unwrap();
    s.add_item(cart_id, sku, qty).unwrap();
}

// ---- catalogue and cart lines ----

#[test]
fn should_store_products_and_report_their_stock() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();

    assert_eq!(s.stock("pen"), Ok(5));
    s.set_stock("pen", 12).unwrap();
    assert_eq!(s.stock("pen"), Ok(12));
    s.set_stock("pen", 0).unwrap();
    assert_eq!(s.stock("pen"), Ok(0));
}

#[test]
fn should_reject_invalid_catalogue_input() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();

    assert_eq!(s.add_product("pen", "Again", 100, "stationery", 1), Err(CartError::AlreadyExists));
    assert_eq!(s.add_product("a", "A", 0, "c", 1), Err(CartError::InvalidArgument));
    assert_eq!(s.add_product("a", "A", -5, "c", 1), Err(CartError::InvalidArgument));
    assert_eq!(s.add_product(" ", "A", 5, "c", 1), Err(CartError::InvalidArgument));
    assert_eq!(s.add_product("a", "", 5, "c", 1), Err(CartError::InvalidArgument));
    assert_eq!(s.add_product("a", "A", 5, " ", 1), Err(CartError::InvalidArgument));
    assert_eq!(s.set_stock("nope", 1), Err(CartError::UnknownSku));
    assert_eq!(s.stock("nope"), Err(CartError::UnknownSku));
    assert_eq!(s.set_tax_rate("stationery", 10_001), Err(CartError::InvalidArgument));
    assert_eq!(s.set_tax_rate(" ", 10), Err(CartError::InvalidArgument));
    assert_eq!(s.stock("pen"), Ok(5));
}

#[test]
fn should_add_to_an_existing_line_instead_of_creating_a_second_one() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 10).unwrap();
    s.create_cart("c1", "u1").unwrap();

    s.add_item("c1", "pen", 2).unwrap();
    s.add_item("c1", "pen", 3).unwrap();

    assert_eq!(s.quantity("c1", "pen"), Ok(5));
    assert_eq!(s.total("c1").unwrap(), "1250|0|0|0|1250");
}

#[test]
fn should_reject_adding_beyond_stock_and_leave_the_cart_unchanged() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();
    cart(&s, "c1", "u1", "pen", 4);

    assert_eq!(s.add_item("c1", "pen", 2), Err(CartError::InsufficientStock("pen".into())));
    assert_eq!(s.quantity("c1", "pen"), Ok(4));
    s.add_item("c1", "pen", 1).unwrap();
    assert_eq!(s.quantity("c1", "pen"), Ok(5));
    assert_eq!(s.add_item("c1", "pen", 1), Err(CartError::InsufficientStock("pen".into())));
    assert_eq!(s.quantity("c1", "pen"), Ok(5));
}

#[test]
fn should_not_reserve_stock_when_items_are_added_to_carts() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 1).unwrap();

    cart(&s, "c1", "u1", "pen", 1);
    cart(&s, "c2", "u2", "pen", 1);

    assert_eq!(s.stock("pen"), Ok(1));
    assert_eq!(s.quantity("c1", "pen"), Ok(1));
    assert_eq!(s.quantity("c2", "pen"), Ok(1));
}

#[test]
fn should_set_quantity_exactly_and_remove_the_line_at_zero() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();
    cart(&s, "c1", "u1", "pen", 2);

    s.set_quantity("c1", "pen", 4).unwrap();
    assert_eq!(s.quantity("c1", "pen"), Ok(4));
    s.set_quantity("c1", "pen", 1).unwrap();
    assert_eq!(s.quantity("c1", "pen"), Ok(1));
    assert_eq!(s.set_quantity("c1", "pen", 6), Err(CartError::InsufficientStock("pen".into())));
    assert_eq!(s.quantity("c1", "pen"), Ok(1));
    s.set_quantity("c1", "pen", 0).unwrap();
    assert_eq!(s.quantity("c1", "pen"), Ok(0));
    assert_eq!(s.total("c1").unwrap(), "0|0|0|0|0");
    s.set_quantity("c1", "pen", 0).unwrap(); // nothing to remove: no error
    s.set_quantity("c1", "pen", 3).unwrap(); // creates the line
    assert_eq!(s.quantity("c1", "pen"), Ok(3));
}

#[test]
fn should_remove_a_line_and_reject_removing_one_that_is_not_there() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();
    s.add_product("ink", "Ink", 400, "stationery", 5).unwrap();
    cart(&s, "c1", "u1", "pen", 2);

    s.remove_item("c1", "pen").unwrap();

    assert_eq!(s.quantity("c1", "pen"), Ok(0));
    assert_eq!(s.remove_item("c1", "pen"), Err(CartError::NoSuchLine));
    assert_eq!(s.remove_item("c1", "ink"), Err(CartError::NoSuchLine));
    assert_eq!(s.remove_item("c1", "ghost"), Err(CartError::UnknownSku));
}

#[test]
fn should_reject_unknown_ids_duplicate_carts_and_bad_quantities() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();
    s.create_cart("c1", "u1").unwrap();

    assert_eq!(s.create_cart("c1", "u2"), Err(CartError::AlreadyExists));
    assert_eq!(s.create_cart(" ", "u2"), Err(CartError::InvalidArgument));
    assert_eq!(s.create_cart("c2", " "), Err(CartError::InvalidArgument));
    assert_eq!(s.add_item("nope", "pen", 1), Err(CartError::UnknownCart));
    assert_eq!(s.add_item("c1", "ghost", 1), Err(CartError::UnknownSku));
    assert_eq!(s.add_item("c1", "pen", 0), Err(CartError::InvalidArgument));
    assert_eq!(s.total("nope"), Err(CartError::UnknownCart));
    assert_eq!(s.coupon_status("nope"), Err(CartError::UnknownCart));
    assert_eq!(s.checkout("nope"), Err(CartError::UnknownCart));
    assert_eq!(s.quantity("c1", "pen"), Ok(0));
}

// ---- pricing ----

#[test]
fn should_total_an_empty_cart_as_zeros() {
    let s = sut!(create());
    s.create_cart("c1", "u1").unwrap();

    assert_eq!(s.total("c1").unwrap(), "0|0|0|0|0");
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::None));
}

#[test]
fn should_give_y_units_free_in_every_complete_group_of_x_plus_y() {
    let s = sut!(create());
    s.add_product("tea", "Tea", 1000, "food", 100).unwrap();
    s.set_buy_x_get_y("tea", 2, 1).unwrap();
    for qty in [1u32, 2, 3, 5, 6, 7] {
        let id = format!("c{qty}");
        cart(&s, &id, &format!("u{qty}"), "tea", qty);
    }

    assert_eq!(s.total("c1").unwrap(), "1000|0|0|0|1000");
    assert_eq!(s.total("c2").unwrap(), "2000|0|0|0|2000");
    assert_eq!(s.total("c3").unwrap(), "3000|1000|0|0|2000");
    assert_eq!(s.total("c5").unwrap(), "5000|1000|0|0|4000");
    assert_eq!(s.total("c6").unwrap(), "6000|2000|0|0|4000");
    assert_eq!(s.total("c7").unwrap(), "7000|2000|0|0|5000");
}

#[test]
fn should_take_a_percent_off_the_line_subtotal_with_floor_division() {
    let s = sut!(create());
    s.add_product("gum", "Gum", 333, "food", 100).unwrap();
    s.set_percent_off("gum", 10).unwrap();
    cart(&s, "c1", "u1", "gum", 3);
    cart(&s, "c2", "u2", "gum", 1);

    assert_eq!(s.total("c1").unwrap(), "999|99|0|0|900");
    assert_eq!(s.total("c2").unwrap(), "333|33|0|0|300");
}

#[test]
fn should_keep_only_one_promotion_per_sku_and_allow_clearing_it() {
    let s = sut!(create());
    s.add_product("tea", "Tea", 1000, "food", 100).unwrap();
    cart(&s, "c1", "u1", "tea", 3);

    s.set_buy_x_get_y("tea", 2, 1).unwrap();
    assert_eq!(s.total("c1").unwrap(), "3000|1000|0|0|2000");
    s.set_percent_off("tea", 50).unwrap();
    assert_eq!(s.total("c1").unwrap(), "3000|1500|0|0|1500");
    s.clear_promotion("tea").unwrap();
    assert_eq!(s.total("c1").unwrap(), "3000|0|0|0|3000");
    s.clear_promotion("tea").unwrap();
}

#[test]
fn should_reject_invalid_promotions() {
    let s = sut!(create());
    s.add_product("tea", "Tea", 1000, "food", 100).unwrap();

    assert_eq!(s.set_percent_off("tea", 0), Err(CartError::InvalidArgument));
    assert_eq!(s.set_percent_off("tea", 91), Err(CartError::InvalidArgument));
    assert_eq!(s.set_buy_x_get_y("tea", 0, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.set_buy_x_get_y("tea", 2, 0), Err(CartError::InvalidArgument));
    assert_eq!(s.set_percent_off("ghost", 10), Err(CartError::UnknownSku));
    assert_eq!(s.set_buy_x_get_y("ghost", 1, 1), Err(CartError::UnknownSku));
    cart(&s, "c1", "u1", "tea", 3);
    assert_eq!(s.total("c1").unwrap(), "3000|0|0|0|3000");
}

#[test]
fn should_take_a_percent_coupon_off_the_post_promotion_subtotal_with_floor_division() {
    let s = sut!(create());
    s.add_product("gum", "Gum", 333, "food", 100).unwrap();
    s.define_coupon("TEN", CouponKind::Percent, 10, 0, 5).unwrap();
    cart(&s, "c1", "u1", "gum", 1);
    s.apply_coupon("c1", "TEN").unwrap();

    assert_eq!(s.total("c1").unwrap(), "333|0|33|0|300");

    s.set_percent_off("gum", 50).unwrap(); // promo first: 333 - 166 = 167, then floor(16.7) = 16
    assert_eq!(s.total("c1").unwrap(), "333|166|16|0|151");
}

#[test]
fn should_cap_a_fixed_coupon_at_the_subtotal() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 10).unwrap();
    s.define_coupon("OFF800", CouponKind::Fixed, 800, 0, 5).unwrap();
    cart(&s, "c1", "u1", "pen", 2);
    cart(&s, "c2", "u2", "pen", 4);
    s.apply_coupon("c1", "OFF800").unwrap();
    s.apply_coupon("c2", "OFF800").unwrap();

    assert_eq!(s.total("c1").unwrap(), "500|0|500|0|0");
    assert_eq!(s.total("c2").unwrap(), "1000|0|800|0|200");
}

#[test]
fn should_replace_the_cart_coupon_when_another_is_applied() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 1000, "stationery", 10).unwrap();
    s.define_coupon("TEN", CouponKind::Percent, 10, 0, 5).unwrap();
    s.define_coupon("FLAT50", CouponKind::Fixed, 50, 0, 5).unwrap();
    cart(&s, "c1", "u1", "pen", 1);

    s.apply_coupon("c1", "TEN").unwrap();
    assert_eq!(s.total("c1").unwrap(), "1000|0|100|0|900");
    s.apply_coupon("c1", "FLAT50").unwrap();
    assert_eq!(s.total("c1").unwrap(), "1000|0|50|0|950");
    s.remove_coupon("c1").unwrap();
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::None));
    assert_eq!(s.total("c1").unwrap(), "1000|0|0|0|1000");
    assert_eq!(s.apply_coupon("c1", "GHOST"), Err(CartError::UnknownCoupon));
    assert_eq!(s.apply_coupon("nope", "TEN"), Err(CartError::UnknownCart));
}

#[test]
fn should_ignore_a_coupon_while_the_minimum_subtotal_is_not_met() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 1000, "stationery", 10).unwrap();
    s.define_coupon("BIG", CouponKind::Percent, 10, 3000, 5).unwrap();
    cart(&s, "c1", "u1", "pen", 2);
    s.apply_coupon("c1", "BIG").unwrap();

    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::NotEligible));
    assert_eq!(s.total("c1").unwrap(), "2000|0|0|0|2000");

    s.add_item("c1", "pen", 1).unwrap(); // exactly the minimum
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::Applied));
    assert_eq!(s.total("c1").unwrap(), "3000|0|300|0|2700");

    s.set_quantity("c1", "pen", 2).unwrap();
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::NotEligible));
    assert_eq!(s.total("c1").unwrap(), "2000|0|0|0|2000");
}

#[test]
fn should_measure_the_minimum_subtotal_after_promotions() {
    let s = sut!(create());
    s.add_product("tea", "Tea", 1000, "food", 100).unwrap();
    s.set_buy_x_get_y("tea", 1, 1).unwrap();
    s.define_coupon("BIG", CouponKind::Percent, 10, 3000, 5).unwrap();
    cart(&s, "c1", "u1", "tea", 4);
    s.apply_coupon("c1", "BIG").unwrap();

    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::NotEligible)); // 4000 before promos, 2000 after
    assert_eq!(s.total("c1").unwrap(), "4000|2000|0|0|2000");

    s.set_quantity("c1", "tea", 8).unwrap();
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::Applied));
    assert_eq!(s.total("c1").unwrap(), "8000|4000|400|0|3600");
}

#[test]
fn should_tax_each_line_by_its_category_and_floor_per_line() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 10).unwrap();
    s.add_product("milk", "Milk", 199, "grocery", 10).unwrap();
    s.add_product("x1", "X1", 15, "small", 10).unwrap();
    s.add_product("x2", "X2", 15, "small", 10).unwrap();
    s.set_tax_rate("stationery", 725).unwrap();
    s.set_tax_rate("small", 1000).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "pen", 3).unwrap(); // 750 * 7.25% = 54.375 -> 54
    s.add_item("c1", "milk", 1).unwrap(); // no rate for "grocery" -> 0
    s.create_cart("c2", "u2").unwrap();
    s.add_item("c2", "x1", 1).unwrap(); // 15 * 10% = 1.5 -> 1
    s.add_item("c2", "x2", 1).unwrap(); // 1.5 -> 1 (floored per line, not on the sum)

    assert_eq!(s.total("c1").unwrap(), "949|0|0|54|1003");
    assert_eq!(s.total("c2").unwrap(), "30|0|0|2|32");
}

#[test]
fn should_spread_the_coupon_over_lines_by_largest_remainder_before_taxing() {
    let s = sut!(create());
    for sku in ["a", "b", "c"] {
        s.add_product(sku, sku, 100, "x", 10).unwrap();
    }
    s.set_tax_rate("x", 1000).unwrap();
    s.define_coupon("OFF100", CouponKind::Fixed, 100, 0, 5).unwrap();
    s.create_cart("c1", "u1").unwrap();
    for sku in ["c", "b", "a"] {
        s.add_item("c1", sku, 1).unwrap();
    }
    s.apply_coupon("c1", "OFF100").unwrap();

    // shares 33.33 each: 34 to "a" (remainders tie, lowest sku first), 33, 33 -> taxable 66, 67, 67 -> tax 6 + 6 + 6
    assert_eq!(s.total("c1").unwrap(), "300|0|100|18|218");
}

#[test]
fn should_compute_the_worked_example_from_the_readme() {
    let s = sut!(create());
    s.add_product("A", "Anvil", 1000, "goods", 20).unwrap();
    s.add_product("B", "Bolt", 333, "food", 20).unwrap();
    s.set_tax_rate("goods", 1000).unwrap();
    s.set_tax_rate("food", 500).unwrap();
    s.set_buy_x_get_y("A", 2, 1).unwrap();
    s.set_percent_off("B", 10).unwrap();
    s.define_coupon("SAVE10", CouponKind::Percent, 10, 5000, 3).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.add_item("c1", "A", 7).unwrap();
    s.add_item("c1", "B", 3).unwrap();
    s.apply_coupon("c1", "SAVE10").unwrap();

    assert_eq!(s.total("c1").unwrap(), "7999|2099|590|490|5800");
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::Applied));
}

// ---- coupon definitions ----

#[test]
fn should_reject_invalid_coupon_definitions() {
    let s = sut!(create());
    s.define_coupon("OK", CouponKind::Percent, 10, 0, 1).unwrap();

    assert_eq!(s.define_coupon("OK", CouponKind::Fixed, 5, 0, 1), Err(CartError::AlreadyExists));
    assert_eq!(s.define_coupon(" ", CouponKind::Fixed, 5, 0, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.define_coupon("A", CouponKind::Percent, 0, 0, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.define_coupon("A", CouponKind::Percent, 101, 0, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.define_coupon("A", CouponKind::Fixed, 0, 0, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.define_coupon("A", CouponKind::Fixed, 5, -1, 1), Err(CartError::InvalidArgument));
    assert_eq!(s.define_coupon("A", CouponKind::Fixed, 5, 0, 0), Err(CartError::InvalidArgument));
    assert_eq!(s.coupon_uses("GHOST"), Err(CartError::UnknownCoupon));
    assert_eq!(s.coupon_uses("OK"), Ok(0));
}

// ---- checkout ----

#[test]
fn should_check_out_and_decrement_stock_empty_the_cart_and_record_the_order() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 5).unwrap();
    s.set_tax_rate("stationery", 1000).unwrap();
    cart(&s, "c1", "u1", "pen", 2);

    assert_eq!(s.checkout("c1").unwrap(), "ORD-1");

    assert_eq!(s.stock("pen"), Ok(3));
    assert_eq!(s.quantity("c1", "pen"), Ok(0));
    assert_eq!(s.total("c1").unwrap(), "0|0|0|0|0");
    assert_eq!(s.orders("u1").unwrap(), vec!["ORD-1|550".to_string()]);
    s.add_item("c1", "pen", 3).unwrap(); // the cart is reusable
    assert_eq!(s.add_item("c1", "pen", 1), Err(CartError::InsufficientStock("pen".into())));
}

#[test]
fn should_name_the_first_lacking_sku_in_sku_order_and_change_nothing() {
    let s = sut!(create());
    for sku in ["zeta", "alpha", "mid"] {
        s.add_product(sku, sku, 100, "x", 5).unwrap();
    }
    s.define_coupon("TEN", CouponKind::Percent, 10, 0, 5).unwrap();
    s.create_cart("c1", "u1").unwrap();
    for sku in ["zeta", "alpha", "mid"] {
        s.add_item("c1", sku, 5).unwrap();
    }
    s.apply_coupon("c1", "TEN").unwrap();
    s.set_stock("zeta", 1).unwrap();
    s.set_stock("mid", 1).unwrap();

    assert_eq!(s.checkout("c1"), Err(CartError::InsufficientStock("mid".into())));
    assert_eq!(s.stock("alpha"), Ok(5));
    assert_eq!(s.stock("mid"), Ok(1));
    assert_eq!(s.stock("zeta"), Ok(1));
    assert_eq!(s.quantity("c1", "alpha"), Ok(5));
    assert_eq!(s.coupon_uses("TEN"), Ok(0));
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::Applied));
    assert!(s.orders("u1").unwrap().is_empty());
}

#[test]
fn should_reject_checking_out_an_empty_cart() {
    let s = sut!(create());
    s.create_cart("c1", "u1").unwrap();

    assert_eq!(s.checkout("c1"), Err(CartError::EmptyCart));
    assert!(s.orders("u1").unwrap().is_empty());
}

#[test]
fn should_consume_a_coupon_use_only_on_successful_checkouts_and_ignore_it_once_exhausted() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 1000, "stationery", 10).unwrap();
    s.define_coupon("ONCE", CouponKind::Percent, 10, 0, 1).unwrap();
    cart(&s, "c1", "u1", "pen", 1);
    cart(&s, "c2", "u2", "pen", 1);
    s.apply_coupon("c1", "ONCE").unwrap();
    s.apply_coupon("c2", "ONCE").unwrap();
    assert_eq!(s.coupon_status("c2"), Ok(CouponStatus::Applied));

    assert_eq!(s.checkout("c1").unwrap(), "ORD-1");

    assert_eq!(s.coupon_uses("ONCE"), Ok(1));
    assert_eq!(s.orders("u1").unwrap(), vec!["ORD-1|900".to_string()]);
    assert_eq!(s.coupon_status("c1"), Ok(CouponStatus::None)); // checkout detaches the coupon
    assert_eq!(s.coupon_status("c2"), Ok(CouponStatus::NotEligible));
    assert_eq!(s.total("c2").unwrap(), "1000|0|0|0|1000");
    assert_eq!(s.checkout("c2").unwrap(), "ORD-2");
    assert_eq!(s.coupon_uses("ONCE"), Ok(1));
    assert_eq!(s.orders("u2").unwrap(), vec!["ORD-2|1000".to_string()]);
}

#[test]
fn should_check_out_without_the_coupon_when_it_became_ineligible_and_not_consume_it() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 1000, "stationery", 10).unwrap();
    s.define_coupon("BIG", CouponKind::Fixed, 500, 2000, 5).unwrap();
    cart(&s, "c1", "u1", "pen", 2);
    s.apply_coupon("c1", "BIG").unwrap();
    assert_eq!(s.total("c1").unwrap(), "2000|0|500|0|1500");

    s.set_quantity("c1", "pen", 1).unwrap();
    s.checkout("c1").unwrap();

    assert_eq!(s.coupon_uses("BIG"), Ok(0));
    assert_eq!(s.orders("u1").unwrap(), vec!["ORD-1|1000".to_string()]);
}

#[test]
fn should_list_orders_per_user_in_checkout_order() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 50).unwrap();
    s.create_cart("c1", "u1").unwrap();
    s.create_cart("c2", "u2").unwrap();
    s.create_cart("c3", "u1").unwrap();

    s.add_item("c1", "pen", 1).unwrap();
    s.add_item("c2", "pen", 2).unwrap();
    s.add_item("c3", "pen", 3).unwrap();
    s.checkout("c3").unwrap();
    s.checkout("c2").unwrap();
    s.checkout("c1").unwrap();

    assert_eq!(s.orders("u1").unwrap(), vec!["ORD-1|750".to_string(), "ORD-3|250".to_string()]);
    assert_eq!(s.orders("u2").unwrap(), vec!["ORD-2|500".to_string()]);
    assert!(s.orders("nobody").unwrap().is_empty());
    assert_eq!(s.orders(" "), Err(CartError::InvalidArgument));
}

// ---- concurrency ----

#[test]
fn should_let_exactly_one_of_many_carts_check_out_the_last_unit() {
    let s = sut!(create());
    s.add_product("ps5", "Console", 49_900, "tech", 1).unwrap();
    let carts = 16;
    for i in 0..carts {
        cart(&s, &format!("c{i}"), &format!("u{i}"), "ps5", 1);
    }

    let results = collect_concurrently(carts, |i| s.checkout(&format!("c{i}")).is_ok());

    assert_eq!(results.iter().filter(|b| **b).count(), 1);
    assert_eq!(s.stock("ps5"), Ok(0));
    let orders: usize = (0..carts).map(|i| s.orders(&format!("u{i}")).unwrap().len()).sum();
    assert_eq!(orders, 1);
}

#[test]
fn should_never_oversell_under_concurrent_checkouts() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 250, "stationery", 30).unwrap();
    let carts = 40;
    for i in 0..carts {
        cart(&s, &format!("c{i}"), &format!("u{i}"), "pen", 1 + (i as u32) % 2);
    }

    let sold = collect_concurrently(carts, |i| {
        if s.checkout(&format!("c{i}")).is_ok() { 1 + (i as u32) % 2 } else { 0 }
    });

    let units: u32 = sold.iter().sum();
    assert!(units <= 30);
    assert_eq!(s.stock("pen"), Ok(30 - units));
    assert!(units >= 29); // only a 2-unit cart can be left stranded by 1 unit
}

#[test]
fn should_honour_a_coupon_max_uses_under_concurrent_checkouts() {
    let s = sut!(create());
    s.add_product("pen", "Blue pen", 1000, "stationery", 100).unwrap();
    s.define_coupon("THREE", CouponKind::Percent, 10, 0, 3).unwrap();
    let carts = 12;
    for i in 0..carts {
        cart(&s, &format!("c{i}"), &format!("u{i}"), "pen", 1);
        s.apply_coupon(&format!("c{i}"), "THREE").unwrap();
    }

    run_concurrently(carts, |i| {
        s.checkout(&format!("c{i}")).unwrap();
    });

    assert_eq!(s.coupon_uses("THREE"), Ok(3));
    let all: Vec<String> = (0..carts).flat_map(|i| s.orders(&format!("u{i}")).unwrap()).collect();
    assert_eq!(all.len(), carts);
    assert_eq!(all.iter().filter(|o| o.ends_with("|900")).count(), 3);
    assert_eq!(all.iter().filter(|o| o.ends_with("|1000")).count(), 9);
}

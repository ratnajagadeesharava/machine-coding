#![allow(unused_imports, clippy::needless_range_loop)]
use p04_coffee_machine::{contract::*, create};
use playground_support::{collect_concurrently, sut};

#[test]
fn should_dispense_drink_and_consume_ingredients() {
    let m = sut!(create());
    m.refill("water", 100);
    m.refill("coffee", 10);
    m.add_recipe("espresso", &[("water", 30), ("coffee", 2)]);

    assert_eq!(m.dispense("espresso"), "OK");

    assert_eq!(m.stock("water"), 70);
    assert_eq!(m.stock("coffee"), 8);
}

#[test]
fn should_reject_unknown_drink() {
    let m = sut!(create());
    assert_eq!(m.dispense("tea"), "UNKNOWN_DRINK");
}

#[test]
fn should_report_missing_ingredient() {
    let m = sut!(create());
    m.refill("water", 100);
    m.refill("coffee", 10);
    m.refill("milk", 2);
    m.add_recipe("latte", &[("water", 30), ("coffee", 2), ("milk", 5)]);

    assert_eq!(m.dispense("latte"), "INSUFFICIENT:milk");
}

#[test]
fn should_not_consume_anything_when_dispense_fails() {
    let m = sut!(create());
    m.refill("water", 100);
    m.refill("coffee", 10);
    m.refill("milk", 2);
    m.add_recipe("latte", &[("water", 30), ("coffee", 2), ("milk", 5)]);

    m.dispense("latte");

    assert_eq!(m.stock("water"), 100);
    assert_eq!(m.stock("coffee"), 10);
    assert_eq!(m.stock("milk"), 2);
}

#[test]
fn should_accumulate_refills() {
    let m = sut!(create());
    m.refill("milk", 5);
    m.refill("milk", 7);

    assert_eq!(m.stock("milk"), 12);
    assert_eq!(m.stock("unobtainium"), 0);
}

#[test]
fn should_reject_invalid_recipes() {
    let m = sut!(create());

    assert!(!m.add_recipe("nothing", &[]));
    assert!(!m.add_recipe("bad", &[("water", 0)]));
    assert_eq!(m.dispense("nothing"), "UNKNOWN_DRINK");
    assert_eq!(m.dispense("bad"), "UNKNOWN_DRINK");
}

#[test]
fn should_replace_existing_recipe() {
    let m = sut!(create());
    m.refill("water", 100);
    m.refill("coffee", 100);
    m.add_recipe("coffee-drink", &[("water", 10)]);
    m.add_recipe("coffee-drink", &[("coffee", 4)]);

    m.dispense("coffee-drink");

    assert_eq!(m.stock("water"), 100);
    assert_eq!(m.stock("coffee"), 96);
}

#[test]
fn should_share_ingredients_between_drinks() {
    let m = sut!(create());
    m.refill("water", 50);
    m.add_recipe("a", &[("water", 30)]);
    m.add_recipe("b", &[("water", 30)]);

    assert_eq!(m.dispense("a"), "OK");
    assert_eq!(m.dispense("b"), "INSUFFICIENT:water");
}

#[test]
fn should_allow_consuming_stock_exactly_to_zero() {
    let m = sut!(create());
    m.refill("water", 30);
    m.add_recipe("a", &[("water", 30)]);

    assert_eq!(m.dispense("a"), "OK");
    assert_eq!(m.stock("water"), 0);
    assert_eq!(m.dispense("a"), "INSUFFICIENT:water");
}

#[test]
fn should_list_low_stock_ingredients_sorted() {
    let m = sut!(create());
    m.refill("water", 100);
    m.refill("milk", 3);
    m.refill("coffee", 4);
    m.refill("sugar", 10);

    assert_eq!(m.low_stock(5), vec!["coffee", "milk"]);
    assert_eq!(m.low_stock(4), vec!["milk"]);
    assert!(m.low_stock(1).is_empty());
}

#[test]
fn should_serve_exactly_as_many_drinks_as_stock_allows_when_outlets_race_for_one_ingredient() {
    let m = sut!(create());
    m.refill("milk", 50);
    m.add_recipe("latte", &[("milk", 5)]);

    let results = collect_concurrently(40, |_| m.dispense("latte"));

    assert_eq!(results.iter().filter(|r| *r == "OK").count(), 10);
    assert_eq!(
        results.iter().filter(|r| *r == "INSUFFICIENT:milk").count(),
        30
    );
    assert_eq!(m.stock("milk"), 0);
}

#[test]
fn should_conserve_stock_under_concurrent_mixed_drinks() {
    let m = sut!(create());
    m.refill("water", 1000);
    m.refill("coffee", 120);
    m.refill("milk", 90);
    m.add_recipe("americano", &[("water", 3), ("coffee", 1)]);
    m.add_recipe("latte", &[("water", 2), ("milk", 1), ("coffee", 1)]);

    let results = collect_concurrently(16, |t| {
        let (mut a, mut l) = (0u32, 0u32);
        for i in 0..40 {
            if (i + t) % 2 == 0 {
                if m.dispense("americano") == "OK" {
                    a += 1;
                }
            } else if m.dispense("latte") == "OK" {
                l += 1;
            }
        }
        (a, l)
    });

    let americanos: u32 = results.iter().map(|r| r.0).sum();
    let lattes: u32 = results.iter().map(|r| r.1).sum();
    assert_eq!(m.stock("water"), 1000 - 3 * americanos - 2 * lattes);
    assert_eq!(m.stock("coffee"), 120 - americanos - lattes);
    assert_eq!(m.stock("milk"), 90 - lattes);
}

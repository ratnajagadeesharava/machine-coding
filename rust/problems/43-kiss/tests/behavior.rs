use p43_kiss::late_fee;

const SOURCE: &str = include_str!("../src/lib.rs");

#[test]
fn should_not_charge_for_on_time_or_early_returns() {
    assert_eq!(late_fee(0, false), 0);
    assert_eq!(late_fee(-3, false), 0);
    assert_eq!(late_fee(-3, true), 0);
}

#[test]
fn should_charge_twenty_five_cents_per_day_for_standard_members() {
    assert_eq!(late_fee(1, false), 25);
    assert_eq!(late_fee(10, false), 250);
}

#[test]
fn should_charge_ten_cents_per_day_for_premium_members() {
    assert_eq!(late_fee(1, true), 10);
    assert_eq!(late_fee(30, true), 300);
}

#[test]
fn should_cap_the_fee() {
    assert_eq!(late_fee(40, false), 1000);
    assert_eq!(late_fee(41, false), 1000);
    assert_eq!(late_fee(50, true), 500);
    assert_eq!(late_fee(51, true), 500);
}

#[test]
fn should_not_overflow_for_absurdly_late_returns() {
    assert_eq!(late_fee(i32::MAX, false), 1000);
    assert_eq!(late_fee(i32::MAX, true), 500);
}

#[test]
fn should_fit_in_one_small_function_with_no_machinery() {
    assert_eq!(late_fee(2, false), 50);

    let code: Vec<&str> = SOURCE
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();
    assert!(code.len() <= 25, "code lines {} (budget 25)", code.len());
    let joined = code.join("\n");
    for banned in ["trait ", "dyn ", "struct ", "enum ", "impl ", "Box<", "macro_rules", "mod "] {
        assert!(!joined.contains(banned), "over-engineering smell: {banned}");
    }
}

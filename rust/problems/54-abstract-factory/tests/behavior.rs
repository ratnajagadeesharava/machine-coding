use p54_abstract_factory::contract::*;
use p54_abstract_factory::{kit_for_region, summarize, EuKit, IndiaKit, UsKit};

/// A whole family written in the test: the client must work with it without knowing anything concrete.
struct TestKit;

struct OneCent;
impl TaxCalculator for OneCent {
    fn tax(&self, _net: u64) -> u64 {
        1
    }
}
struct Hash;
impl PriceFormatter for Hash {
    fn format(&self, cents: u64) -> String {
        format!("#{cents}")
    }
}
struct Never;
impl AddressValidator for Never {
    fn is_valid_postal_code(&self, _code: &str) -> bool {
        false
    }
}

impl RegionKit for TestKit {
    fn region(&self) -> &str {
        "TEST"
    }
    fn tax_calculator(&self) -> Box<dyn TaxCalculator> {
        Box::new(OneCent)
    }
    fn price_formatter(&self) -> Box<dyn PriceFormatter> {
        Box::new(Hash)
    }
    fn address_validator(&self) -> Box<dyn AddressValidator> {
        Box::new(Never)
    }
}

#[test]
fn should_give_the_us_family() {
    let kit = UsKit;

    assert_eq!(kit.region(), "US");
    assert_eq!(kit.tax_calculator().tax(10_000), 800);
    assert_eq!(kit.tax_calculator().tax(1257), 101);
    assert_eq!(kit.tax_calculator().tax(1255), 100);
    assert_eq!(kit.price_formatter().format(1250), "$12.50");
    assert_eq!(kit.price_formatter().format(5), "$0.05");
    assert_eq!(kit.price_formatter().format(100_000), "$1000.00");
    let v = kit.address_validator();
    assert!(v.is_valid_postal_code("12345"));
    assert!(v.is_valid_postal_code("12345-6789"));
    assert!(!v.is_valid_postal_code("1234"));
    assert!(!v.is_valid_postal_code("ABCDE"));
}

#[test]
fn should_give_the_eu_family() {
    let kit = EuKit;

    assert_eq!(kit.region(), "EU");
    assert_eq!(kit.tax_calculator().tax(10_000), 1900);
    assert_eq!(kit.price_formatter().format(1250), "12,50 EUR");
    assert_eq!(kit.price_formatter().format(5), "0,05 EUR");
    assert!(kit.address_validator().is_valid_postal_code("10115"));
    assert!(!kit.address_validator().is_valid_postal_code("12345-6789"));
}

#[test]
fn should_give_the_indian_family() {
    let kit = IndiaKit;

    assert_eq!(kit.region(), "IN");
    assert_eq!(kit.tax_calculator().tax(10_000), 1800);
    assert_eq!(kit.price_formatter().format(1250), "Rs 12.50");
    assert!(kit.address_validator().is_valid_postal_code("560001"));
    assert!(!kit.address_validator().is_valid_postal_code("12345"));
}

#[test]
fn should_choose_the_factory_by_region_code() {
    assert_eq!(kit_for_region("us").unwrap().region(), "US");
    assert_eq!(kit_for_region("EU").unwrap().region(), "EU");
    assert_eq!(kit_for_region("In").unwrap().region(), "IN");
    assert!(matches!(kit_for_region("JP"), Err(KitError::UnknownRegion)));
    assert!(matches!(kit_for_region(""), Err(KitError::UnknownRegion)));
}

#[test]
fn should_summarise_with_everything_from_one_family() {
    assert_eq!(summarize(&UsKit, 10_000, "12345"), "US: net $100.00, tax $8.00, total $108.00, postal valid");
    assert_eq!(
        summarize(&EuKit, 10_000, "10115"),
        "EU: net 100,00 EUR, tax 19,00 EUR, total 119,00 EUR, postal valid"
    );
    assert_eq!(summarize(&IndiaKit, 10_000, "5600"), "IN: net Rs 100.00, tax Rs 18.00, total Rs 118.00, postal invalid");
}

#[test]
fn should_summarise_with_a_family_it_has_never_seen() {
    assert_eq!(summarize(&TestKit, 100, "whatever"), "TEST: net #100, tax #1, total #101, postal invalid");
}

#[test]
fn should_give_usable_products_on_every_call() {
    let kit = UsKit;

    assert_eq!(kit.tax_calculator().tax(100), kit.tax_calculator().tax(100));
    assert_eq!(kit.price_formatter().format(1), "$0.01");
    assert!(!kit.address_validator().is_valid_postal_code(""));
}

#[test]
fn should_not_mix_products_across_families() {
    let kits: Vec<Box<dyn RegionKit>> = vec![Box::new(UsKit), Box::new(EuKit), Box::new(IndiaKit)];

    let formatted: Vec<String> = kits.iter().map(|k| k.price_formatter().format(1250)).collect();
    let taxes: Vec<u64> = kits.iter().map(|k| k.tax_calculator().tax(10_000)).collect();

    assert_eq!(formatted, vec!["$12.50", "12,50 EUR", "Rs 12.50"]);
    assert_eq!(taxes, vec![800, 1900, 1800]);
}

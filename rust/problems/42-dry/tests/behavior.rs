use p42_dry::contract::*;
use p42_dry::{Documents, StandardRules};

fn standard() -> Documents {
    Documents::new(Box::new(StandardRules::new(1800, 10_000, 1000)))
}

/// Rules written in the test: every document must use whatever rules it is given.
struct Fixed {
    discount: i64,
    tax: i64,
}

impl PricingRules for Fixed {
    fn discount(&self, _net: i64) -> i64 {
        self.discount
    }
    fn tax(&self, _taxable: i64) -> i64 {
        self.tax
    }
}

#[test]
fn should_format_a_quote() {
    assert_eq!(standard().quote(&[5000]).unwrap(), "QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00");
}

#[test]
fn should_format_an_invoice_with_the_bulk_discount() {
    assert_eq!(
        standard().invoice("INV-1", &[12_000, 8_000]).unwrap(),
        "INVOICE #INV-1 net=200.00 discount=20.00 tax=32.40 total=212.40"
    );
}

#[test]
fn should_format_a_credit_note_with_a_negative_total() {
    assert_eq!(
        standard().credit_note("INV-9", &[5000]).unwrap(),
        "CREDIT NOTE #INV-9 net=50.00 discount=0.00 tax=9.00 total=-59.00"
    );
}

#[test]
fn should_round_tax_half_up_to_whole_cents() {
    assert_eq!(standard().quote(&[1005]).unwrap(), "QUOTE net=10.05 discount=0.00 tax=1.81 total=11.86");
    assert_eq!(standard().quote(&[5]).unwrap(), "QUOTE net=0.05 discount=0.00 tax=0.01 total=0.06");
}

#[test]
fn should_apply_the_discount_only_from_the_threshold() {
    assert!(standard().quote(&[9_999]).unwrap().contains("discount=0.00"));
    assert!(standard().quote(&[10_000]).unwrap().contains("discount=10.00"));
}

#[test]
fn should_use_the_same_injected_rules_in_every_document() {
    let docs = Documents::new(Box::new(Fixed { discount: 100, tax: 7 }));

    assert_eq!(docs.quote(&[1000]).unwrap(), "QUOTE net=10.00 discount=1.00 tax=0.07 total=9.07");
    assert_eq!(docs.invoice("A", &[1000]).unwrap(), "INVOICE #A net=10.00 discount=1.00 tax=0.07 total=9.07");
    assert_eq!(
        docs.credit_note("A", &[1000]).unwrap(),
        "CREDIT NOTE #A net=10.00 discount=1.00 tax=0.07 total=-9.07"
    );
}

#[test]
fn should_change_every_document_when_only_the_rate_changes() {
    let low = Documents::new(Box::new(StandardRules::new(1000, 10_000, 0)));
    let high = Documents::new(Box::new(StandardRules::new(5000, 10_000, 0)));

    for text in [low.quote(&[1000]).unwrap(), low.invoice("X", &[1000]).unwrap(), low.credit_note("X", &[1000]).unwrap()] {
        assert!(text.contains("tax=1.00"), "{text}");
    }
    for text in [high.quote(&[1000]).unwrap(), high.invoice("X", &[1000]).unwrap(), high.credit_note("X", &[1000]).unwrap()] {
        assert!(text.contains("tax=5.00"), "{text}");
    }
}

#[test]
fn should_validate_input_the_same_way_everywhere() {
    let docs = standard();

    assert_eq!(docs.quote(&[]), Err(DocError::InvalidLines));
    assert_eq!(docs.invoice("A", &[100, -1]), Err(DocError::InvalidLines));
    assert_eq!(docs.credit_note("A", &[-5]), Err(DocError::InvalidLines));
    assert_eq!(docs.invoice(" ", &[100]), Err(DocError::InvalidNumber));
    assert_eq!(docs.credit_note("", &[100]), Err(DocError::InvalidNumber));
    assert_eq!(docs.quote(&[0]).unwrap(), "QUOTE net=0.00 discount=0.00 tax=0.00 total=0.00");
}

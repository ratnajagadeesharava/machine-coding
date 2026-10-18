use p53_factory_method::contract::*;
use p53_factory_method::{BankTransferCheckout, CardCheckout, CheckoutFlow};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// A gateway written in the test.
struct FakeGateway {
    charges: Rc<RefCell<Vec<String>>>,
    accept: bool,
}

impl PaymentGateway for FakeGateway {
    fn name(&self) -> &str {
        "fake"
    }
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool {
        self.charges.borrow_mut().push(format!("{customer_id}:{amount_cents}"));
        self.accept
    }
}

/// A creator written in the test: it only implements the factory method.
struct RecordingCheckout {
    charges: Rc<RefCell<Vec<String>>>,
    created: Cell<u32>,
    accept: bool,
}

impl RecordingCheckout {
    fn new(accept: bool) -> Self {
        RecordingCheckout { charges: Rc::new(RefCell::new(Vec::new())), created: Cell::new(0), accept }
    }
}

impl CheckoutFlow for RecordingCheckout {
    fn create_gateway(&self) -> Box<dyn PaymentGateway> {
        self.created.set(self.created.get() + 1);
        Box::new(FakeGateway { charges: self.charges.clone(), accept: self.accept })
    }
}

#[test]
fn should_check_out_by_card_with_the_card_gateway() {
    assert_eq!(CardCheckout.checkout("cust-1", 5000).unwrap(), "RECEIPT card cust-1 5000");
    assert_eq!(CardCheckout.checkout("cust-1", 100_001).unwrap(), "DECLINED card cust-1 100001");
    assert_eq!(CardCheckout.checkout("cust-2", 100_000).unwrap(), "RECEIPT card cust-2 100000");
}

#[test]
fn should_check_out_by_bank_transfer_with_the_bank_gateway() {
    assert_eq!(BankTransferCheckout.checkout("cust-1", 250_000).unwrap(), "RECEIPT bank cust-1 250000");
    assert_eq!(BankTransferCheckout.checkout("cust-1", 999).unwrap(), "DECLINED bank cust-1 999");
    assert_eq!(BankTransferCheckout.checkout("cust-1", 1000).unwrap(), "RECEIPT bank cust-1 1000");
}

#[test]
fn should_use_whatever_gateway_the_implementor_creates() {
    let flow = RecordingCheckout::new(true);

    let receipt = flow.checkout("cust-7", 1234).unwrap();

    assert_eq!(receipt, "RECEIPT fake cust-7 1234");
    assert_eq!(*flow.charges.borrow(), vec!["cust-7:1234"]);
}

#[test]
fn should_report_a_decline_from_the_created_gateway() {
    let flow = RecordingCheckout::new(false);

    assert_eq!(flow.checkout("cust-7", 50).unwrap(), "DECLINED fake cust-7 50");
}

#[test]
fn should_ask_the_factory_method_exactly_once_per_checkout() {
    let flow = RecordingCheckout::new(true);

    flow.checkout("a", 1).unwrap();
    flow.checkout("b", 2).unwrap();
    flow.checkout("c", 3).unwrap();

    assert_eq!(flow.created.get(), 3);
}

#[test]
fn should_validate_before_creating_any_gateway() {
    let flow = RecordingCheckout::new(true);

    assert_eq!(flow.checkout(" ", 100), Err(CheckoutError::InvalidCustomer));
    assert_eq!(flow.checkout("c", 0), Err(CheckoutError::InvalidAmount));

    assert_eq!(flow.created.get(), 0);
    assert_eq!(flow.checkout("c", 5).unwrap(), "RECEIPT fake c 5");
}

#[test]
fn should_let_different_creators_be_used_through_the_same_trait() {
    let flows: Vec<Box<dyn CheckoutFlow>> = vec![Box::new(CardCheckout), Box::new(BankTransferCheckout), Box::new(RecordingCheckout::new(true))];

    let receipts: Vec<String> = flows.iter().map(|f| f.checkout("c", 2000).unwrap()).collect();

    assert_eq!(receipts, vec!["RECEIPT card c 2000", "RECEIPT bank c 2000", "RECEIPT fake c 2000"]);
}

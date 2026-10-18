use p45_law_of_demeter::contract::*;
use p45_law_of_demeter::{Cashier, ShippingCalculator, SimpleWallet, StandardCustomer};

/// A customer whose wallet is off limits: any code that reaches through it fails the test.
struct StrictCustomer {
    funds: u64,
    pay_calls: u32,
}

impl StrictCustomer {
    fn new(funds: u64) -> Self {
        StrictCustomer { funds, pay_calls: 0 }
    }
}

impl Customer for StrictCustomer {
    fn name(&self) -> &str {
        "strict"
    }
    fn can_afford(&self, amount: u64) -> bool {
        self.funds >= amount
    }
    fn pay(&mut self, amount: u64) -> Result<(), PayError> {
        self.pay_calls += 1;
        self.funds -= amount;
        Ok(())
    }
    fn wallet(&self) -> &dyn Wallet {
        panic!("Law of Demeter: do not reach through the customer to its wallet")
    }
}

/// An order whose customer is off limits.
struct StrictOrder {
    city: &'static str,
    items: u64,
}

impl Order for StrictOrder {
    fn customer(&self) -> &dyn Customer {
        panic!("Law of Demeter: do not reach through the order to its customer")
    }
    fn shipping_city(&self) -> &str {
        self.city
    }
    fn item_total(&self) -> u64 {
        self.items
    }
}

#[test]
fn should_approve_and_take_payment_when_the_customer_can_afford() {
    let mut cashier = Cashier::new();
    let mut customer = StrictCustomer::new(500);

    assert_eq!(cashier.checkout(&mut customer, 300), Ok(Outcome::Approved));

    assert_eq!(customer.pay_calls, 1);
    assert_eq!(customer.funds, 200);
    assert_eq!(cashier.revenue(), 300);
}

#[test]
fn should_decline_without_taking_money_when_the_customer_cannot_afford() {
    let mut cashier = Cashier::new();
    let mut customer = StrictCustomer::new(100);

    assert_eq!(cashier.checkout(&mut customer, 300), Ok(Outcome::Declined));

    assert_eq!(customer.pay_calls, 0);
    assert_eq!(cashier.revenue(), 0);
}

#[test]
fn should_accumulate_revenue_across_customers() {
    let mut cashier = Cashier::new();

    cashier.checkout(&mut StrictCustomer::new(1000), 400).unwrap();
    cashier.checkout(&mut StrictCustomer::new(50), 400).unwrap();
    cashier.checkout(&mut StrictCustomer::new(1000), 250).unwrap();

    assert_eq!(cashier.revenue(), 650);
}

#[test]
fn should_let_the_customer_pay_from_its_own_wallet_and_hide_the_details() {
    let mut ann = StandardCustomer::new("Ann", Box::new(SimpleWallet::new(1000)));

    assert_eq!(ann.name(), "Ann");
    assert!(ann.can_afford(1000));
    assert!(!ann.can_afford(1001));
    ann.pay(400).unwrap();

    assert_eq!(ann.wallet().balance(), 600);
    assert_eq!(ann.pay(601), Err(PayError::InsufficientFunds));
    assert_eq!(ann.wallet().balance(), 600);
}

#[test]
fn should_work_end_to_end_with_the_real_customer() {
    let mut cashier = Cashier::new();
    let mut bob = StandardCustomer::new("Bob", Box::new(SimpleWallet::new(500)));

    assert_eq!(cashier.checkout(&mut bob, 500), Ok(Outcome::Approved));
    assert_eq!(cashier.checkout(&mut bob, 1), Ok(Outcome::Declined));

    assert_eq!(bob.wallet().balance(), 0);
    assert_eq!(cashier.revenue(), 500);
}

#[test]
fn should_price_shipping_from_the_order_without_reaching_through_to_the_customer() {
    let shipping = ShippingCalculator::new();

    assert_eq!(shipping.cost(&StrictOrder { city: "Paris", items: 5000 }), 1500);
    assert_eq!(shipping.cost(&StrictOrder { city: "LYON", items: 5000 }), 1800);
    assert_eq!(shipping.cost(&StrictOrder { city: "Berlin", items: 5000 }), 2500);
    assert_eq!(shipping.cost(&StrictOrder { city: "Paris", items: 10_000 }), 0);
    assert_eq!(shipping.cost(&StrictOrder { city: "Berlin", items: 9_999 }), 2500);
}

#[test]
fn should_reject_invalid_amounts() {
    let mut cashier = Cashier::new();
    let mut customer = StrictCustomer::new(100);
    let mut wallet = SimpleWallet::new(10);

    assert_eq!(cashier.checkout(&mut customer, 0), Err(PayError::InvalidAmount));
    assert_eq!(wallet.debit(0), Err(PayError::InvalidAmount));
    assert_eq!(wallet.debit(11), Err(PayError::InsufficientFunds));
    assert_eq!(wallet.balance(), 10);
    assert_eq!(customer.pay_calls, 0);
}

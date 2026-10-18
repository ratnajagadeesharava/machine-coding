use p51_solid_summary::contract::*;
use p51_solid_summary::{CardPayment, CashOnDelivery, DistanceBandFee, OrderPricer, OrderProcessor};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

type Events = Rc<RefCell<Vec<String>>>;

fn meal() -> Vec<Item> {
    vec![
        Item { name: "burger".into(), quantity: 2, unit_cents: 800 },
        Item { name: "fries".into(), quantity: 1, unit_cents: 300 },
    ]
}

struct FakeRepository {
    events: Events,
    statuses: RefCell<HashMap<String, String>>,
}

impl OrderRepository for FakeRepository {
    fn save(&self, order_id: &str, status: &str, total_cents: u64) {
        self.events.borrow_mut().push(format!("save:{order_id}:{status}:{total_cents}"));
        self.statuses.borrow_mut().insert(order_id.to_string(), status.to_string());
    }
    fn status_of(&self, order_id: &str) -> Option<String> {
        self.statuses.borrow().get(order_id).cloned()
    }
}

struct FakeNotifier(Events);

impl Notifier for FakeNotifier {
    fn notify(&self, order_id: &str, message: &str) {
        self.0.borrow_mut().push(format!("notify:{order_id}:{message}"));
    }
}

struct FakePayment {
    events: Events,
    approve: bool,
    calls: Cell<u32>,
}

impl PaymentMethod for FakePayment {
    fn charge(&self, amount_cents: u64) -> Result<PaymentResult, OrderError> {
        self.calls.set(self.calls.get() + 1);
        self.events.borrow_mut().push(format!("charge:{amount_cents}"));
        Ok(PaymentResult { approved: self.approve, reference: if self.approve { "fake-1".into() } else { String::new() } })
    }
}

fn setup() -> (OrderProcessor, Events) {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let processor = OrderProcessor::new(
        OrderPricer::new(),
        Box::new(FakeRepository { events: events.clone(), statuses: RefCell::new(HashMap::new()) }),
        Box::new(FakeNotifier(events.clone())),
    );
    (processor, events)
}

fn fake_payment(events: &Events, approve: bool) -> FakePayment {
    FakePayment { events: events.clone(), approve, calls: Cell::new(0) }
}

/// The shared Liskov suite: every `PaymentMethod` must pass ALL of it.
fn payment_contract(make: impl Fn() -> Box<dyn PaymentMethod>, declined_amount: u64) {
    let ok = make().charge(500).unwrap();
    assert!(ok.approved && !ok.reference.trim().is_empty(), "ordinary charge: {ok:?}");

    let declined = make().charge(declined_amount).expect("a decline is not an error");
    assert!(!declined.approved && declined.reference.is_empty(), "decline: {declined:?}");

    let m = make();
    for amount in [1u64, 10_000, 1_000_000_000_000, u64::MAX] {
        assert!(m.charge(amount).is_ok(), "charge({amount}) must not fail");
    }
    assert_eq!(make().charge(0), Err(OrderError::InvalidAmount));

    let m = make();
    let first = m.charge(100).unwrap().reference;
    let second = m.charge(100).unwrap().reference;
    assert_ne!(first, second, "references must be unique");
}

#[test]
fn should_price_items_plus_delivery() {
    let pricer = OrderPricer::new();

    assert_eq!(pricer.total(&meal(), 2500, &DistanceBandFee::new()), Ok(2200));
    assert_eq!(pricer.total(&meal(), 500, &DistanceBandFee::new()), Ok(1900));
    assert_eq!(pricer.total(&[], 500, &DistanceBandFee::new()), Err(OrderError::InvalidItems));
    let zero_qty = [Item { name: "x".into(), quantity: 0, unit_cents: 5 }];
    assert_eq!(pricer.total(&zero_qty, 500, &DistanceBandFee::new()), Err(OrderError::InvalidItems));
}

#[test]
fn should_price_delivery_in_distance_bands() {
    let fee = DistanceBandFee::new();

    assert_eq!(fee.fee(0), 0);
    assert_eq!(fee.fee(1000), 0);
    assert_eq!(fee.fee(1001), 300);
    assert_eq!(fee.fee(3000), 300);
    assert_eq!(fee.fee(3001), 400);
    assert_eq!(fee.fee(4000), 400);
    assert_eq!(fee.fee(4001), 500);
}

#[test]
fn should_charge_then_save_then_notify_when_payment_is_approved() {
    let (processor, events) = setup();
    let payment = fake_payment(&events, true);

    let placement = processor.place("o1", &meal(), 2500, &DistanceBandFee::new(), &payment).unwrap();

    assert_eq!(placement, Placement { status: "PAID".into(), total_cents: 2200, reference: "fake-1".into() });
    assert_eq!(*events.borrow(), vec!["charge:2200", "save:o1:PAID:2200", "notify:o1:Order o1 PAID"]);
}

#[test]
fn should_record_a_declined_payment() {
    let (processor, events) = setup();
    let payment = fake_payment(&events, false);

    let placement = processor.place("o2", &meal(), 500, &DistanceBandFee::new(), &payment).unwrap();

    assert_eq!(placement, Placement { status: "PAYMENT_DECLINED".into(), total_cents: 1900, reference: String::new() });
    assert_eq!(
        *events.borrow(),
        vec!["charge:1900", "save:o2:PAYMENT_DECLINED:1900", "notify:o2:Order o2 PAYMENT_DECLINED"]
    );
    assert_eq!(processor.status("o2").as_deref(), Some("PAYMENT_DECLINED"));
    assert_eq!(processor.status("nope"), None);
}

#[test]
fn should_not_charge_anything_for_a_free_order() {
    let (processor, events) = setup();
    let payment = fake_payment(&events, true);
    let sample = [Item { name: "sample".into(), quantity: 1, unit_cents: 0 }];

    let placement = processor.place("free", &sample, 500, &DistanceBandFee::new(), &payment).unwrap();

    assert_eq!(placement, Placement { status: "PAID".into(), total_cents: 0, reference: String::new() });
    assert_eq!(payment.calls.get(), 0);
}

struct FlatFee;
impl DeliveryFeePolicy for FlatFee {
    fn fee(&self, _distance_m: u32) -> u64 {
        999
    }
}

struct Voucher;
impl PaymentMethod for Voucher {
    fn charge(&self, _amount: u64) -> Result<PaymentResult, OrderError> {
        Ok(PaymentResult { approved: true, reference: "voucher-7".into() })
    }
}

#[test]
fn should_accept_new_fee_policies_and_payment_methods_without_any_change() {
    let (processor, _events) = setup();

    let placement = processor.place("o3", &meal(), 123_456, &FlatFee, &Voucher).unwrap();

    assert_eq!(placement, Placement { status: "PAID".into(), total_cents: 1900 + 999, reference: "voucher-7".into() });
}

#[test]
fn should_refund_through_anything_that_is_refundable_even_if_it_cannot_charge() {
    struct RefundOnly;
    impl Refundable for RefundOnly {
        fn refund(&self, reference: &str, _amount: u64) -> bool {
            reference == "ok"
        }
    }
    let (processor, events) = setup();

    assert!(processor.refund("o1", "ok", 500, &RefundOnly));
    assert_eq!(*events.borrow(), vec!["save:o1:REFUNDED:500", "notify:o1:Order o1 REFUNDED"]);
    events.borrow_mut().clear();
    assert!(!processor.refund("o1", "unknown", 500, &RefundOnly));
    assert!(events.borrow().is_empty());
}

#[test]
fn should_work_end_to_end_with_the_real_card_and_refund_within_the_amount_paid() {
    let (processor, _events) = setup();
    let card = CardPayment::new(100_000);
    let placement = processor.place("o9", &meal(), 2500, &DistanceBandFee::new(), &card).unwrap();

    assert_eq!(placement.reference, "card-1");
    assert!(processor.refund("o9", &placement.reference, 2000, &card));
    assert!(!processor.refund("o9", &placement.reference, 300, &card));
    assert_eq!(processor.status("o9").as_deref(), Some("REFUNDED"));
}

#[test]
fn should_refuse_a_blank_order_id_before_doing_anything() {
    let (processor, events) = setup();
    let payment = fake_payment(&events, true);

    assert_eq!(processor.place(" ", &meal(), 500, &DistanceBandFee::new(), &payment), Err(OrderError::InvalidId));

    assert_eq!(payment.calls.get(), 0);
    assert!(events.borrow().is_empty());
}

#[test]
fn card_payment_should_honour_the_payment_contract() {
    payment_contract(|| Box::new(CardPayment::new(10_000)), 10_001);
}

#[test]
fn cash_on_delivery_should_honour_the_payment_contract() {
    payment_contract(|| Box::new(CashOnDelivery::new(50_000)), 50_001);
}

#[test]
fn should_number_references_and_limit_refunds_on_the_card() {
    let card = CardPayment::new(1000);
    assert_eq!(card.charge(10).unwrap().reference, "card-1");
    assert!(!card.charge(5000).unwrap().approved);
    assert_eq!(card.charge(10).unwrap().reference, "card-2");

    let card = CardPayment::new(10_000);
    let reference = card.charge(1000).unwrap().reference;
    assert!(card.refund(&reference, 600));
    assert!(!card.refund(&reference, 500));
    assert!(card.refund(&reference, 400));
    assert!(!card.refund("card-99", 1));
    assert!(!card.refund(&reference, 1));

    let cash = CashOnDelivery::new(50_000);
    assert_eq!(cash.charge(10).unwrap().reference, "cod-1");
    assert_eq!(cash.charge(50_000).unwrap().reference, "cod-2");
    assert!(!cash.charge(50_001).unwrap().approved);
}

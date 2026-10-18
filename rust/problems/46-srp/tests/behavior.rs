use p46_srp::contract::*;
use p46_srp::{InvoiceService, StandardCalculator, TextRenderer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Events = Rc<RefCell<Vec<String>>>;

fn lines() -> Vec<InvoiceLine> {
    vec![
        InvoiceLine { description: "Widget".into(), quantity: 2, unit_cents: 500 },
        InvoiceLine { description: "Gadget".into(), quantity: 1, unit_cents: 250 },
    ]
}

const CANNED: InvoiceTotals = InvoiceTotals { subtotal: 111, tax: 22, total: 133 };

struct FakeCalculator {
    events: Events,
    fail: bool,
}

impl TotalsCalculator for FakeCalculator {
    fn calculate(&self, lines: &[InvoiceLine], tax_bps: u64) -> Result<InvoiceTotals, InvoiceError> {
        self.events.borrow_mut().push(format!("calculate:{}:{}", lines.len(), tax_bps));
        if self.fail {
            Err(InvoiceError::InvalidLines)
        } else {
            Ok(CANNED)
        }
    }
}

struct FakeRenderer {
    events: Events,
    seen: Rc<RefCell<Option<InvoiceTotals>>>,
}

impl InvoiceRenderer for FakeRenderer {
    fn render(&self, invoice_id: &str, _lines: &[InvoiceLine], totals: &InvoiceTotals) -> String {
        self.events.borrow_mut().push(format!("render:{invoice_id}"));
        *self.seen.borrow_mut() = Some(*totals);
        "RENDERED".to_string()
    }
}

struct MemoryRepository {
    events: Events,
    data: Rc<RefCell<HashMap<String, String>>>,
}

impl InvoiceRepository for MemoryRepository {
    fn save(&self, invoice_id: &str, text: &str) {
        self.events.borrow_mut().push(format!("save:{invoice_id}:{text}"));
        self.data.borrow_mut().insert(invoice_id.to_string(), text.to_string());
    }
    fn find(&self, invoice_id: &str) -> Option<String> {
        self.data.borrow().get(invoice_id).cloned()
    }
}

struct Fakes {
    events: Events,
    seen: Rc<RefCell<Option<InvoiceTotals>>>,
    data: Rc<RefCell<HashMap<String, String>>>,
    service: InvoiceService,
}

fn fakes(fail: bool) -> Fakes {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::new(RefCell::new(None));
    let data = Rc::new(RefCell::new(HashMap::new()));
    let service = InvoiceService::new(
        Box::new(FakeCalculator { events: events.clone(), fail }),
        Box::new(FakeRenderer { events: events.clone(), seen: seen.clone() }),
        Box::new(MemoryRepository { events: events.clone(), data: data.clone() }),
    );
    Fakes { events, seen, data, service }
}

#[test]
fn should_calculate_totals_without_knowing_about_layout_or_storage() {
    let calc = StandardCalculator::new();

    assert_eq!(calc.calculate(&lines(), 1800).unwrap(), InvoiceTotals { subtotal: 1250, tax: 225, total: 1475 });
    let one = [InvoiceLine { description: "x".into(), quantity: 1, unit_cents: 1005 }];
    assert_eq!(calc.calculate(&one, 1800).unwrap().tax, 181);
}

#[test]
fn should_reject_invalid_input_in_the_calculator() {
    let calc = StandardCalculator::new();
    let zero_qty = [InvoiceLine { description: "x".into(), quantity: 0, unit_cents: 10 }];

    assert_eq!(calc.calculate(&[], 1800), Err(InvoiceError::InvalidLines));
    assert_eq!(calc.calculate(&zero_qty, 1800), Err(InvoiceError::InvalidLines));
}

#[test]
fn should_lay_out_an_invoice_from_totals_it_is_given() {
    let text = TextRenderer::new().render("A-1", &lines(), &InvoiceTotals { subtotal: 1250, tax: 225, total: 1475 });

    assert_eq!(
        text,
        ["INVOICE A-1", "Widget x2 @ 5.00 = 10.00", "Gadget x1 @ 2.50 = 2.50", "SUBTOTAL 12.50", "TAX 2.25", "TOTAL 14.75"]
            .join("\n")
    );
}

#[test]
fn should_print_the_given_totals_even_when_they_do_not_match_the_lines() {
    // A renderer that adds things up itself would print 12.50 here.
    let text = TextRenderer::new().render("A-2", &lines(), &InvoiceTotals { subtotal: 999, tax: 1, total: 1000 });

    assert!(text.contains("SUBTOTAL 9.99"));
    assert!(text.contains("TAX 0.01"));
    assert!(text.contains("TOTAL 10.00"));
}

#[test]
fn should_only_coordinate_when_issuing_an_invoice() {
    let f = fakes(false);

    let result = f.service.issue("A-1", &lines(), 1800).unwrap();

    assert_eq!(result, "RENDERED");
    assert_eq!(*f.seen.borrow(), Some(CANNED));
    assert_eq!(*f.events.borrow(), vec!["calculate:2:1800", "render:A-1", "save:A-1:RENDERED"]);
    assert_eq!(f.service.lookup("A-1").as_deref(), Some("RENDERED"));
    assert_eq!(f.service.lookup("nope"), None);
}

#[test]
fn should_touch_neither_renderer_nor_repository_when_the_calculation_fails() {
    let f = fakes(true);

    assert_eq!(f.service.issue("A-1", &lines(), 1800), Err(InvoiceError::InvalidLines));

    assert_eq!(*f.events.borrow(), vec!["calculate:2:1800"]);
    assert!(f.data.borrow().is_empty());
}

#[test]
fn should_refuse_a_blank_id_before_calling_anyone() {
    let f = fakes(false);

    assert_eq!(f.service.issue(" ", &lines(), 1800), Err(InvoiceError::InvalidId));

    assert!(f.events.borrow().is_empty());
}

#[test]
fn should_work_end_to_end_with_the_real_parts() {
    let events: Events = Rc::new(RefCell::new(Vec::new()));
    let data = Rc::new(RefCell::new(HashMap::new()));
    let service = InvoiceService::new(
        Box::new(StandardCalculator::new()),
        Box::new(TextRenderer::new()),
        Box::new(MemoryRepository { events, data: data.clone() }),
    );

    let text = service.issue("A-1", &lines(), 1800).unwrap();

    assert!(text.ends_with("SUBTOTAL 12.50\nTAX 2.25\nTOTAL 14.75"));
    assert_eq!(data.borrow().get("A-1"), Some(&text));
}

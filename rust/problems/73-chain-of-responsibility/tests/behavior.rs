use p73_chain_of_responsibility::contract::*;
use p73_chain_of_responsibility::{ApprovalChain, Director, Manager, PolicyGuard, TeamLead};
use std::cell::RefCell;
use std::rc::Rc;

fn claim(cents: i64) -> ExpenseRequest {
    ExpenseRequest { employee: "ann".into(), amount_cents: cents, category: "meals".into() }
}

fn categorised(cents: i64, category: &str) -> ExpenseRequest {
    ExpenseRequest { employee: "ann".into(), amount_cents: cents, category: category.into() }
}

/// A handler written in the test: logs every request it sees and never decides.
struct Audit(Rc<RefCell<Vec<String>>>);

impl Decider for Audit {
    fn decide(&self, request: &ExpenseRequest) -> Option<String> {
        self.0.borrow_mut().push(format!("{}:{}", request.employee, request.amount_cents));
        None
    }
}

fn full_chain() -> ApprovalChain {
    ApprovalChain::new().then(Box::new(TeamLead::new())).then(Box::new(Manager::new())).then(Box::new(Director::new()))
}

#[test]
fn should_let_the_team_lead_approve_small_claims_up_to_the_limit() {
    let chain = ApprovalChain::new().then(Box::new(TeamLead::new()));

    assert_eq!(chain.handle(&claim(1)), "APPROVED:team lead");
    assert_eq!(chain.handle(&claim(50_000)), "APPROVED:team lead");
    assert_eq!(chain.handle(&claim(50_001)), "REJECTED:no approver for 50001");
}

#[test]
fn should_pass_claims_up_the_chain_until_someone_can_approve() {
    let chain = full_chain();

    assert_eq!(chain.handle(&claim(50_000)), "APPROVED:team lead");
    assert_eq!(chain.handle(&claim(50_001)), "APPROVED:manager");
    assert_eq!(chain.handle(&claim(500_000)), "APPROVED:manager");
    assert_eq!(chain.handle(&claim(500_001)), "APPROVED:director");
    assert_eq!(chain.handle(&claim(5_000_000)), "APPROVED:director");
    assert_eq!(chain.handle(&claim(5_000_001)), "REJECTED:no approver for 5000001");
}

#[test]
fn should_send_travel_claims_past_the_team_lead_even_when_they_are_small() {
    let chain = full_chain();

    assert_eq!(chain.handle(&categorised(10_000, "Travel")), "APPROVED:manager");
    assert_eq!(chain.handle(&categorised(10_000, "meals")), "APPROVED:team lead");
}

#[test]
fn should_let_the_order_of_the_chain_change_who_answers() {
    let chain = ApprovalChain::new().then(Box::new(Director::new())).then(Box::new(TeamLead::new()));

    assert_eq!(chain.handle(&claim(100)), "APPROVED:director");
}

#[test]
fn should_stop_at_the_guard_and_never_ask_the_rest_of_the_chain() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let chain = ApprovalChain::new()
        .then(Box::new(PolicyGuard::new()))
        .then(Box::new(Audit(seen.clone())))
        .then(Box::new(TeamLead::new()));

    assert_eq!(chain.handle(&categorised(100, "Gambling")), "REJECTED:policy violation");
    assert_eq!(chain.handle(&claim(0)), "REJECTED:policy violation");
    assert_eq!(chain.handle(&claim(-5)), "REJECTED:policy violation");
    assert!(seen.borrow().is_empty());
    assert_eq!(chain.handle(&claim(100)), "APPROVED:team lead");
    assert_eq!(*seen.borrow(), vec!["ann:100"]);
}

#[test]
fn should_accept_a_handler_it_has_never_seen_anywhere_in_the_chain() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let chain = ApprovalChain::new().then(Box::new(Audit(seen.clone()))).then(Box::new(Manager::new())).then(Box::new(Director::new()));
    let bob = ExpenseRequest { employee: "bob".into(), amount_cents: 400_000, category: "gear".into() };
    let cy = ExpenseRequest { employee: "cy".into(), amount_cents: 9_000_000, category: "gear".into() };

    assert_eq!(chain.handle(&bob), "APPROVED:manager");
    assert_eq!(chain.handle(&cy), "REJECTED:no approver for 9000000");
    assert_eq!(*seen.borrow(), vec!["bob:400000", "cy:9000000"]);
}

#[test]
fn should_reject_everything_when_the_chain_is_empty() {
    assert_eq!(ApprovalChain::new().handle(&claim(100)), "REJECTED:no approver for 100");
}

#[test]
fn should_decide_with_each_handler_on_its_own_too() {
    assert_eq!(TeamLead::new().decide(&claim(100)).as_deref(), Some("APPROVED:team lead"));
    assert_eq!(TeamLead::new().decide(&claim(100_000)), None);
    assert_eq!(Manager::new().decide(&claim(100_000)).as_deref(), Some("APPROVED:manager"));
    assert_eq!(Manager::new().decide(&claim(600_000)), None);
    assert_eq!(Director::new().decide(&claim(600_000)).as_deref(), Some("APPROVED:director"));
    assert_eq!(PolicyGuard::new().decide(&claim(100)), None);
    assert_eq!(PolicyGuard::new().decide(&claim(0)).as_deref(), Some("REJECTED:policy violation"));
}

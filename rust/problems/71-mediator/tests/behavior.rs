use p71_mediator::contract::*;
use p71_mediator::ControlTower;
use std::cell::RefCell;
use std::rc::Rc;

type Calls = Rc<RefCell<Vec<String>>>;

/// An aircraft written in the test: records every instruction it is given.
struct Plane {
    id: String,
    calls: Calls,
}

impl Aircraft for Plane {
    fn id(&self) -> &str {
        &self.id
    }
    fn on_cleared_to_land(&self) {
        self.calls.borrow_mut().push(format!("{}:CLEARED", self.id));
    }
    fn on_hold(&self, position: usize) {
        self.calls.borrow_mut().push(format!("{}:HOLD {position}", self.id));
    }
}

fn plane(id: &str, calls: &Calls) -> Rc<dyn Aircraft> {
    Rc::new(Plane { id: id.to_string(), calls: calls.clone() })
}

fn calls() -> Calls {
    Rc::new(RefCell::new(Vec::new()))
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_clear_the_first_aircraft_at_once_when_the_runway_is_free() {
    let log = calls();
    let mut tower = ControlTower::new();

    tower.request_landing(plane("A", &log)).unwrap();

    assert_eq!(*log.borrow(), strs(&["A:CLEARED"]));
    assert_eq!(tower.on_runway().as_deref(), Some("A"));
    assert!(tower.queue().is_empty());
}

#[test]
fn should_queue_later_aircraft_and_tell_them_their_position() {
    let log = calls();
    let mut tower = ControlTower::new();
    tower.request_landing(plane("A", &log)).unwrap();
    tower.request_landing(plane("B", &log)).unwrap();
    tower.request_landing(plane("C", &log)).unwrap();

    assert_eq!(*log.borrow(), strs(&["A:CLEARED", "B:HOLD 1", "C:HOLD 2"]));
    assert_eq!(tower.queue(), strs(&["B", "C"]));
}

#[test]
fn should_clear_the_next_aircraft_when_the_runway_is_freed_and_renumber_the_rest() {
    let log = calls();
    let mut tower = ControlTower::new();
    for id in ["A", "B", "C", "D"] {
        tower.request_landing(plane(id, &log)).unwrap();
    }
    log.borrow_mut().clear();

    tower.landed("A").unwrap();

    assert_eq!(*log.borrow(), strs(&["B:CLEARED", "C:HOLD 1", "D:HOLD 2"]));
    assert_eq!(tower.on_runway().as_deref(), Some("B"));
    assert_eq!(tower.queue(), strs(&["C", "D"]));
}

#[test]
fn should_free_the_runway_when_the_cleared_aircraft_diverts() {
    let log = calls();
    let mut tower = ControlTower::new();
    tower.request_landing(plane("A", &log)).unwrap();
    tower.request_landing(plane("B", &log)).unwrap();
    log.borrow_mut().clear();

    assert!(tower.cancel("A"));

    assert_eq!(*log.borrow(), strs(&["B:CLEARED"]));
    assert_eq!(tower.on_runway().as_deref(), Some("B"));
    assert!(!tower.cancel("A"));
}

#[test]
fn should_remove_a_waiting_aircraft_and_renumber_only_those_behind_it() {
    let log = calls();
    let mut tower = ControlTower::new();
    for id in ["A", "B", "C", "D"] {
        tower.request_landing(plane(id, &log)).unwrap();
    }
    log.borrow_mut().clear();

    assert!(tower.cancel("B"));

    assert_eq!(*log.borrow(), strs(&["C:HOLD 1", "D:HOLD 2"]));
    assert_eq!(tower.queue(), strs(&["C", "D"]));
}

#[test]
fn should_let_an_emergency_overtake_the_queue() {
    let log = calls();
    let mut tower = ControlTower::new();
    for id in ["A", "B", "C", "D"] {
        tower.request_landing(plane(id, &log)).unwrap();
    }
    log.borrow_mut().clear();

    tower.declare_emergency("D").unwrap();

    assert_eq!(*log.borrow(), strs(&["D:HOLD 1", "B:HOLD 2", "C:HOLD 3"]));
    assert_eq!(tower.queue(), strs(&["D", "B", "C"]));
    log.borrow_mut().clear();
    tower.declare_emergency("D").unwrap();
    assert!(log.borrow().is_empty());
}

#[test]
fn should_reject_requests_that_make_no_sense() {
    let log = calls();
    let mut tower = ControlTower::new();
    tower.request_landing(plane("A", &log)).unwrap();
    tower.request_landing(plane("B", &log)).unwrap();

    assert_eq!(tower.request_landing(plane("A", &log)), Err(TowerError::Duplicate));
    assert_eq!(tower.request_landing(plane("B", &log)), Err(TowerError::Duplicate));
    assert_eq!(tower.landed("B"), Err(TowerError::NotOnRunway));
    assert_eq!(tower.landed("Z"), Err(TowerError::NotOnRunway));
    assert_eq!(tower.declare_emergency("A"), Err(TowerError::Unknown));
    assert_eq!(tower.declare_emergency("Z"), Err(TowerError::Unknown));
    assert_eq!(tower.on_runway().as_deref(), Some("A"));
    assert_eq!(tower.queue(), strs(&["B"]));
}

#[test]
fn should_return_to_an_empty_airspace_when_everyone_has_landed() {
    let log = calls();
    let mut tower = ControlTower::new();
    tower.request_landing(plane("A", &log)).unwrap();
    tower.request_landing(plane("B", &log)).unwrap();

    tower.landed("A").unwrap();
    tower.landed("B").unwrap();

    assert_eq!(tower.on_runway(), None);
    assert!(tower.queue().is_empty());
    tower.request_landing(plane("C", &log)).unwrap();
    assert_eq!(log.borrow().last().unwrap(), "C:CLEARED");
}

//! Mediator — `ControlTower` coordinates the aircraft; aircraft know nothing about each other.
//!
//! Rust adaptation: the tower keeps the aircraft as `Rc<dyn Aircraft>` and identifies them by `id()`. (The Java
//! version also has a `PassengerJet`; here the aircraft are written in the tests.)
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::rc::Rc;

/// The mediator.
///
/// Notification rules (the tests check the exact calls):
/// - runway free on request: `on_cleared_to_land()`; otherwise the aircraft joins the queue and gets `on_hold(position)`;
/// - when the runway is freed (`landed`, or `cancel` of the aircraft on the runway) and somebody is waiting: the first
///   in the queue moves to the runway and gets `on_cleared_to_land()`, THEN every aircraft still waiting gets
///   `on_hold(new_position)`;
/// - cancelling a waiting aircraft removes it and tells each aircraft BEHIND it its new position;
/// - an emergency moves the aircraft to the front: first it gets `on_hold(1)`, then each aircraft it overtook gets
///   `on_hold(new_position)`, in queue order.
pub struct ControlTower;

impl ControlTower {
    pub fn new() -> ControlTower {
        todo!()
    }

    /// Duplicate (on the runway or already queued, compared by id) -> `Err(TowerError::Duplicate)`.
    pub fn request_landing(&mut self, aircraft: Rc<dyn Aircraft>) -> Result<(), TowerError> {
        todo!()
    }

    /// `id` must be the aircraft on the runway, otherwise `Err(TowerError::NotOnRunway)`.
    pub fn landed(&mut self, id: &str) -> Result<(), TowerError> {
        todo!()
    }

    /// A diversion: on the runway (like `landed`) or waiting. Returns false if the id is unknown.
    pub fn cancel(&mut self, id: &str) -> bool {
        todo!()
    }

    /// Moves a waiting aircraft to the front. Unknown or already cleared -> `Err(TowerError::Unknown)`. Already first
    /// in the queue -> nothing changes.
    pub fn declare_emergency(&mut self, id: &str) -> Result<(), TowerError> {
        todo!()
    }

    /// Id of the aircraft currently cleared to use the runway.
    pub fn on_runway(&self) -> Option<String> {
        todo!()
    }

    /// Ids of the waiting aircraft in landing order.
    pub fn queue(&self) -> Vec<String> {
        todo!()
    }
}

//! Mediator — `ControlTower` coordinates the aircraft.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::rc::Rc;

/// Coordinates the aircraft landing on one runway.
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

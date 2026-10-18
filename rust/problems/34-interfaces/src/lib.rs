//! Interfaces — implement the sensors and the hub. The `Sensor` trait is in `contract.rs`.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Unit "C". `read` fails with `SensorFault` when the probe returns less than -273.15.
pub struct TemperatureSensor;

impl TemperatureSensor {
    pub fn new(id: &str, probe: Box<dyn Fn() -> f64>) -> Self {
        todo!()
    }
}

impl Sensor for TemperatureSensor {
    fn id(&self) -> &str {
        todo!()
    }
    fn unit(&self) -> &str {
        todo!()
    }
    fn read(&self) -> Result<f64, SensorFault> {
        todo!()
    }
}

/// Unit "%". `read` fails with `SensorFault` when the probe is outside 0..=100.
pub struct HumiditySensor;

impl HumiditySensor {
    pub fn new(id: &str, probe: Box<dyn Fn() -> f64>) -> Self {
        todo!()
    }
}

impl Sensor for HumiditySensor {
    fn id(&self) -> &str {
        todo!()
    }
    fn unit(&self) -> &str {
        todo!()
    }
    fn read(&self) -> Result<f64, SensorFault> {
        todo!()
    }
}

/// Works with any `Sensor`, including types it has never seen.
pub struct SensorHub;

impl SensorHub {
    pub fn new() -> Self {
        todo!()
    }

    /// `Err(HubError::DuplicateId)` if the id is taken; nothing changes then.
    pub fn register(&mut self, sensor: Box<dyn Sensor>) -> Result<(), HubError> {
        todo!()
    }

    /// True if a sensor with that id was removed.
    pub fn unregister(&mut self, id: &str) -> bool {
        todo!()
    }

    /// Ids in registration order.
    pub fn ids(&self) -> Vec<String> {
        todo!()
    }

    /// (id, reading) in registration order; faulty sensors are left out.
    pub fn read_all(&mut self) -> Vec<(String, f64)> {
        todo!()
    }

    /// Ids that were faulty during the last `read_all` (empty before the first call).
    pub fn last_faults(&self) -> Vec<String> {
        todo!()
    }

    /// Mean of the healthy sensors with this unit, `None` if there are none.
    pub fn average(&self, unit: &str) -> Option<f64> {
        todo!()
    }
}

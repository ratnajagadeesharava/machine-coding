//! Interfaces.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Temperature sensor.
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

/// Humidity sensor.
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

/// Collects readings from registered sensors.
pub struct SensorHub;

impl SensorHub {
    pub fn new() -> Self {
        todo!()
    }

    pub fn register(&mut self, sensor: Box<dyn Sensor>) -> Result<(), HubError> {
        todo!()
    }

    pub fn unregister(&mut self, id: &str) -> bool {
        todo!()
    }

    pub fn ids(&self) -> Vec<String> {
        todo!()
    }

    pub fn read_all(&mut self) -> Vec<(String, f64)> {
        todo!()
    }

    pub fn last_faults(&self) -> Vec<String> {
        todo!()
    }

    pub fn average(&self, unit: &str) -> Option<f64> {
        todo!()
    }
}

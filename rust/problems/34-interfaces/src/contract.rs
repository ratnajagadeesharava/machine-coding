//! Provided contract: anything that can report a measurement is a `Sensor`.

#[derive(Debug, Clone, PartialEq)]
pub struct SensorFault(pub String);

pub trait Sensor {
    fn id(&self) -> &str;
    /// Unit of the reading, e.g. "C" or "%".
    fn unit(&self) -> &str;
    /// Current reading, or a fault.
    fn read(&self) -> Result<f64, SensorFault>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubError {
    DuplicateId,
}

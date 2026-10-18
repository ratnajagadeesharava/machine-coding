//! OCP — extend by adding a `FeePolicy`, never by editing the calculator.

pub mod calculator;
pub mod contract;
pub mod policies;

pub use calculator::FeeCalculator;
pub use policies::{CarPolicy, MotorcyclePolicy};

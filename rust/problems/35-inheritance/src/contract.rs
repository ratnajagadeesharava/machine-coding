//! Provided: the error type used by the payroll types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayrollError {
    InvalidName,
    NegativeSalary,
    InvalidLevel,
}

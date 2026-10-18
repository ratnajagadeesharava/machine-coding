//! Inheritance, the Rust way: shared data lives in `Staff`, shared behaviour lives in the default
//! methods of the `Payable` trait, and each kind of employee wraps a `Staff` and overrides what differs.
#![allow(unused_variables)]

pub mod contract;

use contract::PayrollError;

/// The data every employee has: a name and a base salary.
pub struct Staff;

impl Staff {
    /// Blank name -> `InvalidName`; negative salary -> `NegativeSalary`.
    pub fn new(name: &str, base_monthly: i64) -> Result<Staff, PayrollError> {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }

    pub fn base(&self) -> u64 {
        todo!()
    }
}

/// The "base class". Implementors only have to say where their `Staff` is; everything else has a default
/// body that you write here once.
pub trait Payable {
    fn staff(&self) -> &Staff;

    /// Default "Employee".
    fn role(&self) -> String {
        todo!()
    }

    /// Default: the base salary.
    fn monthly_pay(&self) -> u64 {
        todo!()
    }

    /// "{role}: {name} earns {monthly_pay}". Implementors must NOT override this.
    fn describe(&self) -> String {
        todo!()
    }
}

/// Pay = base * (100 + 20 * (level - 1)) / 100. Level must be 1..=3 (`InvalidLevel`). Role "Engineer".
pub struct Engineer;

impl Engineer {
    pub fn new(staff: Staff, level: u32) -> Result<Engineer, PayrollError> {
        todo!()
    }
}

impl Payable for Engineer {
    fn staff(&self) -> &Staff {
        todo!()
    }
}

/// Pay = base / 2. Role "Intern".
pub struct Intern;

impl Intern {
    pub fn new(staff: Staff) -> Intern {
        todo!()
    }
}

impl Payable for Intern {
    fn staff(&self) -> &Staff {
        todo!()
    }
}

/// Pay = base + (sum of the reports' pay) / 20. Role "Manager". Reports may be managers.
pub struct Manager;

impl Manager {
    pub fn new(staff: Staff, reports: Vec<Box<dyn Payable>>) -> Manager {
        todo!()
    }
}

impl Payable for Manager {
    fn staff(&self) -> &Staff {
        todo!()
    }
}

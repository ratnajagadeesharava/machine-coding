//! Inheritance.
#![allow(unused_variables)]

pub mod contract;

use contract::PayrollError;

/// Name and base salary.
pub struct Staff;

impl Staff {
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

/// Payable staff member.
pub trait Payable {
    fn staff(&self) -> &Staff;

    fn role(&self) -> String {
        todo!()
    }

    fn monthly_pay(&self) -> u64 {
        todo!()
    }

    fn describe(&self) -> String {
        todo!()
    }
}

/// Engineer.
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

/// Intern.
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

/// Manager.
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

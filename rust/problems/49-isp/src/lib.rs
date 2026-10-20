//! Office devices and jobs.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// A basic printer.
pub struct BasicPrinter;

impl BasicPrinter {
    pub fn new() -> BasicPrinter {
        todo!()
    }
}

impl Printer for BasicPrinter {
    fn print(&self, document: &str) -> Result<String, DeviceError> {
        todo!()
    }
}

/// A flatbed scanner.
pub struct FlatbedScanner;

impl FlatbedScanner {
    pub fn new() -> FlatbedScanner {
        todo!()
    }
}

impl Scanner for FlatbedScanner {
    fn scan(&mut self) -> String {
        todo!()
    }
}

/// All-in-one office machine.
pub struct AllInOne;

impl AllInOne {
    pub fn new() -> AllInOne {
        todo!()
    }
}

impl Printer for AllInOne {
    fn print(&self, document: &str) -> Result<String, DeviceError> {
        todo!()
    }
}

impl Scanner for AllInOne {
    fn scan(&mut self) -> String {
        todo!()
    }
}

impl FaxMachine for AllInOne {
    fn fax(&mut self, number: &str, document: &str) -> Result<bool, DeviceError> {
        todo!()
    }
}

/// Print job.
pub fn print_job(printer: &dyn Printer, documents: &[&str]) -> Result<Vec<String>, DeviceError> {
    todo!()
}

/// Scan job.
pub fn scan_job(scanner: &mut dyn Scanner, pages: u32) -> Result<Vec<String>, DeviceError> {
    todo!()
}

/// Fax job.
pub fn fax_job(machine: &mut dyn FaxMachine, number: &str, document: &str) -> Result<bool, DeviceError> {
    todo!()
}

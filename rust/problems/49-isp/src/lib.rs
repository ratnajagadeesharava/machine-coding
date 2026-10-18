//! ISP — small traits, devices implement only what they can do, jobs ask for only what they need.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// A printer and nothing else: it implements ONLY `Printer`.
/// `print` returns "PRINTED: " + document; a blank document -> `Err(BlankDocument)`.
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

/// A scanner and nothing else: it implements ONLY `Scanner`. Pages are "SCAN-1", "SCAN-2", ...
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

/// Does everything. It implements the three small traits directly.
/// print: "PRINTED: " + document (blank -> `Err(BlankDocument)`). scan: "SCAN-1", "SCAN-2", ...
/// fax: a blank document -> `Err(BlankDocument)`; the number must be an optional '+' followed by 7 to 15
/// digits, otherwise `Ok(false)`; a good number gives `Ok(true)`.
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

/// Prints every document in order and returns the outputs. Asks for a `Printer`, nothing bigger.
pub fn print_job(printer: &dyn Printer, documents: &[&str]) -> Result<Vec<String>, DeviceError> {
    todo!()
}

/// Scans `pages` pages in order (`pages == 0` -> `Err(InvalidPages)`). Asks for a `Scanner`, nothing bigger.
pub fn scan_job(scanner: &mut dyn Scanner, pages: u32) -> Result<Vec<String>, DeviceError> {
    todo!()
}

/// Sends one fax. Asks for a `FaxMachine`, nothing bigger.
pub fn fax_job(machine: &mut dyn FaxMachine, number: &str, document: &str) -> Result<bool, DeviceError> {
    todo!()
}

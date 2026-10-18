use p49_isp::contract::*;
use p49_isp::{fax_job, print_job, scan_job, AllInOne, BasicPrinter, FlatbedScanner};
use std::cell::RefCell;

/// Test devices that can do exactly ONE thing: the jobs must accept them.
struct PrinterOnly {
    seen: RefCell<Vec<String>>,
}

impl Printer for PrinterOnly {
    fn print(&self, document: &str) -> Result<String, DeviceError> {
        self.seen.borrow_mut().push(document.to_string());
        Ok(format!("fake:{document}"))
    }
}

struct ScannerOnly(u32);

impl Scanner for ScannerOnly {
    fn scan(&mut self) -> String {
        self.0 += 1;
        format!("page{}", self.0)
    }
}

struct FaxOnly {
    sent: Vec<String>,
}

impl FaxMachine for FaxOnly {
    fn fax(&mut self, number: &str, document: &str) -> Result<bool, DeviceError> {
        self.sent.push(format!("{number}:{document}"));
        Ok(true)
    }
}

#[test]
fn should_print_with_a_basic_printer() {
    let printer = BasicPrinter::new();

    assert_eq!(printer.print("report").unwrap(), "PRINTED: report");
    assert_eq!(printer.print(" "), Err(DeviceError::BlankDocument));
}

#[test]
fn should_number_scanned_pages_from_one() {
    let mut scanner = FlatbedScanner::new();

    assert_eq!(scanner.scan(), "SCAN-1");
    assert_eq!(scanner.scan(), "SCAN-2");
}

#[test]
fn should_let_an_all_in_one_be_used_as_each_of_the_three_devices() {
    let mut machine = AllInOne::new();

    assert_eq!((&machine as &dyn Printer).print("a").unwrap(), "PRINTED: a");
    assert_eq!((&mut machine as &mut dyn Scanner).scan(), "SCAN-1");
    let fax: &mut dyn FaxMachine = &mut machine;
    assert_eq!(fax.fax("+4915112345678", "a"), Ok(true));
    assert_eq!(fax.fax("12", "a"), Ok(false));
    assert_eq!(fax.fax("not-a-number", "a"), Ok(false));
    assert_eq!(fax.fax("5551234", "a"), Ok(true));
    assert_eq!(fax.fax("5551234", ""), Err(DeviceError::BlankDocument));
}

#[test]
fn should_run_a_print_job_on_any_printer_even_one_that_can_do_nothing_else() {
    let printer_only = PrinterOnly { seen: RefCell::new(Vec::new()) };

    let out = print_job(&printer_only, &["a", "b"]).unwrap();

    assert_eq!(out, vec!["fake:a", "fake:b"]);
    assert_eq!(*printer_only.seen.borrow(), vec!["a", "b"]);
    assert_eq!(print_job(&BasicPrinter::new(), &["x"]).unwrap(), vec!["PRINTED: x"]);
}

#[test]
fn should_run_a_scan_job_on_any_scanner_even_one_that_can_do_nothing_else() {
    let mut scanner_only = ScannerOnly(0);

    assert_eq!(scan_job(&mut scanner_only, 3).unwrap(), vec!["page1", "page2", "page3"]);
    assert_eq!(scan_job(&mut AllInOne::new(), 2).unwrap(), vec!["SCAN-1", "SCAN-2"]);
}

#[test]
fn should_run_a_fax_job_on_any_fax_machine_even_one_that_can_do_nothing_else() {
    let mut fax_only = FaxOnly { sent: Vec::new() };

    assert_eq!(fax_job(&mut fax_only, "5551234", "contract"), Ok(true));
    assert_eq!(fax_only.sent, vec!["5551234:contract"]);
    assert_eq!(fax_job(&mut AllInOne::new(), "bad", "contract"), Ok(false));
}

#[test]
fn should_reject_a_scan_job_with_no_pages_and_handle_an_empty_print_job() {
    assert_eq!(scan_job(&mut FlatbedScanner::new(), 0), Err(DeviceError::InvalidPages));
    assert_eq!(print_job(&BasicPrinter::new(), &[]).unwrap(), Vec::<String>::new());
    assert_eq!(print_job(&BasicPrinter::new(), &["ok", " "]), Err(DeviceError::BlankDocument));
}

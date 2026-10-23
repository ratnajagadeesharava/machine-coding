//! Invoice issuing.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// subtotal = sum of quantity * unit_cents; tax = subtotal * tax_bps / 10000 rounded half up;
/// total = subtotal + tax. Empty lines or a quantity of 0 -> `InvalidLines`.
pub struct StandardCalculator;

impl StandardCalculator {
    pub fn new() -> StandardCalculator {
        todo!()
    }
}

impl TotalsCalculator for StandardCalculator {
    fn calculate(&self, lines: &[InvoiceLine], tax_bps: u64) -> Result<InvoiceTotals, InvoiceError> {
        todo!()
    }
}

/// Lays out an invoice exactly like this (lines joined with "\n", no trailing newline):
///
/// ```text
/// INVOICE A-1
/// Widget x2 @ 5.00 = 10.00
/// Gadget x1 @ 2.50 = 2.50
/// SUBTOTAL 12.50
/// TAX 2.25
/// TOTAL 14.75
/// ```
///
/// It prints the totals it is given.
pub struct TextRenderer;

impl TextRenderer {
    pub fn new() -> TextRenderer {
        todo!()
    }
}

impl InvoiceRenderer for TextRenderer {
    fn render(&self, invoice_id: &str, lines: &[InvoiceLine], totals: &InvoiceTotals) -> String {
        todo!()
    }
}

/// Issues and looks up invoices.
pub struct InvoiceService;

impl InvoiceService {
    pub fn new(
        calculator: Box<dyn TotalsCalculator>,
        renderer: Box<dyn InvoiceRenderer>,
        repository: Box<dyn InvoiceRepository>,
    ) -> InvoiceService {
        todo!()
    }

    /// Blank id -> `Err(InvalidId)` before anything is called. Otherwise calculate -> render -> save, in that
    /// order, returning the rendered text. If the calculator fails, its error is returned and neither the
    /// renderer nor the repository is touched.
    pub fn issue(&self, invoice_id: &str, lines: &[InvoiceLine], tax_bps: u64) -> Result<String, InvoiceError> {
        todo!()
    }

    pub fn lookup(&self, invoice_id: &str) -> Option<String> {
        todo!()
    }
}

//! Provided: value types, the three single-purpose traits and the error type. Amounts are in cents.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceLine {
    pub description: String,
    pub quantity: u32,
    pub unit_cents: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvoiceTotals {
    pub subtotal: u64,
    pub tax: u64,
    pub total: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvoiceError {
    /// Empty lines, quantity 0.
    InvalidLines,
    /// Blank invoice id.
    InvalidId,
}

pub trait TotalsCalculator {
    fn calculate(&self, lines: &[InvoiceLine], tax_bps: u64) -> Result<InvoiceTotals, InvoiceError>;
}

pub trait InvoiceRenderer {
    fn render(&self, invoice_id: &str, lines: &[InvoiceLine], totals: &InvoiceTotals) -> String;
}

pub trait InvoiceRepository {
    fn save(&self, invoice_id: &str, text: &str);
    fn find(&self, invoice_id: &str) -> Option<String>;
}

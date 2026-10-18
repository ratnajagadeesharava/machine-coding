//! DRY — one pricing path, one money formatter, one validator.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// discount(net): `net * discount_bps / 10000` when `net >= bulk_threshold`, else 0.
/// tax(taxable): `taxable * tax_bps / 10000`. Both round HALF UP to whole cents (1005 * 1800 bps = 180.9 -> 181).
pub struct StandardRules;

impl StandardRules {
    pub fn new(tax_bps: i64, bulk_threshold: i64, bulk_discount_bps: i64) -> StandardRules {
        todo!()
    }
}

impl PricingRules for StandardRules {
    fn discount(&self, net: i64) -> i64 {
        todo!()
    }
    fn tax(&self, taxable: i64) -> i64 {
        todo!()
    }
}

/// Produces the three customer documents:
///
/// ```text
/// quote(lines)                "QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00"
/// invoice("INV-1", lines)     "INVOICE #INV-1 net=50.00 discount=0.00 tax=9.00 total=59.00"
/// credit_note("INV-1", lines) "CREDIT NOTE #INV-1 net=50.00 discount=0.00 tax=9.00 total=-59.00"
/// ```
///
/// net = sum of lines; total = net - discount + tax; a credit note prints the total negated. Money prints as
/// major.minor with two digits and a leading '-' when negative. Empty lines or a negative line ->
/// `InvalidLines`; a blank number -> `InvalidNumber` (invoice and credit note).
pub struct Documents;

impl Documents {
    pub fn new(rules: Box<dyn PricingRules>) -> Documents {
        todo!()
    }

    pub fn quote(&self, lines: &[i64]) -> Result<String, DocError> {
        todo!()
    }

    pub fn invoice(&self, number: &str, lines: &[i64]) -> Result<String, DocError> {
        todo!()
    }

    pub fn credit_note(&self, invoice_number: &str, lines: &[i64]) -> Result<String, DocError> {
        todo!()
    }
}

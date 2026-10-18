//! Abstract Factory — each `RegionKit` creates a consistent family of products; the client uses only the traits.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Region "US". Tax 8% (800 bps, half up). Prices like "$12.50". Postal codes: 5 digits, optionally followed by
/// "-" and 4 more digits ("12345", "12345-6789").
pub struct UsKit;

impl RegionKit for UsKit {
    fn region(&self) -> &str {
        todo!()
    }
    fn tax_calculator(&self) -> Box<dyn TaxCalculator> {
        todo!()
    }
    fn price_formatter(&self) -> Box<dyn PriceFormatter> {
        todo!()
    }
    fn address_validator(&self) -> Box<dyn AddressValidator> {
        todo!()
    }
}

/// Region "EU". Tax 19% (1900 bps, half up). Prices like "12,50 EUR" (decimal comma). Postal codes: exactly 5 digits.
pub struct EuKit;

impl RegionKit for EuKit {
    fn region(&self) -> &str {
        todo!()
    }
    fn tax_calculator(&self) -> Box<dyn TaxCalculator> {
        todo!()
    }
    fn price_formatter(&self) -> Box<dyn PriceFormatter> {
        todo!()
    }
    fn address_validator(&self) -> Box<dyn AddressValidator> {
        todo!()
    }
}

/// Region "IN". Tax 18% (1800 bps, half up). Prices like "Rs 12.50". Postal codes: exactly 6 digits.
pub struct IndiaKit;

impl RegionKit for IndiaKit {
    fn region(&self) -> &str {
        todo!()
    }
    fn tax_calculator(&self) -> Box<dyn TaxCalculator> {
        todo!()
    }
    fn price_formatter(&self) -> Box<dyn PriceFormatter> {
        todo!()
    }
    fn address_validator(&self) -> Box<dyn AddressValidator> {
        todo!()
    }
}

/// "US", "EU" or "IN" in any letter case; anything else -> `Err(KitError::UnknownRegion)`.
pub fn kit_for_region(code: &str) -> Result<Box<dyn RegionKit>, KitError> {
    todo!()
}

/// The client. Returns `"<region>: net <net>, tax <tax>, total <total>, postal <valid|invalid>"` where every amount
/// is written by the kit's price formatter, tax comes from the kit's tax calculator (on the net amount),
/// total = net + tax, and the postal code is judged by the kit's address validator.
/// Example (US): `"US: net $100.00, tax $8.00, total $108.00, postal valid"`.
pub fn summarize(kit: &dyn RegionKit, net_cents: u64, postal_code: &str) -> String {
    todo!()
}

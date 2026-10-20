//! Abstract Factory.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Region kit for the US.
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

/// Region kit for the EU.
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

/// Region kit for India.
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

/// Region kit lookup.
pub fn kit_for_region(code: &str) -> Result<Box<dyn RegionKit>, KitError> {
    todo!()
}

/// Builds the checkout summary line.
pub fn summarize(kit: &dyn RegionKit, net_cents: u64, postal_code: &str) -> String {
    todo!()
}

//! Provided: three product traits, the abstract factory trait and the error type. Amounts are in cents.

pub trait TaxCalculator {
    /// Tax due on a net amount, rounded half up to whole cents.
    fn tax(&self, net_cents: u64) -> u64;
}

pub trait PriceFormatter {
    /// Human-readable price for an amount in cents.
    fn format(&self, cents: u64) -> String;
}

pub trait AddressValidator {
    fn is_valid_postal_code(&self, postal_code: &str) -> bool;
}

/// The abstract factory: one kit = one consistent FAMILY of products for a region. A client takes all of its tax,
/// price and address behaviour from the same kit, so it can never mix US tax with EU formatting.
pub trait RegionKit {
    /// Region code: "US", "EU" or "IN".
    fn region(&self) -> &str;
    fn tax_calculator(&self) -> Box<dyn TaxCalculator>;
    fn price_formatter(&self) -> Box<dyn PriceFormatter>;
    fn address_validator(&self) -> Box<dyn AddressValidator>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KitError {
    UnknownRegion,
}

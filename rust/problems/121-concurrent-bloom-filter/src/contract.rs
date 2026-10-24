//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BloomError {
    /// blank item (empty after trim)
    InvalidItem,
}

pub trait ConcurrentBloomFilter: Send + Sync {
    /// Adds the item. Returns `true` if the filter changed, i.e. this call flipped at least one bit from 0 to 1;
    /// `false` if every bit of the item was already set (a duplicate, or a new item whose bits all happen to be
    /// set). Items are case-sensitive and are used exactly as given (no trimming, no lower-casing).
    /// `InvalidItem` for a blank (empty after trim) item; the filter is unchanged.
    fn add(&self, item: &str) -> Result<bool, BloomError>;

    /// Adds every item in order and returns how many of those adds returned `true`. All items are validated first:
    /// if any is blank, `InvalidItem` and nothing is added.
    fn add_all(&self, items: &[String]) -> Result<usize, BloomError>;

    /// `false` means the item was definitely never added since the last clear. `true` means it was added, or it is
    /// a false positive. Never changes the filter. `InvalidItem` for a blank item.
    fn might_contain(&self, item: &str) -> Result<bool, BloomError>;

    /// Estimate of the number of distinct items added: round(-(m/k) * ln(1 - X/m)), see the README.
    fn approximate_size(&self) -> u64;

    /// Number of bits currently set to 1 (X in the formula).
    fn set_bits(&self) -> u64;

    /// Size of the bit array, m.
    fn bit_count(&self) -> u64;

    /// Number of bit positions per item, k.
    fn hash_count(&self) -> u32;

    /// Forgets every item: no bit is set afterwards.
    fn clear(&self);
}

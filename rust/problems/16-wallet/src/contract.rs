//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait WalletApi: Send + Sync {
    /// Creates an empty wallet. False if the id is blank or already exists.
    fn create_wallet(&self, wallet_id: &str) -> bool;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT or UNKNOWN_WALLET.
    fn credit(&self, key: &str, wallet_id: &str, amount: i64) -> String;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS.
    fn debit(&self, key: &str, wallet_id: &str, amount: i64) -> String;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS.
    fn transfer(&self, key: &str, from_wallet_id: &str, to_wallet_id: &str, amount: i64) -> String;

    fn balance(&self, wallet_id: &str) -> Option<i64>;

    /// Signed entries such as "+100" and "-30", oldest first. Empty for unknown wallets.
    fn statement(&self, wallet_id: &str) -> Vec<String>;
}

//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create` (epoch milliseconds). Amounts are whole currency units.

#[derive(Debug, PartialEq, Eq)]
pub enum AuctionError {
    /// unknown auction, duplicate id, non-positive amount/starting price, end time not in the future
    InvalidInput,
    /// the auction has ended (or was cancelled), is still open when a result is asked, or cannot be cancelled
    InvalidState,
    /// the seller bids on their own auction, or somebody else cancels it
    NotAllowed,
}

pub type Auc<T> = Result<T, AuctionError>;

pub trait Auctions: Send + Sync {
    /// Opens an auction that runs from now until `end_time_ms`. `starting_price` is the lowest allowed first bid;
    /// `reserve_price` is the price the winner must reach for the item to sell.
    fn create_auction(
        &self,
        id: &str,
        seller: &str,
        starting_price: u64,
        reserve_price: u64,
        end_time_ms: u64,
    ) -> Auc<()>;

    /// Places a bid. Accepted (`Ok(true)`) only if the auction is open, the bidder is not the seller, and the amount is
    /// at least the starting price and strictly higher than the current highest bid. A lower or equal bid is simply
    /// not accepted (`Ok(false)`). A bid accepted in the last 5 minutes before the end moves the end to
    /// "bid time + 5 minutes" (anti-sniping).
    fn place_bid(&self, id: &str, bidder: &str, amount: u64) -> Auc<bool>;

    /// "bidder:amount" of the current highest bid, if any.
    fn highest_bid(&self, id: &str) -> Auc<Option<String>>;

    /// Accepted bids in the order they were accepted, "bidder:amount".
    fn bid_history(&self, id: &str) -> Auc<Vec<String>>;

    /// The (possibly extended) end time.
    fn end_time_of(&self, id: &str) -> Auc<u64>;

    /// Outcome of a finished auction: "SOLD:bidder:amount" if the highest bid reached the reserve,
    /// "RESERVE_NOT_MET", or "NO_BIDS". `InvalidState` while the auction is still open.
    fn result(&self, id: &str) -> Auc<String>;

    /// The seller may cancel an auction that has no bids yet.
    fn cancel(&self, id: &str, seller: &str) -> Auc<()>;

    /// Messages for a user, oldest first: "OUTBID:auctionId:newHighestAmount" each time another bidder takes the
    /// lead from them.
    fn notifications(&self, user: &str) -> Vec<String>;
}

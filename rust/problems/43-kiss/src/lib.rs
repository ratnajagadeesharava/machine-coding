//! Keep it simple: the whole solution is one function.
#![allow(unused_variables)]

/// Late fee in cents for a returned library item.
/// `days_late <= 0` -> 0. Otherwise 25 cents per day (premium members: 10 cents per day),
/// capped at 1000 cents (premium members: 500 cents).
pub fn late_fee(days_late: i32, premium: bool) -> u64 {
    todo!()
}

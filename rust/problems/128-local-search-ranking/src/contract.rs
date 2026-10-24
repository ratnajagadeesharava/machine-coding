//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
#![allow(clippy::too_many_arguments)]

#[derive(Debug, PartialEq, Eq)]
pub enum SearchError {
    /// `add_place` with an id that already exists
    DuplicatePlace,
    /// `update_place` / `remove_place` with an id that does not exist
    UnknownPlace,
    /// blank id / name / category, rating outside 0..=50, price level outside 1..=4, `k <= 0`, `radius < 0`
    InvalidArgument,
}

pub trait PlaceSearch: Send + Sync {
    /// Adds a place. `InvalidArgument` if the id, name or category is blank, `rating_times10` is outside 0..=50
    /// or `price_level` is outside 1..=4; `DuplicatePlace` if the id already exists.
    fn add_place(
        &self,
        place_id: &str,
        name: &str,
        category: &str,
        x: i32,
        y: i32,
        rating_times10: i32,
        price_level: i32,
    ) -> Result<(), SearchError>;

    /// Replaces every attribute of an existing place in one step (same validation as `add_place`).
    /// `UnknownPlace` for an unknown id; a failed update changes nothing.
    fn update_place(
        &self,
        place_id: &str,
        name: &str,
        category: &str,
        x: i32,
        y: i32,
        rating_times10: i32,
        price_level: i32,
    ) -> Result<(), SearchError>;

    /// Removes a place. `UnknownPlace` for an unknown id. The id may be added again later.
    fn remove_place(&self, place_id: &str) -> Result<(), SearchError>;

    /// Ranked place ids, best first, at most `k`. Filters, score and tie-breaks are defined in the README.
    /// `query` and `category` may be `None` (no filter). `InvalidArgument` if `k <= 0` or `radius < 0`.
    fn search(
        &self,
        query: Option<&str>,
        user_x: i32,
        user_y: i32,
        radius: i32,
        category: Option<&str>,
        min_rating_times10: i32,
        max_price_level: i32,
        k: i32,
    ) -> Result<Vec<String>, SearchError>;

    /// The `k` places closest to the user (Manhattan distance), ties by place id ascending.
    /// `InvalidArgument` if `k <= 0`.
    fn nearest(&self, user_x: i32, user_y: i32, k: i32) -> Result<Vec<String>, SearchError>;

    /// One `"category:count"` entry per category that has at least one place, sorted by category ascending.
    fn categories(&self) -> Vec<String>;
}

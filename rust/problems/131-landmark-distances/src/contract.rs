//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum MapError {
    /// a location named in the call does not exist
    UnknownLocation,
    /// `add_location` with a name that already exists
    DuplicateLocation,
    /// blank name or type, self-loop, distance <= 0, k <= 0, radius < 0
    InvalidArgument,
}

/// If a call has both an unknown location and another invalid argument, `UnknownLocation` is reported.
/// A failed call changes nothing.
pub trait CityMap: Send + Sync {
    /// Adds a location. Names are case-sensitive and used as given. Blank (empty or whitespace only): `InvalidArgument`.
    fn add_location(&self, name: &str) -> Result<(), MapError>;

    /// Adds an undirected road of `distance` (> 0) between two different, existing locations. Several roads may join
    /// the same pair; the shortest one is the one that matters for routing.
    fn add_road(&self, a: &str, b: &str, distance: i64) -> Result<(), MapError>;

    /// Removes every road joining `a` and `b` and returns how many were removed (0 if there were none).
    fn remove_road(&self, a: &str, b: &str) -> Result<usize, MapError>;

    /// Tags a location as a landmark of `kind`. Kinds are trimmed and case-insensitive ("Cafe " == "cafe"); a blank
    /// kind is `InvalidArgument`. A location may carry several kinds; tagging twice is harmless.
    fn tag_landmark(&self, location: &str, kind: &str) -> Result<(), MapError>;

    /// Removes the tag; returns whether it was present. Same kind rules as `tag_landmark`.
    fn untag_landmark(&self, location: &str, kind: &str) -> Result<bool, MapError>;

    /// Length of the shortest route in total distance; 0 if `a == b`; -1 if unreachable.
    fn shortest_distance(&self, a: &str, b: &str) -> Result<i64, MapError>;

    /// Location names along a shortest route, `a` first and `b` last (`a == b` gives `[a]`). Among routes of equal
    /// total distance the one whose list of names is lexicographically smallest wins (names compared as `str`
    /// byte order, element by element). Empty if unreachable.
    fn shortest_path(&self, a: &str, b: &str) -> Result<Vec<String>, MapError>;

    /// Up to `k` landmarks of `kind` reachable from `source`, as `"location:distance"` strings ordered by distance
    /// ascending, then location name ascending. The source counts (distance 0) if tagged. `k <= 0`:
    /// `InvalidArgument`. A kind nobody carries: empty.
    fn nearest_landmarks(&self, source: &str, kind: &str, k: i32) -> Result<Vec<String>, MapError>;

    /// Names of all locations at distance <= `radius` from `source` (source included), sorted by name.
    /// `radius < 0`: `InvalidArgument`.
    fn within_radius(&self, source: &str, radius: i64) -> Result<Vec<String>, MapError>;
}

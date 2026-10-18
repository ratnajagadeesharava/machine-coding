//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait CoffeeMachineApi: Send + Sync {
    fn refill(&self, ingredient: &str, units: u32);

    /// Defines or replaces a recipe. Returns false if the recipe is invalid.
    fn add_recipe(&self, drink: &str, ingredients: &[(&str, u32)]) -> bool;

    /// "OK", "UNKNOWN_DRINK" or "INSUFFICIENT:<ingredient>".
    fn dispense(&self, drink: &str) -> String;

    fn stock(&self, ingredient: &str) -> u32;

    /// Ingredients with stock strictly below the threshold, sorted alphabetically.
    fn low_stock(&self, threshold: u32) -> Vec<String>;
}

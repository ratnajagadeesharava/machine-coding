//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Every word and prefix is trimmed and lower-cased before use. After that, a word may contain only the characters
//! a-z, 0-9 and '-'. A blank word or any other character is `InvalidArgument`. A blank prefix is valid and means
//! "everything".

#[derive(Debug, PartialEq, Eq)]
pub enum TrieError {
    /// blank word / forbidden character / `limit == 0`
    InvalidArgument,
}

pub trait WordSet: Send + Sync {
    /// Adds the word. Returns true if it was not present before, false if it already was.
    fn insert(&self, word: &str) -> Result<bool, TrieError>;

    /// True if exactly this word is present.
    fn contains(&self, word: &str) -> Result<bool, TrieError>;

    /// True if at least one present word starts with the prefix (a word equal to the prefix counts).
    fn starts_with(&self, prefix: &str) -> Result<bool, TrieError>;

    /// Number of present words that start with the prefix (a word equal to the prefix counts).
    fn count_with_prefix(&self, prefix: &str) -> Result<usize, TrieError>;

    /// Up to `limit` present words that start with the prefix, in ascending lexicographic (plain character) order,
    /// without duplicates. `limit == 0` is `InvalidArgument`.
    fn words_with_prefix(&self, prefix: &str, limit: usize) -> Result<Vec<String>, TrieError>;

    /// Removes the word. Returns true if it was present. Other words that share its prefix are not affected.
    fn delete(&self, word: &str) -> Result<bool, TrieError>;

    /// Number of words currently present.
    fn size(&self) -> usize;
}

//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Every method that takes a word trims it, lower-cases it and requires it to be non-blank and made only of the
//! ASCII letters a-z (any case), '-' and the apostrophe '. Anything else is `InvalidArgument`. Sorted always means
//! ascending by plain character code (so the apostrophe sorts before '-', which sorts before the letters).

#[derive(Debug, PartialEq, Eq)]
pub enum DictError {
    /// invalid word, unknown part of speech, blank meaning / search text, limit or k <= 0,
    /// a word linked to itself
    InvalidArgument,
    /// the word is not in the dictionary (where the operation needs it to be)
    UnknownWord,
    /// meaning index outside the word's meanings (negative included)
    InvalidIndex,
    /// `update_meaning` would make two entries with the same part of speech and the same meaning
    DuplicateMeaning,
}

/// Checks run in this order: argument validity, unknown word, index, duplicate.
pub trait DictionaryApp: Send + Sync {
    /// Adds a meaning to the word (creating the word if needed). `part_of_speech` is NOUN, VERB, ADJECTIVE or
    /// ADVERB (any case, surrounding whitespace ignored); `meaning` is trimmed and must not be blank.
    /// Returns `Ok(false)` and changes nothing when the word already has exactly this (part of speech, meaning)
    /// pair (meaning compared case-sensitively); `Ok(true)` otherwise.
    fn add_word(
        &self,
        word: &str,
        part_of_speech: &str,
        meaning: &str,
    ) -> Result<bool, DictError>;

    /// The word's meanings in insertion order, each as `"POS: meaning"` (POS in upper case).
    /// Unknown word: empty list.
    fn lookup(&self, word: &str) -> Result<Vec<String>, DictError>;

    /// Replaces the meaning text at the 0-based `index`, keeping its part of speech and position.
    /// Errors: `UnknownWord`, `InvalidIndex`, `InvalidArgument` (blank text), `DuplicateMeaning` (the trimmed text
    /// equals the meaning of a different entry with the same part of speech). Re-applying the entry's own text
    /// is allowed.
    fn update_meaning(&self, word: &str, index: i32, new_meaning: &str) -> Result<(), DictError>;

    /// Removes the meaning at the 0-based index; later meanings shift down by one. Removing a word's last
    /// meaning removes the word itself. Errors: `UnknownWord`, `InvalidIndex`.
    fn remove_meaning(&self, word: &str, index: i32) -> Result<(), DictError>;

    /// Removes the word with all its meanings and synonym links. Returns whether the word existed.
    fn remove_word(&self, word: &str) -> Result<bool, DictError>;

    /// Links two words as direct synonyms (symmetric; links are not transitive). `Ok(false)` if they were already
    /// linked. Errors: `InvalidArgument` (invalid word, or both normalise to the same word), `UnknownWord`.
    fn add_synonym(&self, word: &str, other: &str) -> Result<bool, DictError>;

    /// The direct synonyms of the word, sorted. Unknown word: empty list.
    fn synonyms(&self, word: &str) -> Result<Vec<String>, DictError>;

    /// Words starting with the (normalised, non-blank) prefix, sorted, at most `limit`. `limit <= 0` is
    /// `InvalidArgument`.
    fn prefix(&self, prefix: &str, limit: i32) -> Result<Vec<String>, DictError>;

    /// Words having at least one meaning that contains `text` (trimmed, non-blank, compared case-insensitively),
    /// sorted by word, at most `limit`. Only meaning texts are searched. `limit <= 0` is `InvalidArgument`.
    fn search(&self, text: &str, limit: i32) -> Result<Vec<String>, DictError>;

    /// If the word is not in the dictionary: up to `k` existing words at Levenshtein distance exactly 1 (one
    /// insertion, deletion or substitution), sorted. A known word gives an empty list. `k <= 0` is
    /// `InvalidArgument`.
    fn suggest(&self, word: &str, k: i32) -> Result<Vec<String>, DictError>;

    /// The word at index `day_number.rem_euclid(size)` in sorted order, or `None` when the dictionary is empty.
    fn word_of_the_day(&self, day_number: i64) -> Option<String>;

    /// Number of words.
    fn size(&self) -> usize;

    /// Total number of meaning entries over all words.
    fn meaning_count(&self) -> usize;
}

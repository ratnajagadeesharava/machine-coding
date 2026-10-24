# 134 · Dictionary App (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A small in-memory dictionary, like the one behind a reading app. A **word** can have several **meanings**, each tagged with a part of speech (noun, verb, adjective, adverb). Editors add, correct and remove meanings; readers look words up, browse by prefix, search inside the definitions, follow synonym links, and get "did you mean ...?" suggestions for words that are not in the dictionary. The app also picks a deterministic "word of the day". Several editors can work at the same time, and no edit may be lost.

## Problem

Build the dictionary behind a thin, string-and-number API. Words are case-insensitive and whitespace-padded input is tolerated, so `" Bank "` and `"bank"` are the same word. A word's meanings keep the order in which they were added. Everything a caller sees (lists of words, synonyms) comes back sorted so answers are reproducible.

## Requirements

1. **Words.** Every method that takes a word trims it, lower-cases it, and requires it to be non-blank and made only of the ASCII letters `a-z` (any case on input), the hyphen `-` and the apostrophe `'`. Anything else (spaces inside, digits, accents, underscores) is an error. This applies to lookups, queries and removals as well as to writes.
2. **`add_word(word, part_of_speech, meaning)`** adds a meaning entry to the word, creating the word if it is new. The part of speech is one of `NOUN`, `VERB`, `ADJECTIVE`, `ADVERB` (any case, surrounding whitespace ignored); anything else is an error. The meaning is trimmed and must not be blank. If the word already has an entry with the same part of speech and the same meaning text (compared case-sensitively, after trimming), nothing changes and the result is `false`; otherwise the entry is appended and the result is `true`. An invalid call changes nothing (it does not create the word).
3. **`lookup(word)`** returns the word's entries in insertion order, each as `"<POS>: <meaning>"` with the part of speech in upper case, e.g. `"NOUN: a financial institution"`. An unknown word gives an empty list. The returned list is yours to keep: later edits must not change it.
4. **`update_meaning(word, index, new_meaning)`** replaces the text of the entry at the 0-based `index`; its part of speech and its position stay. `new_meaning` is trimmed. It is an error if the word is unknown, the index is outside `0..entries-1`, the text is blank, or the new text equals the text of a *different* entry of the same word with the *same part of speech* (it is fine to re-apply an entry's own text, or to use the text of an entry with another part of speech).
5. **`remove_meaning(word, index)`** removes the entry at the index; later entries shift down by one. Removing the last remaining entry of a word removes the word itself (as if `remove_word` had been called). Unknown word or bad index is an error.
6. **`remove_word(word)`** removes the word, all of its entries and all of its synonym links, and returns `true`; an unknown word gives `false`.
7. **Synonyms.** `add_synonym(word, other)` links two words directly and symmetrically and returns `true`; if they were already linked it returns `false`. Links are not transitive: if A-B and B-C are linked, A and C are not. Both words must exist, and a word cannot be linked to itself (after normalising); otherwise it is an error. `synonyms(word)` returns the direct synonyms sorted; an unknown word gives an empty list. Removing a word (or its last meaning) removes its links, so a word that is later added again starts with no synonyms.
8. **`prefix(prefix, limit)`** returns the words that start with the prefix (trimmed, lower-cased, same character rules as a word, non-blank), sorted, at most `limit` of them (the first ones in sorted order). `limit <= 0` is an error.
9. **`search(text, limit)`** returns the words for which at least one entry's meaning text contains `text` as a substring, ignoring case. `text` is trimmed and must not be blank. Only meaning texts are searched (not the word, not the part of speech). The result is sorted by word, each word at most once, at most `limit` entries. `limit <= 0` is an error.
10. **`suggest(word, k)`**: if the word is in the dictionary the result is empty. Otherwise it is the first `k` (in sorted order) of the dictionary words whose Levenshtein distance from the normalised input is exactly 1 (one insertion, one deletion or one substitution; a swap of two letters is distance 2). `k <= 0` is an error.
11. **`word_of_the_day(day_number)`** (`day_number` is an `i64`, may be negative) returns the word at index `day_number.rem_euclid(size)` in sorted order, or `None` when the dictionary is empty.
12. **`size()`** is the number of words; **`meaning_count()`** is the total number of entries over all words.
13. **Sorted** always means ascending by plain character code: the apostrophe sorts before the hyphen, which sorts before the letters (`a`, `a'b`, `a-b`, `ab`). Every failing call leaves the dictionary unchanged.

## Constraints

- The dictionary is shared by many threads. Concurrent `add_word` calls with different meanings for the same word must lose none; the same `(part of speech, meaning)` added by many threads at once is accepted exactly once.
- A reader never sees a half-applied edit (for example a synonym link to a word that is mid-removal, or entries out of order).
- Linking `(a, b)` on one thread and `(b, a)` on another must not deadlock.
- Prefix queries and lookups stay fast on tens of thousands of words.

## Examples

### Worked trace 1: meanings and normalisation

```
add_word("Bank",   "noun", "a financial institution")  -> true
add_word(" bank ", "VERB", "to tilt an aircraft")      -> true
add_word("bank",   "NOUN", "a financial institution")  -> false    (same part of speech and meaning)
add_word("bank",   "NOUN", "the land beside a river")  -> true
lookup("BANK") -> ["NOUN: a financial institution",
                   "VERB: to tilt an aircraft",
                   "NOUN: the land beside a river"]
size() -> 1      meaning_count() -> 3

update_meaning("bank", 1, "to lean into a turn")   lookup(...)[1] -> "VERB: to lean into a turn"
update_meaning("bank", 0, "the land beside a river") -> error      (same part of speech, other entry)
remove_meaning("bank", 0)  -> lookup now ["VERB: to lean into a turn", "NOUN: the land beside a river"]
```

### Worked trace 2: browsing a small dictionary

```
cat   NOUN "small domesticated feline"      car  NOUN "a road vehicle"
cart  NOUN "a vehicle pulled by an animal"  dog  NOUN "a domesticated canine"
hound NOUN "a dog used for hunting"

prefix("ca", 10) -> ["car", "cart", "cat"]        prefix("ca", 2) -> ["car", "cart"]
search("DOMESTICATED", 10) -> ["cat", "dog"]       search("vehicle", 10) -> ["car", "cart"]
search("hound", 10) -> []                          (the word itself is not searched)
suggest("cag", 5)  -> ["car", "cat"]               suggest("cats", 5) -> ["cat"]     ("cart" is 2 edits away)
suggest("cat", 5)  -> []                           (a known word needs no suggestion)
word_of_the_day(0) -> "car"   word_of_the_day(7) -> "cat"   word_of_the_day(-1) -> "hound"    (5 words)
```

### Worked trace 3: synonyms

```
add_synonym("dog", "hound") -> true     add_synonym("hound", "dog") -> false
add_synonym("dog", "cat")   -> true
synonyms("dog") -> ["cat", "hound"]    synonyms("hound") -> ["dog"]    synonyms("cat") -> ["dog"]   (no link hound-cat)
add_synonym("dog", "dog") -> error      add_synonym("dog", "wolf") -> error  (wolf is not in the dictionary)
remove_word("dog") -> true              synonyms("hound") -> []    synonyms("cat") -> []
add_word("dog", "noun", "a canine")     synonyms("dog") -> []
```

### Edge cases

- `add_word("word", "PREPOSITION", "x")`, `add_word("two words", ...)`, `add_word("abc1", ...)`, a blank meaning: errors, and the dictionary stays empty.
- The pair "same part of speech, same text" is the duplicate rule: `"A financial institution"` (capital A) is a different meaning from `"a financial institution"`, and the same text under `VERB` is a different entry from the `NOUN` one.
- `suggest` compares the normalised input: `suggest("CAG", 1)` behaves like `suggest("cag", 1)`.
- After `remove_meaning` empties a word, `remove_word` of it returns `false`, and its synonym links are gone.
- `word_of_the_day` follows the dictionary: after a removal the same day number may point to a different word.

## Glossary

- **Entry / meaning**: one `(part of speech, text)` pair belonging to a word.
- **Normalising**: trimming and lower-casing a word before using it.
- **Synonym link**: a direct, symmetric, non-transitive relation between two existing words.
- **Edit distance (Levenshtein)**: the minimum number of single-character insertions, deletions and substitutions turning one string into another.
- **Euclidean modulo**: a modulo whose result is never negative for a positive divisor (`(-1_i64).rem_euclid(5) = 4`).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, DictError>` with `InvalidArgument` (invalid word, part of speech or text; `limit` or `k` <= 0; self link), `UnknownWord`, `InvalidIndex` (negative or too large) and `DuplicateMeaning` (`update_meaning` only). Checks run in this order: argument validity, unknown word, index, duplicate. `index`, `limit` and `k` are `i32`; `day_number` is `i64`; `size` and `meaning_count` are `usize`.
- Lists are owned `Vec<String>`; `word_of_the_day` returns `Option<String>`; `lookup`, `synonyms`, `prefix`, `search` and `suggest` return `Ok(vec![])` rather than an error for "nothing found" (and `lookup` / `synonyms` for an unknown word).
- Normalising, sorting and the checks described above apply to every method listed in the requirements.
- Nothing else is promised: for example the maximum length of a word or meaning is not limited.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
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
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn DictionaryApp>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No test depends on time or randomness. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Adding, looking up, ordering of meanings, the duplicate rule, normalisation of words and parts of speech, invalid input.
- Updating and removing meanings (index shifting, the last meaning removes the word), removing words.
- Synonyms: symmetry, sorting, no transitivity, error cases, cleanup on removal.
- Prefix, search and suggest: sorting, limits, case-insensitivity, errors; word of the day including negative days and the empty dictionary; character-code ordering of `'` and `-`.
- Concurrency: 16 threads adding different meanings to one word lose none; the same meaning from 16 threads is accepted once; 8 threads creating words at the same time.

## Running

```bash
cargo test -p p134_dictionary_app
STRICT=1 cargo test -p p134_dictionary_app   # fail instead of skip while unimplemented
cargo test -p p134_dictionary_app --features hidden   # interviewer scenarios
cargo play run 134             # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `sut!`): `rust/support`.

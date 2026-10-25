# 134 · Dictionary App (Java)

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

1. **Words.** Every method that takes a word trims it, lower-cases it, and requires it to be non-blank and made only of the ASCII letters `a-z` (any case on input), the hyphen `-` and the apostrophe `'`. Anything else (spaces inside, digits, accents, underscores, `null`) is an error. This applies to lookups, queries and removals as well as to writes.
2. **`addWord(word, partOfSpeech, meaning)`** adds a meaning entry to the word, creating the word if it is new. The part of speech is one of `NOUN`, `VERB`, `ADJECTIVE`, `ADVERB` (any case, surrounding whitespace ignored); anything else is an error. The meaning is trimmed and must not be blank. If the word already has an entry with the same part of speech and the same meaning text (compared case-sensitively, after trimming), nothing changes and the result is `false`; otherwise the entry is appended and the result is `true`. An invalid call changes nothing (it does not create the word).
3. **`lookup(word)`** returns the word's entries in insertion order, each as `"<POS>: <meaning>"` with the part of speech in upper case, e.g. `"NOUN: a financial institution"`. An unknown word gives an empty list. The returned list is yours to keep: later edits must not change it.
4. **`updateMeaning(word, index, newMeaning)`** replaces the text of the entry at the 0-based `index`; its part of speech and its position stay. `newMeaning` is trimmed. It is an error if the word is unknown, the index is outside `0..entries-1`, the text is blank, or the new text equals the text of a *different* entry of the same word with the *same part of speech* (it is fine to re-apply an entry's own text, or to use the text of an entry with another part of speech).
5. **`removeMeaning(word, index)`** removes the entry at the index; later entries shift down by one. Removing the last remaining entry of a word removes the word itself (as if `removeWord` had been called). Unknown word or bad index is an error.
6. **`removeWord(word)`** removes the word, all of its entries and all of its synonym links, and returns `true`; an unknown word gives `false`.
7. **Synonyms.** `addSynonym(word, other)` links two words directly and symmetrically and returns `true`; if they were already linked it returns `false`. Links are not transitive: if A-B and B-C are linked, A and C are not. Both words must exist, and a word cannot be linked to itself (after normalising); otherwise it is an error. `synonyms(word)` returns the direct synonyms sorted; an unknown word gives an empty list. Removing a word (or its last meaning) removes its links, so a word that is later added again starts with no synonyms.
8. **`prefix(prefix, limit)`** returns the words that start with the prefix (trimmed, lower-cased, same character rules as a word, non-blank), sorted, at most `limit` of them (the first ones in sorted order). `limit <= 0` is an error.
9. **`search(text, limit)`** returns the words for which at least one entry's meaning text contains `text` as a substring, ignoring case. `text` is trimmed and must not be blank. Only meaning texts are searched (not the word, not the part of speech). The result is sorted by word, each word at most once, at most `limit` entries. `limit <= 0` is an error.
10. **`suggest(word, k)`**: if the word is in the dictionary the result is empty. Otherwise it is the first `k` (in sorted order) of the dictionary words whose Levenshtein distance from the normalised input is exactly 1 (one insertion, one deletion or one substitution; a swap of two letters is distance 2). `k <= 0` is an error.
11. **`wordOfTheDay(dayNumber)`** (`dayNumber` is a `long`, may be negative) returns the word at index `floorMod(dayNumber, size())` in sorted order, or an empty optional when the dictionary is empty.
12. **`size()`** is the number of words; **`meaningCount()`** is the total number of entries over all words.
13. **Sorted** always means ascending by plain character code: the apostrophe sorts before the hyphen, which sorts before the letters (`a`, `a'b`, `a-b`, `ab`). Every failing call leaves the dictionary unchanged.

## Constraints

- The dictionary is shared by many threads. Concurrent `addWord` calls with different meanings for the same word must lose none; the same `(part of speech, meaning)` added by many threads at once is accepted exactly once.
- A reader never sees a half-applied edit (for example a synonym link to a word that is mid-removal, or entries out of order).
- Linking `(a, b)` on one thread and `(b, a)` on another must not deadlock.
- Prefix queries and lookups stay fast on tens of thousands of words.

## Examples

### Worked trace 1: meanings and normalisation

```
addWord("Bank",   "noun", "a financial institution")  -> true
addWord(" bank ", "VERB", "to tilt an aircraft")      -> true
addWord("bank",   "NOUN", "a financial institution")  -> false    (same part of speech and meaning)
addWord("bank",   "NOUN", "the land beside a river")  -> true
lookup("BANK") -> ["NOUN: a financial institution",
                   "VERB: to tilt an aircraft",
                   "NOUN: the land beside a river"]
size() -> 1      meaningCount() -> 3

updateMeaning("bank", 1, "to lean into a turn")   lookup(...)[1] -> "VERB: to lean into a turn"
updateMeaning("bank", 0, "the land beside a river") -> error      (same part of speech, other entry)
removeMeaning("bank", 0)  -> lookup now ["VERB: to lean into a turn", "NOUN: the land beside a river"]
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
wordOfTheDay(0) -> "car"   wordOfTheDay(7) -> "cat"   wordOfTheDay(-1) -> "hound"    (5 words)
```

### Worked trace 3: synonyms

```
addSynonym("dog", "hound") -> true     addSynonym("hound", "dog") -> false
addSynonym("dog", "cat")   -> true
synonyms("dog") -> ["cat", "hound"]    synonyms("hound") -> ["dog"]    synonyms("cat") -> ["dog"]   (no link hound-cat)
addSynonym("dog", "dog") -> error      addSynonym("dog", "wolf") -> error  (wolf is not in the dictionary)
removeWord("dog") -> true              synonyms("hound") -> []    synonyms("cat") -> []
addWord("dog", "noun", "a canine")     synonyms("dog") -> []
```

### Edge cases

- `addWord("word", "PREPOSITION", "x")`, `addWord("two words", ...)`, `addWord("abc1", ...)`, a blank meaning: errors, and the dictionary stays empty.
- The pair "same part of speech, same text" is the duplicate rule: `"A financial institution"` (capital A) is a different meaning from `"a financial institution"`, and the same text under `VERB` is a different entry from the `NOUN` one.
- `suggest` compares the normalised input: `suggest("CAG", 1)` behaves like `suggest("cag", 1)`.
- After `removeMeaning` empties a word, `removeWord` of it returns `false`, and its synonym links are gone.
- `wordOfTheDay` follows the dictionary: after a removal the same day number may point to a different word.

## Glossary

- **Entry / meaning**: one `(part of speech, text)` pair belonging to a word.
- **Normalising**: trimming and lower-casing a word before using it.
- **Synonym link**: a direct, symmetric, non-transitive relation between two existing words.
- **Edit distance (Levenshtein)**: the minimum number of single-character insertions, deletions and substitutions turning one string into another.
- **Floor modulo**: a modulo whose result is never negative for a positive divisor (`floorMod(-1, 5) = 4`).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: every error is an `IllegalArgumentException` (invalid word, part of speech or text; unknown word; bad index; duplicate on update; self link; `limit` or `k` <= 0), including `null` arguments. Lists are returned as `List<String>`; `wordOfTheDay` returns `Optional<String>`.
- Normalising, sorting and the checks described above apply to every method listed in the requirements.
- Nothing else is promised: for example the maximum length of a word or meaning is not limited, and the returned lists may or may not be modifiable.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Optional;

public interface DictionaryApp {
    boolean addWord(String word, String partOfSpeech, String meaning);
    List<String> lookup(String word);
    void updateMeaning(String word, int index, String newMeaning);
    void removeMeaning(String word, int index);
    boolean removeWord(String word);
    boolean addSynonym(String word, String other);
    List<String> synonyms(String word);
    List<String> prefix(String prefix, int limit);
    List<String> search(String text, int limit);
    List<String> suggest(String word, int k);
    Optional<String> wordOfTheDay(long dayNumber);
    int size();
    int meaningCount();
}
```

The full Javadoc is in `src/main/java/contract/DictionaryApp.java`. Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static DictionaryApp create()
}
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
mvn test                 # from this directory (or `mvn -pl problems/134-dictionary-app test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

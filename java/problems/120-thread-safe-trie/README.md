# 120 · Thread-Safe Trie (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

An autocomplete box, a spell checker, a profanity filter and a URL router all ask the same question about a large set of words: *is this word there, and what else starts the same way?* You are building a shared, in-memory **set of words** that answers those questions quickly, and that is used by many threads at once: some add and remove words while others look things up, and every caller must get a correct answer no matter how the calls interleave.

## Problem

Build a thread-safe word set with prefix queries. `insert` adds a word, `delete` removes one, `contains` asks for an exact word, `startsWith` and `countWithPrefix` ask about prefixes, and `wordsWithPrefix` lists matches in alphabetical order. The set holds at least a hundred thousand words, answers ten thousand prefix queries in a few seconds, and stays correct when 16 threads hammer it.

## Requirements

1. **Normalisation.** Every word and every prefix is trimmed and lower-cased first (locale-independent). After that, a word may contain only the characters `a-z`, `0-9` and `-`. A blank word, or a word with any other character (spaces inside, `_`, `.`, accented letters, ...), is an error. A blank prefix is valid and means "every word". A rejected call changes nothing.
2. `insert(word)` adds the word and returns `true` if it was not present, `false` if it already was.
3. `contains(word)` is `true` only for a word that is present exactly (`app` is not contained just because `apple` is).
4. `startsWith(prefix)` is `true` if at least one present word starts with the prefix. A word equal to the prefix counts.
5. `countWithPrefix(prefix)` returns how many present words start with the prefix (a word equal to the prefix counts; the empty prefix counts all words).
6. `wordsWithPrefix(prefix, limit)` returns at most `limit` present words that start with the prefix, in ascending lexicographic order (plain character order: `-` before digits before letters; a word before its own extensions). The empty prefix lists all words. `limit` must be positive. The result is a fresh list the caller may modify; it never contains a word twice.
7. `delete(word)` removes exactly that word and returns `true`; a word that is not present gives `false`. Other words, including ones that share its prefix or extend it, are untouched, and prefix queries stop reporting prefixes that no remaining word has.
8. `size()` returns the number of distinct words currently present.
9. **Concurrency.** Every operation is atomic and linearisable: it takes effect at one instant between its start and its end. In particular:
   - a word inserted concurrently is either fully visible (to `contains`, `startsWith`, `countWithPrefix` and `wordsWithPrefix`) or not visible at all, never partly;
   - `wordsWithPrefix` returns a consistent snapshot: sorted, without duplicates, and containing only words that were present at some instant during the call; a word that is present for the whole call is in the result (subject to `limit`);
   - when several threads insert the same word at the same time, exactly one gets `true`; the same holds for `delete`;
   - inserts and deletes of different words never lose each other's effect; after all writers finish, `size()` equals the number of distinct words that should be present.
10. **Scale.** 100,000 words can be inserted, queried and removed within a generous time bound; words may be up to a thousand characters long.

## Examples

```
insert("apple") -> true     insert("apple") -> false      insert("  APP ") -> true
contains("app") -> true     contains("ap") -> false       size() -> 2

(words: car, card, care, careful, cat, dog)
startsWith("car")              -> true
startsWith("cart")             -> false
countWithPrefix("car")         -> 4         (car, card, care, careful)
countWithPrefix("")            -> 6
wordsWithPrefix("car", 3)      -> [car, card, care]
wordsWithPrefix("ca", 10)      -> [car, card, care, careful, cat]
wordsWithPrefix("", 2)         -> [car, card]

(words: app, apple, apply)
delete("app")                  -> true
contains("app")                -> false     startsWith("app") -> true
wordsWithPrefix("ap", 10)      -> [apple, apply]
delete("app")                  -> false

insert("two words")            -> error     insert("   ") -> error     wordsWithPrefix("a", 0) -> error
```

### Edge cases

- `insert(" Hello ")` and `insert("HELLO")` are the same word `hello`; `startsWith(" HEL ")` is `true`. Words are returned in their stored form (trimmed, lower-case).
- `wordsWithPrefix("", n)` on an empty set is an empty list, `startsWith("")` is `false` there and `countWithPrefix("")` is `0`.
- Deleting the only word below a prefix makes `startsWith` of that prefix `false`; deleting a short word leaves its longer extensions, and deleting a long word leaves the short one.
- A deleted word can be inserted again and `insert` then returns `true`.
- A prefix longer than every word, or one that no word has, gives `false` / `0` / an empty list.
- `limit` larger than the number of matches returns all matches.
- `contains`, `delete`, `startsWith`, `countWithPrefix` and `wordsWithPrefix` apply the same normalisation and the same character rules as `insert`: `contains("a b")` is an error, not `false`.

## Glossary

- **Word**: a normalised string of `a-z`, `0-9` and `-`, as stored.
- **Prefix**: the (normalised) leading characters of a word; may be empty; a word is a prefix of itself.
- **Linearisable**: each call appears to happen at a single instant between its start and its end, and those instants are consistent with the order in which completed calls were observed.
- **Snapshot**: the contents of the set at one instant.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for a null argument, a blank word, a forbidden character, and `limit <= 0`. `insert`, `contains`, `delete` and `wordsWithPrefix` take `String`; sizes and counts are `int`.
- Normalisation: trim, then lower-case, then validate characters. A blank prefix is valid; a blank word is not.
- Order of `wordsWithPrefix`: ascending by character code of the stored words (`-` < `0..9` < `a..z`).
- Rejected calls leave no trace.
- A completed call is visible to every call that starts after it, from any thread.
- Results of concurrent calls need only be explainable by *some* order of the calls that respects the time order of non-overlapping ones.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below. It is **not** your domain model: it is the doorway. Create whatever you like next to it and wire it up.

```java
package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every word and prefix is trimmed and lower-cased (Locale.ROOT) before use. After that, a word may contain only
 * the characters a-z, 0-9 and '-'. A null argument, a blank word, or any other character is an
 * IllegalArgumentException. A blank prefix is valid and means "everything".
 */
public interface WordSet {

    /** Adds the word. Returns true if it was not present before, false if it already was. */
    boolean insert(String word);

    /** True if exactly this word is present. */
    boolean contains(String word);

    /** True if at least one present word starts with the prefix (a word equal to the prefix counts). */
    boolean startsWith(String prefix);

    /** Number of present words that start with the prefix (a word equal to the prefix counts). */
    int countWithPrefix(String prefix);

    /**
     * Up to {@code limit} present words that start with the prefix, in ascending lexicographic (plain character) order,
     * without duplicates. {@code limit <= 0} is an IllegalArgumentException. The list is a fresh copy.
     */
    List<String> wordsWithPrefix(String prefix, int limit);

    /** Removes the word. Returns true if it was present. Other words that share its prefix are not affected. */
    boolean delete(String word);

    /** Number of words currently present. */
    int size();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    private Solution() {}

    public static WordSet create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. Nothing sleeps; concurrency is driven by threads that start at the same instant.

- Insert/contains/startsWith/countWithPrefix/wordsWithPrefix/delete/size on small sets, ordering rules, limits, the empty prefix, prefix-sharing words.
- Normalisation, digits and hyphens, invalid input (blank, forbidden characters, bad limit) and that rejected calls change nothing.
- Deleting words that share prefixes with others; missing words; reinsertion; very long words.
- Concurrency: disjoint inserts lose nothing; the same word inserted or deleted by 16 threads has exactly one winner; overlapping writers give the right `size`; inserts and deletes of disjoint words together; no half-inserted word visible; sorted duplicate-free snapshots while writers churn.
- Scale: 100,000 words, 10,000 prefix queries and 50,000 deletes within a generous bound.
- More (a randomised comparison with a brute-force model, 200,000 words from 8 threads, races on a tiny word set, sibling words under concurrent deletes) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/120-thread-safe-trie test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

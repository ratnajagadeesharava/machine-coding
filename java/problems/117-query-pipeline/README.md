# 117 · Search Query Pipeline (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

When someone types a query into a search box, the raw text rarely goes straight to the index. It is cleaned up, mis-spellings are repaired, synonyms widen it, unwanted words are dropped, the matching documents are ranked and the matched words are highlighted in the snippets that come back. You are building that query-processing flow for a small in-memory search service. The flow has a fixed default order of built-in stages, one stage that fails must not take the whole query down, and the people who run the service must be able to add their own stage, switch a stage off or remove it without the built-in stages being touched.

## Problem

A search service is configured once with a dictionary of correct words, a synonym table, a list of banned words and a small corpus of documents. `process(rawQuery)` takes what the user typed and returns three things: the final query text, the ranked result lines (with the matched words wrapped in `[ ]`), and a trace listing the stages that actually changed something. Operators can register their own query-transforming stage at a chosen position, and can remove or disable any stage by name. A stage that blows up is skipped and reported in the trace; the rest of the flow carries on.

## Requirements

Default order: **parse → spell → synonyms → filter → rank → highlight**. The first four transform the query text and are called the *query phase*; rank and highlight then produce the result lines.

1. **parse**: trim the query, collapse every run of whitespace to one space, lower-case it. A null or blank raw query is an error (nothing runs).
2. **spell**: the dictionary words are configured at creation. Each word of the query that is not in the dictionary but is within edit distance 1 of **exactly one** dictionary word (one insertion, deletion or substitution of a character; a swap of two neighbours is two edits) is replaced by that dictionary word. A word that is in the dictionary, a word with no dictionary word within 1 edit, and a word that is within 1 edit of two or more dictionary words are left unchanged.
3. **synonyms**: the synonym table maps a word to a list of synonyms. Every query word that has an entry is replaced by an OR group written exactly `(` + original + ` OR ` + synonym1 + ` OR ` + synonym2 ... + `)`: the original word first, then the synonyms in table order, duplicates (and the original itself) listed once. Words with no entry (or whose entry adds nothing) stay as they are. Synonyms are not expanded again.
4. **filter**: configured banned words are removed from the query wherever they occur, both as plain words and as members of an OR group. A group left with one member is written as that plain word (no parentheses); a group left with no members disappears. Remaining terms keep their order, separated by single spaces. A query that is empty afterwards gives an empty result (see below).
5. **rank**: the corpus (document id → text) is provided at creation. The query is read as a list of *terms*: a plain word is one term, an OR group is one term. Repeated identical terms count once. A document matches a term when any word of the document (compared ignoring case; documents are split on whitespace) equals the term's word, or any member of its group. A document's score is the number of distinct terms it matches. Documents with score 0 are not returned. Results are ordered by score, highest first, ties by document id (plain text order, `d10` before `d2`). Every matching document is returned.
6. **highlight**: each result line is `<docId>: <document text>`, the document's words joined by single spaces, and every word of the document that equals (ignoring case) any word in the query's terms (including all members of OR groups) is wrapped in `[ ]`, keeping the document's own letter case.
7. **trace**: lists, in execution order, the name of every step that changed something: a query-phase step whose output differs from its input; `rank` when at least one document was returned; `highlight` when it was run on at least one returned line. A step that failed appears as `<name>:failed` instead. Steps that were removed, disabled or did not change anything do not appear.
8. **extensibility**: a caller can register its own step (a name plus a text-to-text function) at the end of the query phase, immediately before, or immediately after a named query-phase step (built-in or registered). A registered step behaves exactly like a built-in one (trace, failure handling, removal). The built-in steps cannot be touched by registration. Any step (built-in or registered) can be removed permanently by name or disabled and enabled again. A disabled step is not run and not traced. Removing `rank` leaves the result lines empty, removing or disabling `highlight` leaves the lines unhighlighted.
9. **failure policy**: a step that throws, or returns null, is skipped: the query it received is passed on unchanged, `<name>:failed` is added to the trace and the flow continues.
10. **empty query**: as soon as any step leaves the query empty or blank, the flow stops: the result has query `""`, no result lines, and the trace up to and including the step that emptied it. Later steps do not run.

## Constraints

- Many threads call `process` while others register, remove, disable and enable steps. The pipeline must be thread-safe.
- One `process` call must see one consistent list of steps: it never sees a registration half applied, and a change made while a call is running does not alter that call.
- The caller's dictionary, synonym table, banned list and corpus are copied at creation: changing them afterwards has no effect.
- Every call is independent; returned lists are not shared with later calls.

## Examples

Configuration used below:

```
dictionary: phone, case, cheap, charger, usb, ...
synonyms:   phone -> [mobile, cell], cheap -> [budget]
banned:     free, spam
corpus:     d1 "cheap phone case"   d2 "wireless phone charger"   d3 "usb charger cable"   d4 "Budget mobile deals"
```

```
process("  Cheep   PHONE  ")
  parse    -> "cheep phone"
  spell    -> "cheap phone"
  synonyms -> "(cheap OR budget) (phone OR mobile OR cell)"
  query  = "(cheap OR budget) (phone OR mobile OR cell)"
  results = ["d1: [cheap] [phone] case",        (2 terms)
             "d4: [Budget] [mobile] deals",     (2 terms, later id)
             "d2: wireless [phone] charger"]    (1 term)
  trace  = [parse, spell, synonyms, rank, highlight]

process("free phone")   -> filter removes "free" -> query "(phone OR mobile OR cell)"
process("FREE spam")    -> query "", results [], trace [parse, filter]
process("zzzz")         -> query "zzzz", results [], trace []   (nothing changed, nothing matched)
```

Registering a step `add-usb` (appends ` usb`) at the end of the query phase, then `process("free case")`: query `case usb`, trace `[filter, add-usb, rank, highlight]`.
A registered step that throws, between parse and spell: `process("CHEEP")` has trace `[parse, boom:failed, spell, synonyms, rank, highlight]` and the same query as without it.

### Edge cases

- `cap` is one edit from both `car` and `cat` (both in the dictionary): left as `cap`. `car` is itself in the dictionary: left alone even though `cat` is one edit away.
- With `cell` banned, `phone` becomes `(phone OR mobile)`; with `mobile` and `cell` banned it becomes `phone`; with `phone` banned it becomes `(mobile OR cell)`.
- `cable cable charger` has two distinct terms, so a document containing only `cable` scores 1, not 2.
- A step registered before `filter` has its output filtered; a step registered at the end of the query phase has not.
- A step registered before `parse` receives the raw, untrimmed text.
- A name used by a built-in step can never be registered, even after that built-in was removed; the name of a removed registered step can be reused.
- Registering relative to `rank`, `highlight`, an unknown name or a removed step is an error; so are a null step, a blank step name and a duplicate step name. Removing, disabling or enabling an unknown name is an error. Nothing changes on error.
- Configuration words (dictionary, synonyms, banned words) are given in lower case.

## Glossary

- **Step**: one stage of the flow with a unique name. Built-in names: `parse`, `spell`, `synonyms`, `filter`, `rank`, `highlight`.
- **Query phase**: the steps that turn query text into query text (parse, spell, synonyms, filter and anything registered).
- **Term**: a plain word or one OR group in the query; the unit rank counts.
- **OR group**: `(a OR b OR c)`, matched by a document when it contains any of its words.
- **Trace**: the ordered names of the steps that changed something (or failed).
- **Edit distance 1**: one inserted, deleted or substituted character.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for a null/blank raw query, a null step, a null/blank or duplicate step name, an unknown/removed/non-query-phase anchor, and an unknown name for remove/disable/enable.
- `process` returns `Result(query, results, trace)`; `Step` is `name()` + `transform(String)` and is implemented by the tests.
- Names of the built-in steps are the constants on the contract (`parse`, `spell`, `synonyms`, `filter`, `rank`, `highlight`).
- Configuration is passed to `Solution.create(dictionary, synonyms, bannedWords, corpus)`.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

public interface QueryPipeline {
    String PARSE = "parse"; String SPELL = "spell"; String SYNONYMS = "synonyms";
    String FILTER = "filter"; String RANK = "rank"; String HIGHLIGHT = "highlight";

    interface Step { String name(); String transform(String query); }   // implemented by the tests
    record Result(String query, List<String> results, List<String> trace) {}

    Result process(String rawQuery);
    void register(Step step);                              // end of the query phase
    void registerBefore(String stepName, Step step);
    void registerAfter(String stepName, Step step);
    void removeStep(String name);
    void disableStep(String name);
    void enableStep(String name);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static QueryPipeline create(Collection<String> dictionary, Map<String, List<String>> synonyms,
                                       Collection<String> bannedWords, Map<String, String> corpus)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Test steps are defined inside the tests; no clock or sleeping is involved.

- parse (whitespace, case, already-clean input) and blank/null rejection.
- spell: insertion / deletion / substitution, dictionary words, far words, swaps, ambiguity.
- synonyms: group format, duplicates, no re-expansion, spell before synonyms.
- filter, including an all-banned query; rank ordering, repeated terms, group-as-one-term; highlight and no-match.
- The full trace for a worked query; registration at the end / before / after a built-in, including before parse.
- Remove, disable, enable, reuse of a removed name; independent instances; configuration copied.
- Failure policy (throwing and null-returning steps), a step that empties the query, invalid registrations.
- Concurrency: 8 threads processing while another registers/disables/removes steps.
- More edge cases (banned words inside groups, id tie-breaks, step positions, concurrent order consistency) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/117-query-pipeline test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.

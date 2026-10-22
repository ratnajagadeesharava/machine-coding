# 108 · Document Comment Service (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy-Medium | 30 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A review tool lets several reviewers leave remarks on documents ("typo in section 2", "please add a diagram"). You are building the in-memory core of its comment service. The service is created with a fixed set of document ids. Reviewers add comments to those documents and anyone can read back all comments of a document, in the order they were written. Comments for documents the service does not know are refused, empty reviewer names or empty comments are refused, and many reviewers comment at the same time without losing, duplicating or reordering anything.

## Problem

Build a Document Comment Service that is initialised with a set of document ids. A reviewer can add a comment to an existing
document and gets back the id of the new comment. Anybody can retrieve all the comments of a document; they come back in the
order in which they were added. What a caller got back must not change behind their back, and what a caller does to what they
got back must not change the service. Many threads add comments at the same time.

## Requirements

1. The service is created from a set of document ids. Those are the only documents that will ever exist; there is no way to add a document later. An empty set is allowed.
2. `addComment(documentId, reviewerId, text)` appends a comment to an existing document and returns the id of the new comment.
3. Comment ids are decimal strings of a service-wide sequence: the first accepted comment gets `"1"`, the next `"2"`, and so on, across all documents, in the order the service accepted them. A rejected call consumes no id.
4. `comments(documentId)` returns every comment of that document, oldest first (insertion order). Each comment is the single string `reviewerId + ": " + text`, with both parts exactly as they were given (no trimming, no case changes). A document without comments gives an empty list.
5. Adding to, or reading from, a document id the service does not know is an error. A blank (null, empty or whitespace-only) document id, reviewer id or text is an error. Blank arguments are checked before the document is looked up, so a blank argument is always an invalid-argument error, even for an unknown document.
6. A rejected call changes nothing: no comment, no id consumed, no document created.
7. The list returned by `comments` is a copy. Later `addComment` calls do not change a list that was already returned, and modifying a returned list (if the language allows it) does not change the service.
8. The same reviewer may comment many times on the same document, and identical comments are allowed and are kept separately.

## Constraints

- Many threads call `addComment` and `comments` at the same time, on the same and on different documents. The service must be thread-safe.
- No accepted comment may be lost or duplicated, and ids are never reused.
- Comments of one document must be listed in id order: if comment `"5"` was accepted before comment `"9"`, `"5"` is listed first, even when added from different threads at once.
- A reader sees the comments of a document as they were at one moment: an earlier prefix of the final history, never a list with a gap in the middle.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, CommentError>` with `UnknownDocument` (non-blank id the service does not know) and `InvalidArgument` (blank document id, reviewer id or text). `create` takes the document ids as `&[&str]` (assumed non-blank, duplicates are harmless).
- Ids are `"1"`, `"2"`, ... (service-wide, accepted order). Failed calls consume no id.
- `comments` entries are exactly `reviewerId + ": " + text`; text is stored verbatim (colons, newlines, surrounding spaces and non-ASCII characters included).

## Examples

```
create({"doc-1", "doc-2"})
addComment("doc-1", "alice", "Fix the typo")      -> "1"
addComment("doc-2", "bob",   "Add a diagram")     -> "2"
addComment("doc-1", "carol", "Looks good")        -> "3"
comments("doc-1")  -> ["alice: Fix the typo", "carol: Looks good"]
comments("doc-2")  -> ["bob: Add a diagram"]
```

```
create({"doc-1"})
comments("doc-1")                           -> []
addComment("doc-9", "alice", "hi")          -> error (unknown document)
addComment("doc-1", "alice", "hi")          -> "1"   (the failed call used no id)
```

```
create({"doc-1"})
before = comments("doc-1")                  -> []
addComment("doc-1", "alice", "first")       -> "1"
before                                      -> []                      (a list you already hold never changes)
comments("doc-1")                           -> ["alice: first"]
```

```
create({"doc-1"})
addComment("doc-1", " ", "hi")              -> error (blank reviewer)
addComment("doc-1", "alice", "")            -> error (blank text)
addComment("doc-1", "alice", "see: line 3") -> "1"
comments("doc-1")                           -> ["alice: see: line 3"]
```

### Edge cases

- Blank means null (Java), empty, or only whitespace. Text such as `" a "` is valid and is kept with its spaces.
- A blank argument beats an unknown document: `addComment("nope", " ", "x")` is an invalid-argument error.
- The ids come from one sequence for the whole service, not one per document.
- Two identical calls produce two comments with two ids.
- Documents are fully independent: comments of one never show up in another.
- A document id that differs only in case or by surrounding spaces is a different (probably unknown) document.
- 16 threads adding 100 comments each to one document produce exactly 1,600 comments, ids `"1"`..`"1600"` each used once, listed in id order.

## Glossary

- **Document**: something that can be commented on, identified by a string id fixed when the service is created.
- **Reviewer**: the person leaving a comment, identified by a string id.
- **Comment**: a reviewer's text attached to one document, with a service-wide id.
- **Insertion order**: the order in which the service accepted the comments (equal to increasing id order).
- **Defensive copy**: a copy handed to the caller so that neither side can change the other's data afterwards.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Create whatever types you like and wire them up in `create` in `src/lib.rs`.

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum CommentError {
    /// non-blank document id the service does not know
    UnknownDocument,
    /// blank document id / reviewer id / text
    InvalidArgument,
}

pub trait CommentService: Send + Sync {
    /// Appends a comment to an existing document and returns its id: "1", "2", ... (one sequence for the whole
    /// service, in accepted order; a rejected call uses no id). Blank arguments are checked first.
    fn add_comment(&self, document_id: &str, reviewer_id: &str, text: &str) -> Result<String, CommentError>;

    /// All comments of the document, oldest first, each as `reviewer_id: text` (verbatim). Empty if there are none.
    fn comments(&self, document_id: &str) -> Result<Vec<String>, CommentError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(document_ids: &[&str]) -> Option<Arc<dyn CommentService>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Nothing depends on the wall clock; concurrency tests use fixed thread counts and check the result, never timing.

- Empty documents, ids `"1"`, `"2"`, ... across documents, comment format and verbatim text, insertion order, independence of documents, repeated and identical comments.
- Error cases: unknown document (add and read), blank document id / reviewer / text, blank beating unknown, failed calls consuming no id and creating nothing.
- Defensive copies: an earlier list never changes, mutating a returned list does not touch the service.
- Concurrency: 16 threads adding to one document lose and duplicate nothing and keep id order; 8 threads on different documents; readers running during writers.
- More edge cases (prefix-consistent reads under load, failed adds racing with good ones, large histories) are in `interviewer/`.

## Running

```bash
cargo test -p p108_document_comments
STRICT=1 cargo test -p p108_document_comments   # fail instead of skip while unimplemented
cargo test -p p108_document_comments --features hidden   # interviewer scenarios
cargo play run 108             # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `sut!`): `rust/support`.

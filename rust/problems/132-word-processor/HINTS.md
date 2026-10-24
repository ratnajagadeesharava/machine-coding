# Hints — 132 Word Processor (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Count what a document of one million identically formatted characters looks like to a user: how many *different things* are in it? Store that, not a million of something.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Make a format an immutable value that is compared and shared by value (a "change" produces another value, never edits one). Describe the formatting of the document as consecutive blocks of (length, format) next to the text. Then three small operations carry most of the problem: split a block at a position, change the formats inside a span, and merge neighbours that became equal. `insert`, `delete`, `applyFormat`, `runs`, `distinctFormatCount` and `replaceAll` are all built from them.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Document { text buffer, runs: list of (length, shared Format), line alignments: list indexed by line }` guarded by one lock. Normalise (drop empty blocks, merge equal neighbours) at the end of every mutating call, so `runs()` is just a walk. `insert` = split at `pos`, add a block with the format of the previous character (default at 0), keep the line list in step with the newlines in the new text. `delete` = split at both ends, drop the blocks in between, remove one line entry per deleted newline after the line that holds `from`. `applyFormat` = validate everything first, split at both ends, rewrite the formats of the blocks inside. `replaceAll` = collect the non-overlapping matches on the original text first, then apply them from the last to the first (so earlier positions stay valid), each as delete + insert with the format of the match's first character.

**Rust:** derive `Clone, PartialEq, Eq, Hash` on the format and share it as `Arc<Format>` (or store small copies - it is only a few words). `Vec<char>` or `String` for the text; `Vec<(usize, Format)>` for the runs. Hold everything in one `Mutex<Doc>` so every `&self` method locks once.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

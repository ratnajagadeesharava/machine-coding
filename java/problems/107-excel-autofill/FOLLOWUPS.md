# Follow-ups — 107 Excel-Like Autofill Engine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Month and weekday names
`Jan, Feb` continues `Mar, Apr`; `Mon` continues `Tue`; both also in full (`January`) and in upper/lower case, which is preserved. The cycle wraps (`Dec` then `Jan`) instead of stopping. How much of your code changes to add this, and where does the decision "which kind of series is this?" live?

### V3 — Text with a number
`Item 1, Item 2` continues `Item 3`; `Q1` continues `Q2`, `Q3`, `Q4`; `A1` continues `A2`. Zero padding is now meaningful (`file007` gives `file008`). How do you decide between this kind and the plain letter kind when a seed such as `A1` or `AB` could be read two ways? What if the prefixes differ?

### V4 — Non-linear and custom series
Add geometric series (`2, 4` gives `8, 16`) and user-defined lists registered at runtime (`Low, Medium, High`). Two kinds now accept the same seeds (`2, 4` is arithmetic and geometric; `1, 1` is both). Who wins, and how is that decided deterministically? Is registration thread-safe, and can a registered list change the answer for seeds that were already valid?

### V5 — Dates and a two-dimensional fill
Dates (`2024-01-30` plus one month gives `2024-02-29` or `2024-02-28`? decide and state it) and filling a block of cells in two directions at once. How does your design carry a direction, and how do you report which cell was unrecognised?

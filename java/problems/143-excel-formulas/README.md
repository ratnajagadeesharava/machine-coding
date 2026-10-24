# 143 · Excel Cell Values and Sum Formulas (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — the entry point and the tests are provided; the cell model, the formula handling and the recomputation are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The calculation core of a tiny spreadsheet. Every cell holds nothing, a number, a piece of text or a **formula** such as `=A1+SUM(B1:B10)*2`. When someone changes a cell, every formula that depends on it, directly or through other formulas, must show the new result the next time anybody looks, and the engine must not redo work that did not need doing: changing a cell that nobody refers to recalculates nothing, and a formula that is reachable by two different routes is still calculated only once per change. Formulas can refer to each other in circles, point at cells that do not exist, divide by zero or add up text, and each of these has a defined outcome. Layout, fonts, rows and columns inserted or deleted are somebody else's problem: this problem is only about **values and formulas**.

## Problem

The sheet has 26 columns (`A`..`Z`) and 100 rows (`1`..`100`), 2,600 cells in all. You implement `set` (put something into a cell), `get` (what does the cell show), and a few read-only queries that expose how cells relate to each other, plus a counter of how many formula evaluations were performed so far, which is how the tests observe that you recompute exactly what has to be recomputed.

## Requirements

### 1. Cell addresses

1. An address is one column letter `A`..`Z` followed by a row number `1`..`100` written without leading zeros: `A1`, `Z100`, `B12`. Letters are case-insensitive; every address you report is upper case.
2. Anything else is invalid: `""`, `A0`, `A101`, `AA1`, `A01`, `1A`, `A`, `A 1`, ` A1`, `A1 `, `null`. Every operation that takes an address rejects an invalid one (see "Behavioral contract") and changes nothing.
3. Whenever the API returns a list of cells (`dependencies`, `dependents`, `usedCells`) it is sorted **row by row**: `A1, B1, ..., Z1, A2, B2, ...`, without duplicates.

### 2. What `set(cell, input)` stores

`set` replaces whatever the cell held. The input is classified by its exact characters, in this order:

1. The empty string `""` **clears** the cell. A cleared cell is empty.
2. A string whose first character is `=` is a **formula** (requirement 3). There is no trimming: ` =1+1` (leading space) is text.
3. A string made of an optional sign `+`/`-`, one or more digits, and optionally a `.` followed by one or more digits is a **number**. So `5`, `-0`, `+7`, `007`, `3.50`, `-2.25` are numbers; `5.`, `.5`, ` 5`, `5 `, `1e3`, `1,000`, `--5` are not.
4. Everything else is **text**, kept exactly as given (including spaces).

After `set` returns, **every cell that directly or indirectly depends on the changed cell is up to date**: `get` never returns a stale value.

### 3. Formulas

A formula is `=` followed by an expression. Spaces and tabs are allowed between any two tokens (and after the `=`), nowhere inside a token.

```
expression := term   (('+' | '-') term)*            left associative
term       := factor (('*' | '/') factor)*          left associative
factor     := '-' factor | primary                  unary minus, may be repeated: --3
primary    := number | reference | SUM '(' argument (',' argument)* ')' | '(' expression ')'
argument   := reference ':' reference | reference | ['-'] number
number     := digits ['.' digits]                   (no sign, no ".5", no "5.")
reference  := one or more letters followed by one or more digits   (case-insensitive)
```

- `SUM` is case-insensitive and must be followed by `(`. There are no other functions, no unary plus, no `%`, no `$`, no strings. A `SUM` argument is a range, a single cell, or a number that may be written with a leading `-`; arbitrary expressions and nested `SUM`s are not arguments.
- A range `X:Y` is the inclusive rectangle spanned by the two cells; the corners may come in any order (`B3:A1` equals `A1:B3`).
- A formula that does not match the grammar (unbalanced parentheses, a missing operand, a stray token, an empty `SUM()`, a range outside `SUM`, a letters-only word that is not `SUM`, ...) has the value `#ERR!`. It refers to nothing.
- A reference that is syntactically fine but not a valid address (`Z101`, `AA1`, `A0`, `A01`, `ABC12`) is a `#REF!` error **at evaluation time**: the formula parses, and its value is `#REF!` if evaluation reaches it (see requirement 5). A range with an invalid corner is invalid as a whole.
- `formulaOf(cell)` is the formula text exactly as given with its letters upper-cased (`=a1 + sum(b1:b3)` becomes `=A1 + SUM(B1:B3)`; nothing else changes, whitespace included). It is `""` for a cell that is not a formula; a formula with a syntax error still has its text.

### 4. Values

A cell's value is one of: empty, a number (a 64-bit floating-point value), text, or an error (`#DIV/0!`, `#VALUE!`, `#REF!`, `#CYCLE!`, `#ERR!`).

- A number cell holds the parsed number; a text cell holds its text; a formula cell holds the result of its expression.
- A reference evaluates to the value of the cell it names (empty, number, text or error).
- Arithmetic (`+ - * /` and unary minus) works on numbers; an **empty operand counts as 0**. A **text operand** makes the result `#VALUE!`. An **error operand** makes the result that error (an error wins over a text operand; if both operands are errors, the left one wins). Division by an operand that is `0` (or an empty cell) is `#DIV/0!`; this is checked only after the error and text checks (`text/0` is `#VALUE!`). A result that is not a finite number (overflow) is `#VALUE!`.
- The result of the whole formula is what its expression evaluates to. So `=A1` where `A1` holds text shows that text, `=A1` where `A1` is empty shows `0`, and `=A1` where `A1` is an error shows that error.
- `SUM(arguments)` adds up its arguments in order. A range is visited **row by row** (top-left to bottom-right, whatever order the corners were written in). Number cells are added, **text and empty cells are ignored**, and the first **error** met while visiting (in that order, argument by argument) is the result of the `SUM`. An invalid reference or range reached while visiting gives `#REF!`. A `SUM` with no number at all gives `0`.
- Errors **propagate**: a formula that reads an error cell through arithmetic or `SUM` takes on that error.

### 5. Cycles

A cell is **on a cycle** if following references (including through ranges) from it leads back to it: `=A1` in `A1`, `=B1` in `A1` and `=A1` in `B1`, `=SUM(A1:A3)` in `A2`, and so on. Every cell on a cycle shows `#CYCLE!`. A formula that merely reads a cycle cell (without being on the cycle) reads the error `#CYCLE!` like any other error and therefore shows `#CYCLE!` too. When the cycle is broken by a later `set` (for instance one of its cells gets a number), every affected cell shows its normal value again.

### 6. Display strings (`get`)

- Empty cell: `""`. Text: as stored. Errors: `#DIV/0!`, `#VALUE!`, `#REF!`, `#CYCLE!`, `#ERR!`.
- Numbers: round to 10 decimal places (half-even on the exact binary value of the double), write them in plain decimal notation (never an exponent), strip trailing zeros and a trailing decimal point; a result of zero, including `-0` and a tiny negative number that rounds to zero, is `0`. So `5.0` is `5`, `1/3` is `0.3333333333`, `0.1+0.2` is `0.3`, `-7/2` is `-3.5`, `0.00000000004` is `0`, `100000*100000` is `10000000000`.

### 7. Recomputation and the counter

1. `recomputeCount()` is the number of **formula evaluations** performed since the sheet was created. A formula evaluation is the computation of one formula cell's value, including a formula with a syntax error (it evaluates to `#ERR!`).
2. `set(c, input)` evaluates `c` itself once if `c` is a formula that is not on a cycle, then re-evaluates **every cell that directly or indirectly depends on `c`** (formula cells that read `c`, directly or through a range, and so on) **exactly once**, after everything they read has its new value. It does so even if the cell's value happens to stay the same, even if the same input is set again, and also when `c` is cleared.
3. Cells that are on a cycle are marked `#CYCLE!` **without being evaluated**: they do not add to the counter. A cell that depends on a cycle without being on it is evaluated (and counted) once like any other dependent.
4. Setting a cell nobody depends on to a number, to text or to `""` adds 0. Setting a cell that nobody refers to does not touch any other formula. `get`, the queries and calls that fail never change the counter.
5. A chain of 100 formulas where each reads the previous one: changing the first cell evaluates exactly 100 formulas. A formula reachable by two routes (a diamond: `B1=A1+1`, `C1=A1+2`, `D1=B1+C1`) is evaluated once per change of `A1`, not twice.

### 8. Queries

- `dependencies(cell)`: the valid cells referenced directly by the formula in `cell`, ranges expanded, without duplicates, row by row. Empty if the cell is not a formula, has a syntax error, or references nothing valid. Invalid references contribute nothing.
- `dependents(cell)`: the cells whose formulas reference `cell` directly (also through a range), row by row. It is meaningful for empty cells too (a formula may point at a cell that is still empty). When a formula is replaced or the cell is cleared, its old references stop counting.
- `usedCells()`: the non-empty cells (anything but a cleared or never-set cell), row by row.

### 9. Constraints

- **Thread safety.** `set`, `get` and the queries are called from several threads (`testkit.Concurrently`). Readers must never see a half-updated sheet: after `set` returns, all dependents are up to date, and a `get` during a `set` returns either the state before or after it, never a value computed from a mixture.
- **Scale.** All 2,600 cells may form one chain (each formula reads the previous cell) or one big ring; the head of such a chain is updated 100 times, in a thread with the **default stack size**, and must neither overflow the stack nor be slow (the tests allow generous time bounds; they are looking for a design problem, not a micro-optimisation). A single formula may read a range of 2,574 cells, and 26 formulas may read the same range.

## Examples

### Worked trace 1: values and propagation

```
set(A1, "5")   set(A2, "7")   set(A3, "=A1+A2*2")   set(B1, "=SUM(A1:A3)")
get(A3) -> "19"        get(B1) -> "31"        formulaOf(A3) -> "=A1+A2*2"
dependencies(B1) -> [A1, A2, A3]              dependents(A1) -> [B1, A3]

set(A1, "10")
get(A3) -> "24"        get(B1) -> "41"

set(A2, "text")        (text in arithmetic)
get(A3) -> "#VALUE!"   get(B1) -> "#VALUE!"  (SUM ignores the text, but reads the error in A3)
```

### Worked trace 2: text, empty cells and errors

```
set(A1, "hello")
set(B1, "=A1+1")       get(B1) -> "#VALUE!"
set(C1, "=SUM(A1,5)")  get(C1) -> "5"            (SUM ignores text)
set(D1, "=A1")         get(D1) -> "hello"        (a bare reference copies the value)
set(E1, "=Z9*2+7")     get(E1) -> "7"            (an empty cell is 0)
set(F1, "=1/0")        get(F1) -> "#DIV/0!"
set(G1, "=F1+B1")      get(G1) -> "#DIV/0!"      (left error first)
set(H1, "=B1+F1")      get(H1) -> "#VALUE!"
set(I1, "=Z101+1/0")   get(I1) -> "#REF!"        (evaluated left to right)
set(J1, "=1+")         get(J1) -> "#ERR!"
set(K1, "=1/3")        get(K1) -> "0.3333333333"
```

### Worked trace 3: cycles

```
set(A1, "=B1+1")      get(A1) -> "1"
set(B1, "=A1+1")      get(A1) -> "#CYCLE!"   get(B1) -> "#CYCLE!"
set(C1, "=A1*2")      get(C1) -> "#CYCLE!"   (depends on a cycle, not on it)
set(B1, "4")          get(A1) -> "5"         get(C1) -> "10"
```

### Worked trace 4: counting evaluations

```
recomputeCount() -> 0
set(A1, "1")                                        recomputeCount() -> 0   (a number is not a formula)
set(B1, "=A1+1")   set(C1, "=A1+2")   set(D1, "=B1+C1")
recomputeCount() -> 3                               get(D1) -> "5"
set(A1, "10")                                       recomputeCount() -> 6   (B1, C1, D1 once each)
set(F9, "123")                                      recomputeCount() -> 6   (nobody reads F9)
set(B1, "=A1+3")                                    recomputeCount() -> 8   (B1 itself, then D1)
```

## Edge cases

- `get` of a valid but untouched cell is `""`; `usedCells()` of a new sheet is empty.
- A text that looks like a formula but starts with a space (` =1+1`) is text; `=` alone, `= not a formula` and `=SUM(A1:B2` are formulas with a syntax error (`#ERR!`).
- `=SUM(A1:B2)` when the cell itself lies inside the range is a cycle.
- An error cell referenced through `SUM(A1:B2)`: the first error in row-by-row order wins, so with `B1` = `#DIV/0!` and `A2` = `#REF!` the result is `#DIV/0!`, although `A2` comes first column-by-column.
- A syntax error wins over everything else in the same formula: `=Z101+` is `#ERR!`, not `#REF!`.
- A bare reference to a text cell copies the text; `=-A1`, `=A1+0` and `=A1*1` on a text cell are `#VALUE!`; `SUM` of text cells is `0`.
- Clearing a cell that other formulas read turns it into 0 for them (and in `SUM`, into nothing) and keeps those formulas alive.
- `-0` never shows: `=0*-1` is `0`.
- The counter counts evaluations, not value changes: setting `B1` to the same input again re-evaluates every dependent of `B1`.
- Setting the cell that completes a cycle: the cells on the cycle are not evaluated and so add 0 to the counter; the cells hanging off the cycle are evaluated once each.
- An invalid address (or a `null` input in Java) is rejected with no effect, no counter change, no partial update.

## Glossary

- **Cell**: one of the 2,600 slots of the sheet, addressed by column letter and row number.
- **Formula**: a cell input starting with `=`, evaluated to a value that is kept up to date.
- **Reference**: a cell address used inside a formula; **range**: a rectangle of cells written `X:Y`.
- **Dependency / dependent**: if the formula in `P` reads `Q` then `Q` is a dependency of `P` and `P` is a dependent of `Q`. *Direct* means by one step; *indirect* through other formulas.
- **Cycle**: a group of formulas that (transitively) read themselves.
- **Evaluation**: computing the value of one formula cell once.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for an invalid address (on every method that takes one) and for a `null` input to `set`. Nothing is changed by a call that throws. The other methods never throw. Lists are `List<String>` of upper-case addresses; `recomputeCount()` is a `long`.
- Lists are sorted row by row (`A1, B1, ..., Z1, A2, ...`) and contain no duplicates.
- Display strings, error values, evaluation order of errors and the counting rules are exactly as described in the requirements.
- `set` returns only after every dependent has been brought up to date.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever types you like and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface FormulaSheet {
    void set(String cell, String input);          // "" clears, "=..." formula, number literal, otherwise text
    String get(String cell);                      // display string, "" for an empty cell
    String formulaOf(String cell);                // formula text, letters upper-cased; "" if not a formula
    List<String> dependencies(String cell);       // valid cells this formula refers to directly (ranges expanded)
    List<String> dependents(String cell);         // cells whose formulas refer to this cell directly
    List<String> usedCells();                     // all non-empty cells
    long recomputeCount();                        // formula evaluations performed so far
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static FormulaSheet create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. No clock, no randomness, no sleeping.

- Numbers, text and empty cells; which inputs are numbers and which are text; clearing; address validation and case-insensitivity.
- Number formatting; precedence, associativity, unary minus, whitespace; empty cells as 0.
- `SUM` over rectangles in any corner order, over lists of ranges, cells and numbers; text and empty cells in `SUM` and in arithmetic.
- `#DIV/0!`, `#VALUE!`, `#REF!`, `#ERR!` and their propagation, including which error wins.
- `formulaOf`, `dependencies`, `dependents`, `usedCells`, also after a formula is replaced.
- Cycles (self, pair, through a range, long) and their repair.
- The evaluation counter: unrelated changes, diamonds, a chain of 100, cycles.
- Scale: a chain through all 2,600 cells updated 100 times on a default stack.
- Concurrency: 8 threads building columns at once with exact counter value; readers racing writers never see a half-updated chain.
- More edge-case scenarios (backward-built chains, rings of 2,600 cells, wide fans, huge ranges, racing readers) are in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/143-excel-formulas test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

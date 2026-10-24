# 133 · Spreadsheet Grid (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The grid part of a spreadsheet such as Excel. Cells are addressed like `A1` or `ZZ1000000`, hold a number or a piece of text, and carry a visual style (bold, font size, colours, number format). You can fill, clear, copy and move rectangles of cells, insert and delete whole rows and columns, and sort a block of rows. Only a tiny fraction of the million-by-702 grid is ever used, and a sheet may hold hundreds of thousands of cells while the user keeps inserting rows, so the memory and time you spend must follow the cells that exist, not the size of the grid. There are **no formulas** in this problem: a cell holds exactly what was typed into it.

## Requirements

### Addresses and ranges

1. A cell address is one or two letters followed by a row number: columns `A`..`Z`, `AA`..`ZZ` (1..702, `A`=1, `Z`=26, `AA`=27, `ZZ`=702) and rows 1..1,000,000 written without leading zeros. Letters are case-insensitive (`b3` is `B3`); every other form is an error: empty or null, no digits, no letters, `A0`, `A01`, `AAA1`, `A1000001`, embedded or surrounding whitespace, signs, `$`, decimals, `A1:B2`.
2. A range is given as two addresses, top-left and bottom-right. If the top-left is below or to the right of the bottom-right the range is an error. A column argument (`insertColumn`, `sortRange`) is the column letters only (`"C"`, case-insensitive); anything else is an error.
3. Every error is an `IllegalArgumentException` and **leaves the grid completely unchanged** (no half-applied operation).

### Values

4. `setNumber(cell, value)` stores a number; NaN and infinities are errors. `setText(cell, text)` stores text (null is an error); text that looks like a number stays text. `setText(cell, "")` empties the value but keeps the style. `clear(cell)` removes **both** value and style: the cell becomes an unused default cell.
5. `valueType(cell)` is `EMPTY`, `NUMBER` or `TEXT`. An unused or cleared cell is `EMPTY`; so is a cell that only has a style.
6. `getValue(cell)` returns the display string: `""` for `EMPTY`, the text exactly as stored for `TEXT`, and for `NUMBER` the rendering below. Text is never affected by number formats.

### Number display

Let *d* be the shortest decimal representation of the stored double that reads back to the same double (so `0.1 + 0.2` is `0.30000000000000004`). All rounding is **half away from zero** on that decimal. A result that rounds to zero is shown as `0` / `0.00` / `0%`, never with a minus sign; `-0` is stored and shown as `0`. Numbers are never shown with an exponent.

| Number format | Rendering |
|---|---|
| `GENERAL` | rounded to at most 10 decimals, trailing zeros (and a dangling point) removed: `1.5`, `2`, `0.3`, `0.0000000001`, `1000000000000000` |
| `INTEGER` | rounded to a whole number: `2.5` -> `3`, `-2.5` -> `-3`, `-0.4` -> `0` |
| `TWO_DECIMALS` | exactly two decimals: `3` -> `3.00`, `1.005` -> `1.01`, `-0.001` -> `0.00` |
| `PERCENT` | *d* x 100 rounded to a whole number, then `%`: `0.256` -> `26%`, `0.005` -> `1%`, `-0.5` -> `-50%`, `12` -> `1200%` |

Changing the format of a cell changes how its stored number is displayed from then on; the stored number itself is untouched.

### Styles

7. A style is the combination (bold, italic, font size, text colour, fill colour, number format). The default style is `(false, false, 11, "black", "none", GENERAL)`; every cell has a style, unused cells have the default one.
8. `StyleChange` (provided) has optional fields; a missing field leaves that part of the style as it is. Valid values: font size 8..72; a colour is a non-empty name without whitespace and without `|`, stored in **lower case**; the number format is one of `GENERAL`, `INTEGER`, `TWO_DECIMALS`, `PERCENT`, accepted case-insensitively and stored in **upper case**. An invalid field, or a null change, is an error and nothing at all is applied (also for ranges: no cell changes).
9. `setStyle(cell, change)` changes one cell, `setStyleRange(topLeft, bottomRight, change)` every cell of a rectangle (values are kept). Styling an empty cell makes it a *styled* cell. A cell whose style equals the default style and whose value is empty is simply an unused cell again.
10. `styleOf(cell)` returns the style as the string `bold|italic|fontSize|textColor|fillColor|numberFormat`, with `true`/`false` in lower case, e.g. `true|false|11|black|none|GENERAL` or `false|true|14|red|light-blue|PERCENT`.
11. `distinctStyleCount()` is the number of different **non-default** styles held, right now, by at least one cell. Two cells with equal styles count once, whichever way they got them; a style nobody holds any more does not count.

### Range operations

12. `fillNumber(topLeft, bottomRight, value)` / `fillText(...)` set the value of every cell of the rectangle and keep the styles (`fillText` with `""` empties the values).
13. `clearRange(topLeft, bottomRight)` clears value **and** style of every cell inside.
14. `copy(srcTopLeft, srcBottomRight, destTopLeft)` copies values **and** styles of the source rectangle onto the same-sized rectangle whose top-left is `destTopLeft`. Every destination cell is overwritten, so an empty (unused) source cell empties the destination cell completely (value and style). Overlapping ranges behave as if the source had first been copied through a temporary buffer. If the destination rectangle would leave the grid (past row 1,000,000 or column ZZ), it is an error.
15. `move(...)` is cut-and-paste: the same as `copy`, and afterwards every source cell that is **not** inside the destination rectangle is cleared (value and style).
16. Limit: `setStyleRange`, `fillNumber`, `fillText`, `copy`, `move` and `sortRange` reject a range with more than 1,000,000 cells (width x height). `clearRange` has no limit and must stay fast on `A1:ZZ1000000`.

### Structure

17. `insertRow(r)` shifts every cell of rows `r` and below down by one (values and styles) and leaves row `r` empty (it does not inherit any style). A cell pushed past row 1,000,000 vanishes. `deleteRow(r)` removes row `r`: its cells vanish and the rows below move up. `insertColumn(c)` / `deleteColumn(c)` do the same for columns (shifting right/left, cells pushed past `ZZ` vanish). A row outside 1..1,000,000 or an invalid column is an error. Deleting a completely empty row or column is fine.

### Queries

18. `usedRange()` is the bounding box of all cells that hold a value **or** a non-default style, written `TOPLEFT:BOTTOMRIGHT` (e.g. `B2:D7`; a single cell is `C3:C3`), or `""` when there are none. `rowCount()` is the largest used row number and `columnCount()` the largest used column number (`AA` is 27), `0` when nothing is used.

### Sorting

19. `sortRange(topLeft, bottomRight, byColumn, ascending)` reorders the **rows** of the rectangle by the value in column `byColumn`, which must lie inside the rectangle's columns (otherwise an error). Whole rows of the rectangle move, with values and styles; cells outside the rectangle are untouched.
20. Order: numbers first (by numeric value), then text (compared case-insensitively, by code point after lower-casing), then empty values. In descending order only the order **inside** the numbers group and **inside** the text group is reversed; the groups keep their positions and empty values stay last. The sort is **stable** in both directions: rows with equal keys (`2` and `2.0`; `a` and `A`; two empty keys) keep their original relative order.

## Constraints

- The structure is shared by many threads: every operation is atomic as seen by the others, and no write may be lost. Eight threads writing distinct cells all land; fifty `insertRow(1)` calls from each of eight threads move a cell from `A1` to `A401`.
- Sparse: a grid with 200,000 used cells, followed by 1,000 row/column insertions and deletions, has to finish in seconds, and `clearRange` over the full grid must not walk 700 million addresses.
- Memory follows the number of used cells (and distinct styles), not the grid size.

## Examples

```
setNumber("a1", 1234.5678)       getValue("A1") -> "1234.5678"        valueType("A1") -> NUMBER
setStyle("A1", numberFormat=TWO_DECIMALS)   getValue("A1") -> "1234.57"
setStyle("A1", numberFormat=INTEGER)        getValue("A1") -> "1235"
setStyle("A1", numberFormat=PERCENT)        getValue("A1") -> "123457%"
setText("A2", "1234.5678"), same style      getValue("A2") -> "1234.5678"   (text is never reformatted)
```

```
setStyle("B2", bold=true)                              styleOf("B2") -> "true|false|11|black|none|GENERAL"
setStyle("b2", italic=true, fontSize=14, textColor="RED", fillColor="Light-Blue", numberFormat="percent")
                                                       styleOf("B2") -> "true|true|14|red|light-blue|PERCENT"
setStyle("B2", bold=false, fillColor="none")           styleOf("B2") -> "false|true|14|red|none|PERCENT"
setStyle("A1", fontSize=7)  -> IllegalArgumentException (A1 is unchanged)

setStyleRange("A1","A100", bold=true)   distinctStyleCount() -> 1
setStyle("B1", bold=true, italic=true)  distinctStyleCount() -> 2
setStyle("C1", bold=true)               distinctStyleCount() -> 2     (same style as A1..A100)
clear("B1")                             distinctStyleCount() -> 1
```

```
A1=1  A2=2  A3=3                     copy("A1","A3","A2")  ->  A1..A4 = 1,1,2,3   (as if through a buffer)

A1=1 B1=2 A2=3 B2=4                  move("A1","B2","B2")  (destination is B2:C3)
   -> B2=1 C2=2 B3=3 C3=4, and A1, B1, A2 are now empty (outside the destination), usedRange() -> "B2:C3"

A1=1 A2=2 B3=3(bold)                 insertRow(2)  -> A1=1, row 2 empty, A3=2, B4=3 (still bold)
A1=1 A2=2 A3=3 C3=33                 deleteRow(2)  -> A1=1, A2=3, C2=33, usedRange() -> "A1:C2"
setNumber("ZZ1", 5) ; insertColumn("A") -> the cell is pushed past ZZ and vanishes
```

```
rows (key in A, payload in B):  1:"b"  2:"A"  3:"a"  4:2  5:1  6:(empty key)  7:"B"  8:"x"   payload = row number
sortRange("A1","B8","A",true)   keys -> 1,2,A,a,b,B,x,(empty)   payloads -> 5,4,2,3,1,7,8,6
sortRange("A1","B8","A",false)  keys -> 2,1,x,b,B,A,a,(empty)   payloads -> 4,5,8,1,7,2,3,6
```

### Edge cases

- `A01`, `A0`, `a 1`, `A1000001`, `AAA1` and `""` are all errors, and so is a null address (Java). `ZZ1000000` and `a1` are fine.
- A style-only cell is *used*: `setStyle("C3", bold=true)` gives `usedRange() == "C3:C3"` while `getValue("C3")` is `""`; setting `bold=false` again makes it unused and `usedRange()` is `""` again.
- `setText(c, "")` keeps the style, `clear(c)` does not.
- `setNumber("A1", 0.00000000005)` shows `0.0000000001`; `-0.00000000001` shows `0`; `1.005` in `TWO_DECIMALS` shows `1.01`.
- `copy("A1","A2","A1000000")` leaves the grid (rows 1,000,000 and 1,000,001) and is an error that changes nothing; `copy("A1","A2","A999999")` is fine.
- `move` onto itself (same top-left) changes nothing.
- A range of 1,000,002 cells given to `fillNumber` is an error; `clearRange("A1","ZZ1000000")` is fine.
- `deleteRow` of an empty row still pulls the rows below it up.
- `sortRange` by a column outside the rectangle, or with a reversed rectangle, is an error and changes nothing.

## Glossary

- **Cell**: one address; unused unless it holds a value or a non-default style.
- **Used cell**: a cell with a value or a non-default style (these are the cells `usedRange`, `rowCount` and `columnCount` look at).
- **Style**: the immutable combination of the six formatting attributes; equal combinations are the same style.
- **Range / rectangle**: all cells between a top-left and a bottom-right address, inclusive.
- **Structure operation**: insert or delete a whole row or column.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `IllegalArgumentException` for every invalid address, range, column, value, style field or destination described above; the grid is unchanged afterwards.
- `StyleChange` and `ValueType` are provided in `contract`; numbers are `double`, rows are `int`, columns are `String` letters.
- Number display, the style string and the ordering rules are exactly as written above.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract in `src/main/java/contract/SpreadsheetGrid.java` (read it for the per-method documentation). It is **not** your domain model — it is the doorway. Create whatever you like and wire it up in `solution/Solution.java`.

```java
public interface SpreadsheetGrid {
    void setNumber(String cell, double value);      void setText(String cell, String text);
    void clear(String cell);                        String getValue(String cell);
    ValueType valueType(String cell);
    void setStyle(String cell, StyleChange change); String styleOf(String cell);
    void setStyleRange(String topLeft, String bottomRight, StyleChange change);
    int distinctStyleCount();
    void fillNumber(String topLeft, String bottomRight, double value);
    void fillText(String topLeft, String bottomRight, String text);
    void clearRange(String topLeft, String bottomRight);
    void copy(String srcTopLeft, String srcBottomRight, String destTopLeft);
    void move(String srcTopLeft, String srcBottomRight, String destTopLeft);
    void insertRow(int row);        void deleteRow(int row);
    void insertColumn(String column);  void deleteColumn(String column);
    String usedRange();  int rowCount();  int columnCount();
    void sortRange(String topLeft, String bottomRight, String byColumn, boolean ascending);
}

public final class Solution { public static SpreadsheetGrid create() }   // solution/Solution.java
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works.

- Empty grid, addresses (valid, case, every kind of invalid), bad numbers.
- Number display for each format including the rounding edge cases; re-rendering when the format changes; text untouched.
- Partial style updates, normalisation of names, atomic rejection of bad style changes, `distinctStyleCount` through setting, sharing and clearing, styled-but-empty cells.
- Rectangle styling, fill, clear, the range limits and reversed ranges.
- Copy (values + styles, empties overwrite), overlapping copy, destination outside the grid, move.
- Insert/delete rows and columns, cells pushed off the grid, `usedRange`/counts.
- Sorting: groups, stability, descending, partial rectangles, bad keys.
- Concurrency: distinct-cell writes from 8 threads, concurrent row insertions.
- Scale: 200,000 cells and 1,000 structure operations in bounded time.
- More scenarios (a randomised comparison against a small model, grid corners, sorting 50,000 rows, readers during structure changes) are in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/133-spreadsheet-grid test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

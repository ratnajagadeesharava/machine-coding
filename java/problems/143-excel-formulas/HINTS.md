# Hints — 143 Excel Cell Values and Sum Formulas (Java)

Try the problem for at least 20 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

When `A1` changes, which cells have to be looked at, and in what order must they be looked at so that nobody reads a value that is about to change again? Draw the cells and "reads" arrows for a diamond and for a chain of five and write down the order.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep, for every cell, both who it reads and who reads it, and update both sides whenever a formula is replaced. Turn formula text into a small tree once, when it is set; the tree is what you evaluate afterwards and also where the list of referenced cells comes from. On a change, collect everything that depends on the changed cell, put it in an order where every cell comes after everything it reads, and evaluate in that order. Cells that cannot be put in such an order are exactly the ones involved in cycles or hanging off them; deciding which is which needs a little more than "could not be ordered".

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Cell { input, value, tree (optional), reads: sorted set of cells }` plus `readers: cell -> sorted set of cells`. `set`: remove the cell from the `readers` of its old `reads`, parse the new input, add it to the `readers` of the new `reads`, then walk `readers` from the cell to collect the affected set, find strongly connected components in that sub-graph (a component of size > 1 or with a self edge is a cycle: mark `#CYCLE!`, do not count), process the components in dependency order and evaluate the rest once each. A recursive-descent parser with one function per precedence level handles the grammar; evaluation is a recursive walk over the tree whose depth is the nesting of the formula, not the length of the chain of cells.

Do all graph walks (collecting the affected cells, ordering, finding cycles) with an explicit stack or queue: the chain and the ring through 2,600 cells are deeper than a default thread stack allows for recursion. **Java:** a `record`/sealed interface per node type and a `switch` over it keeps the evaluator small; `ArrayDeque` for the explicit stack; format numbers with `BigDecimal` (`setScale(10, RoundingMode.HALF_EVEN)`, `stripTrailingZeros().toPlainString()`), not `String.format`/`Double.toString` (exponents, locale).

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

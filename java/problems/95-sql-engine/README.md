# 95 · Mini SQL Engine (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

We want a tiny in-memory SQL database for an embedded product: a handful of tables, a restricted SQL dialect, no persistence.
Callers hand it one statement as a string and get back column names and rows as text, or an error. It has to be correct for
the dialect described below (it is intentionally small, but exact), reject everything else with a clear error without half-applying
a statement, and stay usable when several threads issue statements at the same time.

## Requirements

1. `execute(sql)` runs exactly one statement and returns `QueryResult(columns, rows, affected)`. Anything invalid throws `SqlException` (Rust: `Err(SqlError)`) and **changes nothing**.
2. Grammar (keywords case-insensitive, table and column names case-sensitive, one optional trailing `;`): `CREATE TABLE t (col INT|TEXT [PRIMARY KEY], ...)`; `INSERT INTO t VALUES (literal, ...)`; `SELECT * | item, ... FROM t [JOIN u ON t.a = u.b] [WHERE cond] [GROUP BY col] [ORDER BY col [ASC|DESC]] [LIMIT n]`; `UPDATE t SET col = literal | col = col + n | col = col - n [, ...] [WHERE cond]`; `DELETE FROM t [WHERE cond]`.
3. Select items: `col`, `t.col`, `COUNT(*)`, `SUM(col)`, `MIN(col)`, `MAX(col)`. Literals: integers (`42`, `-7`) and single-quoted text with `''` as an escaped quote (`'it''s'`).
4. Conditions: `col (= | != | <> | < | <= | > | >=) literal`, combined with `AND`, `OR` and parentheses; **AND binds tighter than OR**.
5. Types: `INT` and `TEXT`. A literal must match its column's type (also in comparisons and `SET`). At most one `PRIMARY KEY` column; its values are unique and an INSERT or UPDATE that would duplicate one fails and changes nothing.
6. Rows come back in insertion order unless `ORDER BY` says otherwise; ordering is numeric for INT and alphabetical (by code point) for TEXT; ties keep insertion order (stable sort).
7. Aggregates without `GROUP BY` return exactly one row (`COUNT(*)` of nothing is `0`); with `GROUP BY col` one row per distinct value, in order of first appearance unless `ORDER BY` is given. `SUM` works on INT only; `MIN`/`MAX` work on INT (numeric) and TEXT (alphabetical).
8. `JOIN ... ON a = b` is an inner equi-join of exactly two tables; matched row pairs follow the left table's row order, then the right table's.
9. Result values are text: integers in decimal, text as is. `affected` is the number of rows inserted, updated or deleted (0 for SELECT and CREATE TABLE).
10. Statements are serialised: each behaves as if it ran alone, before or after any other concurrent statement.

## Constraints

- All of the calls on one engine may come from many threads; every statement must be atomic with respect to the others (no lost inserts, no lost increments, no torn reads of an aggregate).
- A multi-row `UPDATE` is all-or-nothing: if it fails on the third row (for example a primary-key clash) the first two rows are not changed.
- Reads should not see a half-applied statement; whether readers run in parallel is up to you.
- No clocks and no randomness are involved; behaviour must be deterministic.

## Examples

```
CREATE TABLE users (id INT PRIMARY KEY, name TEXT, age INT, city TEXT)
INSERT INTO users VALUES (1, 'ann', 31, 'blr')     -- affected = 1
SELECT city, COUNT(*), SUM(age) FROM users GROUP BY city ORDER BY city
  columns: city | COUNT(*) | SUM(age)
  rows:    blr, 2, 71  /  del, 1, 25  /  mum, 1, 25
SELECT id FROM users WHERE id = 1 OR id = 2 AND age = 99     -- id=1 OR (id=2 AND age=99)
SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.uid
  columns: users.name | orders.amount
UPDATE users SET age = age + 1 WHERE city = 'blr'            -- affected = 2
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `SqlEngine.execute(String) -> QueryResult(List<String> columns, List<List<String>> rows, int affected)`, errors are `SqlException` (unchecked), factory `Solution.create()`. Rust: `execute(&self, &str) -> Result<QueryResult, SqlError>` with `usize affected`, `SqlError(pub String)`.
- Output column names: a plain column is named as written in the select list (`city`, or `users.name` if written qualified); `SELECT *` yields the table's columns in declaration order; aggregates are named exactly as written with canonical spelling: `COUNT(*)`, `SUM(age)`, `MIN(age)`, `MAX(name)`.
- With a JOIN, `SELECT *` yields all columns of the left table then the right, each qualified: `users.id, users.name, ..., orders.oid, ...`. Unqualified column names are accepted in a join when unambiguous (`name`, `amount`, `id = uid` in the tests); an ambiguous one should be an error.
- ORDER BY may name a select column, a table column or an aggregate expression (`ORDER BY COUNT(*) DESC`, `ORDER BY orders.amount DESC`). LIMIT is applied after ORDER BY; `LIMIT -1` is an error.
- Mixing plain columns with aggregates is an error unless the plain column is the GROUP BY column (`SELECT name, COUNT(*) FROM users` fails).
- Errors (all `SqlException`): unknown statement or syntax (`SELEC`, empty string, dangling `WHERE`), unknown table or column, type mismatch (`age > 'x'`, `'six'` for an INT column, `name = name + 1`), wrong value count on INSERT, `SUM` of TEXT, duplicate table, unsupported column type (`FLOAT`), primary-key violation.
- `UPDATE ... SET id = 7` without WHERE on four rows fails (the rows would share a key) and leaves all rows untouched. `SET col = col + n` is allowed on INT columns only.
- A failed statement never changes the data and never leaves a table half-created.
- Integers are 64-bit-safe in decimal text; `-7` is a literal, `col - n` is the only arithmetic form (no expressions in SELECT or WHERE).

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/**
 * What a statement returns. Provided.
 *
 * @param columns output column names (empty for statements that return no rows)
 * @param rows    every value as text: integers in decimal, text as is
 * @param affected rows inserted / updated / deleted (0 for SELECT and CREATE TABLE)
 */
public record QueryResult(List<String> columns, List<List<String>> rows, int affected) {}

package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe (statements behave as if they ran one
 * after the other).
 *
 * <p>Supported SQL (keywords case-insensitive, names case-sensitive, optional trailing ';'):
 * <pre>
 * CREATE TABLE t (col INT|TEXT [PRIMARY KEY], ...)
 * INSERT INTO t VALUES (literal, ...)                      -- one row; literals: 42, -7, 'text', 'it''s'
 * SELECT * | item, ... FROM t [JOIN u ON t.a = u.b]
 *        [WHERE cond] [GROUP BY col] [ORDER BY col [ASC|DESC]] [LIMIT n]
 *      item := col | t.col | COUNT(*) | SUM(col) | MIN(col) | MAX(col)
 * UPDATE t SET col = literal | col = col + n | col = col - n [, ...] [WHERE cond]
 * DELETE FROM t [WHERE cond]
 *      cond := comparison | cond AND cond | cond OR cond | ( cond )     -- AND binds tighter than OR
 *      comparison := col (= | != | &lt;&gt; | &lt; | &lt;= | &gt; | &gt;=) literal
 * </pre>
 */
public interface SqlEngine {

    /** Runs one statement. Throws {@link SqlException} for anything invalid. */
    QueryResult execute(String sql);
}

package contract;

/** Any syntax or semantic error. The statement that caused it changed nothing. Provided. */
public class SqlException extends RuntimeException {
    public SqlException(String message) {
        super(message);
    }
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static SqlEngine create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- SELECT * returns columns in declaration order and rows in insertion order; projection order; lower-case keywords; trailing semicolon accepted.
- WHERE: all six comparison operators (`!=` and `<>`), AND/OR precedence, parentheses.
- ORDER BY numeric vs alphabetical (id 10 after 4, not after 1), DESC, LIMIT; stable ties.
- Aggregates with and without GROUP BY, including `COUNT(*)` = 0 on no match and ordering by an aggregate.
- Two-table JOIN with qualified and unqualified columns, correct column names for `SELECT *`.
- UPDATE with literal and `col +/- n`, affected counts; DELETE with and without WHERE.
- Primary-key enforcement on INSERT and UPDATE, with nothing changed on failure; quote escaping `'it''s'`; text containing commas.
- A list of 16 invalid statements all raise `SqlException`.
- Concurrency: 8 threads x 250 inserts lose nothing; 16 threads racing for one primary key - exactly one wins; 16 x 250 `n = n + 1` end at exactly 4000; readers summing balances while writers update never see a torn single statement; **repeat 100x** a mixed insert/update workload.
- More edge cases and scale scenarios in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

A hand-written tokenizer plus recursive descent is shorter than regex soup; keep `''` handling inside the string-literal scanner. `LinkedHashMap` gives first-seen order for groups and `List.sort` is stable, which is exactly the tie rule.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/95-sql-engine test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.

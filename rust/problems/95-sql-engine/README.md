# 95 · Mini SQL Engine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building a tiny in-memory SQL database for an embedded product: a few tables, a deliberately small SQL dialect, no
persistence. The caller hands the engine one statement as a string and gets back column names and rows as text, or an error.
The dialect is small but must be implemented exactly: it supports creating tables, inserting, selecting (with filtering, joining
two tables, grouping, aggregating, ordering and limiting), updating and deleting. Anything outside the dialect is rejected with a
clear error, and a rejected statement must change nothing. Several threads may issue statements at the same time and each
statement must behave as if it ran alone.

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
11. `QueryResult.columns` is empty and `rows` is empty for `CREATE TABLE`, `INSERT`, `UPDATE` and `DELETE`; only `affected` is meaningful there (`CREATE TABLE` reports 0, `INSERT` 1).
12. Comparisons are `column OP literal` with the literal on the right; INT columns compare numerically, TEXT columns compare alphabetically by code point (so `'Z' < 'a'`). `=` and `!=`/`<>` are equality and inequality.
13. Evaluation order of the clauses: filter rows (`WHERE`), group/aggregate (`GROUP BY`), sort (`ORDER BY`), cut (`LIMIT`).
14. On `SELECT` with a `JOIN`, a `WHERE`/`ORDER BY`/`GROUP BY` column may be written qualified (`orders.amount`) or, when only one table has it, unqualified (`amount`). Rows whose join columns have no partner on the other side are dropped (inner join).

## Constraints

- All of the calls on one engine may come from many threads; every statement must be atomic with respect to the others (no lost inserts, no lost increments, no torn reads of an aggregate).
- A multi-row `UPDATE` is all-or-nothing: if it fails on the third row (for example a primary-key clash) the first two rows are not changed.
- Reads should not see a half-applied statement; whether readers run in parallel is up to you.
- No clocks and no randomness are involved; behaviour must be deterministic.

## Glossary

- **Literal**: a constant value in the statement: an integer (`42`, `-7`) or a single-quoted text (`'ann'`, `'it''s'` means `it's`).
- **Primary key**: a column whose values must be unique within the table; inserting or updating a row into a duplicate fails.
- **Aggregate**: a function over several rows giving one value: `COUNT(*)` rows, `SUM(col)` of an INT column, `MIN(col)` / `MAX(col)` of an INT or TEXT column.
- **GROUP BY**: splits the rows by the value of one column and computes the aggregates per group; groups come out in order of first appearance unless `ORDER BY` says otherwise.
- **Inner equi-join**: pairs every row of the left table with every row of the right table whose joined column has an equal value.
- **Stable sort**: rows that compare equal under `ORDER BY` keep their relative insertion order.

## Examples

Each example assumes the `users` table below freshly created (they are independent, not one running session):

```
CREATE TABLE users (id INT PRIMARY KEY, name TEXT, age INT, city TEXT)
INSERT INTO users VALUES (1, 'ann', 31, 'blr')      -- affected = 1
INSERT INTO users VALUES (2, 'bob', 25, 'del')
INSERT INTO users VALUES (3, 'cy',  40, 'blr')
INSERT INTO users VALUES (4, 'dee', 25, 'mum');     -- optional trailing ';'
```

| Statement | columns | rows (each row in `[...]`) |
|---|---|---|
| `SELECT * FROM users` | id, name, age, city | [1,ann,31,blr] [2,bob,25,del] [3,cy,40,blr] [4,dee,25,mum] |
| `select city, name from users where id = 3` | city, name | [blr,cy] |
| `SELECT name FROM users WHERE age > 25` | name | [ann] [cy] |
| `SELECT id FROM users WHERE id = 1 OR id = 2 AND age = 99` | id | [1]  (`AND` first: `id=1 OR (id=2 AND age=99)`) |
| `SELECT id FROM users WHERE (id = 1 OR id = 2) AND age = 25` | id | [2] |
| `SELECT name FROM users ORDER BY age` | name | [bob] [dee] [ann] [cy]  (ties in insertion order) |
| `SELECT id FROM users ORDER BY id DESC LIMIT 2` | id | [4] [3] |
| `SELECT COUNT(*), SUM(age), MIN(age), MAX(name) FROM users WHERE city = 'blr'` | COUNT(*), SUM(age), MIN(age), MAX(name) | [2,71,31,cy] |
| `SELECT COUNT(*) FROM users WHERE id = 99` | COUNT(*) | [0] |
| `SELECT city, COUNT(*), SUM(age) FROM users GROUP BY city ORDER BY city` | city, COUNT(*), SUM(age) | [blr,2,71] [del,1,25] [mum,1,25] |

**Updates, deletes and failures**

```
UPDATE users SET age = age + 1 WHERE city = 'blr'            -- affected = 2 (ann 32, cy 41)
UPDATE users SET city = 'pune', age = age - 5 WHERE id = 2   -- affected = 1
UPDATE users SET age = 0 WHERE id = 999                      -- affected = 0 (no error)
DELETE FROM users WHERE age = 25                             -- affected = number of removed rows
DELETE FROM users                                            -- removes all rows
INSERT INTO users VALUES (1, 'dup', 1, 'x')                  -- error: primary key 1 exists, nothing changes
UPDATE users SET id = 7                                      -- error: four rows would share key 7; no row changes
INSERT INTO users VALUES (5, 'it''s', 1, 'a,b')              -- stores name  it's  and city  a,b
SELECT name, COUNT(*) FROM users                             -- error: plain column next to an aggregate without GROUP BY
```

**Join** (with `orders (oid INT PRIMARY KEY, uid INT, amount INT)` holding (100,1,50) (101,1,70) (102,3,20) (103,99,5))

```
SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.uid ORDER BY orders.amount DESC
  columns: users.name | orders.amount        rows: [ann,70] [ann,50] [cy,20]      (order 103 has no user)
SELECT name, SUM(amount) FROM users JOIN orders ON id = uid GROUP BY name ORDER BY name
  rows: [ann,120] [cy,20]
SELECT * FROM users JOIN orders ON users.id = orders.uid
  columns: users.id, users.name, users.age, users.city, orders.oid, orders.uid, orders.amount
```

## Edge cases

- Keywords are case-insensitive (`select`, `Select`); table and column names are case-sensitive (`Users` is not `users`).
- Errors are `SqlError` for: an unknown statement or syntax error (`SELEC ...`, an empty string, `WHERE` with nothing after it), unknown table or column, type mismatch (`age > 'x'`, `'six'` into an INT column, `name = name + 1`), wrong number of values in `INSERT`, `SUM` of a TEXT column, creating a table twice, a column type other than `INT`/`TEXT` (`FLOAT`), a primary-key duplicate, `LIMIT -1`.
- An error never leaves partial changes: a multi-row `UPDATE` that fails on the third row leaves the first two rows unchanged; a failed `CREATE TABLE` creates nothing.
- `SELECT` aggregates without `GROUP BY` always return exactly one row, also on an empty table (`COUNT(*)` = `0`).
- Output column names are the select items as written: `city`, `users.name`, and the canonical `COUNT(*)`, `SUM(age)`, `MIN(age)`, `MAX(name)`.
- No arithmetic anywhere except `SET col = col + n` / `col - n` on INT columns.
- Values come back as text: integers in decimal (`-7`), text exactly as stored (without the quotes).

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

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`); statements behave as if they ran one after the other.
//!
//! Supported SQL (keywords case-insensitive, names case-sensitive, optional trailing ';'):
//!
//! ```text
//! CREATE TABLE t (col INT|TEXT [PRIMARY KEY], ...)
//! INSERT INTO t VALUES (literal, ...)                      -- one row; literals: 42, -7, 'text', 'it''s'
//! SELECT * | item, ... FROM t [JOIN u ON t.a = u.b]
//!        [WHERE cond] [GROUP BY col] [ORDER BY col [ASC|DESC]] [LIMIT n]
//!      item := col | t.col | COUNT(*) | SUM(col) | MIN(col) | MAX(col)
//! UPDATE t SET col = literal | col = col + n | col = col - n [, ...] [WHERE cond]
//! DELETE FROM t [WHERE cond]
//!      cond := comparison | cond AND cond | cond OR cond | ( cond )     -- AND binds tighter than OR
//!      comparison := col (= | != | <> | < | <= | > | >=) literal
//! ```

/// Any syntax or semantic error. The statement that caused it changed nothing. Provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlError(pub String);

/// What a statement returns. Provided.
///
/// * `columns`: output column names (empty for statements that return no rows)
/// * `rows`: every value as text: integers in decimal, text as is
/// * `affected`: rows inserted / updated / deleted (0 for SELECT and CREATE TABLE)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected: usize,
}

pub trait SqlEngine: Send + Sync {
    /// Runs one statement. `Err(SqlError)` for anything invalid.
    fn execute(&self, sql: &str) -> Result<QueryResult, SqlError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn SqlEngine>> { ... }
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

## Running

```bash
cargo test -p p95_sql_engine
STRICT=1 cargo test -p p95_sql_engine   # fail instead of skip while unimplemented
cargo test -p p95_sql_engine --features hidden   # interviewer scenarios
cargo play run 95              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

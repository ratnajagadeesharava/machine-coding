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

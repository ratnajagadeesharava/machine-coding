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

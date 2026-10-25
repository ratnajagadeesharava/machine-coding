package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Addresses are {@code "A1"} style: one or two letters (A..ZZ, columns 1..702) followed by a row number 1..1,000,000
 * without leading zeros. Input is case-insensitive (normalised to upper case); surrounding whitespace is NOT allowed.
 * A malformed or out-of-range address, a null argument, or any other invalid argument raises
 * {@link IllegalArgumentException} and leaves the grid unchanged. A "range" is a top-left and a bottom-right address;
 * the top-left must not be below or right of the bottom-right. Columns are passed as letters ("C").
 */
public interface SpreadsheetGrid {

    // ---- cells ----

    /** Stores a number (NaN and infinities are rejected). The cell's style is kept. */
    void setNumber(String cell, double value);

    /** Stores text (null is rejected). Empty text empties the cell's value but keeps its style. */
    void setText(String cell, String text);

    /** Removes the value AND the style of the cell: it becomes an unused default cell. */
    void clear(String cell);

    /** Display string: "" for an empty cell, the text as is, or the number rendered by its number format. */
    String getValue(String cell);

    ValueType valueType(String cell);

    // ---- styles ----

    /** Applies the non-null fields of {@code change} to the cell's style (all or nothing). */
    void setStyle(String cell, StyleChange change);

    /** {@code bold|italic|fontSize|textColor|fillColor|numberFormat}, e.g. {@code true|false|11|black|none|GENERAL}. */
    String styleOf(String cell);

    /** Applies {@code change} to every cell of the range (at most 1,000,000 cells). */
    void setStyleRange(String topLeft, String bottomRight, StyleChange change);

    /** Number of different non-default styles currently held by at least one cell. */
    int distinctStyleCount();

    // ---- range operations (area limit 1,000,000 cells unless stated) ----

    /** Sets the value of every cell of the range; styles are kept. */
    void fillNumber(String topLeft, String bottomRight, double value);

    /** Like {@link #fillNumber} with text. Empty text empties the values. */
    void fillText(String topLeft, String bottomRight, String text);

    /** Clears value and style of every cell of the range (no area limit). */
    void clearRange(String topLeft, String bottomRight);

    /**
     * Copies values and styles of the source range to a same-sized area whose top-left is {@code destTopLeft}.
     * Empty source cells clear the destination cell (value and style). The result is as if the source were first copied
     * to a temporary buffer. A destination that leaves the grid is an error.
     */
    void copy(String srcTopLeft, String srcBottomRight, String destTopLeft);

    /**
     * Cut and paste: like {@link #copy}, then every source cell that is not inside the destination area is cleared
     * (value and style).
     */
    void move(String srcTopLeft, String srcBottomRight, String destTopLeft);

    // ---- structure ----

    /** Shifts rows {@code row..} down by one, leaving an empty row; cells pushed past row 1,000,000 vanish. */
    void insertRow(int row);

    /** Removes the row; the rows below move up by one. */
    void deleteRow(int row);

    /** Shifts columns {@code column..} right by one, leaving an empty column; cells pushed past ZZ vanish. */
    void insertColumn(String column);

    /** Removes the column; the columns to its right move left by one. */
    void deleteColumn(String column);

    // ---- queries ----

    /** Bounding box of all cells that hold a value or a non-default style, e.g. {@code "B2:D7"}; "" if there are none. */
    String usedRange();

    /** Largest row number of a used cell (0 if none). */
    int rowCount();

    /** Largest column number of a used cell (A=1 ... ZZ=702; 0 if none). */
    int columnCount();

    // ---- sorting ----

    /**
     * Stable sort of the rows of the range by the values in column {@code byColumn} (which must lie inside the range).
     * Whole rows of the range (values and styles) move. Order: numbers, then text (compared case-insensitively), then
     * empty values. Descending reverses the order inside the numbers group and inside the text group only; equal keys
     * keep their original order in both directions.
     */
    void sortRange(String topLeft, String bottomRight, String byColumn, boolean ascending);
}

package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A cell address is a column letter A..Z followed by a row number 1..100 (no leading zeros), e.g. "B12". Addresses
 * are case-insensitive and are reported in upper case. Every method that takes an address throws
 * IllegalArgumentException for an invalid one (null, "A0", "A101", "AA1", "A01", " A1", ...) and then changes nothing.
 * All lists are sorted row by row (A1, B1, ..., Z1, A2, ...), without duplicates.
 */
public interface FormulaSheet {

    /**
     * Stores {@code input} in the cell, replacing whatever was there. "" clears the cell, a number literal makes a
     * number, input starting with '=' makes a formula, anything else is text. A null input is an
     * IllegalArgumentException. After this call every cell that directly or indirectly depends on the cell is up to
     * date.
     */
    void set(String cell, String input);

    /** The display string of the cell ("" for an empty cell); see the README for the exact formats. */
    String get(String cell);

    /** The formula text with letters upper-cased and nothing else changed; "" if the cell is not a formula. */
    String formulaOf(String cell);

    /** The valid cells the formula in this cell refers to directly (ranges expanded); empty if it has none. */
    List<String> dependencies(String cell);

    /** The cells whose formulas refer to this cell directly (also through a range). */
    List<String> dependents(String cell);

    /** All non-empty cells. */
    List<String> usedCells();

    /** Number of formula evaluations performed so far, including the one made when a formula is set. */
    long recomputeCount();
}

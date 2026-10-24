package contract;

/**
 * A visiting order, as a pure function from "which visit is this" to "which cell". PROVIDED: the tests implement it and
 * register their own instances.
 */
public interface TraversalStrategy {

    /**
     * The cell visited by the {@code index}-th visit (0-based) of a {@code rows x cols} matrix, as {@code {row, col}}.
     * Called with {@code 0 <= index < rows * cols}. A correct strategy maps those indexes onto every cell exactly once.
     * The same arguments always give the same answer.
     */
    int[] position(int rows, int cols, long index);
}

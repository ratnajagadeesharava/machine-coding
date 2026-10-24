package contract;

/**
 * A read-only rows x cols grid of ints. PROVIDED: the tests implement it (some of them over matrices far too big to
 * hold in memory, and they count the calls to {@link #get}). Implementations are immutable and thread-safe.
 */
public interface MatrixSource {

    /** Number of rows. */
    int rows();

    /** Number of columns. */
    int cols();

    /** The value at (r, c), {@code 0 <= r < rows()}, {@code 0 <= c < cols()}. */
    int get(int r, int c);

    /**
     * Convenience: an immutable, defensive COPY of {@code grid}.
     * IllegalArgumentException if the grid is null, has no rows, has an empty or null row, or its rows differ in length.
     */
    static MatrixSource of(int[][] grid) {
        if (grid == null || grid.length == 0) {
            throw new IllegalArgumentException("matrix must have at least one row");
        }
        int cols = grid[0] == null ? 0 : grid[0].length;
        if (cols == 0) {
            throw new IllegalArgumentException("matrix must have at least one column");
        }
        int[][] copy = new int[grid.length][];
        for (int r = 0; r < grid.length; r++) {
            if (grid[r] == null || grid[r].length != cols) {
                throw new IllegalArgumentException("row " + r + " has a different length");
            }
            copy[r] = grid[r].clone();
        }
        return new MatrixSource() {
            @Override
            public int rows() {
                return copy.length;
            }

            @Override
            public int cols() {
                return cols;
            }

            @Override
            public int get(int r, int c) {
                return copy[r][c];
            }
        };
    }
}

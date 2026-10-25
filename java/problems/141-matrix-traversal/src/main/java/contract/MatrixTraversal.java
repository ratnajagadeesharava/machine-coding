package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Built-in strategy names (case-sensitive): ROW_MAJOR, COLUMN_MAJOR, SNAKE, SPIRAL, SPIRAL_COUNTER, DIAGONAL, ZIGZAG,
 * BOUSTROPHEDON_COLUMNS, REVERSE_ROW_MAJOR (exact orders in the README).
 *
 * <p>Errors: IllegalArgumentException for a null/unknown strategy name, a bad argument, an unknown or closed cursor and
 * a registration that is rejected; IllegalStateException when a registered strategy misbehaves.
 */
public interface MatrixTraversal {

    /** All values in the order of the strategy. IllegalArgumentException for an unknown strategy name. */
    List<Integer> traverse(String strategy);

    /**
     * Makes {@code strategy} usable by name in every operation of this service. IllegalArgumentException if the name is
     * null or blank, equals a built-in name or an already registered name, or if the strategy is null.
     */
    void registerStrategy(String name, TraversalStrategy strategy);

    /** Opens a cursor positioned before the first visit and returns its id; ids are 1, 2, 3, ... and never reused. */
    int start(String strategy);

    /** The next value of the cursor, or empty once all rows*cols values were returned. Unknown/closed cursor: IAE. */
    Optional<Integer> next(int cursorId);

    /** True while at least one value is left. */
    boolean hasNext(int cursorId);

    /** How many values the cursor has not returned yet. */
    long remaining(int cursorId);

    /** Puts the cursor back before the first visit. */
    void reset(int cursorId);

    /** Releases the cursor; every later call with this id (a second close included) is an IllegalArgumentException. */
    void close(int cursorId);

    /** Sum of the first {@code k} visited values; {@code k} above rows*cols counts as rows*cols. k &lt; 0: IAE. */
    long pathSum(String strategy, long k);

    /** The value of the {@code k}-th visit, 0-based. k &lt; 0 or k &gt;= rows*cols: IAE. */
    int kthValue(String strategy, long k);
}

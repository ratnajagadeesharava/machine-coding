package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Single-threaded: one caller at a time.
 *
 * <p>Coordinates are (row, col) with (0,0) top-left; rows run 0..height-1, columns 0..width-1.
 * Cells are always rendered as the string {@code "row,col"} (no spaces).
 */
public interface SnakeGame {

    /**
     * Moves the head one cell. {@code direction} is exactly one of "U" (row-1), "D" (row+1), "L" (col-1), "R" (col+1);
     * anything else (null, lower case, longer strings) is an IllegalArgumentException, even after the game is over,
     * and changes nothing.
     *
     * <p>Returns the score after the move, or -1 if the game is over after this move (a fatal move, a winning move,
     * or any move made when the game was already over; moves after game over change nothing).
     */
    int move(String direction);

    /** Foods eaten so far (always {@code length() - 1}). */
    int score();

    /** True once the snake hit a wall or itself, reversed on itself, or filled the board. */
    boolean isOver();

    /** True only if the game ended because the snake fills every cell of the board. */
    boolean isWon();

    /** Number of cells the snake occupies (starts at 1). */
    int length();

    /** The head cell as "row,col". */
    String head();

    /** Every snake cell as "row,col", head first, tail last. */
    List<String> body();

    /** The cell of the food currently on the board as "row,col", or "none". */
    String foodPosition();
}

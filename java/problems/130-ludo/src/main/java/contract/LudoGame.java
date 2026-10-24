package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point for one game of Ludo. Your design lives behind it. Single-threaded use.
 *
 * <p>Colours are the strings {@code "RED"}, {@code "GREEN"}, {@code "YELLOW"}, {@code "BLUE"}; a game with N players
 * uses the first N of them, in that turn order. Tokens are numbered 0..3. A token's <em>progress</em> is 0 in the yard,
 * 1..51 on the shared track, 52..56 in its home column and 57 once it is HOME. The full rules are in the README.
 */
public interface LudoGame {

    /**
     * Rolls the next injected die for the current player and returns it (1..6).
     * IllegalStateException if the game is over, if the previous roll still waits for a {@link #move}, or if the
     * injected dice are used up (message "dice exhausted"; nothing changes).
     * If the roll gives no legal move, or it is the third consecutive 6, the turn is resolved automatically
     * before this method returns.
     */
    int roll();

    /** Token indexes the current player may move with the pending roll, ascending. Empty if no roll is pending. */
    List<Integer> legalMoves();

    /**
     * Moves the chosen token by the pending roll and returns {@code "moved"}, {@code "home"} or
     * {@code "captured:<COLOUR>:<n>"} (several captured colours are joined with {@code ","} in turn order).
     * IllegalStateException if the game is over or no roll is pending; IllegalArgumentException if the token is not
     * one of {@link #legalMoves()} (including an index outside 0..3).
     */
    String move(int tokenIndex);

    /** The colour whose turn it is (after the game is over: the winner). */
    String currentPlayer();

    /** Progress 0..57 of the token. IllegalArgumentException for a colour not in this game or a token outside 0..3. */
    int tokenProgress(String colour, int tokenIndex);

    /** Shared track cell 0..51 the token stands on, or -1 if it is in the yard, in its home column or HOME. */
    int tokenCell(String colour, int tokenIndex);

    /** The winning colour, empty while the game is running. */
    Optional<String> winner();

    boolean isOver();
}

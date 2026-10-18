package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface TicTacToeApi {

    /** OK, WIN, DRAW, GAME_OVER, NOT_YOUR_TURN, INVALID_PLAYER, OUT_OF_BOUNDS or OCCUPIED. */
    String play(String player, int row, int col);

    /** "X" or "O"; empty once the game is over. */
    Optional<String> nextPlayer();

    Optional<String> winner();

    /** "X", "O", or empty for an empty/out-of-bounds cell. */
    Optional<String> cellAt(int row, int col);
}

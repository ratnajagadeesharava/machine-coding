package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. One instance is one game in progress. */
public interface BoardGameApi {

    /** Returns OK, WIN, DRAW, INVALID_MOVE, NOT_YOUR_TURN or GAME_OVER. */
    String play(String player, String move);

    /** "X" (P1), "O" (P2) or "." (empty or out of the board). Connect4: row 0 is the bottom row. */
    String cellAt(int row, int col);

    /** One line per row from the visual top to the bottom, joined with "\n". */
    String render();

    /** "P1" or "P2"; empty once the game is over. */
    Optional<String> nextPlayer();
}

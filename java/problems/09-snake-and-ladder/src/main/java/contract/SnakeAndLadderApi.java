package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface SnakeAndLadderApi {

    /** Applies the dice value for the current player. Returns the mover's new position, or -1 if refused. */
    int roll(int dice);

    /** Empty after the game is over. */
    Optional<String> currentPlayer();

    /** 0 before entering the board; -1 for unknown players. */
    int positionOf(String player);

    Optional<String> winner();
}

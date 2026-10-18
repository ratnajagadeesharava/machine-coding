package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface ChessApi {

    enum MoveResult { OK, ILLEGAL, GAME_OVER }

    enum Status { IN_PROGRESS, CHECK, CHECKMATE, STALEMATE }

    /** Applies a move in coordinate notation ("e2e4", "e7e8q"). */
    MoveResult move(String move);

    /** "WHITE" or "BLACK". */
    String turn();

    Status status();

    /** "wP", "bK", ... or empty for an empty/invalid square. */
    Optional<String> pieceAt(String square);

    /** All legal moves for the side to move, sorted alphabetically. */
    List<String> legalMoves();
}

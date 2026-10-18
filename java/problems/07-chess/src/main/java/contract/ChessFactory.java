package contract;

/** Creates games. FEN is used by tests to set up positions. */
public interface ChessFactory {
    ChessApi newGame();

    ChessApi fromFen(String fen);
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ChessApi;
import contract.ChessApi.MoveResult;
import contract.ChessApi.Status;
import contract.ChessFactory;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private ChessFactory chess() {
        return Candidate.of(Solution::create);
    }

    private ChessApi play(ChessApi game, String... moves) {
        for (String m : moves) {
            assertThat(game.move(m)).as("move %s", m).isEqualTo(MoveResult.OK);
        }
        return game;
    }

    private ChessApi game(String fen) {
        return chess().fromFen(fen);
    }

    /** Number of leaf positions after the given depth, found by replaying move sequences on fresh games. */
    private long perft(ChessFactory f, String fen, int depth) {
        return perft(f, fen, List.of(), depth);
    }

    private long perft(ChessFactory f, String fen, List<String> prefix, int depth) {
        ChessApi g = fen == null ? f.newGame() : f.fromFen(fen);
        for (String m : prefix) {
            if (g.move(m) != MoveResult.OK) {
                throw new AssertionError("replay failed on " + prefix);
            }
        }
        List<String> moves = g.legalMoves();
        if (depth == 1) {
            return moves.size();
        }
        long total = 0;
        for (String m : moves) {
            List<String> next = new java.util.ArrayList<>(prefix);
            next.add(m);
            total += perft(f, fen, next, depth - 1);
        }
        return total;
    }

    @Test
    void shouldStartWithWhiteToMoveAndStandardSetup() {
        ChessApi g = chess().newGame();

        assertThat(g.turn()).isEqualTo("WHITE");
        assertThat(g.status()).isEqualTo(Status.IN_PROGRESS);
        assertThat(g.pieceAt("e1")).hasValue("wK");
        assertThat(g.pieceAt("d8")).hasValue("bQ");
        assertThat(g.pieceAt("a2")).hasValue("wP");
        assertThat(g.pieceAt("h7")).hasValue("bP");
        assertThat(g.pieceAt("e4")).isEmpty();
        assertThat(g.pieceAt("z9")).isEmpty();
    }

    @Test
    void shouldHaveTwentyLegalMovesInTheStartingPosition() {
        ChessApi g = chess().newGame();

        assertThat(g.legalMoves()).hasSize(20).contains("e2e4", "g1f3", "a2a3").doesNotContain("e2e5");
    }

    @Test
    void shouldApplyLegalMoveAndSwitchTurn() {
        ChessApi g = chess().newGame();

        assertThat(g.move("e2e4")).isEqualTo(MoveResult.OK);

        assertThat(g.turn()).isEqualTo("BLACK");
        assertThat(g.pieceAt("e4")).hasValue("wP");
        assertThat(g.pieceAt("e2")).isEmpty();
    }

    @Test
    void shouldRejectIllegalMovesWithoutChangingState() {
        ChessApi g = chess().newGame();

        assertThat(g.move("e2e5")).isEqualTo(MoveResult.ILLEGAL);   // pawn cannot advance three
        assertThat(g.move("e7e5")).isEqualTo(MoveResult.ILLEGAL);   // not black's turn
        assertThat(g.move("e3e4")).isEqualTo(MoveResult.ILLEGAL);   // empty square
        assertThat(g.move("a1a3")).isEqualTo(MoveResult.ILLEGAL);   // rook blocked by pawn
        assertThat(g.move("nonsense")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.move("")).isEqualTo(MoveResult.ILLEGAL);

        assertThat(g.turn()).isEqualTo("WHITE");
        assertThat(g.legalMoves()).hasSize(20);
        assertThat(g.pieceAt("a1")).hasValue("wR");
    }

    @Test
    void shouldNotAllowPieceToMoveThroughOthers() {
        ChessApi g = chess().newGame();

        assertThat(g.move("f1c4")).isEqualTo(MoveResult.ILLEGAL); // bishop blocked by e2
        assertThat(g.move("d1d3")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.move("e2e4")).isEqualTo(MoveResult.OK);
        assertThat(g.move("e7e5")).isEqualTo(MoveResult.OK);
        assertThat(g.move("f1c4")).isEqualTo(MoveResult.OK);      // now open
    }

    @Test
    void shouldAllowKnightsToJump() {
        ChessApi g = chess().newGame();

        assertThat(g.move("b1c3")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("c3")).hasValue("wN");
    }

    @Test
    void shouldNotAllowPawnToCaptureForwardOrMoveDiagonallyWithoutCapture() {
        ChessApi g = chess().newGame();
        play(g, "e2e4", "e7e5");

        assertThat(g.move("e4e5")).isEqualTo(MoveResult.ILLEGAL);   // blocked head-on
        assertThat(g.move("e4d5")).isEqualTo(MoveResult.ILLEGAL);   // diagonal without capture
    }

    @Test
    void shouldUpdateBoardOnCapture() {
        ChessApi g = chess().newGame();
        play(g, "e2e4", "d7d5");

        assertThat(g.move("e4d5")).isEqualTo(MoveResult.OK);

        assertThat(g.pieceAt("d5")).hasValue("wP");
        assertThat(g.pieceAt("e4")).isEmpty();
    }

    @Test
    void shouldNotLetKingMoveIntoCheck() {
        // White king e1, black rook on d8 controls the d-file; black king far away.
        ChessApi g = game("3rk3/8/8/8/8/8/8/4K3 w - - 0 1");

        assertThat(g.move("e1d1")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.move("e1d2")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.legalMoves()).containsExactly("e1e2", "e1f1", "e1f2");
    }

    @Test
    void shouldNotLetPinnedPieceExposeItsKing() {
        ChessApi g = game("4r2k/8/8/8/8/8/4B3/4K3 w - - 0 1");

        assertThat(g.move("e2d3")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.legalMoves()).noneMatch(m -> m.startsWith("e2"));
    }

    @Test
    void shouldReportCheckAndOnlyAllowMovesThatResolveIt() {
        ChessApi g = game("4r2k/8/8/8/8/8/8/R3K3 w - - 0 1");

        assertThat(g.status()).isEqualTo(Status.CHECK);
        assertThat(g.legalMoves()).containsExactly("e1d1", "e1d2", "e1f1", "e1f2");
    }

    @Test
    void shouldDetectCheckmate() {
        ChessApi g = chess().newGame();

        play(g, "f2f3", "e7e5", "g2g4", "d8h4");

        assertThat(g.status()).isEqualTo(Status.CHECKMATE);
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.move("a2a3")).isEqualTo(MoveResult.GAME_OVER);
    }

    @Test
    void shouldDetectStalemate() {
        ChessApi g = game("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");

        assertThat(g.status()).isEqualTo(Status.STALEMATE);
        assertThat(g.legalMoves()).isEmpty();
        assertThat(g.move("h8g8")).isEqualTo(MoveResult.GAME_OVER);
    }

    @Test
    void shouldCastleKingsideAndQueenside() {
        ChessApi g = game("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");

        assertThat(g.move("e1g1")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("g1")).hasValue("wK");
        assertThat(g.pieceAt("f1")).hasValue("wR");
        assertThat(g.pieceAt("h1")).isEmpty();

        assertThat(g.move("e8c8")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("c8")).hasValue("bK");
        assertThat(g.pieceAt("d8")).hasValue("bR");
        assertThat(g.pieceAt("a8")).isEmpty();
    }

    @Test
    void shouldNotCastleOutOfThroughOrIntoCheck() {
        // in check
        assertThat(game("4r2k/8/8/8/8/8/8/R3K2R w KQ - 0 1").move("e1g1")).isEqualTo(MoveResult.ILLEGAL);
        // through an attacked square (f1)
        assertThat(game("5r1k/8/8/8/8/8/8/R3K2R w KQ - 0 1").move("e1g1")).isEqualTo(MoveResult.ILLEGAL);
        // into an attacked square (g1)
        assertThat(game("6rk/8/8/8/8/8/8/R3K2R w KQ - 0 1").move("e1g1")).isEqualTo(MoveResult.ILLEGAL);
        // the queenside is unaffected by an attack on g1/f1
        assertThat(game("6rk/8/8/8/8/8/8/R3K2R w KQ - 0 1").move("e1c1")).isEqualTo(MoveResult.OK);
    }

    @Test
    void shouldNotCastleWhenPathIsBlockedOrRightsAreLost() {
        assertThat(game("4k3/8/8/8/8/8/8/R2QK2R w KQ - 0 1").move("e1c1")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(game("4k3/8/8/8/8/8/8/R3K2R w - - 0 1").move("e1g1")).isEqualTo(MoveResult.ILLEGAL);

        ChessApi g = game("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        play(g, "e1e2", "e8e7", "e2e1", "e7e8");
        assertThat(g.move("e1g1")).isEqualTo(MoveResult.ILLEGAL);
    }

    @Test
    void shouldCaptureEnPassantOnlyImmediatelyAfterTwoSquareAdvance() {
        ChessApi g = chess().newGame();
        play(g, "e2e4", "a7a6", "e4e5", "d7d5");

        assertThat(g.move("e5d6")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("d6")).hasValue("wP");
        assertThat(g.pieceAt("d5")).isEmpty();

        ChessApi late = chess().newGame();
        play(late, "e2e4", "a7a6", "e4e5", "d7d5", "h2h3", "a6a5");
        assertThat(late.move("e5d6")).isEqualTo(MoveResult.ILLEGAL);
    }

    @Test
    void shouldPromotePawnAndRequireAPieceChoice() {
        ChessApi g = game("8/P6k/8/8/8/8/8/K7 w - - 0 1");

        assertThat(g.move("a7a8")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.legalMoves()).contains("a7a8q", "a7a8r", "a7a8b", "a7a8n");
        assertThat(g.move("a7a8q")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("a8")).hasValue("wQ");
        assertThat(g.pieceAt("a7")).isEmpty();
    }

    @Test
    void shouldAllowUnderpromotionAndRejectInvalidPromotionPieces() {
        ChessApi g = game("8/P6k/8/8/8/8/8/K7 w - - 0 1");

        assertThat(g.move("a7a8k")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.move("a7a8p")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.move("a7a8n")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("a8")).hasValue("wN");
    }

    @Test
    void shouldCountLegalMovesCorrectlyInThePerftPositions() {
        ChessFactory f = chess();
        assertThat(f.newGame().legalMoves()).hasSize(20);
        assertThat(f.fromFen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").legalMoves()).hasSize(48);
        assertThat(f.fromFen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").legalMoves()).hasSize(14);
        assertThat(f.fromFen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").legalMoves()).hasSize(6);
        assertThat(f.fromFen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").legalMoves()).hasSize(44);
    }

    @Test
    void shouldMatchKnownPerftToDepthThreeFromTheStartingPosition() {
        ChessFactory f = chess();

        assertThat(perft(f, null, 2)).isEqualTo(400);
        assertThat(perft(f, null, 3)).isEqualTo(8902);
    }

    @Test
    void shouldMatchKnownPerftToDepthTwoInTrickyPositions() {
        ChessFactory f = chess();

        assertThat(perft(f, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 2)).isEqualTo(2039);
        assertThat(perft(f, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 2)).isEqualTo(191);
        assertThat(perft(f, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 2)).isEqualTo(264);
        assertThat(perft(f, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 2)).isEqualTo(1486);
    }
}

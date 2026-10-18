package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ChessApi;
import contract.ChessApi.MoveResult;
import contract.ChessApi.Status;
import contract.ChessFactory;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private ChessFactory chess() {
        return Candidate.of(Solution::create);
    }

    private long perft(ChessFactory f, String fen, List<String> prefix, int depth) {
        ChessApi g = f.fromFen(fen);
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
            List<String> next = new ArrayList<>(prefix);
            next.add(m);
            total += perft(f, fen, next, depth - 1);
        }
        return total;
    }

    @Test
    void shouldMatchKnownPerftToDepthThreeInTrickyPositions() {
        ChessFactory f = chess();
        assertThat(perft(f, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", List.of(), 3)).isEqualTo(2812);
        assertThat(perft(f, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", List.of(), 3)).isEqualTo(9467);
        assertThat(perft(f, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", List.of(), 3)).isEqualTo(97862);
    }

    @Test
    void shouldNotAllowEnPassantThatExposesTheKingAlongTheRank() {
        // White king a5, black rook h5: capturing e5xd6 e.p. would remove both pawns from the 5th rank.
        ChessApi g = chess().fromFen("8/8/8/K2pP2r/8/8/8/7k w - d6 0 1");
        assertThat(g.move("e5d6")).isEqualTo(MoveResult.ILLEGAL);
    }

    @Test
    void shouldLoseCastlingRightWhenRookIsCapturedEvenIfAnotherRookReturnsToTheSquare() {
        ChessApi g = chess().fromFen("4k3/8/8/8/8/7R/6b1/R3K2R b KQ - 0 1");
        assertThat(g.move("g2h1")).isEqualTo(MoveResult.OK); // bishop takes the h1 rook
        assertThat(g.move("h3h1")).isEqualTo(MoveResult.OK); // another rook retakes on h1
        assertThat(g.move("e8e7")).isEqualTo(MoveResult.OK);
        assertThat(g.move("e1g1")).isEqualTo(MoveResult.ILLEGAL);
        assertThat(g.legalMoves()).contains("e1c1");
    }

    @Test
    void shouldAnswerDoubleCheckWithKingMoveOnly() {
        // Black king e8 is in double check from rook e1 and bishop b5.
        ChessApi g = chess().fromFen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1");
        assertThat(g.status()).isEqualTo(Status.CHECK);
        assertThat(g.legalMoves()).allMatch(m -> m.startsWith("e8"));
    }

    @Test
    void shouldAllowPromotionWithCapture() {
        ChessApi g = chess().fromFen("1n5k/P7/8/8/8/8/8/K7 w - - 0 1");
        assertThat(g.legalMoves()).contains("a7a8q", "a7b8q", "a7b8n");
        assertThat(g.move("a7b8q")).isEqualTo(MoveResult.OK);
        assertThat(g.pieceAt("b8")).hasValue("wQ");
    }

    @Test
    void shouldDetectBackRankMate() {
        ChessApi g = chess().fromFen("6k1/5ppp/8/8/8/8/8/R3K3 w - - 0 1");
        assertThat(g.move("a1a8")).isEqualTo(MoveResult.OK);
        assertThat(g.status()).isEqualTo(Status.CHECKMATE);
    }
}

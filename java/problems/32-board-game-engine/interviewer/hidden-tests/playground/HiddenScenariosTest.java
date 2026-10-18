package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.BoardGameApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private BoardGameApi game(String name) {
        return Candidate.of(() -> Solution.create(name)).orElseThrow();
    }

    private static String playAll(BoardGameApi g, String... moves) {
        String last = "";
        for (int i = 0; i < moves.length; i++) {
            last = g.play(i % 2 == 0 ? "P1" : "P2", moves[i]);
            if (i < moves.length - 1) {
                assertThat(last).as("move %d (%s)", i, moves[i]).isEqualTo("OK");
            }
        }
        return last;
    }

    @Test
    void shouldWinTicTacToeOnTheAntiDiagonal() {
        BoardGameApi g = game("tictactoe");

        assertThat(playAll(g, "0,2", "0,0", "1,1", "0,1", "2,0")).isEqualTo("WIN");
    }

    @Test
    void shouldWinConnect4OnADescendingDiagonal() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "6", "5", "5", "4", "3", "4", "4", "3", "3", "0", "3")).isEqualTo("WIN");
    }

    @Test
    void shouldWinConnect4WhenThePlacedPieceFillsTheMiddleOfTheLine() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "0", "0", "1", "1", "3", "3", "2")).isEqualTo("WIN");
    }

    @Test
    void shouldCountSixInARowAsAWinInGomoku() {
        BoardGameApi g = game("gomoku");

        assertThat(playAll(g, "7,3", "0,0", "7,4", "2,0", "7,5", "4,0", "7,7", "6,0", "7,8", "8,0", "7,6"))
                .isEqualTo("WIN");
    }

    @Test
    void shouldWinGomokuOnAColumnAndOnTheAntiDiagonal() {
        BoardGameApi vertical = game("gomoku");
        assertThat(playAll(vertical, "3,7", "0,0", "4,7", "0,2", "5,7", "0,4", "6,7", "0,6", "7,7"))
                .isEqualTo("WIN");

        BoardGameApi anti = game("gomoku");
        assertThat(playAll(anti, "3,7", "0,0", "4,6", "0,2", "5,5", "0,4", "6,4", "0,6", "7,3")).isEqualTo("WIN");
    }

    @Test
    void shouldTreatUnknownPlayersAsNotTheirTurnAndHugeNumbersAsInvalid() {
        BoardGameApi g = game("tictactoe");

        assertThat(g.play("P3", "0,0")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.play("p1", "0,0")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.play("", "0,0")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.play("P2", "x")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.play("P1", "99999999999,0")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", "0,99999999999999999999")).isEqualTo("INVALID_MOVE");
        assertThat(g.render()).isEqualTo("...\n...\n...");
    }

    @Test
    void shouldAnswerGameOverBeforeTurnAndMoveChecksOnConnect4() {
        BoardGameApi g = game("connect4");
        playAll(g, "0", "1", "0", "1", "0", "1", "0");

        assertThat(g.play("P1", "9")).isEqualTo("GAME_OVER");
        assertThat(g.play("nobody", "x")).isEqualTo("GAME_OVER");
        assertThat(g.cellAt(4, 0)).isEqualTo(".");
    }
}

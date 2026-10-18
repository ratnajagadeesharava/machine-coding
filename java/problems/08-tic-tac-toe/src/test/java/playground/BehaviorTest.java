package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.TicTacToeApi;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private TicTacToeApi game(int size, int k) {
        return Candidate.of(() -> Solution.create(size, k));
    }

    private TicTacToeApi classic() {
        return game(3, 3);
    }

    /** Plays alternating X, O moves from (row,col) pairs; returns the result of the last move. */
    private String play(TicTacToeApi g, int... rc) {
        String last = null;
        String[] players = {"X", "O"};
        for (int i = 0; i < rc.length; i += 2) {
            last = g.play(players[(i / 2) % 2], rc[i], rc[i + 1]);
        }
        return last;
    }

    @Test
    void shouldLetXStartAndAlternate() {
        TicTacToeApi g = classic();

        assertThat(g.nextPlayer()).hasValue("X");
        assertThat(g.play("X", 0, 0)).isEqualTo("OK");
        assertThat(g.nextPlayer()).hasValue("O");
        assertThat(g.cellAt(0, 0)).hasValue("X");
        assertThat(g.cellAt(1, 1)).isEmpty();
    }

    @Test
    void shouldRejectMoveOutOfTurnWithoutConsumingTheTurn() {
        TicTacToeApi g = classic();

        assertThat(g.play("O", 0, 0)).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.cellAt(0, 0)).isEmpty();
        assertThat(g.play("X", 0, 0)).isEqualTo("OK");
        assertThat(g.play("X", 1, 1)).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.nextPlayer()).hasValue("O");
    }

    @Test
    void shouldRejectUnknownPlayer() {
        TicTacToeApi g = classic();

        assertThat(g.play("Z", 0, 0)).isEqualTo("INVALID_PLAYER");
        assertThat(g.nextPlayer()).hasValue("X");
    }

    @Test
    void shouldRejectOccupiedCellWithoutConsumingTheTurn() {
        TicTacToeApi g = classic();
        g.play("X", 1, 1);

        assertThat(g.play("O", 1, 1)).isEqualTo("OCCUPIED");

        assertThat(g.nextPlayer()).hasValue("O");
        assertThat(g.cellAt(1, 1)).hasValue("X");
        assertThat(g.play("O", 0, 0)).isEqualTo("OK");
    }

    @Test
    void shouldRejectOutOfBoundsCells() {
        TicTacToeApi g = classic();

        assertThat(g.play("X", -1, 0)).isEqualTo("OUT_OF_BOUNDS");
        assertThat(g.play("X", 0, 3)).isEqualTo("OUT_OF_BOUNDS");
        assertThat(g.play("X", 3, 3)).isEqualTo("OUT_OF_BOUNDS");
        assertThat(g.nextPlayer()).hasValue("X");
    }

    @Test
    void shouldDetectRowWin() {
        TicTacToeApi g = classic();

        assertThat(play(g, 0, 0, 1, 0, 0, 1, 1, 1, 0, 2)).isEqualTo("WIN");

        assertThat(g.winner()).hasValue("X");
        assertThat(g.nextPlayer()).isEmpty();
    }

    @Test
    void shouldDetectColumnWin() {
        TicTacToeApi g = classic();

        assertThat(play(g, 0, 0, 0, 1, 1, 0, 1, 1, 2, 0)).isEqualTo("WIN");
        assertThat(g.winner()).hasValue("X");
    }

    @Test
    void shouldDetectDiagonalWin() {
        TicTacToeApi g = classic();

        assertThat(play(g, 0, 0, 0, 1, 1, 1, 0, 2, 2, 2)).isEqualTo("WIN");
        assertThat(g.winner()).hasValue("X");
    }

    @Test
    void shouldDetectAntiDiagonalWinForSecondPlayer() {
        TicTacToeApi g = classic();

        assertThat(play(g, 0, 0, 0, 2, 0, 1, 1, 1, 2, 2, 2, 0)).isEqualTo("WIN");
        assertThat(g.winner()).hasValue("O");
    }

    @Test
    void shouldDetectDraw() {
        TicTacToeApi g = classic();

        String result = play(g, 0, 0, 0, 1, 0, 2, 1, 1, 1, 0, 1, 2, 2, 1, 2, 0, 2, 2);

        assertThat(result).isEqualTo("DRAW");
        assertThat(g.winner()).isEmpty();
        assertThat(g.nextPlayer()).isEmpty();
    }

    @Test
    void shouldRejectMovesAfterTheGameEnds() {
        TicTacToeApi g = classic();
        play(g, 0, 0, 1, 0, 0, 1, 1, 1, 0, 2);

        assertThat(g.play("O", 2, 2)).isEqualTo("GAME_OVER");
        assertThat(g.play("X", 2, 2)).isEqualTo("GAME_OVER");
        assertThat(g.cellAt(2, 2)).isEmpty();
    }

    @Test
    void shouldSupportLargerBoardsWithShorterWinningLine() {
        TicTacToeApi g = game(5, 4);

        // X builds (0,0)..(3,3) diagonal; O plays elsewhere in row 4.
        assertThat(play(g, 0, 0, 4, 0, 1, 1, 4, 1, 2, 2, 4, 2, 3, 3)).isEqualTo("WIN");
        assertThat(g.winner()).hasValue("X");
    }

    @Test
    void shouldNotTreatAShorterLineAsAWinOnALargerBoard() {
        TicTacToeApi g = game(5, 4);

        assertThat(play(g, 0, 0, 4, 0, 0, 1, 4, 1, 0, 2)).isEqualTo("OK");
        assertThat(g.winner()).isEmpty();
        assertThat(g.nextPlayer()).hasValue("O");
    }

    @Test
    void shouldHandleLargeBoardsQuickly() {
        TicTacToeApi g = game(1000, 5);
        List<int[]> xs = new ArrayList<>();
        List<int[]> os = new ArrayList<>();
        for (int r = 0; r < 20; r++) {
            for (int c = 0; c < 500; c++) {
                (((c / 2) + r) % 2 == 0 ? xs : os).add(new int[] {r, c});
            }
        }
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < xs.size(); i++) {
                assertThat(g.play("X", xs.get(i)[0], xs.get(i)[1])).isEqualTo("OK");
                assertThat(g.play("O", os.get(i)[0], os.get(i)[1])).isEqualTo("OK");
            }
        });
    }
}

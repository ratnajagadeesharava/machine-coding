package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.TicTacToeApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private TicTacToeApi game(int size, int k) {
        return Candidate.of(() -> Solution.create(size, k));
    }

    private String play(TicTacToeApi g, int... rc) {
        String last = null;
        String[] players = {"X", "O"};
        for (int i = 0; i < rc.length; i += 2) {
            last = g.play(players[(i / 2) % 2], rc[i], rc[i + 1]);
        }
        return last;
    }

    @Test
    void shouldWinByFillingTheMiddleOfALine() {
        TicTacToeApi g = game(3, 3);
        assertThat(play(g, 0, 0, 1, 0, 0, 2, 1, 1, 0, 1)).isEqualTo("WIN");
    }

    @Test
    void shouldWinWithALineLongerThanK() {
        TicTacToeApi g = game(6, 3);
        // X: (0,0) (0,1) (0,3) (0,4) then (0,2) joins into five; O elsewhere.
        play(g, 0, 0, 5, 0, 0, 1, 5, 2, 0, 3, 5, 4, 0, 4, 4, 0);
        assertThat(g.play("X", 0, 2)).isEqualTo("WIN");
    }

    @Test
    void lastCellThatWinsShouldBeWinNotDraw() {
        TicTacToeApi g = game(3, 3);
        // X O X
        // O X O
        // O X X  -> X wins on the diagonal with the final move at (2,2)
        String r = play(g, 0, 0, 0, 1, 0, 2, 1, 0, 1, 1, 1, 2, 2, 1, 2, 0, 2, 2);
        assertThat(r).isEqualTo("WIN");
        assertThat(g.winner()).hasValue("X");
    }

    @Test
    void shouldDetectAntiDiagonalWinInTheCornerOfALargeBoard() {
        TicTacToeApi g = game(4, 3);
        assertThat(play(g, 0, 3, 3, 0, 1, 2, 3, 1, 2, 1)).isEqualTo("WIN");
    }

    @Test
    void shouldNotWinOnANonConsecutiveLine() {
        TicTacToeApi g = game(5, 3);
        assertThat(play(g, 0, 0, 4, 4, 0, 1, 4, 3, 0, 3)).isEqualTo("OK");
        assertThat(g.winner()).isEmpty();
    }
}

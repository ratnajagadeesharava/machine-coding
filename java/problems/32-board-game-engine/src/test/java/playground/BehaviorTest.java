package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.BoardGameApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private BoardGameApi game(String name) {
        return Candidate.of(() -> Solution.create(name)).orElseThrow();
    }

    /** Plays the moves alternately starting with P1; every move but the last must be OK. Returns the last result. */
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

    private static final int[] C4_DRAW = {
        1, 3, 1, 1, 1, 1, 1, 3, 5, 2, 3, 0, 4, 3, 2, 5, 5, 2, 6, 0, 6, 4, 0, 0, 0, 4, 2, 6, 3, 0, 5, 4, 2, 2, 4, 4,
        6, 5, 5, 6, 6, 3
    };

    @Test
    void shouldHostKnownGamesAndRefuseUnknownOnes() {
        assertThat(Candidate.of(() -> Solution.create("tictactoe"))).isPresent();
        assertThat(Candidate.of(() -> Solution.create("connect4"))).isPresent();
        assertThat(Candidate.of(() -> Solution.create("gomoku"))).isPresent();
        assertThat(Candidate.of(() -> Solution.create("chess"))).isEmpty();
        assertThat(Candidate.of(() -> Solution.create("TicTacToe"))).isEmpty();
        assertThat(Candidate.of(() -> Solution.create(""))).isEmpty();
    }

    @Test
    void shouldLetP1StartAndAlternateTurns() {
        BoardGameApi g = game("tictactoe");
        assertThat(g.nextPlayer()).hasValue("P1");

        assertThat(g.play("P1", "1,1")).isEqualTo("OK");
        assertThat(g.nextPlayer()).hasValue("P2");
        assertThat(g.play("P2", "0,0")).isEqualTo("OK");
        assertThat(g.nextPlayer()).hasValue("P1");

        assertThat(g.cellAt(1, 1)).isEqualTo("X");
        assertThat(g.cellAt(0, 0)).isEqualTo("O");
        assertThat(g.cellAt(2, 2)).isEqualTo(".");
    }

    @Test
    void shouldRefuseOutOfTurnMovesWithoutChangingAnything() {
        BoardGameApi g = game("tictactoe");

        assertThat(g.play("P2", "0,0")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.cellAt(0, 0)).isEqualTo(".");
        assertThat(g.play("P1", "0,0")).isEqualTo("OK");
        assertThat(g.play("P1", "0,1")).isEqualTo("NOT_YOUR_TURN");
        assertThat(g.cellAt(0, 1)).isEqualTo(".");
        assertThat(g.nextPlayer()).hasValue("P2");
    }

    @Test
    void shouldRejectInvalidMovesWithoutConsumingTheTurn() {
        BoardGameApi g = game("tictactoe");
        g.play("P1", "1,1");

        assertThat(g.play("P2", "1,1")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "3,0")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "0,3")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "-1,0")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "x")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "1, 2")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "1,2,0")).isEqualTo("INVALID_MOVE");

        assertThat(g.nextPlayer()).hasValue("P2");
        assertThat(g.play("P2", "0,0")).isEqualTo("OK");
    }

    @Test
    void shouldReturnBlankForCoordinatesOutsideTheBoard() {
        BoardGameApi g = game("tictactoe");

        assertThat(g.cellAt(-1, 0)).isEqualTo(".");
        assertThat(g.cellAt(0, 3)).isEqualTo(".");
        assertThat(g.cellAt(99, 99)).isEqualTo(".");
    }

    @Test
    void shouldWinTicTacToeOnARow() {
        BoardGameApi g = game("tictactoe");

        assertThat(playAll(g, "0,0", "1,0", "0,1", "1,1", "0,2")).isEqualTo("WIN");
    }

    @Test
    void shouldWinTicTacToeOnAColumn() {
        BoardGameApi g = game("tictactoe");

        assertThat(playAll(g, "0,0", "0,1", "1,0", "1,1", "2,0")).isEqualTo("WIN");
    }

    @Test
    void shouldWinTicTacToeOnTheMainDiagonal() {
        BoardGameApi g = game("tictactoe");

        assertThat(playAll(g, "0,0", "0,1", "1,1", "0,2", "2,2")).isEqualTo("WIN");
    }

    @Test
    void shouldLetP2WinToo() {
        BoardGameApi g = game("tictactoe");

        assertThat(playAll(g, "0,0", "1,0", "0,1", "1,1", "2,2", "1,2")).isEqualTo("WIN");
    }

    @Test
    void shouldRefuseEverythingAfterTheGameIsWon() {
        BoardGameApi g = game("tictactoe");
        playAll(g, "0,0", "1,0", "0,1", "1,1", "0,2");

        assertThat(g.play("P2", "2,2")).isEqualTo("GAME_OVER");
        assertThat(g.play("P1", "2,2")).isEqualTo("GAME_OVER");
        assertThat(g.play("P1", "garbage")).isEqualTo("GAME_OVER");
        assertThat(g.cellAt(2, 2)).isEqualTo(".");
        assertThat(g.nextPlayer()).isEmpty();
    }

    @Test
    void shouldDeclareADrawWhenTheBoardFillsUpWithoutAWinner() {
        BoardGameApi g = game("tictactoe");

        String last = playAll(g, "0,0", "0,1", "0,2", "1,1", "1,0", "1,2", "2,1", "2,0", "2,2");

        assertThat(last).isEqualTo("DRAW");
        assertThat(g.play("P2", "0,0")).isEqualTo("GAME_OVER");
        assertThat(g.nextPlayer()).isEmpty();
    }

    @Test
    void shouldPreferWinOverDrawWhenTheLastCellCompletesALine() {
        BoardGameApi g = game("tictactoe");

        String last = playAll(g, "0,0", "0,1", "0,2", "1,0", "1,1", "1,2", "2,1", "2,0", "2,2");

        assertThat(last).isEqualTo("WIN");
    }

    @Test
    void shouldRenderTheTicTacToeBoardTopRowFirst() {
        BoardGameApi g = game("tictactoe");
        g.play("P1", "0,0");
        g.play("P2", "1,1");
        g.play("P1", "2,1");

        assertThat(g.render()).isEqualTo("X..\n.O.\n.X.");
    }

    @Test
    void shouldDropConnect4PiecesToTheLowestFreeRow() {
        BoardGameApi g = game("connect4");

        assertThat(g.play("P1", "3")).isEqualTo("OK");
        assertThat(g.play("P2", "3")).isEqualTo("OK");
        assertThat(g.play("P1", "0")).isEqualTo("OK");

        assertThat(g.cellAt(0, 3)).isEqualTo("X");
        assertThat(g.cellAt(1, 3)).isEqualTo("O");
        assertThat(g.cellAt(2, 3)).isEqualTo(".");
        assertThat(g.cellAt(0, 0)).isEqualTo("X");
        assertThat(g.cellAt(5, 6)).isEqualTo(".");
        assertThat(g.cellAt(6, 0)).isEqualTo(".");
    }

    @Test
    void shouldRenderConnect4WithTheBottomRowLast() {
        BoardGameApi g = game("connect4");
        g.play("P1", "3");
        g.play("P2", "3");
        g.play("P1", "0");

        assertThat(g.render())
                .isEqualTo(String.join("\n", ".......", ".......", ".......", ".......", "...O...", "X..X..."));
    }

    @Test
    void shouldRejectConnect4ColumnsOutOfRangeOrMalformed() {
        BoardGameApi g = game("connect4");

        assertThat(g.play("P1", "7")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", "-1")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", "x")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", "")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", "3,3")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P1", " 3")).isEqualTo("INVALID_MOVE");
        assertThat(g.nextPlayer()).hasValue("P1");
    }

    @Test
    void shouldRejectAMoveInAFullConnect4ColumnWithoutConsumingTheTurn() {
        BoardGameApi g = game("connect4");
        assertThat(playAll(g, "0", "0", "0", "0", "0", "0")).isEqualTo("OK");

        assertThat(g.play("P1", "0")).isEqualTo("INVALID_MOVE");
        assertThat(g.nextPlayer()).hasValue("P1");
        assertThat(g.play("P1", "1")).isEqualTo("OK");
    }

    @Test
    void shouldWinConnect4Vertically() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "0", "1", "0", "1", "0", "1", "0")).isEqualTo("WIN");
    }

    @Test
    void shouldWinConnect4Horizontally() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "0", "0", "1", "1", "2", "2", "3")).isEqualTo("WIN");
        assertThat(g.play("P2", "4")).isEqualTo("GAME_OVER");
    }

    @Test
    void shouldWinConnect4OnAnAscendingDiagonal() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "0", "1", "1", "2", "3", "2", "2", "3", "3", "6", "3")).isEqualTo("WIN");
    }

    @Test
    void shouldNotWinConnect4WithOnlyThreeInARow() {
        BoardGameApi g = game("connect4");

        assertThat(playAll(g, "0", "0", "1", "1", "2")).isEqualTo("OK");
        assertThat(g.play("P2", "5")).isEqualTo("OK");
    }

    @Test
    void shouldDeclareConnect4ADrawWhenTheBoardIsFull() {
        BoardGameApi g = game("connect4");
        String[] moves = new String[C4_DRAW.length];
        for (int i = 0; i < moves.length; i++) {
            moves[i] = String.valueOf(C4_DRAW[i]);
        }

        assertThat(playAll(g, moves)).isEqualTo("DRAW");
        assertThat(g.play("P1", "0")).isEqualTo("GAME_OVER");
        assertThat(g.nextPlayer()).isEmpty();
    }

    @Test
    void shouldPlayGomokuOnA15By15Board() {
        BoardGameApi g = game("gomoku");

        assertThat(g.play("P1", "7,7")).isEqualTo("OK");
        assertThat(g.play("P2", "14,14")).isEqualTo("OK");
        assertThat(g.play("P1", "0,0")).isEqualTo("OK");

        assertThat(g.cellAt(7, 7)).isEqualTo("X");
        assertThat(g.cellAt(14, 14)).isEqualTo("O");
        String[] lines = g.render().split("\n");
        assertThat(lines).hasSize(15);
        assertThat(lines).allSatisfy(line -> assertThat(line).hasSize(15));
        assertThat(lines[7]).isEqualTo(".......X.......");
        assertThat(lines[0]).startsWith("X");
    }

    @Test
    void shouldRejectGomokuMovesOutsideTheBoardOrOnOccupiedCells() {
        BoardGameApi g = game("gomoku");
        g.play("P1", "7,7");

        assertThat(g.play("P2", "15,0")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "0,15")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "7,7")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "-1,3")).isEqualTo("INVALID_MOVE");
        assertThat(g.play("P2", "14,14")).isEqualTo("OK");
    }

    @Test
    void shouldNeedFiveInARowToWinGomoku() {
        BoardGameApi g = game("gomoku");

        assertThat(playAll(g, "7,3", "0,0", "7,4", "2,0", "7,5", "4,0", "7,6")).isEqualTo("OK");
        assertThat(g.play("P2", "6,0")).isEqualTo("OK");
        assertThat(g.play("P1", "7,7")).isEqualTo("WIN");
        assertThat(g.play("P2", "8,0")).isEqualTo("GAME_OVER");
    }

    @Test
    void shouldWinGomokuOnADiagonal() {
        BoardGameApi g = game("gomoku");

        assertThat(playAll(g, "3,3", "0,0", "4,4", "0,2", "5,5", "0,4", "6,6", "0,6", "7,7")).isEqualTo("WIN");
    }

    @Test
    void shouldKeepGamesIndependent() {
        BoardGameApi a = game("tictactoe");
        BoardGameApi b = game("tictactoe");

        a.play("P1", "0,0");

        assertThat(b.cellAt(0, 0)).isEqualTo(".");
        assertThat(b.nextPlayer()).hasValue("P1");
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.SnakeGame;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private static int[] at(int row, int col) {
        return new int[] {row, col};
    }

    private SnakeGame game(int width, int height, int[]... cells) {
        List<int[]> foods = new ArrayList<>(Arrays.asList(cells));
        return Candidate.of(() -> Solution.create(width, height, foods));
    }

    private static void play(SnakeGame g, String moves) {
        for (char c : moves.toCharArray()) {
            g.move(String.valueOf(c));
        }
    }

    @Test
    void shouldCreateAHugeBoardWithoutDependingOnItsArea() {
        SnakeGame g = game(100_000, 100_000, at(0, 3), at(99_999, 99_999));

        assertThat(g.foodPosition()).isEqualTo("0,3");
        play(g, "RRR");
        assertThat(g.score()).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("99999,99999");
        assertThat(g.move("D")).isEqualTo(1);
        assertThat(g.isOver()).isFalse();
        assertThat(g.head()).isEqualTo("1,3");
    }

    @Test
    void shouldEatAlongAHugeCorridorAndWinWithoutSlowingDown() {
        int n = 200_000;
        List<int[]> foods = new ArrayList<>();
        for (int c = 1; c < n; c++) {
            foods.add(at(0, c));
        }
        SnakeGame g = Candidate.of(() -> Solution.create(n, 1, foods));

        for (int i = 1; i < n - 1; i++) {
            assertThat(g.move("R")).isEqualTo(i);
        }
        assertThat(g.isOver()).isFalse();
        assertThat(g.move("R")).isEqualTo(-1);
        assertThat(g.isWon()).isTrue();
        assertThat(g.length()).isEqualTo(n);
        assertThat(g.score()).isEqualTo(n - 1);
        assertThat(g.head()).isEqualTo("0," + (n - 1));
        assertThat(g.body()).hasSize(n);
        assertThat(g.body().get(n - 1)).isEqualTo("0,0");
    }

    @Test
    void shouldFillAWholeBoardInASerpentine() {
        SnakeGame g = game(4, 3,
                at(0, 1), at(0, 2), at(0, 3), at(1, 3), at(1, 2), at(1, 1), at(1, 0), at(2, 0), at(2, 1), at(2, 2), at(2, 3));
        String path = "RRRDLLLDRRR";

        for (int i = 0; i < path.length() - 1; i++) {
            assertThat(g.move(String.valueOf(path.charAt(i)))).isEqualTo(i + 1);
        }
        assertThat(g.isWon()).isFalse();
        assertThat(g.foodPosition()).isEqualTo("2,3");

        assertThat(g.move("R")).isEqualTo(-1);

        assertThat(g.isWon()).isTrue();
        assertThat(g.isOver()).isTrue();
        assertThat(g.length()).isEqualTo(12);
        assertThat(g.score()).isEqualTo(11);
        assertThat(g.foodPosition()).isEqualTo("none");
        assertThat(g.move("L")).isEqualTo(-1);
        assertThat(g.length()).isEqualTo(12);
    }

    @Test
    void shouldChaseItsOwnTailForHundredsOfThousandsOfMoves() {
        SnakeGame g = game(3, 3, at(0, 1), at(1, 1), at(1, 0));
        play(g, "RDL");
        assertThat(g.length()).isEqualTo(4);

        for (int i = 0; i < 100_000; i++) {
            assertThat(g.move("U")).isEqualTo(3);
            assertThat(g.move("R")).isEqualTo(3);
            assertThat(g.move("D")).isEqualTo(3);
            assertThat(g.move("L")).isEqualTo(3);
        }

        assertThat(g.isOver()).isFalse();
        assertThat(g.body()).containsExactly("1,0", "1,1", "0,1", "0,0");
    }

    @Test
    void shouldNotFreeTheTailCellOnTheMoveThatEats() {
        SnakeGame g = game(4, 1, at(0, 1), at(0, 2), at(0, 0), at(0, 1));

        assertThat(g.move("R")).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("0,2");

        assertThat(g.move("R")).isEqualTo(2);

        // (0,0) and (0,1) are both still under the snake: both are skipped, nothing is left
        assertThat(g.foodPosition()).isEqualTo("none");
    }

    @Test
    void shouldSkipALongRunOfFoodsUnderTheSnakeAndKeepTheRest() {
        SnakeGame g = game(4, 1, at(0, 1), at(0, 0), at(0, 1), at(0, 0), at(0, 3), at(0, 1));

        g.move("R");

        assertThat(g.foodPosition()).isEqualTo("0,3");
        g.move("R");
        assertThat(g.move("R")).isEqualTo(2);
        assertThat(g.foodPosition()).isEqualTo("none");
    }

    @Test
    void shouldUseTheLastAppliedDirectionForReversalEvenAfterAnInvalidInput() {
        SnakeGame g = game(4, 4, at(0, 1));
        g.move("R");

        assertThatThrownBy(() -> g.move("l")).isInstanceOf(IllegalArgumentException.class);
        assertThat(g.isOver()).isFalse();
        assertThat(g.body()).containsExactly("0,1", "0,0");

        assertThat(g.move("L")).isEqualTo(-1);
        assertThat(g.isOver()).isTrue();
        assertThat(g.head()).isEqualTo("0,1");
    }

    @Test
    void shouldAllowTurningBackOnTheOtherAxisAfterAReversalWasAvoided() {
        SnakeGame g = game(4, 4, at(0, 1));
        g.move("R");

        assertThat(g.move("D")).isEqualTo(1);
        assertThat(g.move("L")).isEqualTo(1);
        assertThat(g.move("U")).isEqualTo(1);
        assertThat(g.isOver()).isFalse();
        assertThat(g.body()).containsExactly("0,0", "1,0");
    }

    @Test
    void shouldNotBeAffectedByChangesTheCallerMakesToItsFoodList() {
        List<int[]> foods = new ArrayList<>();
        int[] first = at(0, 2);
        foods.add(first);
        foods.add(at(1, 2));
        SnakeGame g = Candidate.of(() -> Solution.create(3, 3, foods));

        first[0] = 2;
        first[1] = 2;
        foods.clear();

        assertThat(g.foodPosition()).isEqualTo("0,2");
        play(g, "RR");
        assertThat(g.score()).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("1,2");
    }

    @Test
    void shouldAcceptTheSameCellMoreThanOnceInTheFoodList() {
        SnakeGame g = game(3, 3, at(2, 2), at(2, 2), at(2, 2));

        assertThat(g.foodPosition()).isEqualTo("2,2");
        play(g, "RRDD");
        assertThat(g.score()).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("none");
    }

    @Test
    void shouldReturnABodySnapshotThatLaterMovesDoNotChange() {
        SnakeGame g = game(5, 1, at(0, 1));
        g.move("R");
        List<String> before = g.body();

        g.move("R");

        assertThat(before).containsExactly("0,1", "0,0");
        assertThat(g.body()).containsExactly("0,2", "0,1");
    }
}

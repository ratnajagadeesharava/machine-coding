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

class BehaviorTest {

    private static List<int[]> foods(int[]... cells) {
        return new ArrayList<>(Arrays.asList(cells));
    }

    private static int[] at(int row, int col) {
        return new int[] {row, col};
    }

    private SnakeGame game(int width, int height, int[]... cells) {
        return Candidate.of(() -> Solution.create(width, height, foods(cells)));
    }

    private static void play(SnakeGame g, String moves) {
        for (char c : moves.toCharArray()) {
            g.move(String.valueOf(c));
        }
    }

    @Test
    void shouldStartWithASingleCellSnakeAtTheTopLeft() {
        SnakeGame g = game(3, 2, at(1, 2), at(0, 1));

        assertThat(g.score()).isZero();
        assertThat(g.isOver()).isFalse();
        assertThat(g.isWon()).isFalse();
        assertThat(g.length()).isEqualTo(1);
        assertThat(g.head()).isEqualTo("0,0");
        assertThat(g.body()).containsExactly("0,0");
        assertThat(g.foodPosition()).isEqualTo("1,2");
    }

    @Test
    void shouldRejectInvalidBoardsAndFoodLists() {
        game(3, 3); // obtains a valid instance first so that "not implemented" is reported properly
        assertThatThrownBy(() -> Solution.create(0, 3, foods())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 0, foods())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-1, 2, foods())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(2, -5, foods())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 3, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 2, foods(at(2, 0)))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 2, foods(at(0, 3)))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 2, foods(at(-1, 0)))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 2, foods(new int[] {1}))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(3, 2, foods(new int[] {1, 1, 1})))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectAnythingButTheFourUpperCaseDirections() {
        SnakeGame g = game(3, 3);

        for (String bad : new String[] {null, "", "u", "r", "UP", "RIGHT", " U", "U ", "X", "UD"}) {
            assertThatThrownBy(() -> g.move(bad)).isInstanceOf(IllegalArgumentException.class);
        }
        assertThat(g.head()).isEqualTo("0,0");
        assertThat(g.isOver()).isFalse();
        assertThat(g.score()).isZero();
    }

    @Test
    void shouldMoveInAllFourDirections() {
        SnakeGame g = game(3, 3);

        assertThat(g.move("R")).isZero();
        assertThat(g.head()).isEqualTo("0,1");
        assertThat(g.move("D")).isZero();
        assertThat(g.head()).isEqualTo("1,1");
        assertThat(g.move("L")).isZero();
        assertThat(g.head()).isEqualTo("1,0");
        assertThat(g.move("U")).isZero();
        assertThat(g.head()).isEqualTo("0,0");
        assertThat(g.isOver()).isFalse();
        assertThat(g.length()).isEqualTo(1);
        assertThat(g.body()).containsExactly("0,0");
    }

    @Test
    void shouldEndTheGameOnEveryWall() {
        for (String dir : new String[] {"U", "L"}) {
            SnakeGame g = game(3, 3);
            assertThat(g.move(dir)).isEqualTo(-1);
            assertThat(g.isOver()).isTrue();
            assertThat(g.isWon()).isFalse();
        }
        SnakeGame right = game(3, 3);
        play(right, "RR");
        assertThat(right.isOver()).isFalse();
        assertThat(right.move("R")).isEqualTo(-1);
        assertThat(right.isOver()).isTrue();

        SnakeGame down = game(3, 3);
        play(down, "DD");
        assertThat(down.isOver()).isFalse();
        assertThat(down.move("D")).isEqualTo(-1);
        assertThat(down.isOver()).isTrue();
    }

    @Test
    void shouldLeaveTheSnakeWhereItWasOnAFatalMove() {
        SnakeGame g = game(3, 1, at(0, 2));
        play(g, "RR");
        assertThat(g.score()).isEqualTo(1);

        assertThat(g.move("R")).isEqualTo(-1);

        assertThat(g.isOver()).isTrue();
        assertThat(g.head()).isEqualTo("0,2");
        assertThat(g.body()).containsExactly("0,2", "0,1");
        assertThat(g.length()).isEqualTo(2);
        assertThat(g.score()).isEqualTo(1);
    }

    @Test
    void shouldIgnoreEveryMoveAfterGameOver() {
        SnakeGame g = game(3, 3, at(0, 1));
        g.move("R");
        g.move("U"); // wall
        assertThat(g.isOver()).isTrue();
        List<String> body = g.body();

        for (String d : new String[] {"D", "L", "R", "U", "D"}) {
            assertThat(g.move(d)).isEqualTo(-1);
        }
        assertThat(g.body()).isEqualTo(body);
        assertThat(g.head()).isEqualTo("0,1");
        assertThat(g.score()).isEqualTo(1);
        assertThat(g.isOver()).isTrue();
        assertThatThrownBy(() -> g.move("x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldGrowAndScoreWhenTheHeadReachesFood() {
        SnakeGame g = game(3, 2, at(1, 2), at(0, 1));

        assertThat(g.move("R")).isZero();
        assertThat(g.move("D")).isZero();
        assertThat(g.move("R")).isEqualTo(1);
        assertThat(g.length()).isEqualTo(2);
        assertThat(g.body()).containsExactly("1,2", "1,1");
        assertThat(g.foodPosition()).isEqualTo("0,1");
        assertThat(g.move("U")).isEqualTo(1);
        assertThat(g.body()).containsExactly("0,2", "1,2");
        assertThat(g.move("L")).isEqualTo(2);
        assertThat(g.body()).containsExactly("0,1", "0,2", "1,2");
        assertThat(g.score()).isEqualTo(2);
        assertThat(g.foodPosition()).isEqualTo("none");
        assertThat(g.move("U")).isEqualTo(-1);
        assertThat(g.isOver()).isTrue();
    }

    @Test
    void shouldKeepTheFoodWhereItIsUntilItIsEaten() {
        SnakeGame g = game(4, 3, at(2, 3), at(0, 1));

        play(g, "RRD");
        assertThat(g.foodPosition()).isEqualTo("2,3");
        assertThat(g.score()).isZero();
        play(g, "LL");
        assertThat(g.foodPosition()).isEqualTo("2,3");
    }

    @Test
    void shouldMoveTheTailForwardWhenNothingIsEaten() {
        SnakeGame g = game(5, 1, at(0, 1), at(0, 2), at(0, 4));
        play(g, "RR");
        assertThat(g.body()).containsExactly("0,2", "0,1", "0,0");

        assertThat(g.move("R")).isEqualTo(2);

        assertThat(g.length()).isEqualTo(3);
        assertThat(g.body()).containsExactly("0,3", "0,2", "0,1");
    }

    @Test
    void shouldSkipAFoodListedUnderTheSnakeAtCreation() {
        SnakeGame g = game(3, 1, at(0, 0), at(0, 2));

        assertThat(g.foodPosition()).isEqualTo("0,2");
        assertThat(g.move("R")).isZero();
        assertThat(g.move("R")).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("none");
    }

    @Test
    void shouldSkipFoodsListedUnderTheSnakeWhenTheNextOneSpawns() {
        SnakeGame g = game(3, 3, at(0, 1), at(0, 1), at(0, 0), at(2, 2));

        assertThat(g.foodPosition()).isEqualTo("0,1");
        assertThat(g.move("R")).isEqualTo(1);

        assertThat(g.foodPosition()).isEqualTo("2,2");
    }

    @Test
    void shouldNeverComeBackToASkippedFood() {
        SnakeGame g = game(4, 2, at(0, 1), at(0, 0), at(1, 3));

        g.move("R");
        assertThat(g.foodPosition()).isEqualTo("1,3");
        play(g, "RRD"); // walks away from (0,0), which stays skipped
        assertThat(g.foodPosition()).isEqualTo("none");
        assertThat(g.score()).isEqualTo(2);
    }

    @Test
    void shouldHaveNoFoodWhenTheListIsEmptyOrUsedUp() {
        SnakeGame g = game(3, 3);
        assertThat(g.foodPosition()).isEqualTo("none");
        play(g, "RRDD");
        assertThat(g.score()).isZero();
        assertThat(g.length()).isEqualTo(1);

        SnakeGame one = game(3, 1, at(0, 1));
        one.move("R");
        assertThat(one.foodPosition()).isEqualTo("none");
        one.move("R");
        assertThat(one.length()).isEqualTo(2);
    }

    @Test
    void shouldAllowAReversalWhileTheSnakeIsOneCellLong() {
        SnakeGame g = game(3, 3);

        assertThat(g.move("R")).isZero();
        assertThat(g.move("L")).isZero();
        assertThat(g.isOver()).isFalse();
        assertThat(g.head()).isEqualTo("0,0");
        assertThat(g.move("D")).isZero();
        assertThat(g.move("U")).isZero();
        assertThat(g.isOver()).isFalse();
    }

    private void assertReversalEndsTheGame(int[][] food, String setup, String reversal, int expectedLength) {
        SnakeGame g = game(4, 4, food);
        play(g, setup);
        assertThat(g.isOver()).isFalse();
        assertThat(g.length()).isEqualTo(expectedLength);
        List<String> body = g.body();
        int score = g.score();

        assertThat(g.move(reversal)).isEqualTo(-1);

        assertThat(g.isOver()).isTrue();
        assertThat(g.isWon()).isFalse();
        assertThat(g.body()).isEqualTo(body);
        assertThat(g.score()).isEqualTo(score);
    }

    @Test
    void shouldEndTheGameOnAReversalOfALongerSnake() {
        assertReversalEndsTheGame(new int[][] {at(0, 1)}, "R", "L", 2);
        assertReversalEndsTheGame(new int[][] {at(1, 0)}, "D", "U", 2);
        assertReversalEndsTheGame(new int[][] {at(0, 1), at(1, 1)}, "RDL", "R", 3);
        assertReversalEndsTheGame(new int[][] {at(0, 1), at(1, 1)}, "RDLU", "D", 3);
    }

    @Test
    void shouldAllowTheSameAxisWhenItIsNotAReversal() {
        SnakeGame g = game(5, 2, at(0, 1));
        g.move("R");

        assertThat(g.move("R")).isEqualTo(1);
        assertThat(g.move("D")).isEqualTo(1);
        assertThat(g.move("D")).isEqualTo(-1); // wall: row 2 does not exist
    }

    @Test
    void shouldAllowMovingIntoTheCellTheTailIsLeaving() {
        SnakeGame g = game(3, 3, at(0, 1), at(1, 1), at(1, 0));
        play(g, "RDL");
        assertThat(g.body()).containsExactly("1,0", "1,1", "0,1", "0,0");
        assertThat(g.score()).isEqualTo(3);

        assertThat(g.move("U")).isEqualTo(3);

        assertThat(g.isOver()).isFalse();
        assertThat(g.body()).containsExactly("0,0", "1,0", "1,1", "0,1");
        assertThat(g.move("R")).isEqualTo(3); // chases its own tail again
        assertThat(g.body()).containsExactly("0,1", "0,0", "1,0", "1,1");
    }

    @Test
    void shouldEndTheGameWhenTheHeadHitsAnyOtherPartOfTheBody() {
        SnakeGame g = game(3, 3, at(0, 1), at(0, 2), at(1, 2), at(1, 1));
        play(g, "RRDL");
        assertThat(g.body()).containsExactly("1,1", "1,2", "0,2", "0,1", "0,0");
        assertThat(g.score()).isEqualTo(4);

        assertThat(g.move("U")).isEqualTo(-1); // (0,1) is a body cell that is not the tail

        assertThat(g.isOver()).isTrue();
        assertThat(g.isWon()).isFalse();
        assertThat(g.head()).isEqualTo("1,1");
        assertThat(g.length()).isEqualTo(5);
    }

    @Test
    void shouldWinAndStopWhenTheSnakeFillsTheBoard() {
        SnakeGame g = game(2, 2, at(0, 1), at(1, 1), at(1, 0));

        assertThat(g.move("R")).isEqualTo(1);
        assertThat(g.move("D")).isEqualTo(2);
        assertThat(g.isOver()).isFalse();
        assertThat(g.move("L")).isEqualTo(-1);

        assertThat(g.isOver()).isTrue();
        assertThat(g.isWon()).isTrue();
        assertThat(g.score()).isEqualTo(3);
        assertThat(g.length()).isEqualTo(4);
        assertThat(g.body()).containsExactly("1,0", "1,1", "0,1", "0,0");
        assertThat(g.foodPosition()).isEqualTo("none");
        assertThat(g.move("U")).isEqualTo(-1);
        assertThat(g.body()).containsExactly("1,0", "1,1", "0,1", "0,0");
        assertThat(g.isWon()).isTrue();
    }

    @Test
    void shouldCountAOneCellBoardAsAlreadyWon() {
        SnakeGame g = game(1, 1, at(0, 0));

        assertThat(g.isOver()).isTrue();
        assertThat(g.isWon()).isTrue();
        assertThat(g.foodPosition()).isEqualTo("none");
        assertThat(g.move("R")).isEqualTo(-1);
        assertThat(g.score()).isZero();
        assertThat(g.length()).isEqualTo(1);
    }

    @Test
    void shouldReportALossAsNotWon() {
        SnakeGame g = game(2, 1, at(0, 1));

        assertThat(g.move("R")).isEqualTo(-1); // eats the last cell: board full
        assertThat(g.isWon()).isTrue();

        SnakeGame lost = game(2, 2);
        lost.move("L");
        assertThat(lost.isOver()).isTrue();
        assertThat(lost.isWon()).isFalse();
    }

    @Test
    void shouldKeepScoreEqualToEatenFoodsAndLengthOneHigher() {
        SnakeGame g = game(6, 1, at(0, 1), at(0, 2), at(0, 3), at(0, 4));

        for (int i = 1; i <= 4; i++) {
            assertThat(g.move("R")).isEqualTo(i);
            assertThat(g.score()).isEqualTo(i);
            assertThat(g.length()).isEqualTo(i + 1);
            assertThat(g.body()).hasSize(i + 1);
        }
        assertThat(g.body()).containsExactly("0,4", "0,3", "0,2", "0,1", "0,0");
    }

    @Test
    void shouldKeepTheFoodShownAfterALoss() {
        SnakeGame g = game(3, 3, at(2, 2));

        g.move("U");

        assertThat(g.isOver()).isTrue();
        assertThat(g.foodPosition()).isEqualTo("2,2");
    }

    @Test
    void shouldHandleFoodInEveryCornerOfALargerBoard() {
        SnakeGame g = game(4, 3, at(0, 3), at(2, 3), at(2, 0));

        play(g, "RRR");
        assertThat(g.score()).isEqualTo(1);
        assertThat(g.foodPosition()).isEqualTo("2,3");
        play(g, "DD");
        assertThat(g.score()).isEqualTo(2);
        assertThat(g.foodPosition()).isEqualTo("2,0");
        assertThat(g.isOver()).isFalse();
        assertThat(g.head()).isEqualTo("2,3");
    }
}

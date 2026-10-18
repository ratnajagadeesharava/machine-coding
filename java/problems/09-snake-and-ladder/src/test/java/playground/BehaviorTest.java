package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SnakeAndLadderApi;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {
    private static final Map<Integer, Integer> SNAKES = Map.of(27, 5, 21, 9);
    private static final Map<Integer, Integer> LADDERS = Map.of(3, 22, 8, 26);

    private SnakeAndLadderApi game(String... players) {
        return Candidate.of(() -> Solution.create(30, SNAKES, LADDERS, List.of(players)));
    }

    @Test
    void shouldStartEveryoneOffTheBoardWithFirstPlayerToMove() {
        SnakeAndLadderApi g = game("a", "b");

        assertThat(g.positionOf("a")).isZero();
        assertThat(g.positionOf("b")).isZero();
        assertThat(g.currentPlayer()).hasValue("a");
        assertThat(g.winner()).isEmpty();
        assertThat(g.positionOf("nobody")).isEqualTo(-1);
    }

    @Test
    void shouldMoveForwardAndPassTheTurn() {
        SnakeAndLadderApi g = game("a", "b");

        assertThat(g.roll(4)).isEqualTo(4);

        assertThat(g.positionOf("a")).isEqualTo(4);
        assertThat(g.currentPlayer()).hasValue("b");
    }

    @Test
    void shouldRotateTurnsThroughAllPlayers() {
        SnakeAndLadderApi g = game("a", "b", "c");

        g.roll(1);
        assertThat(g.currentPlayer()).hasValue("b");
        g.roll(2);
        assertThat(g.currentPlayer()).hasValue("c");
        g.roll(4);
        assertThat(g.currentPlayer()).hasValue("a");
        assertThat(g.positionOf("b")).isEqualTo(2);
    }

    @Test
    void shouldClimbLadders() {
        SnakeAndLadderApi g = game("a", "b");

        assertThat(g.roll(3)).isEqualTo(22);

        assertThat(g.positionOf("a")).isEqualTo(22);
    }

    @Test
    void shouldSlideDownSnakes() {
        SnakeAndLadderApi g = game("a", "b");
        g.roll(3);   // a: 22
        g.roll(1);   // b: 1

        assertThat(g.roll(5)).isEqualTo(5); // a lands on 27 -> 5

        assertThat(g.positionOf("a")).isEqualTo(5);
    }

    @Test
    void shouldNotChainJumps() {
        // 3 -> 22 lands next to the snake at 21 but must not be affected by it.
        SnakeAndLadderApi g = game("a");

        assertThat(g.roll(3)).isEqualTo(22);
        assertThat(g.roll(5)).isEqualTo(5); // 22 + 5 = 27, a snake head
        assertThat(g.roll(3)).isEqualTo(26); // 5 + 3 = 8, a ladder bottom -> 26
    }

    @Test
    void shouldStayPutWhenRollWouldOvershootAndStillPassTheTurn() {
        SnakeAndLadderApi g = game("a", "b");
        // bring a to 29 using b as a bystander
        g.roll(6); g.roll(1);   // a 6, b 1
        g.roll(6); g.roll(1);   // a 12, b 2
        g.roll(6);              // a 18
        g.roll(1);              // b 3 -> ladder -> 22
        g.roll(6);              // a 24
        g.roll(1);              // b 23
        g.roll(5);              // a 29
        g.roll(1);              // b 24
        assertThat(g.positionOf("a")).isEqualTo(29);

        assertThat(g.roll(2)).isEqualTo(29); // a would reach 31: stays
        assertThat(g.currentPlayer()).hasValue("b");
    }

    @Test
    void shouldWinOnExactRollAndRefuseFurtherMoves() {
        SnakeAndLadderApi g = game("solo");
        g.roll(6);
        g.roll(6);
        g.roll(6);
        g.roll(6);
        g.roll(5);
        assertThat(g.positionOf("solo")).isEqualTo(29);

        assertThat(g.roll(1)).isEqualTo(30);

        assertThat(g.winner()).hasValue("solo");
        assertThat(g.currentPlayer()).isEmpty();
        assertThat(g.roll(1)).isEqualTo(-1);
        assertThat(g.positionOf("solo")).isEqualTo(30);
    }

    @Test
    void shouldRefuseInvalidDiceWithoutConsumingTheTurn() {
        SnakeAndLadderApi g = game("a", "b");

        assertThat(g.roll(0)).isEqualTo(-1);
        assertThat(g.roll(7)).isEqualTo(-1);
        assertThat(g.roll(-2)).isEqualTo(-1);

        assertThat(g.currentPlayer()).hasValue("a");
        assertThat(g.positionOf("a")).isZero();
    }

    @Test
    void shouldDeclareWinnerWhoReachesTheEndFirst() {
        SnakeAndLadderApi g = game("a", "b");
        // a climbs via ladder 8 -> 26 and then walks 4 to the end; b crawls.
        g.roll(4);   // a 4
        g.roll(1);   // b 1
        g.roll(4);   // a 8 -> 26
        g.roll(1);   // b 2
        assertThat(g.positionOf("a")).isEqualTo(26);

        assertThat(g.roll(4)).isEqualTo(30);

        assertThat(g.winner()).hasValue("a");
    }
}

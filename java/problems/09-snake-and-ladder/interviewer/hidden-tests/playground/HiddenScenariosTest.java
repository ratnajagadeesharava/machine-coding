package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SnakeAndLadderApi;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void ladderToTheLastSquareShouldWin() {
        SnakeAndLadderApi g = Candidate.of(() -> Solution.create(30, Map.of(), Map.of(5, 30), List.of("a", "b")));

        assertThat(g.roll(5)).isEqualTo(30);
        assertThat(g.winner()).hasValue("a");
        assertThat(g.roll(1)).isEqualTo(-1);
    }

    @Test
    void jumpDestinationShouldNotTriggerAnotherJump() {
        // the snake at 20 ends on 12, which is the bottom of a ladder: sliding down must not climb again
        SnakeAndLadderApi g = Candidate.of(() -> Solution.create(30, Map.of(20, 12), Map.of(12, 28), List.of("a")));

        g.roll(5);
        g.roll(5);
        g.roll(5);
        g.roll(5); // 20 -> snake -> 12
        assertThat(g.positionOf("a")).isEqualTo(12);
    }

    @Test
    void shouldAdvanceTurnEvenWhenOvershooting() {
        SnakeAndLadderApi g = Candidate.of(() -> Solution.create(10, Map.of(), Map.of(), List.of("a", "b")));
        g.roll(6); // a 6
        g.roll(6); // b 6
        assertThat(g.roll(6)).isEqualTo(6); // a would reach 12: stays
        assertThat(g.currentPlayer()).hasValue("b");
        assertThat(g.roll(4)).isEqualTo(10);
        assertThat(g.winner()).hasValue("b");
    }

    @Test
    void invalidDiceAfterGameOverShouldStillBeRefused() {
        SnakeAndLadderApi g = Candidate.of(() -> Solution.create(10, Map.of(), Map.of(6, 10), List.of("a")));
        g.roll(6);
        assertThat(g.winner()).hasValue("a");
        assertThat(g.roll(0)).isEqualTo(-1);
        assertThat(g.roll(3)).isEqualTo(-1);
    }
}

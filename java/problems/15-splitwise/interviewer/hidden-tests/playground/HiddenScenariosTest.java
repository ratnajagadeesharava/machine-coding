package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SplitwiseApi;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private SplitwiseApi app() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldAllocateEqualRemainderStartingFromTheAlphabeticallyFirstEvenWhenListedLast() {
        SplitwiseApi s = app();
        s.addEqualExpense("Z", 101, List.of("C", "B", "A"));
        // 101 / 3 = 33 r 2 -> A and B get 34, C gets 33
        assertThat(s.owed("A", "Z")).isEqualTo(34);
        assertThat(s.owed("B", "Z")).isEqualTo(34);
        assertThat(s.owed("C", "Z")).isEqualTo(33);
    }

    @Test
    void settlingMoreThanOwedShouldFlipTheDirection() {
        SplitwiseApi s = app();
        s.addEqualExpense("A", 100, List.of("A", "B"));
        s.settle("B", "A", 80);
        assertThat(s.owed("B", "A")).isZero();
        assertThat(s.owed("A", "B")).isEqualTo(30);
    }

    @Test
    void simplifyOfAlreadySettledStateShouldBeEmpty() {
        SplitwiseApi s = app();
        s.addEqualExpense("A", 100, List.of("A", "B"));
        s.settle("B", "A", 50);
        assertThat(s.simplify()).isEmpty();
    }

    @Test
    void simplifyBetweenTwoPeopleShouldBeASinglePayment() {
        SplitwiseApi s = app();
        s.addExactExpense("A", 70, Map.of("B", 70L));
        assertThat(s.simplify()).containsExactly("B>A:70");
    }

    @Test
    void refusedExpensesShouldLeaveNoTrace() {
        SplitwiseApi s = app();
        s.addExactExpense("A", 10, Map.of("B", 3L, "C", 3L));
        s.addPercentExpense("A", 10, Map.of("B", 99));
        s.addEqualExpense("A", 10, List.of("B", "B"));
        assertThat(s.balances()).isEmpty();
        assertThat(s.owed("B", "A")).isZero();
    }
}

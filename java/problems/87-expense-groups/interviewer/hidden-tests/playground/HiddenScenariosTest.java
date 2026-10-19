package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ExpenseGroups;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldSettleUpTwentyPeopleInReasonableTime() {
        ExpenseGroups g = Candidate.of(Solution::create);
        List<String> members = new ArrayList<>();
        for (int i = 0; i < 20; i++) {
            members.add("p" + i);
        }
        g.createGroup("big", members);
        java.util.Random r = new java.util.Random(9);
        for (int k = 0; k < 60; k++) {
            int payer = r.nextInt(20);
            int debtor = (payer + 1 + r.nextInt(19)) % 20;
            g.addExactExpense("big", "p" + payer, Map.of("p" + debtor, 1L + r.nextInt(100)));
        }

        long start = System.nanoTime();
        List<String> payments = g.settleUp("big");

        assertThat((System.nanoTime() - start) / 1_000_000).as("ms").isLessThan(10_000);
        assertThat(payments.size()).isLessThanOrEqualTo(19);
    }

    @Test
    void shouldFindThePerfectPairingWhenItExists() {
        ExpenseGroups g = Candidate.of(Solution::create);
        List<String> members = new ArrayList<>();
        for (int i = 0; i < 10; i++) {
            members.add("p" + i);
        }
        g.createGroup("pairs", members);
        for (int i = 0; i < 10; i += 2) { // five independent pairs -> exactly 5 payments
            g.addExactExpense("pairs", "p" + i, Map.of("p" + (i + 1), 10L * (i + 1)));
        }

        assertThat(g.settleUp("pairs")).hasSize(5);
    }
}

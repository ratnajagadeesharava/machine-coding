package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SplitwiseApi;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private SplitwiseApi app() {
        return Candidate.of(Solution::create);
    }

    private static Map<String, Long> afterApplying(Map<String, Long> balances, List<String> payments) {
        Map<String, Long> result = new HashMap<>(balances);
        for (String p : payments) {
            String[] parts = p.split("[>:]");
            long amount = Long.parseLong(parts[2]);
            result.merge(parts[0], amount, Long::sum);   // debtor pays: their balance rises
            result.merge(parts[1], -amount, Long::sum);  // creditor receives: their balance falls
        }
        result.values().removeIf(v -> v == 0);
        return result;
    }

    @Test
    void shouldSplitEquallyBetweenParticipants() {
        SplitwiseApi s = app();

        assertThat(s.addEqualExpense("A", 300, List.of("A", "B", "C"))).isTrue();

        assertThat(s.owed("B", "A")).isEqualTo(100);
        assertThat(s.owed("C", "A")).isEqualTo(100);
        assertThat(s.owed("A", "B")).isZero();
        assertThat(s.balances()).containsOnly(Map.entry("A", 200L), Map.entry("B", -100L), Map.entry("C", -100L));
    }

    @Test
    void shouldGiveEqualSplitRemainderToParticipantsInNameOrder() {
        SplitwiseApi s = app();

        s.addEqualExpense("D", 100, List.of("C", "A", "B"));

        assertThat(s.owed("A", "D")).isEqualTo(34);
        assertThat(s.owed("B", "D")).isEqualTo(33);
        assertThat(s.owed("C", "D")).isEqualTo(33);
    }

    @Test
    void shouldAllowPayerWhoDidNotParticipate() {
        SplitwiseApi s = app();

        s.addEqualExpense("A", 90, List.of("B", "C"));

        assertThat(s.owed("B", "A")).isEqualTo(45);
        assertThat(s.owed("C", "A")).isEqualTo(45);
        assertThat(s.balances()).containsEntry("A", 90L);
    }

    @Test
    void shouldCreateNoDebtWhenPayerIsTheOnlyParticipant() {
        SplitwiseApi s = app();

        assertThat(s.addEqualExpense("A", 50, List.of("A"))).isTrue();

        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldSplitByExactAmounts() {
        SplitwiseApi s = app();

        assertThat(s.addExactExpense("A", 100, Map.of("B", 30L, "C", 70L))).isTrue();

        assertThat(s.owed("B", "A")).isEqualTo(30);
        assertThat(s.owed("C", "A")).isEqualTo(70);
    }

    @Test
    void shouldRefuseExactSplitsThatDoNotAddUp() {
        SplitwiseApi s = app();

        assertThat(s.addExactExpense("A", 100, Map.of("B", 30L, "C", 60L))).isFalse();
        assertThat(s.addExactExpense("A", 100, Map.of("B", 130L, "C", -30L))).isFalse();

        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldSplitByPercent() {
        SplitwiseApi s = app();

        assertThat(s.addPercentExpense("A", 200, Map.of("B", 25, "C", 75))).isTrue();

        assertThat(s.owed("B", "A")).isEqualTo(50);
        assertThat(s.owed("C", "A")).isEqualTo(150);
    }

    @Test
    void shouldDistributePercentRemainderInNameOrder() {
        SplitwiseApi s = app();

        // floor(101*33/100)=33, floor(101*33/100)=33, floor(101*34/100)=34 -> 100, remainder 1 goes to "A"
        assertThat(s.addPercentExpense("P", 101, Map.of("C", 34, "A", 33, "B", 33))).isTrue();

        assertThat(s.owed("A", "P")).isEqualTo(34);
        assertThat(s.owed("B", "P")).isEqualTo(33);
        assertThat(s.owed("C", "P")).isEqualTo(34);
    }

    @Test
    void shouldRefusePercentsThatDoNotSumToOneHundred() {
        SplitwiseApi s = app();

        assertThat(s.addPercentExpense("A", 100, Map.of("B", 50, "C", 49))).isFalse();
        assertThat(s.addPercentExpense("A", 100, Map.of("B", 60, "C", 50))).isFalse();
        assertThat(s.addPercentExpense("A", 100, Map.of("B", 120, "C", -20))).isFalse();
        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldRefuseInvalidEqualExpenses() {
        SplitwiseApi s = app();

        assertThat(s.addEqualExpense("A", 0, List.of("B"))).isFalse();
        assertThat(s.addEqualExpense("A", -5, List.of("B"))).isFalse();
        assertThat(s.addEqualExpense("A", 10, List.of())).isFalse();
        assertThat(s.addEqualExpense("A", 10, List.of("B", "B"))).isFalse();
        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldNetDebtsBetweenTwoPeopleInBothDirections() {
        SplitwiseApi s = app();
        s.addEqualExpense("A", 100, List.of("A", "B"));   // B owes A 50
        s.addEqualExpense("B", 30, List.of("A", "B"));    // A owes B 15

        assertThat(s.owed("B", "A")).isEqualTo(35);
        assertThat(s.owed("A", "B")).isZero();
    }

    @Test
    void shouldReduceDebtWhenSettled() {
        SplitwiseApi s = app();
        s.addEqualExpense("A", 100, List.of("A", "B"));

        assertThat(s.settle("B", "A", 20)).isTrue();

        assertThat(s.owed("B", "A")).isEqualTo(30);
        assertThat(s.settle("B", "A", 30)).isTrue();
        assertThat(s.owed("B", "A")).isZero();
        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldRefuseInvalidSettlements() {
        SplitwiseApi s = app();

        assertThat(s.settle("B", "A", 0)).isFalse();
        assertThat(s.settle("B", "A", -5)).isFalse();
        assertThat(s.settle("A", "A", 5)).isFalse();
    }

    @Test
    void shouldReturnZeroForUnknownPeople() {
        SplitwiseApi s = app();

        assertThat(s.owed("x", "y")).isZero();
        assertThat(s.balances()).isEmpty();
    }

    @Test
    void shouldSimplifyChainsOfDebts() {
        SplitwiseApi s = app();
        s.addEqualExpense("A", 200, List.of("A", "B"));   // B owes A 100
        s.addEqualExpense("B", 200, List.of("B", "C"));   // C owes B 100

        assertThat(s.simplify()).containsExactly("C>A:100");
        assertThat(s.owed("B", "A")).isEqualTo(100);      // simplify must not change the state
    }

    @Test
    void shouldSimplifyAnyScenarioToAtMostNMinusOnePayments() {
        SplitwiseApi s = app();
        Random rnd = new Random(1);
        String[] people = {"a", "b", "c", "d", "e", "f"};
        for (int i = 0; i < 200; i++) {
            String payer = people[rnd.nextInt(people.length)];
            int n = 2 + rnd.nextInt(4);
            java.util.Set<String> chosen = new java.util.LinkedHashSet<>();
            while (chosen.size() < n) {
                chosen.add(people[rnd.nextInt(people.length)]);
            }
            s.addEqualExpense(payer, 100 + rnd.nextInt(900), List.copyOf(chosen));
        }
        Map<String, Long> before = s.balances();
        assertThat(before.values().stream().mapToLong(Long::longValue).sum()).isZero();

        List<String> payments = s.simplify();

        assertThat(payments.size()).isLessThanOrEqualTo(Math.max(0, before.size() - 1));
        assertThat(afterApplying(before, payments)).isEmpty();
    }

    @Test
    void shouldStayExactAndFastWithManyExpenses() {
        SplitwiseApi s = app();
        String[] names = new String[1000];
        for (int i = 0; i < names.length; i++) {
            names[i] = "user" + i;
        }
        org.junit.jupiter.api.Assertions.assertTimeoutPreemptively(java.time.Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 100_000; i++) {
                s.addEqualExpense(names[i % 1000], 100 + i % 50, List.of(names[(i + 1) % 1000], names[(i + 2) % 1000], names[(i + 3) % 1000]));
            }
            assertThat(s.balances().values().stream().mapToLong(Long::longValue).sum()).isZero();
            s.owed("user1", "user2");
            s.simplify();
        });
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ExpenseGroups;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private ExpenseGroups groups() {
        return Candidate.of(Solution::create);
    }

    /** Applies "from>to:amount" payments to the balances and returns the resulting balances (zero ones removed). */
    private static Map<String, Long> apply(Map<String, Long> balances, List<String> payments) {
        Map<String, Long> b = new HashMap<>(balances);
        for (String p : payments) {
            String from = p.substring(0, p.indexOf('>'));
            String to = p.substring(p.indexOf('>') + 1, p.indexOf(':'));
            long amount = Long.parseLong(p.substring(p.indexOf(':') + 1));
            assertThat(amount).as("payment %s", p).isPositive();
            b.merge(from, amount, Long::sum); // paying reduces what from owes (negative balance moves towards 0)
            b.merge(to, -amount, Long::sum);
        }
        b.values().removeIf(v -> v == 0);
        return b;
    }

    /** Fewest payments needed: (people with a balance) - (max number of disjoint zero-sum subsets). */
    private static int optimum(java.util.Collection<Long> balances) {
        long[] v = balances.stream().mapToLong(Long::longValue).toArray();
        int k = v.length;
        if (k == 0) {
            return 0;
        }
        long[] sum = new long[1 << k];
        int[] best = new int[1 << k];
        for (int m = 1; m < (1 << k); m++) {
            sum[m] = sum[m & (m - 1)] + v[Integer.numberOfTrailingZeros(m)];
            int b = 0;
            for (int i = 0; i < k; i++) {
                if ((m >> i & 1) != 0) {
                    b = Math.max(b, best[m ^ (1 << i)]);
                }
            }
            best[m] = b + (sum[m] == 0 ? 1 : 0);
        }
        return k - best[(1 << k) - 1];
    }

    /** Builds a group whose balances are exactly {a:+2, b:+3, c:-4, d:-5, e:+4}. The optimum is 3 payments, greedy needs 4. */
    private ExpenseGroups greedyTrap() {
        ExpenseGroups g = groups();
        g.createGroup("trip", List.of("a", "b", "c", "d", "e"));
        g.addExactExpense("trip", "e", Map.of("c", 4L));
        g.addExactExpense("trip", "a", Map.of("d", 2L));
        g.addExactExpense("trip", "b", Map.of("d", 3L));
        return g;
    }

    @Test
    void shouldSplitAnExpenseEquallyAndGiveTheRemainderToTheFirstNames() {
        ExpenseGroups g = groups();
        g.createGroup("flat", List.of("ann", "bob", "cy"));

        g.addExpense("flat", "ann", 100, List.of("cy", "ann", "bob")); // 34, 33, 33 in name order

        assertThat(g.balances("flat")).containsOnly(Map.entry("ann", 66L), Map.entry("bob", -33L), Map.entry("cy", -33L));
        assertThat(g.totalSpent("flat")).isEqualTo(100);
    }

    @Test
    void shouldSupportExactSharesAndAPayerOutsideTheSplit() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob", "cy"));

        g.addExactExpense("g", "ann", Map.of("bob", 30L, "cy", 70L));

        assertThat(g.balances("g")).containsOnly(Map.entry("ann", 100L), Map.entry("bob", -30L), Map.entry("cy", -70L));
    }

    @Test
    void shouldRejectInvalidInputAndChangeNothing() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob"));

        assertThatThrownBy(() -> g.createGroup("g", List.of("x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.createGroup("h", List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.createGroup("h", List.of("x", "x"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addMember("g", "ann")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("nope", "ann", 10, List.of("ann"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("g", "zed", 10, List.of("ann"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("g", "ann", 10, List.of("ann", "zed"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("g", "ann", 0, List.of("ann"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("g", "ann", 10, List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExpense("g", "ann", 10, List.of("bob", "bob"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.addExactExpense("g", "ann", Map.of("bob", 0L))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.settle("g", "ann", "bob", -5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> g.settle("g", "ann", "ann", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThat(g.balances("g")).isEmpty();
        assertThat(g.totalSpent("g")).isZero();
    }

    @Test
    void shouldKeepBalancesOfDifferentGroupsApart() {
        ExpenseGroups g = groups();
        g.createGroup("a", List.of("ann", "bob"));
        g.createGroup("b", List.of("ann", "bob"));

        g.addExpense("a", "ann", 10, List.of("ann", "bob"));

        assertThat(g.balances("a")).containsOnly(Map.entry("ann", 5L), Map.entry("bob", -5L));
        assertThat(g.balances("b")).isEmpty();
    }

    @Test
    void shouldLetASettlementReduceTheDebt() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob"));
        g.addExpense("g", "ann", 100, List.of("ann", "bob"));

        g.settle("g", "bob", "ann", 20);
        assertThat(g.balances("g")).containsOnly(Map.entry("ann", 30L), Map.entry("bob", -30L));
        g.settle("g", "bob", "ann", 30);

        assertThat(g.balances("g")).isEmpty();
        assertThat(g.totalSpent("g")).isEqualTo(100);
    }

    @Test
    void shouldOnlyLetMembersLeaveWhenTheyAreSquare() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob", "cy"));
        g.addExpense("g", "ann", 90, List.of("ann", "bob"));

        assertThat(g.removeMember("g", "bob")).isFalse();
        assertThat(g.removeMember("g", "cy")).isTrue();
        assertThatThrownBy(() -> g.addExpense("g", "ann", 10, List.of("cy"))).isInstanceOf(IllegalArgumentException.class);
        g.settle("g", "bob", "ann", 45);
        assertThat(g.removeMember("g", "bob")).isTrue();
        assertThat(g.removeMember("g", "ann")).isTrue();
    }

    @Test
    void shouldReturnNoPaymentsWhenEverybodyIsSquare() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob"));
        g.addExpense("g", "ann", 10, List.of("ann", "bob"));
        g.addExpense("g", "bob", 10, List.of("ann", "bob"));

        assertThat(g.settleUp("g")).isEmpty();
    }

    @Test
    void shouldSettleUpAPairWithOnePayment() {
        ExpenseGroups g = groups();
        g.createGroup("g", List.of("ann", "bob"));
        g.addExpense("g", "ann", 100, List.of("ann", "bob"));

        assertThat(g.settleUp("g")).containsExactly("bob>ann:50");
    }

    @Test
    void shouldSettleUpWithTheFewestPossiblePayments() {
        ExpenseGroups g = greedyTrap();
        Map<String, Long> before = g.balances("trip");
        assertThat(before).containsOnly(
                Map.entry("a", 2L), Map.entry("b", 3L), Map.entry("c", -4L), Map.entry("d", -5L), Map.entry("e", 4L));

        List<String> payments = g.settleUp("trip");

        assertThat(payments).as("greedy matching needs 4, the optimum is 3").hasSize(3);
        assertThat(apply(before, payments)).isEmpty();
        assertThat(payments).isSorted();
    }

    @Test
    void shouldFindTheOptimumOnLargerRandomGroups() {
        java.util.Random r = new java.util.Random(5);
        for (int round = 0; round < 20; round++) {
            ExpenseGroups g = groups();
            List<String> members = new ArrayList<>();
            for (int i = 0; i < 12; i++) {
                members.add("m" + i);
            }
            g.createGroup("g", members);
            // two independent sub-groups of 6 that each sum to zero -> optimum is at most 10 payments
            for (int half = 0; half < 2; half++) {
                for (int k = 0; k < 5; k++) {
                    int payer = half * 6 + r.nextInt(6);
                    int debtor = half * 6 + r.nextInt(6);
                    if (payer != debtor) {
                        g.addExactExpense("g", "m" + payer, Map.of("m" + debtor, 1L + r.nextInt(50)));
                    }
                }
            }
            Map<String, Long> before = g.balances("g");

            List<String> payments = g.settleUp("g");

            assertThat(apply(before, payments)).as("round %d", round).isEmpty();
            assertThat(payments.size()).as("round %d: fewer payments exist", round).isEqualTo(optimum(before.values()));
        }
    }

    @Test
    void shouldSplitAcrossThreadsWithoutLosingAnExpense() {
        ExpenseGroups g = groups();
        List<String> all = List.of("a", "b", "c", "d");
        g.createGroup("g", all);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 500; i++) {
                g.addExpense("g", all.get(t % 4), 10, all);
            }
        });

        assertThat(g.totalSpent("g")).isEqualTo(8 * 500 * 10);
        Map<String, Long> b = new TreeMap<>(g.balances("g"));
        assertThat(b.values().stream().mapToLong(Long::longValue).sum()).isZero();
        // every expense of 10 split 4 ways is charged 3,3,2,2 to a,b,c,d (remainder 2 goes to the first two names);
        // threads t=0..7 pay as a,b,c,d,a,b,c,d -> each payer paid 2 * 500 * 10 = 10_000; there are 4_000 expenses
        assertThat(b).containsOnly(
                Map.entry("a", 10_000L - 4_000 * 3L),
                Map.entry("b", 10_000L - 4_000 * 3L),
                Map.entry("c", 10_000L - 4_000 * 2L),
                Map.entry("d", 10_000L - 4_000 * 2L));
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            ExpenseGroups g = groups();
            g.createGroup("g", List.of("a", "b", "c"));

            Concurrently.run(6, t -> {
                for (int i = 0; i < 20; i++) {
                    g.addExpense("g", "a", 7, List.of("a", "b", "c"));
                    g.settle("g", "b", "a", 1);
                }
            });

            // 120 expenses of 7: a is charged 3,2,2 -> a pays 840, owes 3*120; b and c owe 2*120 each; b also settled 120
            assertThat(g.totalSpent("g")).as("run %d", run).isEqualTo(840);
            assertThat(g.balances("g")).as("run %d", run)
                    .containsOnly(Map.entry("a", 480L - 120L), Map.entry("b", -240L + 120L), Map.entry("c", -240L));
        }
    }
}

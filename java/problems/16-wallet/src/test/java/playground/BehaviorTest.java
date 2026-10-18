package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.WalletApi;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private WalletApi service(String... wallets) {
        WalletApi api = Candidate.of(Solution::create);
        for (String w : wallets) {
            assertThat(api.createWallet(w)).isTrue();
        }
        return api;
    }

    private static long count(List<String> results, String what) {
        return results.stream().filter(what::equals).count();
    }

    @Test
    void shouldStartWithAnEmptyWalletAndRefuseDuplicateCreation() {
        WalletApi api = service("alice");

        assertThat(api.balance("alice")).hasValue(0);
        assertThat(api.credit("k1", "alice", 50)).isEqualTo("APPLIED");
        assertThat(api.createWallet("alice")).isFalse();
        assertThat(api.createWallet(" ")).isFalse();
        assertThat(api.createWallet("")).isFalse();
        assertThat(api.balance("alice")).hasValue(50);
        assertThat(api.balance("nobody")).isEmpty();
    }

    @Test
    void shouldCreditAndDebit() {
        WalletApi api = service("alice");

        assertThat(api.credit("k1", "alice", 1000)).isEqualTo("APPLIED");
        assertThat(api.debit("k2", "alice", 400)).isEqualTo("APPLIED");

        assertThat(api.balance("alice")).hasValue(600);
    }

    @Test
    void shouldAllowDebitDownToExactlyZeroButNotBelow() {
        WalletApi api = service("alice");
        api.credit("k1", "alice", 100);

        assertThat(api.debit("k2", "alice", 101)).isEqualTo("INSUFFICIENT_FUNDS");
        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.debit("k3", "alice", 100)).isEqualTo("APPLIED");
        assertThat(api.balance("alice")).hasValue(0);
        assertThat(api.debit("k4", "alice", 1)).isEqualTo("INSUFFICIENT_FUNDS");
    }

    @Test
    void shouldRefuseNonPositiveAmounts() {
        WalletApi api = service("alice", "bob");
        api.credit("k0", "alice", 100);

        assertThat(api.credit("k1", "alice", 0)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.credit("k2", "alice", -5)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.debit("k3", "alice", 0)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.debit("k4", "alice", -1)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.transfer("k5", "alice", "bob", 0)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.transfer("k6", "alice", "bob", -10)).isEqualTo("INVALID_AMOUNT");

        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.balance("bob")).hasValue(0);
    }

    @Test
    void shouldRefuseUnknownWalletsWithoutSideEffects() {
        WalletApi api = service("alice");
        api.credit("k0", "alice", 100);

        assertThat(api.credit("k1", "ghost", 10)).isEqualTo("UNKNOWN_WALLET");
        assertThat(api.debit("k2", "ghost", 10)).isEqualTo("UNKNOWN_WALLET");
        assertThat(api.transfer("k3", "alice", "ghost", 10)).isEqualTo("UNKNOWN_WALLET");
        assertThat(api.transfer("k4", "ghost", "alice", 10)).isEqualTo("UNKNOWN_WALLET");

        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.statement("alice")).containsExactly("+100");
    }

    @Test
    void shouldMoveMoneyBetweenWallets() {
        WalletApi api = service("alice", "bob");
        api.credit("k0", "alice", 1000);

        assertThat(api.transfer("k1", "alice", "bob", 300)).isEqualTo("APPLIED");

        assertThat(api.balance("alice")).hasValue(700);
        assertThat(api.balance("bob")).hasValue(300);
    }

    @Test
    void shouldRefuseTransferBeyondBalanceAndChangeNothing() {
        WalletApi api = service("alice", "bob");
        api.credit("k0", "alice", 100);

        assertThat(api.transfer("k1", "alice", "bob", 101)).isEqualTo("INSUFFICIENT_FUNDS");

        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.balance("bob")).hasValue(0);
        assertThat(api.statement("bob")).isEmpty();
    }

    @Test
    void shouldNotApplyTheSameKeyTwice() {
        WalletApi api = service("alice");

        assertThat(api.credit("k1", "alice", 1000)).isEqualTo("APPLIED");
        assertThat(api.credit("k1", "alice", 1000)).isEqualTo("DUPLICATE");

        assertThat(api.balance("alice")).hasValue(1000);
        assertThat(api.statement("alice")).containsExactly("+1000");
    }

    @Test
    void shouldTreatAKeyAsDuplicateEvenWithDifferentParameters() {
        WalletApi api = service("alice", "bob");
        api.credit("k1", "alice", 100);

        assertThat(api.credit("k1", "alice", 999)).isEqualTo("DUPLICATE");
        assertThat(api.credit("k1", "bob", 100)).isEqualTo("DUPLICATE");
        assertThat(api.debit("k1", "alice", 10)).isEqualTo("DUPLICATE");
        assertThat(api.transfer("k1", "alice", "bob", 10)).isEqualTo("DUPLICATE");

        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.balance("bob")).hasValue(0);
    }

    @Test
    void shouldNotConsumeTheKeyOfARefusedRequest() {
        WalletApi api = service("alice");

        assertThat(api.debit("k1", "alice", 50)).isEqualTo("INSUFFICIENT_FUNDS");
        api.credit("k2", "alice", 100);

        assertThat(api.debit("k1", "alice", 50)).isEqualTo("APPLIED");
        assertThat(api.balance("alice")).hasValue(50);
    }

    @Test
    void shouldMoveMoneyOnlyOnceWhenATransferIsReplayed() {
        WalletApi api = service("alice", "bob");
        api.credit("k0", "alice", 1000);

        assertThat(api.transfer("t1", "alice", "bob", 250)).isEqualTo("APPLIED");
        assertThat(api.transfer("t1", "alice", "bob", 250)).isEqualTo("DUPLICATE");

        assertThat(api.balance("alice")).hasValue(750);
        assertThat(api.balance("bob")).hasValue(250);
    }

    @Test
    void shouldRecordSignedStatementEntriesInOrder() {
        WalletApi api = service("alice", "bob");
        api.credit("k1", "alice", 1000);
        api.transfer("k2", "alice", "bob", 300);
        api.debit("k3", "alice", 50);
        api.transfer("k4", "bob", "alice", 100);
        api.debit("k5", "bob", 9999); // refused
        api.credit("k1", "alice", 5); // duplicate

        assertThat(api.statement("alice")).containsExactly("+1000", "-300", "-50", "+100");
        assertThat(api.statement("bob")).containsExactly("+300", "-100");
        assertThat(api.statement("ghost")).isEmpty();
    }

    @Test
    void shouldKeepTotalMoneyConstantWhenOppositeTransfersRace() {
        WalletApi api = service("a", "b");
        api.credit("seed-a", "a", 1000);
        api.credit("seed-b", "b", 1000);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 300; i++) {
                String key = "t" + t + "-" + i;
                long amount = 1 + i % 3;
                if (t % 2 == 0) {
                    api.transfer(key, "a", "b", amount);
                } else {
                    api.transfer(key, "b", "a", amount);
                }
            }
        });

        long a = api.balance("a").getAsLong();
        long b = api.balance("b").getAsLong();
        assertThat(a).isGreaterThanOrEqualTo(0);
        assertThat(b).isGreaterThanOrEqualTo(0);
        assertThat(a + b).isEqualTo(2000);
    }

    @Test
    void shouldNeverOverdrawWhenManyDebitsRace() {
        WalletApi api = service("alice");
        api.credit("seed", "alice", 100);

        List<String> results = Concurrently.collect(100, i -> api.debit("d" + i, "alice", 3));

        assertThat(count(results, "APPLIED")).isEqualTo(33);
        assertThat(count(results, "INSUFFICIENT_FUNDS")).isEqualTo(67);
        assertThat(api.balance("alice")).hasValue(1);
        assertThat(api.statement("alice")).hasSize(34);
    }

    @Test
    void shouldApplyARacingKeyExactlyOnce() {
        WalletApi api = service("alice");

        List<String> results = Concurrently.collect(32, i -> api.credit("same-key", "alice", 10));

        assertThat(count(results, "APPLIED")).isEqualTo(1);
        assertThat(count(results, "DUPLICATE")).isEqualTo(31);
        assertThat(api.balance("alice")).hasValue(10);
        assertThat(api.statement("alice")).containsExactly("+10");
    }

    @Test
    void shouldConserveMoneyAcrossARingOfWalletsUnderContention() {
        WalletApi api = service("w0", "w1", "w2", "w3");
        for (int i = 0; i < 4; i++) {
            api.credit("seed" + i, "w" + i, 500);
        }

        Concurrently.run(16, t -> {
            for (int i = 0; i < 250; i++) {
                int from = (t + i) % 4;
                int to = (t + 2 * i + 1) % 4;
                if (from == to) {
                    to = (to + 1) % 4;
                }
                api.transfer("r" + t + "-" + i, "w" + from, "w" + to, 1 + (i * 7) % 40);
            }
        });

        long total = 0;
        for (int i = 0; i < 4; i++) {
            long balance = api.balance("w" + i).getAsLong();
            assertThat(balance).isGreaterThanOrEqualTo(0);
            total += balance;
        }
        assertThat(total).isEqualTo(2000);
    }

    @Test
    void shouldKeepStatementsConsistentWithBalancesAfterRaces() {
        WalletApi api = service("a", "b");
        api.credit("seed", "a", 500);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 100; i++) {
                String from = t % 2 == 0 ? "a" : "b";
                String to = t % 2 == 0 ? "b" : "a";
                api.transfer("x" + t + "-" + i, from, to, 5);
            }
        });

        for (String w : List.of("a", "b")) {
            long sum = api.statement(w).stream().mapToLong(Long::parseLong).sum();
            assertThat(sum).isEqualTo(api.balance(w).getAsLong());
        }
    }
}

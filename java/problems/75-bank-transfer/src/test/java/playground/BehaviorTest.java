package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Bank;
import java.util.List;
import java.util.concurrent.ThreadLocalRandom;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private Bank bank() {
        return Candidate.of(Solution::create);
    }

    private static String acc(int i) {
        return "acc-" + i;
    }

    private static Bank bankWith(Bank b, int accounts, long each) {
        for (int i = 0; i < accounts; i++) {
            b.open(acc(i), each);
        }
        return b;
    }

    @Test
    void shouldMoveMoneyBetweenAccounts() {
        Bank b = bankWith(bank(), 2, 100);

        boolean ok = b.transfer(acc(0), acc(1), 30);

        assertThat(ok).isTrue();
        assertThat(b.balance(acc(0))).isEqualTo(70);
        assertThat(b.balance(acc(1))).isEqualTo(130);
        assertThat(b.total()).isEqualTo(200);
    }

    @Test
    void shouldRefuseWhenFundsAreInsufficientAndChangeNothing() {
        Bank b = bankWith(bank(), 2, 100);

        boolean ok = b.transfer(acc(0), acc(1), 101);

        assertThat(ok).isFalse();
        assertThat(b.balance(acc(0))).isEqualTo(100);
        assertThat(b.balance(acc(1))).isEqualTo(100);
    }

    @Test
    void shouldAllowTransferOfTheWholeBalance() {
        Bank b = bankWith(bank(), 2, 100);

        assertThat(b.transfer(acc(0), acc(1), 100)).isTrue();

        assertThat(b.balance(acc(0))).isZero();
        assertThat(b.balance(acc(1))).isEqualTo(200);
    }

    @Test
    void shouldRejectInvalidRequests() {
        Bank b = bankWith(bank(), 2, 100);

        assertThatThrownBy(() -> b.transfer(acc(0), "ghost", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.transfer("ghost", acc(0), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.transfer(acc(0), acc(0), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.transfer(acc(0), acc(1), 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.transfer(acc(0), acc(1), -5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.balance("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.open(acc(0), 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.open("neg", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(b.total()).isEqualTo(200);
    }

    @Test
    void shouldNotDeadlockWhenTwoAccountsTransferToEachOther() {
        Bank b = bankWith(bank(), 2, 1_000_000);

        Concurrently.run(4, i -> {
            String from = acc(i % 2);
            String to = acc((i + 1) % 2);
            for (int n = 0; n < 20_000; n++) {
                b.transfer(from, to, 1);
            }
        });

        assertThat(b.balance(acc(0))).isEqualTo(1_000_000);
        assertThat(b.balance(acc(1))).isEqualTo(1_000_000);
    }

    @Test
    void shouldNotDeadlockWhenTransfersFormACycle() {
        int n = 8;
        Bank b = bankWith(bank(), n, 100_000);

        Concurrently.run(n, i -> {
            for (int k = 0; k < 5_000; k++) {
                b.transfer(acc(i), acc((i + 1) % n), 1);
            }
        });

        assertThat(b.total()).isEqualTo(n * 100_000L);
        for (int i = 0; i < n; i++) {
            assertThat(b.balance(acc(i))).isEqualTo(100_000);
        }
    }

    @Test
    void shouldAcceptExactlyAsManyWithdrawalsAsTheBalanceAllows() {
        Bank b = bank();
        b.open("source", 500);
        b.open("sink", 0);

        List<Boolean> results = Concurrently.collect(100, i -> b.transfer("source", "sink", 10));

        assertThat(results.stream().filter(r -> r).count()).isEqualTo(50);
        assertThat(b.balance("source")).isZero();
        assertThat(b.balance("sink")).isEqualTo(500);
    }

    @Test
    void shouldConserveMoneyUnderRandomLoad() {
        int accounts = 20;
        Bank b = bankWith(bank(), accounts, 1_000);
        AtomicLong succeeded = new AtomicLong();

        Concurrently.run(16, t -> {
            ThreadLocalRandom rnd = ThreadLocalRandom.current();
            for (int k = 0; k < 10_000; k++) {
                int from = rnd.nextInt(accounts);
                int to = (from + 1 + rnd.nextInt(accounts - 1)) % accounts;
                if (b.transfer(acc(from), acc(to), 1 + rnd.nextInt(50))) {
                    succeeded.incrementAndGet();
                }
            }
        });

        assertThat(b.total()).isEqualTo(accounts * 1_000L);
        for (int i = 0; i < accounts; i++) {
            assertThat(b.balance(acc(i))).isGreaterThanOrEqualTo(0);
        }
        assertThat(succeeded.get()).isPositive();
    }

    @Test
    void shouldNeverShowMoneyInFlightToAnAuditor() {
        int accounts = 10;
        long expected = accounts * 1_000L;
        Bank b = bankWith(bank(), accounts, 1_000);
        AtomicBoolean done = new AtomicBoolean();
        AtomicInteger movers = new AtomicInteger(6);
        AtomicLong wrongTotals = new AtomicLong();
        AtomicLong audits = new AtomicLong();

        Concurrently.run(7, t -> {
            if (t == 6) { // auditor
                while (!done.get()) {
                    if (b.total() != expected) {
                        wrongTotals.incrementAndGet();
                    }
                    audits.incrementAndGet();
                }
                return;
            }
            ThreadLocalRandom rnd = ThreadLocalRandom.current();
            for (int k = 0; k < 20_000; k++) {
                int from = rnd.nextInt(accounts);
                int to = (from + 1 + rnd.nextInt(accounts - 1)) % accounts;
                b.transfer(acc(from), acc(to), 1 + rnd.nextInt(100));
            }
            if (movers.decrementAndGet() == 0) {
                done.set(true);
            }
        });

        assertThat(audits.get()).as("auditor must make progress while transfers run").isPositive();
        assertThat(wrongTotals.get()).as("total() observed a half-applied transfer").isZero();
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            int n = 4;
            Bank b = bankWith(bank(), n, 1_000);

            Concurrently.run(n * 2, i -> {
                for (int k = 0; k < 300; k++) {
                    b.transfer(acc(i % n), acc((i + 1) % n), 1);
                    b.transfer(acc((i + 1) % n), acc(i % n), 1);
                }
            });

            assertThat(b.total()).as("run %d", run).isEqualTo(n * 1_000L);
        }
    }
}

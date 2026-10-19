package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Bank;
import java.util.concurrent.ThreadLocalRandom;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandleOneHotAccountReceivingFromEveryone() {
        Bank b = Candidate.of(Solution::create);
        int senders = 32;
        b.open("hot", 0);
        for (int i = 0; i < senders; i++) {
            b.open("s" + i, 10_000);
        }

        Concurrently.run(senders, 60_000, i -> {
            for (int k = 0; k < 2_000; k++) {
                b.transfer("s" + i, "hot", 1);
            }
        });

        assertThat(b.balance("hot")).isEqualTo(senders * 2_000L);
        assertThat(b.total()).isEqualTo(senders * 10_000L);
    }

    @Test
    void shouldKeepTotalWithTransfersBothWaysBetweenManyPairs() {
        Bank b = Candidate.of(Solution::create);
        int n = 50;
        for (int i = 0; i < n; i++) {
            b.open("a" + i, 5_000);
        }

        Concurrently.run(24, 60_000, t -> {
            ThreadLocalRandom r = ThreadLocalRandom.current();
            for (int k = 0; k < 20_000; k++) {
                int x = r.nextInt(n);
                int y = (x + 1 + r.nextInt(n - 1)) % n;
                b.transfer("a" + x, "a" + y, 1 + r.nextInt(20));
            }
        });

        assertThat(b.total()).isEqualTo(n * 5_000L);
    }
}

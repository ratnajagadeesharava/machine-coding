package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.LockService;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private final FakeClock clock = new FakeClock(0);

    private LockService locks() {
        return Candidate.of(() -> Solution.create(clock));
    }

    /** What a storage server does with fencing tokens: it refuses writes carrying a token older than one it has seen. */
    private static final class FencedStore {
        long highest;
        String value = "";

        boolean write(long token, String v) {
            if (token < highest) {
                return false;
            }
            highest = token;
            value = v;
            return true;
        }
    }

    @Test
    void shouldGrantAFreeLockWithTheFirstToken() {
        LockService l = locks();

        Optional<Long> t = l.acquire("job", "ann", 1_000);

        assertThat(t).hasValue(1L);
        assertThat(l.holder("job")).hasValue("ann");
        assertThat(l.latestToken("job")).isEqualTo(1);
    }

    @Test
    void shouldRefuseAnotherOwnerWhileTheLeaseIsValid() {
        LockService l = locks();
        l.acquire("job", "ann", 1_000);

        clock.advance(999);

        assertThat(l.acquire("job", "bob", 1_000)).isEmpty();
        assertThat(l.holder("job")).hasValue("ann");
    }

    @Test
    void shouldKeepLocksIndependent() {
        LockService l = locks();
        l.acquire("a", "ann", 1_000);

        assertThat(l.acquire("b", "bob", 1_000)).hasValue(1L);
        assertThat(l.latestToken("c")).isZero();
    }

    @Test
    void shouldLetAnotherOwnerTakeOverExactlyWhenTheLeaseExpiresWithAHigherToken() {
        LockService l = locks();
        l.acquire("job", "ann", 1_000);

        clock.advance(1_000);

        assertThat(l.holder("job")).isEmpty();
        assertThat(l.acquire("job", "bob", 1_000)).hasValue(2L);
        assertThat(l.holder("job")).hasValue("bob");
    }

    @Test
    void shouldRefreshTheLeaseAndKeepTheTokenWhenTheOwnerAcquiresAgain() {
        LockService l = locks();
        long token = l.acquire("job", "ann", 1_000).orElseThrow();

        clock.advance(900);
        Optional<Long> again = l.acquire("job", "ann", 1_000);
        clock.advance(900);

        assertThat(again).hasValue(token);
        assertThat(l.holder("job")).hasValue("ann"); // 1_800 < 900 + 1_000
        assertThat(l.latestToken("job")).isEqualTo(token);
    }

    @Test
    void shouldIssueANewTokenWhenTheOwnTokenHasExpiredAndIsReacquired() {
        LockService l = locks();
        l.acquire("job", "ann", 1_000);

        clock.advance(1_000);

        assertThat(l.acquire("job", "ann", 1_000)).hasValue(2L);
    }

    @Test
    void shouldRenewOnlyWithTheRightOwnerAndTokenBeforeExpiry() {
        LockService l = locks();
        long token = l.acquire("job", "ann", 1_000).orElseThrow();

        assertThat(l.renew("job", "bob", token, 1_000)).isFalse();
        assertThat(l.renew("job", "ann", token + 1, 1_000)).isFalse();
        clock.advance(500);
        assertThat(l.renew("job", "ann", token, 1_000)).isTrue();
        clock.advance(999);
        assertThat(l.holder("job")).hasValue("ann");
        clock.advance(1);
        assertThat(l.renew("job", "ann", token, 1_000)).as("too late").isFalse();
    }

    @Test
    void shouldReleaseOnlyForTheCurrentHolder() {
        LockService l = locks();
        long token = l.acquire("job", "ann", 1_000).orElseThrow();

        assertThat(l.release("job", "bob", token)).isFalse();
        assertThat(l.release("job", "ann", token + 5)).isFalse();
        assertThat(l.holder("job")).hasValue("ann");
        assertThat(l.release("job", "ann", token)).isTrue();
        assertThat(l.release("job", "ann", token)).isFalse();
        assertThat(l.acquire("job", "bob", 1_000)).hasValue(token + 1);
    }

    @Test
    void shouldNotLetAStaleOwnerReleaseOrRenewTheNewHoldersLock() {
        LockService l = locks();
        long old = l.acquire("job", "ann", 1_000).orElseThrow();
        clock.advance(1_500);
        long fresh = l.acquire("job", "bob", 1_000).orElseThrow();

        assertThat(l.release("job", "ann", old)).isFalse();
        assertThat(l.renew("job", "ann", old, 1_000)).isFalse();

        assertThat(l.holder("job")).hasValue("bob");
        assertThat(fresh).isGreaterThan(old);
    }

    @Test
    void shouldLetAFencedStoreRejectAWriterWhoseLeaseExpiredWhilePaused() {
        LockService l = locks();
        FencedStore store = new FencedStore();
        long annToken = l.acquire("db", "ann", 1_000).orElseThrow();
        // ann stalls (long GC pause) past her lease; bob takes the lock and writes
        clock.advance(2_000);
        long bobToken = l.acquire("db", "bob", 1_000).orElseThrow();
        assertThat(store.write(bobToken, "bob's data")).isTrue();

        // ann wakes up, still believes she holds the lock and writes
        boolean annWrote = store.write(annToken, "ann's stale data");

        assertThat(annWrote).isFalse();
        assertThat(store.value).isEqualTo("bob's data");
    }

    @Test
    void shouldRejectNonPositiveLeases() {
        LockService l = locks();

        assertThatThrownBy(() -> l.acquire("job", "ann", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.acquire("job", "ann", -5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldGiveMutualExclusionAndIncreasingTokensUnderContention() {
        LockService l = locks();
        AtomicInteger inside = new AtomicInteger();
        AtomicInteger violations = new AtomicInteger();
        List<Long> tokens = Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(16, t -> {
            String owner = "worker-" + t;
            for (int i = 0; i < 100; i++) {
                Optional<Long> token;
                while ((token = l.acquire("shared", owner, 60_000)).isEmpty()) {
                    Thread.yield();
                }
                if (inside.incrementAndGet() != 1) {
                    violations.incrementAndGet();
                }
                tokens.add(token.get());
                inside.decrementAndGet();
                if (!l.release("shared", owner, token.get())) {
                    violations.incrementAndGet();
                }
            }
        });

        assertThat(violations).hasValue(0);
        assertThat(tokens).hasSize(1_600);
        for (int i = 1; i < tokens.size(); i++) {
            assertThat(tokens.get(i)).isGreaterThan(tokens.get(i - 1));
        }
        assertThat(l.latestToken("shared")).isEqualTo(1_600);
    }

    @Test
    void shouldLetExactlyOneThreadWinAFreeLock() {
        LockService l = locks();

        List<Optional<Long>> results = Concurrently.collect(16, t -> l.acquire("job", "worker-" + t, 10_000));

        assertThat(results.stream().filter(Optional::isPresent).count()).isEqualTo(1);
        assertThat(l.latestToken("job")).isEqualTo(1);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            FakeClock c = new FakeClock(0);
            LockService l = Candidate.of(() -> Solution.create(c));
            AtomicInteger wins = new AtomicInteger();

            Concurrently.run(8, t -> {
                if (l.acquire("job", "w" + t, 1_000).isPresent()) {
                    wins.incrementAndGet();
                }
            });
            c.advance(1_000);
            Concurrently.run(8, t -> {
                if (l.acquire("job", "w" + t, 1_000).isPresent()) {
                    wins.incrementAndGet();
                }
            });

            assertThat(wins).as("run %d", run).hasValue(2);
            assertThat(l.latestToken("job")).as("run %d", run).isEqualTo(2);
        }
    }
}

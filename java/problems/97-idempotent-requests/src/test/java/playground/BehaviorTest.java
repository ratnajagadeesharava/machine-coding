package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Handler;
import contract.IdempotentProcessor;
import contract.KeyConflictException;
import java.util.HashSet;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long HOUR = 3_600_000;
    private final FakeClock clock = new FakeClock(0);

    private IdempotentProcessor processor() {
        return Candidate.of(() -> Solution.create(clock));
    }

    private static Handler counting(AtomicInteger calls) {
        return body -> "done:" + body + ":" + calls.incrementAndGet();
    }

    private static void sleep(long ms) {
        try {
            Thread.sleep(ms);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    @Test
    void shouldRunTheHandlerOnceAndReplayTheStoredResponse() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();

        String first = p.process("key-1", "pay 10", counting(calls));
        String second = p.process("key-1", "pay 10", counting(calls));

        assertThat(first).isEqualTo("done:pay 10:1");
        assertThat(second).isEqualTo(first);
        assertThat(calls).hasValue(1);
    }

    @Test
    void shouldTreatDifferentKeysIndependently() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();

        p.process("a", "x", counting(calls));
        p.process("b", "x", counting(calls));

        assertThat(calls).hasValue(2);
    }

    @Test
    void shouldRejectAKeyReusedForADifferentBody() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();
        p.process("key-1", "pay 10", counting(calls));

        assertThatThrownBy(() -> p.process("key-1", "pay 99", counting(calls))).isInstanceOf(KeyConflictException.class);

        assertThat(calls).hasValue(1);
        assertThat(p.process("key-1", "pay 10", counting(calls))).isEqualTo("done:pay 10:1");
    }

    @Test
    void shouldRejectInvalidArguments() {
        IdempotentProcessor p = processor();

        assertThatThrownBy(() -> p.process(null, "x", b -> "r")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.process(" ", "x", b -> "r")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> p.process("k", null, b -> "r")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotStoreAFailureAndRunTheHandlerAgainOnRetry() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();
        Handler flaky = body -> {
            if (calls.incrementAndGet() == 1) {
                throw new IllegalStateException("boom");
            }
            return "ok:" + calls.get();
        };

        assertThatThrownBy(() -> p.process("k", "x", flaky)).isInstanceOf(IllegalStateException.class).hasMessage("boom");

        assertThat(p.process("k", "x", flaky)).isEqualTo("ok:2");
        assertThat(p.process("k", "x", flaky)).isEqualTo("ok:2");
        assertThat(calls).hasValue(2);
    }

    @Test
    void shouldExpireStoredResponsesAfter24Hours() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();
        p.process("k", "x", counting(calls));

        clock.advance(24 * HOUR - 1);
        assertThat(p.process("k", "x", counting(calls))).isEqualTo("done:x:1");
        clock.advance(1);

        assertThat(p.process("k", "x", counting(calls))).isEqualTo("done:x:2");
        assertThat(calls).hasValue(2);
    }

    @Test
    void shouldAllowADifferentBodyOnceTheKeyHasExpired() {
        IdempotentProcessor p = processor();
        p.process("k", "old", b -> "r1");

        clock.advance(25 * HOUR);

        assertThat(p.process("k", "new", b -> "r2")).isEqualTo("r2");
    }

    @Test
    void shouldRunTheHandlerOnlyOnceWhenManyThreadsRaceWithTheSameKey() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();
        Handler slow = body -> {
            sleep(100);
            return "r" + calls.incrementAndGet();
        };

        long start = System.nanoTime();
        List<String> responses = Concurrently.collect(32, t -> p.process("same", "body", slow));
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(calls).hasValue(1);
        assertThat(new HashSet<>(responses)).containsExactly("r1");
        assertThat(ms).as("waiters share the single execution").isLessThan(1_500);
    }

    @Test
    void shouldNotMakeDifferentKeysWaitForEachOther() {
        IdempotentProcessor p = processor();
        Handler slow = body -> {
            sleep(150);
            return "r";
        };

        long start = System.nanoTime();
        Concurrently.run(16, t -> p.process("key-" + t, "body", slow));
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).as("16 keys of 150 ms in parallel; a global lock would need 2.4 s").isLessThan(1_200);
    }

    @Test
    void shouldHandOverToAWaitingCallerWhenTheRunningOneFails() {
        IdempotentProcessor p = processor();
        AtomicInteger calls = new AtomicInteger();
        Handler failsFirst = body -> {
            sleep(100);
            if (calls.incrementAndGet() == 1) {
                throw new IllegalStateException("first one fails");
            }
            return "ok";
        };

        List<String> outcomes = Concurrently.collect(8, t -> {
            try {
                return p.process("k", "b", failsFirst);
            } catch (IllegalStateException e) {
                return "failed";
            }
        });

        assertThat(outcomes.stream().filter(o -> o.equals("failed")).count()).isEqualTo(1);
        assertThat(outcomes.stream().filter(o -> o.equals("ok")).count()).isEqualTo(7);
        assertThat(calls).hasValue(2);
    }

    @Test
    void shouldReportAConflictEvenWhileTheFirstCallIsStillRunning() throws Exception {
        IdempotentProcessor p = processor();
        CountDownLatch started = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        Thread first = new Thread(() -> p.process("k", "A", body -> {
            started.countDown();
            try {
                release.await();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return "r";
        }));
        first.setDaemon(true);
        first.start();
        started.await();

        assertThatThrownBy(() -> p.process("k", "B", b -> "x")).isInstanceOf(KeyConflictException.class);

        release.countDown();
        first.join(5_000);
    }

    @Test
    void shouldStayExactlyOnceOver100Runs() {
        for (int run = 0; run < 100; run++) {
            IdempotentProcessor p = processor();
            AtomicInteger calls = new AtomicInteger();

            List<String> responses = Concurrently.collect(8, t -> p.process("k", "b", body -> "r" + calls.incrementAndGet()));

            assertThat(calls).as("run %d", run).hasValue(1);
            assertThat(new HashSet<>(responses)).as("run %d", run).containsExactly("r1");
        }
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CircuitBreakerApi;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.BooleanSupplier;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;
import testkit.FakeClock;

class BehaviorTest {
    private static final long OPEN_MS = 1000;
    private static final BooleanSupplier OK = () -> true;
    private static final BooleanSupplier FAIL = () -> false;

    private final FakeClock clock = new FakeClock(10_000);

    /** threshold 3, cool-down 1000 ms, 2 probe successes required. */
    private CircuitBreakerApi system() {
        return Candidate.of(() -> Solution.create(3, OPEN_MS, 2, clock));
    }

    private static void trip(CircuitBreakerApi cb) {
        for (int i = 0; i < 3; i++) {
            cb.call(FAIL);
        }
    }

    private CircuitBreakerApi halfOpen() {
        CircuitBreakerApi cb = system();
        trip(cb);
        clock.advance(OPEN_MS);
        return cb;
    }

    @Test
    void shouldStartClosedAndPassSuccessfulCalls() {
        CircuitBreakerApi cb = system();

        assertThat(cb.state()).isEqualTo("CLOSED");
        assertThat(cb.call(OK)).isEqualTo("SUCCESS");
        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldReportFailuresBelowThresholdWithoutTripping() {
        CircuitBreakerApi cb = system();

        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");
        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");

        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldTripAfterConsecutiveFailures() {
        CircuitBreakerApi cb = system();

        trip(cb);

        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void shouldResetFailureCountOnSuccess() {
        CircuitBreakerApi cb = system();

        cb.call(FAIL);
        cb.call(FAIL);
        cb.call(OK);
        cb.call(FAIL);
        cb.call(FAIL);
        assertThat(cb.state()).isEqualTo("CLOSED");

        cb.call(FAIL);
        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void shouldRejectWithoutRunningTheOperationWhileOpen() {
        CircuitBreakerApi cb = system();
        trip(cb);
        AtomicInteger runs = new AtomicInteger();

        String result = cb.call(() -> {
            runs.incrementAndGet();
            return true;
        });

        assertThat(result).isEqualTo("REJECTED");
        assertThat(runs).hasValue(0);
        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void shouldTreatAThrowingOperationAsAFailure() {
        // Java only: exceptions are failures; they must not escape and must count towards the threshold.
        CircuitBreakerApi cb = system();
        BooleanSupplier boom = () -> {
            throw new IllegalStateException("boom");
        };

        assertThat(cb.call(boom)).isEqualTo("FAILURE");
        assertThat(cb.call(boom)).isEqualTo("FAILURE");
        assertThat(cb.call(boom)).isEqualTo("FAILURE");

        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void shouldStayOpenUntilTheCoolDownHasElapsed() {
        CircuitBreakerApi cb = system();
        trip(cb);

        clock.advance(OPEN_MS - 1);
        assertThat(cb.state()).isEqualTo("OPEN");
        assertThat(cb.call(OK)).isEqualTo("REJECTED");

        clock.advance(1);
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
    }

    @Test
    void shouldCloseAfterTheRequiredNumberOfSuccessfulTrials() {
        CircuitBreakerApi cb = halfOpen();

        assertThat(cb.call(OK)).isEqualTo("SUCCESS");
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
        assertThat(cb.call(OK)).isEqualTo("SUCCESS");
        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldStartWithAFreshFailureCountAfterClosing() {
        CircuitBreakerApi cb = halfOpen();
        cb.call(OK);
        cb.call(OK);

        cb.call(FAIL);
        cb.call(FAIL);

        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldTripAgainAndRestartTheCoolDownWhenATrialFails() {
        CircuitBreakerApi cb = halfOpen();

        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");
        assertThat(cb.state()).isEqualTo("OPEN");

        clock.advance(OPEN_MS - 1);
        assertThat(cb.state()).isEqualTo("OPEN");
        clock.advance(1);
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
    }

    @Test
    void shouldRequireTheFullNumberOfTrialSuccessesAgainAfterATrialFailure() {
        CircuitBreakerApi cb = halfOpen();
        cb.call(OK);
        cb.call(FAIL);
        clock.advance(OPEN_MS);

        cb.call(OK);
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
        cb.call(OK);
        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldAllowParallelCallsWhileClosed() {
        CircuitBreakerApi cb = system();
        CountDownLatch bothInside = new CountDownLatch(2);
        BooleanSupplier meet = () -> {
            bothInside.countDown();
            try {
                return bothInside.await(10, TimeUnit.SECONDS);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                return false;
            }
        };

        List<String> results = Concurrently.collect(2, i -> cb.call(meet));

        assertThat(results).containsOnly("SUCCESS");
    }

    @Test
    void shouldRejectEveryConcurrentCallWhileOpenWithoutRunningAny() {
        CircuitBreakerApi cb = system();
        trip(cb);
        AtomicInteger runs = new AtomicInteger();

        List<String> results = Concurrently.collect(100, i -> cb.call(() -> {
            runs.incrementAndGet();
            return true;
        }));

        assertThat(results).hasSize(100).containsOnly("REJECTED");
        assertThat(runs).hasValue(0);
    }

    @Test
    void shouldTripExactlyOnceUnderConcurrentFailures() {
        CircuitBreakerApi cb = system();
        AtomicInteger runs = new AtomicInteger();

        List<String> results = Concurrently.collect(100, i -> cb.call(() -> {
            runs.incrementAndGet();
            return false;
        }));

        long failures = results.stream().filter("FAILURE"::equals).count();
        long rejected = results.stream().filter("REJECTED"::equals).count();
        assertThat(failures + rejected).isEqualTo(100);
        assertThat(failures).isGreaterThanOrEqualTo(3);
        assertThat(runs).hasValue((int) failures);
        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void shouldAdmitOnlyOneTrialAtATimeWhenHalfOpen() throws Exception {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(3, OPEN_MS, 3, clock));
        trip(cb);
        clock.advance(OPEN_MS);
        AtomicInteger started = new AtomicInteger();
        CountDownLatch release = new CountDownLatch(1);
        BooleanSupplier slow = () -> {
            started.incrementAndGet();
            try {
                release.await(20, TimeUnit.SECONDS);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            return true;
        };
        ExecutorService pool = Executors.newFixedThreadPool(20);
        try {
            List<Future<String>> futures = new ArrayList<>();
            for (int i = 0; i < 20; i++) {
                futures.add(pool.submit(() -> cb.call(slow)));
            }

            Eventually.assertTrue(
                    () -> started.get() == 1 && futures.stream().filter(Future::isDone).count() == 19,
                    10_000,
                    "one trial running, the other 19 callers refused");
            Eventually.assertStays(() -> started.get() == 1, 100, "no second trial may start");
            assertThat(cb.state()).isEqualTo("HALF_OPEN");

            release.countDown();
            List<String> results = new ArrayList<>();
            for (Future<String> f : futures) {
                results.add(f.get(10, TimeUnit.SECONDS));
            }
            assertThat(results.stream().filter("SUCCESS"::equals).count()).isEqualTo(1);
            assertThat(results.stream().filter("REJECTED"::equals).count()).isEqualTo(19);
            assertThat(cb.state()).isEqualTo("HALF_OPEN");
        } finally {
            release.countDown();
            pool.shutdownNow();
        }
    }

    @Test
    void shouldAdmitTheNextTrialOnceTheRunningOneHasFinished() throws Exception {
        CircuitBreakerApi cb = halfOpen();
        CountDownLatch inside = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        ExecutorService pool = Executors.newSingleThreadExecutor();
        try {
            Future<String> first = pool.submit(() -> cb.call(() -> {
                inside.countDown();
                try {
                    release.await(20, TimeUnit.SECONDS);
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                }
                return true;
            }));
            assertThat(inside.await(10, TimeUnit.SECONDS)).isTrue();

            assertThat(cb.call(OK)).isEqualTo("REJECTED");
            assertThat(cb.state()).isEqualTo("HALF_OPEN");

            release.countDown();
            assertThat(first.get(10, TimeUnit.SECONDS)).isEqualTo("SUCCESS");
            assertThat(cb.call(OK)).isEqualTo("SUCCESS");
            assertThat(cb.state()).isEqualTo("CLOSED");
        } finally {
            release.countDown();
            pool.shutdownNow();
        }
    }
}

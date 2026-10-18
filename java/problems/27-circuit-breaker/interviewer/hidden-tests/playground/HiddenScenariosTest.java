package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CircuitBreakerApi;
import java.util.function.BooleanSupplier;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private static final BooleanSupplier OK = () -> true;
    private static final BooleanSupplier FAIL = () -> false;

    private final FakeClock clock = new FakeClock(5_000);

    @Test
    void thresholdOfOneShouldTripOnTheFirstFailureAndOneProbeShouldClose() {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(1, 200, 1, clock));

        assertThat(cb.call(OK)).isEqualTo("SUCCESS");
        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");
        assertThat(cb.state()).isEqualTo("OPEN");

        clock.advance(200);
        assertThat(cb.call(OK)).isEqualTo("SUCCESS");
        assertThat(cb.state()).isEqualTo("CLOSED");
        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");
        assertThat(cb.state()).isEqualTo("OPEN");
    }

    @Test
    void rejectedCallsShouldNotExtendTheCoolDown() {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(2, 1000, 1, clock));
        cb.call(FAIL);
        cb.call(FAIL);

        clock.advance(600);
        assertThat(cb.call(OK)).isEqualTo("REJECTED");
        clock.advance(400);

        assertThat(cb.state()).isEqualTo("HALF_OPEN");
    }

    @Test
    void aTrialFailureShouldDiscardEarlierTrialSuccesses() {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(2, 100, 3, clock));
        cb.call(FAIL);
        cb.call(FAIL);
        clock.advance(100);
        cb.call(OK);
        cb.call(OK);
        assertThat(cb.state()).isEqualTo("HALF_OPEN");

        assertThat(cb.call(FAIL)).isEqualTo("FAILURE");
        assertThat(cb.state()).isEqualTo("OPEN");

        clock.advance(100);
        cb.call(OK);
        cb.call(OK);
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
        cb.call(OK);
        assertThat(cb.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldStayHalfOpenIndefinitelyUntilATrialDecidesAndNeedNoCallToObserveIt() {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(1, 50, 1, clock));
        cb.call(FAIL);

        clock.advance(1_000_000);

        assertThat(cb.state()).isEqualTo("HALF_OPEN");
        assertThat(cb.state()).isEqualTo("HALF_OPEN");
    }

    @Test
    void shouldKeepIndependentGuardsIndependent() {
        CircuitBreakerApi a = Candidate.of(() -> Solution.create(1, 100, 1, clock));
        CircuitBreakerApi b = Candidate.of(() -> Solution.create(1, 100, 1, clock));

        a.call(FAIL);

        assertThat(a.state()).isEqualTo("OPEN");
        assertThat(b.state()).isEqualTo("CLOSED");
        assertThat(b.call(OK)).isEqualTo("SUCCESS");
    }

    @Test
    void shouldCountOnlyConsecutiveFailuresAcrossAClosingCycle() {
        CircuitBreakerApi cb = Candidate.of(() -> Solution.create(3, 100, 1, clock));
        cb.call(FAIL);
        cb.call(FAIL);
        cb.call(OK);
        cb.call(FAIL);
        cb.call(FAIL);
        assertThat(cb.state()).isEqualTo("CLOSED");

        cb.call(FAIL);
        clock.advance(100);
        cb.call(OK);

        cb.call(FAIL);
        cb.call(FAIL);
        assertThat(cb.state()).isEqualTo("CLOSED");
    }
}

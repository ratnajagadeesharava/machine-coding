package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.TokenSystem;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final long GRACE = 60_000;
    private static final long RECALL = 120_000;

    private static final String D = TokenSystem.DINE_IN;
    private static final String T = TokenSystem.TAKEAWAY;
    private static final String V = TokenSystem.VIP;

    private final FakeClock clock = new FakeClock(1_000_000);

    private TokenSystem system() {
        TokenSystem s = Candidate.of(() -> Solution.create(clock, GRACE, RECALL));
        s.addCounter("all", Set.of(D, T, V));
        return s;
    }

    private static void issue(TokenSystem s, String type, int count) {
        for (int i = 0; i < count; i++) {
            s.issueToken("c", type, 1);
        }
    }

    @Test
    void shouldNumberTokensFromOneInIssueOrder() {
        TokenSystem s = system();

        assertThat(s.issueToken("Asha", D, 4)).isEqualTo("T-1");
        assertThat(s.issueToken("Ben", T, 1)).isEqualTo("T-2");
        assertThat(s.issueToken("Cy", V, 1)).isEqualTo("T-3");
        assertThat(s.status("T-1")).isEqualTo("WAITING");
    }

    @Test
    void shouldRejectInvalidTokenRequests() {
        TokenSystem s = system();

        assertThatThrownBy(() -> s.issueToken(" ", D, 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken(null, D, 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", D, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", D, 13)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", T, 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", V, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", "ROYAL", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.issueToken("a", null, 1)).isInstanceOf(IllegalArgumentException.class);

        // failed requests do not consume numbers
        assertThat(s.issueToken("ok", D, 1)).isEqualTo("T-1");
    }

    @Test
    void shouldAcceptPartySizeBoundariesForDineIn() {
        TokenSystem s = system();

        assertThat(s.issueToken("solo", D, 1)).isEqualTo("T-1");
        assertThat(s.issueToken("big", D, 12)).isEqualTo("T-2");
    }

    @Test
    void shouldServeVipThenTakeawayThenDineInOldestFirstWithinAType() {
        TokenSystem s = system();
        s.issueToken("a", D, 2); // T-1
        s.issueToken("b", T, 1); // T-2
        s.issueToken("c", V, 1); // T-3
        s.issueToken("d", D, 3); // T-4
        s.issueToken("e", V, 1); // T-5

        assertThat(s.queueSnapshot()).containsExactly("T-3", "T-5", "T-2", "T-1", "T-4");
        assertThat(s.callNext("all")).contains("T-3");
        assertThat(s.callNext("all")).contains("T-5");
        assertThat(s.callNext("all")).contains("T-2");
        assertThat(s.callNext("all")).contains("T-1");
        assertThat(s.callNext("all")).contains("T-4");
    }

    @Test
    void shouldMarkTheTokenCalledAndRemoveItFromTheQueue() {
        TokenSystem s = system();
        issue(s, D, 2);

        assertThat(s.callNext("all")).contains("T-1");

        assertThat(s.status("T-1")).isEqualTo("CALLED");
        assertThat(s.queueSnapshot()).containsExactly("T-2");
    }

    @Test
    void shouldReturnEmptyWhenNothingIsWaiting() {
        TokenSystem s = system();

        assertThat(s.callNext("all")).isEqualTo(Optional.empty());
        s.issueToken("a", D, 1);
        s.callNext("all");
        assertThat(s.callNext("all")).isEmpty();
    }

    @Test
    void shouldSkipTokensACounterDoesNotSupportWithoutConsumingThem() {
        TokenSystem s = system();
        s.addCounter("takeaway-only", Set.of(T));
        s.issueToken("a", D, 2); // T-1
        s.issueToken("b", T, 1); // T-2

        assertThat(s.callNext("takeaway-only")).contains("T-2");
        assertThat(s.callNext("takeaway-only")).isEmpty();
        assertThat(s.status("T-1")).isEqualTo("WAITING");
        assertThat(s.queueSnapshot()).containsExactly("T-1");
        assertThat(s.callNext("all")).contains("T-1");
    }

    @Test
    void shouldLetACounterSupportSeveralTypes() {
        TokenSystem s = system();
        s.addCounter("bar", Set.of(T, V));
        s.issueToken("a", D, 1); // T-1
        s.issueToken("b", T, 1); // T-2
        s.issueToken("c", V, 1); // T-3

        assertThat(s.callNext("bar")).contains("T-3");
        assertThat(s.callNext("bar")).contains("T-2");
        assertThat(s.callNext("bar")).isEmpty();
    }

    @Test
    void shouldRejectBadCountersAndUnknownCounters() {
        TokenSystem s = system();

        assertThatThrownBy(() -> s.addCounter("all", Set.of(D))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addCounter(" ", Set.of(D))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addCounter("x", Set.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addCounter("x", Set.of("ROYAL"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.addCounter("x", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.callNext("ghost")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.callNext("x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldMarkServedOnlyForCalledTokens() {
        TokenSystem s = system();
        issue(s, D, 2);

        assertThatThrownBy(() -> s.markServed("T-1")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.markServed("T-99")).isInstanceOf(IllegalArgumentException.class);
        s.callNext("all");
        s.markServed("T-1");
        assertThat(s.status("T-1")).isEqualTo("SERVED");
        assertThatThrownBy(() -> s.markServed("T-1")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.markNoShow("T-1")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldCancelOnlyWaitingTokens() {
        TokenSystem s = system();
        issue(s, D, 3);
        s.callNext("all"); // T-1 called

        s.cancel("T-2");

        assertThat(s.status("T-2")).isEqualTo("CANCELLED");
        assertThat(s.queueSnapshot()).containsExactly("T-3");
        assertThatThrownBy(() -> s.cancel("T-1")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.cancel("T-2")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.cancel("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.callNext("all")).contains("T-3");
    }

    @Test
    void shouldReportPositionsInTheServingOrder() {
        TokenSystem s = system();
        s.issueToken("a", D, 1); // T-1
        s.issueToken("b", V, 1); // T-2
        s.issueToken("c", T, 1); // T-3

        assertThat(s.positionInQueue("T-2")).isEqualTo(1);
        assertThat(s.positionInQueue("T-3")).isEqualTo(2);
        assertThat(s.positionInQueue("T-1")).isEqualTo(3);
        s.callNext("all");
        assertThat(s.positionInQueue("T-1")).isEqualTo(2);
        assertThatThrownBy(() -> s.positionInQueue("T-2")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.positionInQueue("T-42")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldEstimateWaitWithFiveMinutesBeforeAnythingIsServed() {
        TokenSystem s = system();
        issue(s, D, 3);

        assertThat(s.estimatedWaitMs("T-1")).isEqualTo(0);
        assertThat(s.estimatedWaitMs("T-2")).isEqualTo(300_000);
        assertThat(s.estimatedWaitMs("T-3")).isEqualTo(600_000);
        assertThatThrownBy(() -> s.estimatedWaitMs("T-9")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldEstimateWaitFromTheIntegerMeanOfServedDurations() {
        TokenSystem s = system();
        issue(s, D, 5);
        s.callNext("all"); // T-1 at +0
        clock.advance(1_000);
        s.markServed("T-1"); // took 1000
        s.callNext("all"); // T-2 called at +1000
        clock.advance(2_001);
        s.markServed("T-2"); // took 2001

        // mean = (1000 + 2001) / 2 = 1500 (integer division)
        assertThat(s.estimatedWaitMs("T-3")).isEqualTo(0);
        assertThat(s.estimatedWaitMs("T-4")).isEqualTo(1_500);
        assertThat(s.estimatedWaitMs("T-5")).isEqualTo(3_000);
    }

    @Test
    void shouldNotCountNoShowsOrCancellationsInTheAverage() {
        TokenSystem s = system();
        issue(s, D, 5);
        s.callNext("all"); // T-1
        clock.advance(4_000);
        s.markServed("T-1"); // 4000
        s.callNext("all"); // T-2
        clock.advance(90_000);
        s.markNoShow("T-2");
        s.cancel("T-3");

        assertThat(s.estimatedWaitMs("T-5")).isEqualTo(4_000);
    }

    @Test
    void shouldPromoteATokenPassedOverByFiveLaterTokens() {
        TokenSystem s = system();
        s.issueToken("walk-in", D, 2); // T-1
        issue(s, V, 6); // T-2 .. T-7

        // each served VIP (T-2..T-5) is later than T-1 and passes it over: count 1, 2, 3, 4
        assertThat(s.callNext("all")).contains("T-2");
        assertThat(s.callNext("all")).contains("T-3");
        assertThat(s.callNext("all")).contains("T-4");
        assertThat(s.callNext("all")).contains("T-5");
        assertThat(s.queueSnapshot()).containsExactly("T-6", "T-7", "T-1");

        // T-6 is the 5th: T-1 is promoted ahead of the remaining VIP
        assertThat(s.callNext("all")).contains("T-6");
        assertThat(s.queueSnapshot()).containsExactly("T-1", "T-7");
        assertThat(s.positionInQueue("T-1")).isEqualTo(1);
        assertThat(s.callNext("all")).contains("T-1");
        assertThat(s.callNext("all")).contains("T-7");
    }

    @Test
    void shouldNotCountTokensIssuedBeforeTheWaitingOneAsPassingItOver() {
        TokenSystem s = system();
        issue(s, V, 6); // T-1 .. T-6
        s.issueToken("late", D, 2); // T-7
        for (int i = 0; i < 6; i++) {
            s.callNext("all");
        }

        s.issueToken("vip", V, 1); // T-8

        // six earlier tokens were served before T-7 but none was issued after it
        assertThat(s.queueSnapshot()).containsExactly("T-8", "T-7");
    }

    @Test
    void shouldCountPassOversMadeByCountersThatCannotServeTheWaitingToken() {
        TokenSystem s = system();
        s.addCounter("takeaway-only", Set.of(T));
        s.issueToken("a", D, 1); // T-1
        issue(s, T, 6); // T-2 .. T-7
        for (int i = 0; i < 4; i++) {
            s.callNext("takeaway-only"); // T-2..T-5
        }
        assertThat(s.queueSnapshot()).containsExactly("T-6", "T-7", "T-1");

        s.callNext("takeaway-only"); // T-6, the fifth

        assertThat(s.queueSnapshot()).containsExactly("T-1", "T-7");
        assertThat(s.callNext("all")).contains("T-1");
    }

    @Test
    void shouldPromoteSeveralStarvedTokensOldestFirst() {
        TokenSystem s = system();
        s.issueToken("a", D, 1); // T-1
        s.issueToken("b", D, 1); // T-2
        issue(s, V, 6); // T-3 .. T-8
        for (int i = 0; i < 5; i++) {
            s.callNext("all"); // T-3..T-7
        }

        assertThat(s.queueSnapshot()).containsExactly("T-1", "T-2", "T-8");
    }

    @Test
    void shouldExpireACalledTokenOnlyAfterTheGraceTime() {
        TokenSystem s = system();
        issue(s, D, 2);
        s.callNext("all"); // T-1 at t0

        clock.advance(GRACE);
        assertThat(s.expireOverdue()).isEmpty();
        assertThat(s.status("T-1")).isEqualTo("CALLED");

        clock.advance(1);
        assertThat(s.expireOverdue()).containsExactly("T-1");
        assertThat(s.status("T-1")).isEqualTo("NO_SHOW");
        assertThat(s.status("T-2")).isEqualTo("WAITING");
        assertThat(s.expireOverdue()).isEmpty();
        assertThatThrownBy(() -> s.markServed("T-1")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldStillServeAnOverdueTokenUntilExpiryIsRun() {
        TokenSystem s = system();
        issue(s, D, 1);
        s.callNext("all");
        clock.advance(10 * GRACE);

        s.markServed("T-1");

        assertThat(s.status("T-1")).isEqualTo("SERVED");
        assertThat(s.expireOverdue()).isEmpty();
    }

    @Test
    void shouldRecallANoShowToTheFrontOfItsPriorityClass() {
        TokenSystem s = system();
        issue(s, D, 4); // T-1 .. T-4
        s.callNext("all"); // T-1
        s.callNext("all"); // T-2
        s.markNoShow("T-1");
        s.markNoShow("T-2");
        s.issueToken("vip", V, 1); // T-5
        assertThat(s.queueSnapshot()).containsExactly("T-5", "T-3", "T-4");

        s.recall("T-1");
        assertThat(s.status("T-1")).isEqualTo("WAITING");
        assertThat(s.queueSnapshot()).containsExactly("T-5", "T-1", "T-3", "T-4");

        // a later recall goes in front of an earlier one
        s.recall("T-2");
        assertThat(s.queueSnapshot()).containsExactly("T-5", "T-2", "T-1", "T-3", "T-4");
    }

    @Test
    void shouldAllowARecallOnlyOnceAndOnlyForNoShows() {
        TokenSystem s = system();
        issue(s, D, 2);
        assertThatThrownBy(() -> s.recall("T-1")).isInstanceOf(IllegalStateException.class);
        s.callNext("all");
        assertThatThrownBy(() -> s.recall("T-1")).isInstanceOf(IllegalStateException.class);
        s.markNoShow("T-1");

        s.recall("T-1");
        assertThatThrownBy(() -> s.recall("T-1")).isInstanceOf(IllegalStateException.class);

        assertThat(s.callNext("all")).contains("T-1");
        s.markNoShow("T-1");
        assertThatThrownBy(() -> s.recall("T-1")).isInstanceOf(IllegalStateException.class);
        assertThat(s.status("T-1")).isEqualTo("NO_SHOW");
        assertThatThrownBy(() -> s.recall("T-77")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRefuseARecallAfterTheWindowAndExpireTheToken() {
        TokenSystem s = system();
        issue(s, D, 2);
        s.callNext("all"); // T-1
        s.callNext("all"); // T-2
        s.markNoShow("T-1");
        s.markNoShow("T-2");
        clock.advance(RECALL);
        s.recall("T-1"); // exactly at the window edge is still allowed

        clock.advance(1);
        assertThatThrownBy(() -> s.recall("T-2")).isInstanceOf(IllegalStateException.class);
        assertThat(s.status("T-2")).isEqualTo("NO_SHOW");
        s.expireOverdue();
        assertThat(s.status("T-2")).isEqualTo("EXPIRED");
        assertThat(s.status("T-1")).isEqualTo("WAITING");
    }

    @Test
    void shouldReportStatistics() {
        TokenSystem s = system();
        assertThat(s.stats()).isEqualTo("0:0:0:0:0");
        issue(s, D, 5);
        assertThat(s.stats()).isEqualTo("5:0:0:0:5");
        s.callNext("all"); // T-1
        s.callNext("all"); // T-2
        s.callNext("all"); // T-3
        s.markServed("T-1");
        s.markNoShow("T-2");
        s.cancel("T-5");

        // T-3 is CALLED and belongs to no counter; T-4 is waiting
        assertThat(s.stats()).isEqualTo("5:1:1:1:1");
    }

    @Test
    void shouldStartANewDayWithoutLosingCounters() {
        TokenSystem s = system();
        issue(s, D, 3);
        s.callNext("all");
        clock.advance(9_000);
        s.markServed("T-1");

        s.resetDay();

        assertThat(s.stats()).isEqualTo("0:0:0:0:0");
        assertThat(s.queueSnapshot()).isEmpty();
        assertThatThrownBy(() -> s.status("T-1")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.issueToken("new", D, 1)).isEqualTo("T-1");
        s.issueToken("new2", D, 1);
        assertThat(s.estimatedWaitMs("T-2")).isEqualTo(300_000); // average forgotten
        assertThat(s.callNext("all")).contains("T-1");
    }

    @Test
    void shouldIgnoreLaterChangesToTheCallersCounterSet() {
        TokenSystem s = system();
        Set<String> mine = new HashSet<>(Set.of(T));
        s.addCounter("mine", mine);
        mine.add(D);
        s.issueToken("a", D, 1);

        assertThat(s.callNext("mine")).isEmpty();
    }

    @Test
    void shouldIssueUniqueGapFreeNumbersFromSixteenThreads() {
        TokenSystem s = system();

        List<List<String>> perThread = Concurrently.collect(16, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < 100; i++) {
                mine.add(s.issueToken("t" + t, i % 3 == 0 ? D : i % 3 == 1 ? T : V, i % 3 == 0 ? 2 : 1));
            }
            return mine;
        });

        Set<String> all = new HashSet<>();
        perThread.forEach(all::addAll);
        assertThat(all).hasSize(1_600);
        for (int n = 1; n <= 1_600; n++) {
            assertThat(all).contains("T-" + n);
        }
        assertThat(s.stats()).isEqualTo("1600:0:0:0:1600");
        assertThat(s.queueSnapshot()).hasSize(1_600);
    }

    @Test
    void shouldNeverCallTheSameTokenTwiceUnderConcurrentCounters() {
        TokenSystem s = system();
        for (int c = 0; c < 8; c++) {
            s.addCounter("c" + c, Set.of(D, T, V));
        }
        issue(s, D, 400);
        issue(s, V, 400);

        List<List<String>> got = Concurrently.collect(8, c -> {
            List<String> mine = new ArrayList<>();
            while (true) {
                Optional<String> t = s.callNext("c" + c);
                if (t.isEmpty()) {
                    return mine;
                }
                mine.add(t.get());
            }
        });

        List<String> all = new ArrayList<>();
        got.forEach(all::addAll);
        assertThat(all).hasSize(800).doesNotHaveDuplicates();
        assertThat(s.queueSnapshot()).isEmpty();
    }
}

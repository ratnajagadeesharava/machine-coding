package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.TokenSystem;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    private static final long GRACE = 60_000;
    private static final long RECALL = 120_000;
    private static final String D = TokenSystem.DINE_IN;
    private static final String T = TokenSystem.TAKEAWAY;
    private static final String V = TokenSystem.VIP;

    private final FakeClock clock = new FakeClock(5_000_000);

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
    void shouldPromoteStarvedTokensByAgeNotByPriority() {
        TokenSystem s = system();
        s.issueToken("a", D, 2); // T-1
        s.issueToken("b", T, 1); // T-2
        issue(s, V, 7); // T-3 .. T-9
        for (int i = 0; i < 5; i++) {
            s.callNext("all"); // T-3..T-7 pass over both T-1 and T-2
        }

        // both promoted: oldest first even though T-2 has the higher priority; then the remaining VIPs
        assertThat(s.queueSnapshot()).containsExactly("T-1", "T-2", "T-8", "T-9");
    }

    @Test
    void shouldResetThePassedOverCountWhenARecalledTokenReturns() {
        TokenSystem s = system();
        s.issueToken("a", D, 1); // T-1
        s.issueToken("b", D, 1); // T-2
        issue(s, V, 6); // T-3 .. T-8
        for (int i = 0; i < 5; i++) {
            s.callNext("all"); // T-3..T-7 : T-1 and T-2 promoted
        }
        assertThat(s.callNext("all")).contains("T-1");
        s.markNoShow("T-1");

        s.recall("T-1");

        // T-2 is still promoted; the recalled T-1 starts again from zero at the front of the DINE_IN class
        assertThat(s.queueSnapshot()).containsExactly("T-2", "T-8", "T-1");
        assertThat(s.callNext("all")).contains("T-2");
        assertThat(s.callNext("all")).contains("T-8");
        assertThat(s.callNext("all")).contains("T-1");
    }

    @Test
    void shouldPlaceARecalledTokenAheadOfOlderTokensOfItsClassButBehindHigherClasses() {
        TokenSystem s = system();
        s.issueToken("a", T, 1); // T-1
        s.issueToken("b", T, 1); // T-2
        s.issueToken("c", T, 1); // T-3
        s.issueToken("d", D, 1); // T-4
        assertThat(s.callNext("all")).contains("T-1");
        assertThat(s.callNext("all")).contains("T-2");
        s.markNoShow("T-2");
        s.issueToken("e", V, 1); // T-5

        s.recall("T-2");

        // T-2 is newer than nothing here but still jumps T-3; the VIP stays first, the DINE_IN stays last
        assertThat(s.queueSnapshot()).containsExactly("T-5", "T-2", "T-3", "T-4");
    }

    @Test
    void shouldCountAPassOverWhenALaterTokenIsServedWhileARecalledTokenWaits() {
        TokenSystem s = system();
        s.issueToken("a", D, 1); // T-1
        assertThat(s.callNext("all")).contains("T-1");
        s.markNoShow("T-1");
        issue(s, V, 6); // T-2 .. T-7
        s.recall("T-1");
        for (int i = 0; i < 4; i++) {
            s.callNext("all"); // T-2..T-5
        }
        assertThat(s.queueSnapshot()).containsExactly("T-6", "T-7", "T-1");

        s.callNext("all"); // T-6: fifth later token since the recall

        assertThat(s.queueSnapshot()).containsExactly("T-1", "T-7");
    }

    @Test
    void shouldExpireOnlyTheOverdueTokensAndReportThemInTokenOrder() {
        TokenSystem s = system();
        issue(s, D, 4);
        s.callNext("all"); // T-1 at 0
        clock.advance(30_000);
        s.callNext("all"); // T-2 at 30000
        s.callNext("all"); // T-3 at 30000
        clock.advance(30_001); // T-1 is 60001 old, the others 30001

        assertThat(s.expireOverdue()).containsExactly("T-1");

        clock.advance(30_000); // T-2, T-3 are now 60001 old
        s.markServed("T-3");
        assertThat(s.expireOverdue()).containsExactly("T-2");
        assertThat(s.stats()).isEqualTo("4:1:2:0:1");
    }

    @Test
    void shouldMeasureTheRecallWindowFromTheMomentOfTheNoShow() {
        TokenSystem s = system();
        issue(s, D, 1);
        s.callNext("all");
        clock.advance(GRACE + 5_000);
        s.expireOverdue(); // becomes NO_SHOW now, not at the call time
        clock.advance(RECALL);

        s.recall("T-1");

        assertThat(s.status("T-1")).isEqualTo("WAITING");
    }

    @Test
    void shouldExpireNoShowsOnlyAfterTheRecallWindow() {
        TokenSystem s = system();
        issue(s, D, 2);
        s.callNext("all");
        s.markNoShow("T-1");
        clock.advance(RECALL);
        s.expireOverdue();
        assertThat(s.status("T-1")).isEqualTo("NO_SHOW");

        clock.advance(1);
        assertThat(s.expireOverdue()).isEmpty();
        assertThat(s.status("T-1")).isEqualTo("EXPIRED");
        assertThat(s.stats()).isEqualTo("2:0:1:0:1");
        assertThatThrownBy(() -> s.recall("T-1")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldFloorTheAverageServiceTime() {
        TokenSystem s = system();
        issue(s, D, 5);
        long[] took = {1, 1, 2};
        for (int i = 0; i < 3; i++) {
            s.callNext("all");
            clock.advance(took[i]);
            s.markServed("T-" + (i + 1));
        }

        // (1 + 1 + 2) / 3 = 1 (floor)
        assertThat(s.estimatedWaitMs("T-5")).isEqualTo(1);
    }

    @Test
    void shouldKeepPositionsConsistentWithSnapshotAndNextCallAcrossARandomRun() {
        TokenSystem s = system();
        Random rnd = new Random(42);
        String[] types = {D, T, V};
        Set<String> recalled = new HashSet<>();
        for (int step = 0; step < 300; step++) {
            int op = rnd.nextInt(10);
            if (op < 5) {
                String type = types[rnd.nextInt(3)];
                s.issueToken("c", type, type.equals(D) ? 1 + rnd.nextInt(12) : 1);
            } else if (op < 8) {
                List<String> before = s.queueSnapshot();
                Optional<String> got = s.callNext("all");
                assertThat(got.isPresent()).isEqualTo(!before.isEmpty());
                got.ifPresent(t -> {
                    assertThat(t).isEqualTo(before.get(0));
                    if (rnd.nextBoolean()) {
                        clock.advance(rnd.nextInt(1_000));
                        s.markServed(t);
                    } else {
                        s.markNoShow(t);
                        if (rnd.nextBoolean() && recalled.add(t)) {
                            s.recall(t);
                        }
                    }
                });
            } else {
                List<String> q = s.queueSnapshot();
                if (!q.isEmpty()) {
                    s.cancel(q.get(rnd.nextInt(q.size())));
                }
            }
            List<String> snap = s.queueSnapshot();
            for (int i = 0; i < snap.size(); i++) {
                assertThat(s.positionInQueue(snap.get(i))).isEqualTo(i + 1);
            }
        }
    }

    @Test
    void shouldKeepStatisticsConsistentUnderConcurrentWork() {
        TokenSystem s = system();
        for (int c = 0; c < 4; c++) {
            s.addCounter("c" + c, Set.of(D, T, V));
        }
        AtomicInteger cancelled = new AtomicInteger();
        List<List<String>> served = Concurrently.collect(16, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < 100; i++) {
                String tok = s.issueToken("n" + t, i % 2 == 0 ? D : V, 1);
                if (i % 10 == 0) {
                    try {
                        s.cancel(tok); // another thread's counter may already have called it
                        cancelled.incrementAndGet();
                    } catch (IllegalStateException e) {
                        // lost the race: the token is CALLED already; this thread serves nothing for it
                    }
                    continue;
                }
                s.callNext("c" + (t % 4)).ifPresent(called -> {
                    s.markServed(called);
                    mine.add(called);
                });
            }
            return mine;
        });

        Set<String> uniq = new HashSet<>();
        int total = 0;
        for (List<String> l : served) {
            uniq.addAll(l);
            total += l.size();
        }
        assertThat(uniq).hasSize(total);
        String[] parts = s.stats().split(":");
        assertThat(parts[0]).isEqualTo("1600");
        assertThat(Integer.parseInt(parts[1])).isEqualTo(total);
        assertThat(parts[3]).isEqualTo(String.valueOf(cancelled.get()));
        int waiting = Integer.parseInt(parts[4]);
        assertThat(s.queueSnapshot()).hasSize(waiting);
        assertThat(total + cancelled.get() + waiting).isEqualTo(1_600);
    }
}

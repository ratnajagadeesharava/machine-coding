package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.LogService;
import contract.Sink;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    /** Recording sink whose outcome the test controls. */
    static final class FakeSink implements Sink {
        final List<String> lines = Collections.synchronizedList(new ArrayList<>());
        final AtomicBoolean failing = new AtomicBoolean();
        final AtomicBoolean throwing = new AtomicBoolean();

        @Override
        public boolean write(String line) {
            if (throwing.get()) {
                throw new IllegalStateException("disk exploded");
            }
            if (failing.get()) {
                return false;
            }
            lines.add(line);
            return true;
        }
    }

    private final FakeSink file = new FakeSink();
    private final FakeSink tool = new FakeSink();
    private LogService svc;

    private LogService svc() {
        return svc = Candidate.of(() -> Solution.create(file, tool, 8));
    }

    @AfterEach
    void stop() {
        if (svc != null) {
            svc.close();
        }
    }

    private static long seqOf(String line) {
        return Long.parseLong(line.split("\\|", 5)[0]);
    }

    private static List<Long> seqs(List<String> lines) {
        return lines.stream().map(BehaviorTest::seqOf).toList();
    }

    @Test
    void shouldStartAtOneAndIncreaseByOne() {
        LogService s = svc();

        assertThat(s.log("AUTH", "FILE", "INFO", "login")).isEqualTo(1);
        assertThat(s.log("PAYMENTS", "TOOL", "WARN", "retry")).isEqualTo(2);
        assertThat(s.log("SEARCH", "FILE", "DEBUG", "query")).isEqualTo(3);
    }

    @Test
    void shouldHaveWrittenTheLineToTheSinkBeforeAcknowledging() {
        LogService s = svc();

        long seq = s.log("AUTH", "FILE", "WARN", "bad password");

        assertThat(seq).isEqualTo(1);
        assertThat(file.lines).containsExactly("1|AUTH|FILE|WARN|bad password");
    }

    @Test
    void shouldRouteEachLogToItsOwnDestinationOnly() {
        LogService s = svc();
        s.log("AUTH", "FILE", "INFO", "a");
        s.log("AUTH", "TOOL", "INFO", "b");
        s.log("SEARCH", "FILE", "INFO", "c");

        assertThat(file.lines).containsExactly("1|AUTH|FILE|INFO|a", "3|SEARCH|FILE|INFO|c");
        assertThat(tool.lines).containsExactly("2|AUTH|TOOL|INFO|b");
    }

    @Test
    void shouldAcceptEveryFixedModuleAndLevel() {
        LogService s = svc();
        long n = 0;
        for (String m : List.of("AUTH", "PAYMENTS", "SEARCH")) {
            for (String l : List.of("DEBUG", "INFO", "WARN", "ERROR")) {
                assertThat(s.log(m, "FILE", l, "x")).isEqualTo(++n);
            }
        }
    }

    @Test
    void shouldKeepTheActivityTextVerbatim() {
        LogService s = svc();

        s.log("AUTH", "FILE", "INFO", "  a|b | c  ");

        assertThat(s.query(null, null, null, null, null)).containsExactly("1|AUTH|FILE|INFO|  a|b | c  ");
    }

    @Test
    void shouldRejectAnUnknownModule() {
        LogService s = svc();

        assertThatThrownBy(() -> s.log("BILLING", "FILE", "INFO", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.log("auth", "FILE", "INFO", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.log(null, "FILE", "INFO", "x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectUnknownDestinationLevelAndBlankActivity() {
        LogService s = svc();

        assertThatThrownBy(() -> s.log("AUTH", "DISK", "INFO", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.log("AUTH", "FILE", "TRACE", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.log("AUTH", "FILE", "INFO", " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.log("AUTH", "FILE", "INFO", null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotConsumeASequenceNumberOrWriteAnythingForInvalidInput() {
        LogService s = svc();
        assertThatThrownBy(() -> s.log("BILLING", "FILE", "INFO", "x")).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.log("AUTH", "FILE", "INFO", "first")).isEqualTo(1);
        assertThat(file.lines).hasSize(1);
        assertThat(tool.lines).isEmpty();
    }

    @Test
    void shouldReportFailureWhenTheSinkWriteReturnsFalse() {
        LogService s = svc();
        file.failing.set(true);

        assertThat(s.log("AUTH", "FILE", "ERROR", "boom")).isEqualTo(-1);
    }

    @Test
    void shouldReportFailureWhenTheSinkThrows() {
        LogService s = svc();
        tool.throwing.set(true);

        assertThat(s.log("AUTH", "TOOL", "ERROR", "boom")).isEqualTo(-1);
        assertThat(s.log("AUTH", "FILE", "ERROR", "still alive")).isEqualTo(1);
    }

    @Test
    void shouldNotConsumeASequenceNumberWhenTheWriteFails() {
        LogService s = svc();
        assertThat(s.log("AUTH", "FILE", "INFO", "one")).isEqualTo(1);
        file.failing.set(true);
        assertThat(s.log("AUTH", "FILE", "INFO", "lost")).isEqualTo(-1);
        assertThat(s.log("AUTH", "FILE", "INFO", "lost too")).isEqualTo(-1);
        file.failing.set(false);

        assertThat(s.log("AUTH", "FILE", "INFO", "two")).isEqualTo(2);
        assertThat(seqs(s.query(null, null, null, null, null))).containsExactly(1L, 2L);
    }

    @Test
    void shouldNotStoreALogWhoseWriteFailed() {
        LogService s = svc();
        tool.failing.set(true);
        s.log("PAYMENTS", "TOOL", "ERROR", "never stored");
        s.log("PAYMENTS", "FILE", "ERROR", "stored");

        assertThat(s.query(null, null, null, null, null)).containsExactly("1|PAYMENTS|FILE|ERROR|stored");
        assertThat(s.query("TOOL", null, null, null, null)).isEmpty();
    }

    @Test
    void shouldFailOneDestinationWithoutAffectingTheOther() {
        LogService s = svc();
        file.failing.set(true);

        assertThat(s.log("AUTH", "TOOL", "INFO", "ok")).isEqualTo(1);
        assertThat(s.log("AUTH", "FILE", "INFO", "bad")).isEqualTo(-1);
        assertThat(s.log("AUTH", "TOOL", "INFO", "ok again")).isEqualTo(2);
    }

    @Test
    void shouldReturnEverythingInSequenceOrderWithoutFilters() {
        LogService s = svc();
        s.log("AUTH", "FILE", "INFO", "a");
        s.log("SEARCH", "TOOL", "WARN", "b");
        s.log("PAYMENTS", "FILE", "ERROR", "c");

        assertThat(s.query(null, null, null, null, null))
                .containsExactly("1|AUTH|FILE|INFO|a", "2|SEARCH|TOOL|WARN|b", "3|PAYMENTS|FILE|ERROR|c");
    }

    @Test
    void shouldReturnAnEmptyListWhenNothingMatches() {
        LogService s = svc();

        assertThat(s.query(null, null, null, null, null)).isEmpty();
        s.log("AUTH", "FILE", "INFO", "a");
        assertThat(s.query("TOOL", null, null, null, null)).isEmpty();
        assertThat(s.query(null, null, null, 5L, 9L)).isEmpty();
    }

    @Test
    void shouldFilterByDestinationAndByModule() {
        LogService s = svc();
        s.log("AUTH", "FILE", "INFO", "a");
        s.log("AUTH", "TOOL", "INFO", "b");
        s.log("SEARCH", "FILE", "INFO", "c");

        assertThat(seqs(s.query("FILE", null, null, null, null))).containsExactly(1L, 3L);
        assertThat(seqs(s.query(null, "AUTH", null, null, null))).containsExactly(1L, 2L);
        assertThat(seqs(s.query("FILE", "SEARCH", null, null, null))).containsExactly(3L);
    }

    @Test
    void shouldTreatTheLevelFilterAsAMinimumLevel() {
        LogService s = svc();
        s.log("AUTH", "FILE", "DEBUG", "d");
        s.log("AUTH", "FILE", "INFO", "i");
        s.log("AUTH", "FILE", "WARN", "w");
        s.log("AUTH", "FILE", "ERROR", "e");

        assertThat(seqs(s.query(null, null, "DEBUG", null, null))).containsExactly(1L, 2L, 3L, 4L);
        assertThat(seqs(s.query(null, null, "WARN", null, null))).containsExactly(3L, 4L);
        assertThat(seqs(s.query(null, null, "ERROR", null, null))).containsExactly(4L);
    }

    @Test
    void shouldApplyAnInclusiveSequenceRange() {
        LogService s = svc();
        for (int i = 0; i < 6; i++) {
            s.log("AUTH", "FILE", "INFO", "m" + i);
        }

        assertThat(seqs(s.query(null, null, null, 2L, 4L))).containsExactly(2L, 3L, 4L);
        assertThat(seqs(s.query(null, null, null, 5L, null))).containsExactly(5L, 6L);
        assertThat(seqs(s.query(null, null, null, null, 2L))).containsExactly(1L, 2L);
        assertThat(seqs(s.query(null, null, null, 3L, 3L))).containsExactly(3L);
    }

    @Test
    void shouldCombineAllFilters() {
        LogService s = svc();
        s.log("AUTH", "FILE", "ERROR", "1");
        s.log("PAYMENTS", "FILE", "ERROR", "2");
        s.log("PAYMENTS", "TOOL", "ERROR", "3");
        s.log("PAYMENTS", "FILE", "INFO", "4");
        s.log("PAYMENTS", "FILE", "WARN", "5");
        s.log("PAYMENTS", "FILE", "ERROR", "6");

        assertThat(seqs(s.query("FILE", "PAYMENTS", "WARN", 2L, 5L))).containsExactly(2L, 5L);
    }

    @Test
    void shouldRejectInvalidFiltersAndAReversedRange() {
        LogService s = svc();

        assertThatThrownBy(() -> s.query("DISK", null, null, null, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.query(null, "BILLING", null, null, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.query(null, null, "TRACE", null, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.query(null, null, null, 5L, 4L)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReturnASnapshotTheCallerMayModify() {
        LogService s = svc();
        s.log("AUTH", "FILE", "INFO", "a");
        List<String> first = s.query(null, null, null, null, null);
        s.log("AUTH", "FILE", "INFO", "b");

        assertThat(first).hasSize(1);
        first.clear();
        assertThat(s.query(null, null, null, null, null)).hasSize(2);
    }

    @Test
    void shouldRejectLogsAfterCloseButKeepAnsweringQueries() {
        LogService s = svc();
        s.log("AUTH", "FILE", "INFO", "a");

        s.close();
        s.close();

        assertThatThrownBy(() -> s.log("AUTH", "FILE", "INFO", "late")).isInstanceOf(IllegalStateException.class);
        assertThat(s.query(null, null, null, null, null)).hasSize(1);
        assertThat(file.lines).hasSize(1);
    }

    @Test
    void shouldGiveEveryConcurrentLogAUniqueGapFreeNumberAndLoseNothing() {
        LogService s = svc();
        String[] modules = {"AUTH", "PAYMENTS", "SEARCH"};
        String[] dests = {"FILE", "TOOL"};

        List<List<Long>> perThread = Concurrently.collect(16, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 500; i++) {
                mine.add(s.log(modules[(t + i) % 3], dests[i % 2], "INFO", "t" + t + "-" + i));
            }
            return mine;
        });

        Set<Long> all = new HashSet<>();
        perThread.forEach(all::addAll);
        assertThat(all).hasSize(8_000);
        assertThat(all).allMatch(n -> n >= 1 && n <= 8_000);
        assertThat(file.lines.size() + tool.lines.size()).isEqualTo(8_000);
        assertThat(seqs(s.query(null, null, null, null, null))).hasSize(8_000).isSorted();
    }

    @Test
    void shouldDeliverLinesToEachSinkInIncreasingSequenceOrderUnderConcurrency() {
        LogService s = svc();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 300; i++) {
                s.log("SEARCH", i % 2 == 0 ? "FILE" : "TOOL", "DEBUG", "x");
            }
        });

        assertThat(seqs(file.lines)).isSorted().doesNotHaveDuplicates();
        assertThat(seqs(tool.lines)).isSorted().doesNotHaveDuplicates();
    }

    @Test
    void shouldStayGapFreeOverAcknowledgedLogsWhenWritesFailRandomlyUnderConcurrency() {
        AtomicInteger writes = new AtomicInteger();
        Sink flaky = line -> writes.incrementAndGet() % 3 != 0 && file.write(line);
        LogService sv = svc = Candidate.of(() -> Solution.create(flaky, tool, 4));

        List<List<Long>> results = Concurrently.collect(8, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 200; i++) {
                mine.add(sv.log("AUTH", "FILE", "INFO", "x"));
            }
            return mine;
        });

        List<Long> acked = new ArrayList<>();
        results.forEach(r -> r.stream().filter(n -> n > 0).forEach(acked::add));
        Collections.sort(acked);
        assertThat(acked).isNotEmpty();
        for (int i = 0; i < acked.size(); i++) {
            assertThat(acked.get(i)).isEqualTo(i + 1L);
        }
        assertThat(sv.query(null, null, null, null, null)).hasSize(acked.size());
        assertThat(file.lines).hasSize(acked.size());
    }
}

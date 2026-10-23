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
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final String[] MODULES = {"AUTH", "PAYMENTS", "SEARCH"};
    private static final String[] LEVELS = {"DEBUG", "INFO", "WARN", "ERROR"};

    private static final class Rec implements Sink {
        final List<String> lines = Collections.synchronizedList(new ArrayList<>());

        @Override
        public boolean write(String line) {
            lines.add(line);
            return true;
        }
    }

    private static long seqOf(String line) {
        return Long.parseLong(line.split("\\|", 5)[0]);
    }

    @Test
    void shouldLoseNothingWith32ThreadsThroughATinyBuffer() {
        Rec file = new Rec();
        Rec tool = new Rec();
        LogService s = Candidate.of(() -> Solution.create(file, tool, 1));

        List<List<Long>> res = Concurrently.collect(32, 60_000, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 1_000; i++) {
                mine.add(s.log(MODULES[i % 3], i % 2 == 0 ? "FILE" : "TOOL", LEVELS[i % 4], "t" + t));
            }
            return mine;
        });
        s.close();

        Set<Long> all = new HashSet<>();
        res.forEach(all::addAll);
        assertThat(all).hasSize(32_000);
        assertThat(all.stream().mapToLong(Long::longValue).min().getAsLong()).isEqualTo(1);
        assertThat(all.stream().mapToLong(Long::longValue).max().getAsLong()).isEqualTo(32_000);
        assertThat(file.lines.size() + tool.lines.size()).isEqualTo(32_000);
        assertThat(s.query(null, null, null, null, null)).hasSize(32_000);
    }

    @Test
    void shouldMakeAnAcknowledgedLogImmediatelyVisibleToItsOwnProducer() {
        LogService s = Candidate.of(() -> Solution.create(new Rec(), new Rec(), 4));

        Concurrently.run(8, t -> {
            for (int i = 0; i < 500; i++) {
                long seq = s.log(MODULES[t % 3], "FILE", "WARN", "t" + t + "-" + i);
                List<String> one = s.query(null, null, null, seq, seq);
                assertThat(one).hasSize(1);
                assertThat(one.get(0)).isEqualTo(seq + "|" + MODULES[t % 3] + "|FILE|WARN|t" + t + "-" + i);
            }
        });
        s.close();
    }

    @Test
    void shouldReturnOrderedDuplicateFreeSnapshotsWhileProducersAreRunning() {
        LogService s = Candidate.of(() -> Solution.create(new Rec(), new Rec(), 8));

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 300; i++) {
                    List<String> snap = s.query(null, null, null, null, null);
                    long prev = 0;
                    for (String line : snap) {
                        long q = seqOf(line);
                        assertThat(q).isGreaterThan(prev);
                        prev = q;
                    }
                    assertThat(prev).isEqualTo(snap.size());
                }
                return;
            }
            for (int i = 0; i < 400; i++) {
                s.log("AUTH", i % 2 == 0 ? "FILE" : "TOOL", "INFO", "x");
            }
        });
        s.close();
    }

    @Test
    void shouldKeepAcknowledgedNumbersGapFreeWhenOneDestinationFailsIntermittently() {
        Rec file = new Rec();
        AtomicInteger toolWrites = new AtomicInteger();
        Rec toolRec = new Rec();
        Sink flakyTool = line -> toolWrites.incrementAndGet() % 4 == 0 ? false : toolRec.write(line);
        LogService s = Candidate.of(() -> Solution.create(file, flakyTool, 2));

        List<List<Long>> res = Concurrently.collect(12, 60_000, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 400; i++) {
                mine.add(s.log("PAYMENTS", i % 2 == 0 ? "FILE" : "TOOL", "ERROR", "x"));
            }
            return mine;
        });
        s.close();

        List<Long> acked = new ArrayList<>();
        long failures = 0;
        for (List<Long> r : res) {
            for (long n : r) {
                if (n == -1) {
                    failures++;
                } else {
                    acked.add(n);
                }
            }
        }
        Collections.sort(acked);
        for (int i = 0; i < acked.size(); i++) {
            assertThat(acked.get(i)).isEqualTo(i + 1L);
        }
        assertThat(failures).isEqualTo(toolWrites.get() / 4);
        assertThat(file.lines.size() + toolRec.lines.size()).isEqualTo(acked.size());
        assertThat(s.query(null, null, null, null, null)).hasSize(acked.size());
    }

    @Test
    void shouldApplyBackPressureWithASlowSinkAndStillFinishEverything() throws Exception {
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        Rec rec = new Rec();
        Sink slow = line -> {
            entered.countDown();
            try {
                if (!release.await(30, TimeUnit.SECONDS)) {
                    return false;
                }
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                return false;
            }
            return rec.write(line);
        };
        LogService s = Candidate.of(() -> Solution.create(slow, new Rec(), 1));

        Thread releaser = new Thread(() -> {
            try {
                entered.await();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            release.countDown();
        });
        releaser.start();

        List<List<Long>> res = Concurrently.collect(10, 60_000, t -> {
            List<Long> mine = new ArrayList<>();
            for (int i = 0; i < 20; i++) {
                mine.add(s.log("SEARCH", "FILE", "INFO", "x"));
            }
            return mine;
        });
        releaser.join();
        s.close();

        Set<Long> all = new HashSet<>();
        res.forEach(all::addAll);
        assertThat(all).hasSize(200).doesNotContain(-1L);
        assertThat(rec.lines).hasSize(200);
    }

    @Test
    void shouldAccountForEveryLogWhenClosedWhileProducersAreRunning() {
        Rec file = new Rec();
        Rec tool = new Rec();
        LogService s = Candidate.of(() -> Solution.create(file, tool, 4));
        AtomicInteger acked = new AtomicInteger();

        Concurrently.run(9, 60_000, t -> {
            if (t == 8) {
                while (acked.get() < 200) {
                    Thread.onSpinWait();
                }
                s.close();
                return;
            }
            for (int i = 0; i < 2_000; i++) {
                try {
                    long n = s.log("AUTH", i % 2 == 0 ? "FILE" : "TOOL", "INFO", "x");
                    assertThat(n).isPositive();
                    acked.incrementAndGet();
                } catch (IllegalStateException closed) {
                    return;
                }
            }
        });

        assertThat(file.lines.size() + tool.lines.size()).isEqualTo(acked.get());
        assertThat(s.query(null, null, null, null, null)).hasSize(acked.get());
        assertThatThrownBy(() -> s.log("AUTH", "FILE", "INFO", "late")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldMatchABruteForceModelForEveryCombinationOfFilters() {
        LogService s = Candidate.of(() -> Solution.create(new Rec(), new Rec(), 3));
        List<String> model = new ArrayList<>();
        for (int i = 0; i < 240; i++) {
            String m = MODULES[i % 3];
            String d = (i / 3) % 2 == 0 ? "FILE" : "TOOL";
            String l = LEVELS[(i / 6) % 4];
            long seq = s.log(m, d, l, "n" + i);
            model.add(seq + "|" + m + "|" + d + "|" + l + "|n" + i);
        }

        String[] dests = {null, "FILE", "TOOL"};
        String[] mods = {null, "AUTH", "PAYMENTS", "SEARCH"};
        String[] lvls = {null, "DEBUG", "INFO", "WARN", "ERROR"};
        for (String d : dests) {
            for (String m : mods) {
                for (String l : lvls) {
                    List<String> expected = new ArrayList<>();
                    for (String line : model) {
                        String[] p = line.split("\\|", 5);
                        long seq = Long.parseLong(p[0]);
                        boolean ok = seq >= 17 && seq <= 201
                                && (d == null || d.equals(p[2]))
                                && (m == null || m.equals(p[1]))
                                && (l == null || rank(p[3]) >= rank(l));
                        if (ok) {
                            expected.add(line);
                        }
                    }
                    assertThat(s.query(d, m, l, 17L, 201L)).as("%s %s %s", d, m, l).isEqualTo(expected);
                }
            }
        }
        s.close();
    }

    private static int rank(String level) {
        return List.of(LEVELS).indexOf(level);
    }

    @Test
    void shouldNotReuseOrSkipNumbersAcrossLongRunsOfConsecutiveFailures() {
        Rec file = new Rec();
        AtomicInteger calls = new AtomicInteger();
        Sink burst = line -> {
            int c = calls.incrementAndGet();
            return (c < 5 || c > 50) && file.write(line);
        };
        LogService s = Candidate.of(() -> Solution.create(burst, new Rec(), 2));

        List<Long> out = new ArrayList<>();
        for (int i = 0; i < 60; i++) {
            out.add(s.log("AUTH", "FILE", "INFO", "x"));
        }
        s.close();

        List<Long> expected = new ArrayList<>(List.of(1L, 2L, 3L, 4L));
        for (int i = 0; i < 46; i++) {
            expected.add(-1L);
        }
        for (long n = 5; n <= 14; n++) {
            expected.add(n);
        }
        assertThat(out).isEqualTo(expected);
        assertThat(file.lines).hasSize(14);
    }
}

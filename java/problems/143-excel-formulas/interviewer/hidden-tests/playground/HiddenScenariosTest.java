package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.FormulaSheet;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static String cellName(int index) {
        return "" + (char) ('A' + index % 26) + (index / 26 + 1);
    }

    private static void onDefaultStack(Runnable body) {
        AtomicReference<Throwable> failure = new AtomicReference<>();
        Thread t = new Thread(() -> {
            try {
                body.run();
            } catch (Throwable e) {
                failure.set(e);
            }
        });
        t.start();
        try {
            t.join();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new AssertionError(e);
        }
        if (failure.get() != null) {
            throw new AssertionError("Failed on a default-stack thread: " + failure.get(), failure.get());
        }
    }

    @Test
    void shouldBuildTheChainBackwardsAndEvaluateEveryCellOnceWhenTheHeadArrives() {
        FormulaSheet s = Candidate.of(Solution::create);
        onDefaultStack(() -> {
            for (int i = 999; i >= 1; i--) {
                s.set(cellName(i), "=" + cellName(i - 1) + "+1");
            }
            assertThat(s.get(cellName(999))).isEqualTo("999");
            long before = s.recomputeCount();
            s.set("A1", "1");
            assertThat(s.recomputeCount() - before).isEqualTo(999);
            assertThat(s.get(cellName(999))).isEqualTo("1000");
            assertThat(s.get("B1")).isEqualTo("2");
            assertThat(s.dependents("A1")).containsExactly("B1");
        });
    }

    @Test
    void shouldDetectAReferenceCycleThroughAllTwentySixHundredCellsAndRecoverFromIt() {
        FormulaSheet s = Candidate.of(Solution::create);
        onDefaultStack(() -> {
            for (int i = 0; i < 2600; i++) {
                s.set(cellName(i), "=" + cellName((i + 2599) % 2600) + "+1");
            }
            for (int i = 0; i < 2600; i += 137) {
                assertThat(s.get(cellName(i))).isEqualTo("#CYCLE!");
            }
            assertThat(s.get("Z100")).isEqualTo("#CYCLE!");
            assertThat(s.get("A1")).isEqualTo("#CYCLE!");

            long before = s.recomputeCount();
            s.set("A1", "1");
            assertThat(s.recomputeCount() - before).isEqualTo(2599);
            assertThat(s.get("Z100")).isEqualTo("2600");
            assertThat(s.get("B1")).isEqualTo("2");
        });
    }

    @Test
    void shouldNotEvaluateACellOncePerPathThroughALayeredLattice() {
        FormulaSheet s = Candidate.of(Solution::create);
        s.set("A1", "1");
        for (int c = 1; c < 26; c++) {
            s.set(cellName(c), "=A1");
        }
        for (int r = 2; r <= 8; r++) {
            for (int c = 0; c < 26; c++) {
                s.set(cellName((r - 1) * 26 + c), "=SUM(A" + (r - 1) + ":Z" + (r - 1) + ")");
            }
        }
        assertThat(s.get("Z1")).isEqualTo("1");
        assertThat(s.get("A2")).isEqualTo("26");
        assertThat(s.get("Z3")).isEqualTo("676");
        assertThat(s.get("M8")).isEqualTo("8031810176");

        long before = s.recomputeCount();
        s.set("A1", "2");
        assertThat(s.recomputeCount() - before).isEqualTo(25 + 7 * 26);
        assertThat(s.get("M8")).isEqualTo("16063620352");
        assertThat(s.get("A2")).isEqualTo("52");
    }

    @Test
    void shouldHandleTwentySixLargeRangesOverTheWholeGrid() {
        FormulaSheet s = Candidate.of(Solution::create);
        for (int i = 0; i < 99 * 26; i++) {
            s.set(cellName(i), String.valueOf(i % 7));
        }
        for (int c = 0; c < 26; c++) {
            s.set(cellName(99 * 26 + c), "=SUM(A1:Z99)");
        }
        long expected = 0;
        for (int i = 0; i < 99 * 26; i++) {
            expected += i % 7;
        }
        assertThat(s.get("A100")).isEqualTo(String.valueOf(expected));
        assertThat(s.dependencies("Z100")).hasSize(99 * 26);
        assertThat(s.dependents("A1")).hasSize(26);
        assertThat(s.dependents("Z99")).hasSize(26);

        long before = s.recomputeCount();
        long start = System.nanoTime();
        for (int k = 0; k < 100; k++) {
            int i = (k * 37) % (99 * 26);
            s.set(cellName(i), "100");
            expected += 100 - (i % 7);
            if (k % 25 == 0) {
                assertThat(s.get("Q100")).isEqualTo(String.valueOf(expected));
            }
            s.set(cellName(i), String.valueOf(i % 7));
            expected -= 100 - (i % 7);
        }
        assertThat(s.recomputeCount() - before).isEqualTo(100L * 2 * 26);
        assertThat(s.get("Z100")).isEqualTo(String.valueOf(expected));
        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(30_000);
    }

    @Test
    void shouldPropagateErrorsAlongALongChainAndRecoverWhenTheHeadIsFixed() {
        FormulaSheet s = Candidate.of(Solution::create);
        onDefaultStack(() -> {
            s.set("A1", "=1/0");
            for (int i = 1; i < 2600; i++) {
                s.set(cellName(i), "=" + cellName(i - 1) + "+1");
            }
            assertThat(s.get("Z100")).isEqualTo("#DIV/0!");
            s.set("A1", "text");
            assertThat(s.get("Z100")).isEqualTo("#VALUE!");
            s.set("A1", "=Z101");
            assertThat(s.get("M50")).isEqualTo("#REF!");
            s.set("A1", "0");
            assertThat(s.get("Z100")).isEqualTo("2599");
        });
    }

    @Test
    void shouldCountSyntaxErrorsCycleTailsAndIdenticalRewrites() {
        FormulaSheet s = Candidate.of(Solution::create);
        s.set("A1", "=1+");
        assertThat(s.recomputeCount()).isEqualTo(1);
        assertThat(s.get("A1")).isEqualTo("#ERR!");
        s.set("A1", "=1+");
        assertThat(s.recomputeCount()).isEqualTo(2);

        s.set("B1", "5");
        s.set("B2", "=B1+1");
        s.set("B3", "=B2+1");
        long before = s.recomputeCount();
        s.set("B1", "5");
        assertThat(s.recomputeCount() - before).isEqualTo(2);

        // a cycle with a tail: C1 <-> C2, C3 and C4 hang off it
        s.set("C1", "=C2");
        s.set("C2", "=C1");
        s.set("C3", "=C1+1");
        s.set("C4", "=C3+1");
        before = s.recomputeCount();
        s.set("C2", "=C1");
        assertThat(s.recomputeCount() - before).isEqualTo(2);
        assertThat(s.get("C4")).isEqualTo("#CYCLE!");
        s.set("C2", "");
        assertThat(s.get("C1")).isEqualTo("0");
        assertThat(s.get("C4")).isEqualTo("2");
        assertThat(s.dependents("C2")).containsExactly("C1");
    }

    @Test
    void shouldFollowValuesThroughTextNumberAndFormulaTransitions() {
        FormulaSheet s = Candidate.of(Solution::create);
        s.set("A1", "10");
        s.set("B1", "=A1");
        s.set("C1", "=B1*2");
        s.set("D1", "=SUM(A1:C1)");
        assertThat(s.get("D1")).isEqualTo("40");
        s.set("A1", "ten");
        assertThat(s.get("B1")).isEqualTo("ten");
        assertThat(s.get("C1")).isEqualTo("#VALUE!");
        assertThat(s.get("D1")).isEqualTo("#VALUE!");
        s.set("A1", "=B1");
        assertThat(s.get("D1")).isEqualTo("#CYCLE!");
        s.set("A1", "");
        assertThat(s.get("B1")).isEqualTo("0");
        assertThat(s.get("D1")).isEqualTo("0");
        s.set("B1", "0.1");
        s.set("E1", "=SUM(B1,B1,B1,B1,B1,B1,B1,B1,B1,B1)");
        assertThat(s.get("E1")).isEqualTo("1");
        StringBuilder nested = new StringBuilder("=");
        for (int i = 0; i < 200; i++) {
            nested.append('(');
        }
        nested.append("B1+1");
        for (int i = 0; i < 200; i++) {
            nested.append(')');
        }
        s.set("F1", nested.toString());
        assertThat(s.get("F1")).isEqualTo("1.1");
    }

    @Test
    void shouldKeepDerivedCellsMutuallyConsistentWhileWritersRaceReaders() {
        FormulaSheet s = Candidate.of(Solution::create);
        s.set("A1", "1");
        s.set("B1", "=A1");
        s.set("C1", "=A1*2");
        s.set("D1", "=C1-B1-B1");
        s.set("E1", "=SUM(B1:C1)");
        s.set("F1", "=E1-A1*3");

        List<Boolean> results = Concurrently.collect(8, 60_000, t -> {
            if (t < 3) {
                for (int i = 0; i < 2_000; i++) {
                    s.set("A1", String.valueOf(t * 10_000 + i));
                }
                return true;
            }
            boolean ok = true;
            for (int i = 0; i < 5_000; i++) {
                ok &= s.get("D1").equals("0");
                ok &= s.get("F1").equals("0");
                ok &= s.dependencies("D1").equals(List.of("B1", "C1"));
            }
            return ok;
        });
        assertThat(results).doesNotContain(false);
        assertThat(s.recomputeCount()).isEqualTo(5 + 3 * 2_000 * 5L);
    }
}

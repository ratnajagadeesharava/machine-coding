package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

import contract.MatrixSource;
import contract.MatrixTraversal;
import contract.TraversalStrategy;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final List<String> BUILT_INS = List.of(
            "ROW_MAJOR", "COLUMN_MAJOR", "SNAKE", "SPIRAL", "SPIRAL_COUNTER",
            "DIAGONAL", "ZIGZAG", "BOUSTROPHEDON_COLUMNS", "REVERSE_ROW_MAJOR");

    /** value = 3 * r + c, counts get() calls. */
    static final class Grid implements MatrixSource {
        final int rows;
        final int cols;
        final AtomicLong gets = new AtomicLong();

        Grid(int rows, int cols) {
            this.rows = rows;
            this.cols = cols;
        }

        @Override
        public int rows() {
            return rows;
        }

        @Override
        public int cols() {
            return cols;
        }

        @Override
        public int get(int r, int c) {
            if (r < 0 || r >= rows || c < 0 || c >= cols) {
                throw new IllegalArgumentException("get(" + r + "," + c + ") is outside the matrix");
            }
            gets.incrementAndGet();
            return 3 * r + c;
        }

        int value(int r, int c) {
            return 3 * r + c;
        }
    }

    private static MatrixTraversal over(MatrixSource s) {
        return Candidate.of(() -> Solution.create(s));
    }

    // ---- independent oracles: plain simulations, no index maths --------------------------------------------------

    private static List<int[]> spiralOracle(int rows, int cols) {
        List<int[]> out = new ArrayList<>();
        int top = 0;
        int bottom = rows - 1;
        int left = 0;
        int right = cols - 1;
        while (top <= bottom && left <= right) {
            for (int c = left; c <= right; c++) {
                out.add(new int[] {top, c});
            }
            top++;
            for (int r = top; r <= bottom; r++) {
                out.add(new int[] {r, right});
            }
            right--;
            if (top <= bottom) {
                for (int c = right; c >= left; c--) {
                    out.add(new int[] {bottom, c});
                }
                bottom--;
            }
            if (left <= right) {
                for (int r = bottom; r >= top; r--) {
                    out.add(new int[] {r, left});
                }
                left++;
            }
        }
        return out;
    }

    private static List<int[]> oracle(String name, int rows, int cols) {
        List<int[]> out = new ArrayList<>();
        switch (name) {
            case "ROW_MAJOR" -> {
                for (int r = 0; r < rows; r++) {
                    for (int c = 0; c < cols; c++) {
                        out.add(new int[] {r, c});
                    }
                }
            }
            case "COLUMN_MAJOR" -> {
                for (int c = 0; c < cols; c++) {
                    for (int r = 0; r < rows; r++) {
                        out.add(new int[] {r, c});
                    }
                }
            }
            case "SNAKE" -> {
                for (int r = 0; r < rows; r++) {
                    for (int i = 0; i < cols; i++) {
                        out.add(new int[] {r, r % 2 == 0 ? i : cols - 1 - i});
                    }
                }
            }
            case "BOUSTROPHEDON_COLUMNS" -> {
                for (int c = 0; c < cols; c++) {
                    for (int i = 0; i < rows; i++) {
                        out.add(new int[] {c % 2 == 0 ? i : rows - 1 - i, c});
                    }
                }
            }
            case "REVERSE_ROW_MAJOR" -> {
                out = oracle("ROW_MAJOR", rows, cols);
                Collections.reverse(out);
            }
            case "SPIRAL" -> out = spiralOracle(rows, cols);
            case "SPIRAL_COUNTER" -> {
                for (int[] p : spiralOracle(cols, rows)) {
                    out.add(new int[] {p[1], p[0]});
                }
            }
            case "DIAGONAL", "ZIGZAG" -> {
                for (int s = 0; s <= rows + cols - 2; s++) {
                    List<int[]> diag = new ArrayList<>();
                    for (int r = 0; r < rows; r++) {
                        int c = s - r;
                        if (c >= 0 && c < cols) {
                            diag.add(new int[] {r, c});
                        }
                    }
                    if (name.equals("ZIGZAG") && s % 2 == 0) {
                        Collections.reverse(diag);
                    }
                    out.addAll(diag);
                }
            }
            default -> throw new IllegalArgumentException(name);
        }
        return out;
    }

    @Test
    void shouldAgreeWithPlainSimulationsOnEveryShapeUpTo9x9() {
        for (int rows = 1; rows <= 9; rows++) {
            for (int cols = 1; cols <= 9; cols++) {
                Grid g = new Grid(rows, cols);
                MatrixTraversal t = over(g);
                for (String name : BUILT_INS) {
                    List<Integer> expected = new ArrayList<>();
                    for (int[] p : oracle(name, rows, cols)) {
                        expected.add(g.value(p[0], p[1]));
                    }
                    String label = name + " " + rows + "x" + cols;
                    assertThat(t.traverse(name)).as(label).isEqualTo(expected);

                    int cursor = t.start(name);
                    List<Integer> viaCursor = new ArrayList<>();
                    Optional<Integer> v;
                    while ((v = t.next(cursor)).isPresent()) {
                        viaCursor.add(v.get());
                    }
                    assertThat(viaCursor).as(label + " cursor").isEqualTo(expected);

                    long running = 0;
                    for (int k = 0; k < expected.size(); k++) {
                        assertThat(t.kthValue(name, k)).as(label + " kth " + k).isEqualTo(expected.get(k));
                        assertThat(t.pathSum(name, k)).as(label + " sum " + k).isEqualTo(running);
                        running += expected.get(k);
                    }
                    assertThat(t.pathSum(name, expected.size() + 5)).isEqualTo(running);
                }
            }
        }
    }

    @Test
    void shouldAnswerKthValueOnA20000x20000MatrixWithOneReadAndNoScan() {
        Grid g = new Grid(20_000, 20_000);
        MatrixTraversal t = over(g);
        long n = 400_000_000L;

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (String name : BUILT_INS) {
                for (long k : new long[] {0, 1, 19_999, 20_000, 123_456_789L, n / 2, n - 2, n - 1}) {
                    long before = g.gets.get();
                    t.kthValue(name, k);
                    assertThat(g.gets.get() - before).as(name + " k=" + k).isLessThanOrEqualTo(1);
                }
            }
            // last visits, derived by hand
            assertThat(t.kthValue("ROW_MAJOR", n - 1)).isEqualTo(g.value(19_999, 19_999));
            assertThat(t.kthValue("REVERSE_ROW_MAJOR", 0)).isEqualTo(g.value(19_999, 19_999));
            assertThat(t.kthValue("COLUMN_MAJOR", n - 1)).isEqualTo(g.value(19_999, 19_999));
            assertThat(t.kthValue("SNAKE", n - 1)).isEqualTo(g.value(19_999, 0));
            assertThat(t.kthValue("BOUSTROPHEDON_COLUMNS", n - 1)).isEqualTo(g.value(0, 19_999));
            assertThat(t.kthValue("SPIRAL", n - 1)).isEqualTo(g.value(10_000, 9_999));
            assertThat(t.kthValue("SPIRAL_COUNTER", n - 1)).isEqualTo(g.value(9_999, 10_000));
            assertThat(t.kthValue("DIAGONAL", n - 1)).isEqualTo(g.value(19_999, 19_999));
            assertThat(t.kthValue("ZIGZAG", n - 1)).isEqualTo(g.value(19_999, 19_999));
            assertThat(t.kthValue("SPIRAL", 19_999)).isEqualTo(g.value(0, 19_999));
            assertThat(t.kthValue("SPIRAL", 20_000)).isEqualTo(g.value(1, 19_999));
            assertThat(t.kthValue("ZIGZAG", 3)).isEqualTo(g.value(2, 0));
        });
    }

    @Test
    void shouldComputePathSumOfAHugeMatrixReadingAtMostKCells() {
        Grid g = new Grid(20_000, 20_000);
        MatrixTraversal t = over(g);

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (String name : BUILT_INS) {
                long before = g.gets.get();
                long sum = t.pathSum(name, 5_000);
                assertThat(g.gets.get() - before).as(name).isLessThanOrEqualTo(5_000);

                int cursor = t.start(name);
                long viaCursor = 0;
                for (int i = 0; i < 5_000; i++) {
                    viaCursor += t.next(cursor).get();
                }
                assertThat(sum).as(name).isEqualTo(viaCursor);
            }
            assertThat(t.pathSum("ROW_MAJOR", 20_000)).isEqualTo(
                    20_000L * 19_999 / 2); // 3*0 + c for c in 0..19999
        });
    }

    @Test
    void shouldCountMoreCellsThanAnIntCanHold() {
        Grid g = new Grid(100_000, 100_000);
        MatrixTraversal t = over(g);
        long n = 10_000_000_000L;

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            int cursor = t.start("COLUMN_MAJOR");
            assertThat(t.remaining(cursor)).isEqualTo(n);
            assertThat(t.next(cursor)).contains(g.value(0, 0));
            assertThat(t.next(cursor)).contains(g.value(1, 0));
            assertThat(t.remaining(cursor)).isEqualTo(n - 2);
            assertThat(t.kthValue("ROW_MAJOR", n - 1)).isEqualTo(g.value(99_999, 99_999));
            assertThat(t.kthValue("ROW_MAJOR", 4_294_967_296L)).isEqualTo(g.value(42_949, 67_296));
            assertThat(t.kthValue("COLUMN_MAJOR", 4_294_967_296L)).isEqualTo(g.value(67_296, 42_949));
            assertThat(t.kthValue("SPIRAL", n - 1)).isEqualTo(g.value(50_000, 49_999));
            assertThat(t.kthValue("SPIRAL_COUNTER", n - 1)).isEqualTo(g.value(49_999, 50_000));
            assertThat(t.kthValue("DIAGONAL", n - 1)).isEqualTo(g.value(99_999, 99_999));
            assertThat(t.kthValue("ZIGZAG", n - 1)).isEqualTo(g.value(99_999, 99_999));
            assertThat(t.kthValue("REVERSE_ROW_MAJOR", n - 1)).isEqualTo(g.value(0, 0));
            assertThatThrownBy(() -> t.kthValue("ROW_MAJOR", n)).isInstanceOf(IllegalArgumentException.class);
            assertThat(t.pathSum("SPIRAL", 3)).isEqualTo(0 + 1 + 2);
        });
    }

    @Test
    void shouldHandleExtremelyThinMatrices() {
        Grid row = new Grid(1, 1_000_000);
        Grid col = new Grid(1_000_000, 1);
        MatrixTraversal r = over(row);
        MatrixTraversal c = over(col);

        assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (String name : BUILT_INS) {
                boolean reversed = name.equals("REVERSE_ROW_MAJOR");
                assertThat(r.kthValue(name, 0)).as(name).isEqualTo(reversed ? 999_999 : 0);
                assertThat(r.kthValue(name, 999_999)).as(name).isEqualTo(reversed ? 0 : 999_999);
                assertThat(r.kthValue(name, 1234)).as(name).isEqualTo(reversed ? 999_999 - 1234 : 1234);
                assertThat(c.kthValue(name, 0)).as(name).isEqualTo(reversed ? 3 * 999_999 : 0);
                assertThat(c.kthValue(name, 999_999)).as(name).isEqualTo(reversed ? 0 : 3 * 999_999);
                assertThat(c.kthValue(name, 1234)).as(name).isEqualTo(reversed ? 3 * (999_999 - 1234) : 3 * 1234);
            }
        });
    }

    @Test
    void shouldReturnEachValueExactlyOnceWhenThreadsShareOneCursor() {
        Grid g = new Grid(200, 200);
        MatrixTraversal t = over(g);
        int cursor = t.start("SPIRAL");

        List<List<Integer>> parts = Concurrently.collect(8, i -> {
            List<Integer> mine = new ArrayList<>();
            Optional<Integer> v;
            while ((v = t.next(cursor)).isPresent()) {
                mine.add(v.get());
            }
            return mine;
        });

        List<Integer> all = new ArrayList<>();
        parts.forEach(all::addAll);
        assertThat(all).hasSize(40_000);
        assertThat(all).containsExactlyInAnyOrderElementsOf(t.traverse("SPIRAL"));
        assertThat(t.hasNext(cursor)).isFalse();
    }

    @Test
    void shouldLetExactlyOneThreadRegisterAContestedName() {
        MatrixTraversal t = over(new Grid(4, 4));
        TraversalStrategy rowMajor = (rows, cols, index) -> new int[] {(int) (index / cols), (int) (index % cols)};
        AtomicInteger wins = new AtomicInteger();

        Concurrently.run(16, i -> {
            try {
                t.registerStrategy("CONTESTED", rowMajor);
                wins.incrementAndGet();
            } catch (IllegalArgumentException expected) {
                // lost the race
            }
        });

        assertThat(wins.get()).isEqualTo(1);
        assertThat(t.traverse("CONTESTED")).hasSize(16);
    }

    @Test
    void shouldAskARegisteredStrategyOnlyForWhatItNeedsOnAHugeMatrix() {
        Grid g = new Grid(20_000, 20_000);
        MatrixTraversal t = over(g);
        AtomicLong calls = new AtomicLong();
        t.registerStrategy("COUNTED", (rows, cols, index) -> {
            calls.incrementAndGet();
            return new int[] {(int) (index / cols), (int) (index % cols)};
        });

        int cursor = t.start("COUNTED");
        assertThat(calls.get()).isZero();
        assertThat(g.gets.get()).isZero();
        for (int i = 0; i < 10; i++) {
            t.next(cursor);
        }
        assertThat(calls.get()).isLessThanOrEqualTo(10);
        assertThat(g.gets.get()).isLessThanOrEqualTo(100);

        calls.set(0);
        assertThat(t.kthValue("COUNTED", 399_999_999L)).isEqualTo(g.value(19_999, 19_999));
        assertThat(calls.get()).isLessThanOrEqualTo(1);
        calls.set(0);
        assertThat(t.pathSum("COUNTED", 100)).isEqualTo(
                java.util.stream.LongStream.range(0, 100).sum());
        assertThat(calls.get()).isLessThanOrEqualTo(100);
    }

    @Test
    void shouldAcceptARegisteredPermutationAndRejectOneThatIsOnlyMostlyAPermutation() {
        int rows = 30;
        int cols = 17;
        long n = (long) rows * cols;
        List<Integer> shuffled = new ArrayList<>();
        for (int i = 0; i < n; i++) {
            shuffled.add(i);
        }
        Collections.shuffle(shuffled, new Random(7));
        Grid g = new Grid(rows, cols);
        MatrixTraversal t = over(g);
        t.registerStrategy("SHUFFLED", (r, c, index) -> {
            int cell = shuffled.get((int) index);
            return new int[] {cell / c, cell % c};
        });
        t.registerStrategy("ALMOST", (r, c, index) -> {
            int cell = shuffled.get((int) (index == n - 1 ? 0 : index));
            return new int[] {cell / c, cell % c};
        });

        List<Integer> expected = new ArrayList<>();
        for (int cell : shuffled) {
            expected.add(g.value(cell / cols, cell % cols));
        }
        assertThat(t.traverse("SHUFFLED")).isEqualTo(expected);
        assertThatThrownBy(() -> t.traverse("ALMOST"))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining(String.valueOf(n - 1));
        // the very same strategy is fine for the prefix it gets right
        assertThat(t.pathSum("ALMOST", n - 1)).isEqualTo(expected.stream().limit(n - 1).mapToLong(i -> i).sum());
    }

    @Test
    void shouldFailAMidStreamBadPositionOnlyWhenReachedAndKeepOtherCursorsWorking() {
        MatrixTraversal t = over(new Grid(3, 3));
        t.registerStrategy("BAD_AT_5", (rows, cols, index) ->
                index == 5 ? new int[] {rows, cols} : new int[] {(int) (index / cols), (int) (index % cols)});
        int bad = t.start("BAD_AT_5");
        int good = t.start("ROW_MAJOR");

        for (int i = 0; i < 5; i++) {
            assertThat(t.next(bad)).isPresent();
        }
        assertThatThrownBy(() -> t.next(bad)).isInstanceOf(IllegalStateException.class).hasMessageContaining("5");
        assertThatThrownBy(() -> t.next(bad)).isInstanceOf(IllegalStateException.class).hasMessageContaining("5");
        assertThat(t.remaining(bad)).isEqualTo(4);
        t.reset(bad);
        assertThat(t.next(bad)).contains(0);
        assertThat(t.next(good)).contains(0);
        assertThat(t.hasNext(good)).isTrue();
    }

    @Test
    void shouldOpenAndCloseHundredThousandCursors() {
        MatrixTraversal t = over(new Grid(20_000, 20_000));
        int[] ids = new int[100_000];
        for (int i = 0; i < ids.length; i++) {
            ids[i] = t.start(BUILT_INS.get(i % BUILT_INS.size()));
            assertThat(ids[i]).isEqualTo(i + 1);
        }
        for (int id : ids) {
            t.close(id);
        }
        assertThat(t.start("SPIRAL")).isEqualTo(100_001);
        assertThatThrownBy(() -> t.next(ids[0])).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSurviveCursorsClosingWhileOthersKeepReading() {
        MatrixTraversal t = over(new Grid(60, 60));
        List<Integer> expected = t.traverse("ZIGZAG");

        List<List<Integer>> got = Concurrently.collect(16, i -> {
            List<Integer> mine = new ArrayList<>();
            for (int round = 0; round < 20; round++) {
                int scratch = t.start("ROW_MAJOR");
                t.next(scratch);
                t.close(scratch);
            }
            int cursor = t.start("ZIGZAG");
            Optional<Integer> v;
            while ((v = t.next(cursor)).isPresent()) {
                mine.add(v.get());
            }
            t.close(cursor);
            return mine;
        });

        for (List<Integer> mine : got) {
            assertThat(mine).isEqualTo(expected);
        }
    }

    @Test
    void shouldMatchTheOracleSumsOnAnOddSizedThousandSquareGrid() {
        Grid g = new Grid(1_000, 1_001);
        MatrixTraversal t = over(g);

        for (String name : BUILT_INS) {
            long expected = 0;
            for (int[] p : oracle(name, 1_000, 1_001)) {
                expected += g.value(p[0], p[1]);
            }
            assertThat(t.pathSum(name, Long.MAX_VALUE)).as(name).isEqualTo(expected);
        }
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.MatrixSource;
import contract.MatrixTraversal;
import contract.TraversalStrategy;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final int[][] G3 = {{1, 2, 3}, {4, 5, 6}, {7, 8, 9}};
    private static final int[][] G23 = {{1, 2, 3}, {4, 5, 6}};
    private static final int[][] G32 = {{1, 2}, {3, 4}, {5, 6}};
    private static final List<String> BUILT_INS = List.of(
            "ROW_MAJOR", "COLUMN_MAJOR", "SNAKE", "SPIRAL", "SPIRAL_COUNTER",
            "DIAGONAL", "ZIGZAG", "BOUSTROPHEDON_COLUMNS", "REVERSE_ROW_MAJOR");

    /** value = r * cols + c, counts every get(); never held in memory. */
    static final class Counting implements MatrixSource {
        final int rows;
        final int cols;
        final AtomicLong gets = new AtomicLong();

        Counting(int rows, int cols) {
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
            return r * cols + c;
        }
    }

    private static MatrixTraversal of(int[][] grid) {
        return Candidate.of(() -> Solution.create(MatrixSource.of(grid)));
    }

    private static MatrixTraversal over(MatrixSource source) {
        return Candidate.of(() -> Solution.create(source));
    }

    private static List<Integer> drain(MatrixTraversal t, int cursor) {
        List<Integer> out = new ArrayList<>();
        Optional<Integer> v;
        while ((v = t.next(cursor)).isPresent()) {
            out.add(v.get());
        }
        return out;
    }

    /** reverse of COLUMN_MAJOR, defined by the test. */
    private static final TraversalStrategy REVERSE_COLUMN_MAJOR = (rows, cols, index) -> {
        long j = (long) rows * cols - 1 - index;
        return new int[] {(int) (j % rows), (int) (j / rows)};
    };

    @Test
    void shouldTraverseRowMajorAndItsReverse() {
        MatrixTraversal t = of(G3);

        assertThat(t.traverse("ROW_MAJOR")).containsExactly(1, 2, 3, 4, 5, 6, 7, 8, 9);
        assertThat(t.traverse("REVERSE_ROW_MAJOR")).containsExactly(9, 8, 7, 6, 5, 4, 3, 2, 1);
    }

    @Test
    void shouldTraverseColumnMajor() {
        assertThat(of(G3).traverse("COLUMN_MAJOR")).containsExactly(1, 4, 7, 2, 5, 8, 3, 6, 9);
    }

    @Test
    void shouldTraverseSnakeRowsAndSnakeColumns() {
        MatrixTraversal t = of(G3);

        assertThat(t.traverse("SNAKE")).containsExactly(1, 2, 3, 6, 5, 4, 7, 8, 9);
        assertThat(t.traverse("BOUSTROPHEDON_COLUMNS")).containsExactly(1, 4, 7, 8, 5, 2, 3, 6, 9);
    }

    @Test
    void shouldTraverseSpiralClockwiseAndCounterClockwise() {
        MatrixTraversal t = of(G3);

        assertThat(t.traverse("SPIRAL")).containsExactly(1, 2, 3, 6, 9, 8, 7, 4, 5);
        assertThat(t.traverse("SPIRAL_COUNTER")).containsExactly(1, 4, 7, 8, 9, 6, 3, 2, 5);
    }

    @Test
    void shouldTraverseAntiDiagonalsTopRightToBottomLeft() {
        assertThat(of(G3).traverse("DIAGONAL")).containsExactly(1, 2, 4, 3, 5, 7, 6, 8, 9);
    }

    @Test
    void shouldTraverseZigzagLikeLeetCode498() {
        assertThat(of(G3).traverse("ZIGZAG")).containsExactly(1, 2, 4, 7, 5, 3, 6, 8, 9);
    }

    @Test
    void shouldHandleWideMatrices() {
        MatrixTraversal t = of(G23);

        assertThat(t.traverse("ROW_MAJOR")).containsExactly(1, 2, 3, 4, 5, 6);
        assertThat(t.traverse("COLUMN_MAJOR")).containsExactly(1, 4, 2, 5, 3, 6);
        assertThat(t.traverse("SNAKE")).containsExactly(1, 2, 3, 6, 5, 4);
        assertThat(t.traverse("SPIRAL")).containsExactly(1, 2, 3, 6, 5, 4);
        assertThat(t.traverse("SPIRAL_COUNTER")).containsExactly(1, 4, 5, 6, 3, 2);
        assertThat(t.traverse("DIAGONAL")).containsExactly(1, 2, 4, 3, 5, 6);
        assertThat(t.traverse("ZIGZAG")).containsExactly(1, 2, 4, 5, 3, 6);
        assertThat(t.traverse("BOUSTROPHEDON_COLUMNS")).containsExactly(1, 4, 5, 2, 3, 6);
        assertThat(t.traverse("REVERSE_ROW_MAJOR")).containsExactly(6, 5, 4, 3, 2, 1);
    }

    @Test
    void shouldHandleTallMatrices() {
        MatrixTraversal t = of(G32);

        assertThat(t.traverse("ROW_MAJOR")).containsExactly(1, 2, 3, 4, 5, 6);
        assertThat(t.traverse("COLUMN_MAJOR")).containsExactly(1, 3, 5, 2, 4, 6);
        assertThat(t.traverse("SNAKE")).containsExactly(1, 2, 4, 3, 5, 6);
        assertThat(t.traverse("SPIRAL")).containsExactly(1, 2, 4, 6, 5, 3);
        assertThat(t.traverse("SPIRAL_COUNTER")).containsExactly(1, 3, 5, 6, 4, 2);
        assertThat(t.traverse("DIAGONAL")).containsExactly(1, 2, 3, 4, 5, 6);
        assertThat(t.traverse("ZIGZAG")).containsExactly(1, 2, 3, 5, 4, 6);
        assertThat(t.traverse("BOUSTROPHEDON_COLUMNS")).containsExactly(1, 3, 5, 6, 4, 2);
        assertThat(t.traverse("REVERSE_ROW_MAJOR")).containsExactly(6, 5, 4, 3, 2, 1);
    }

    @Test
    void shouldHandleSingleRowSingleColumnAndSingleCell() {
        MatrixTraversal row = of(new int[][] {{7, 8, 9}});
        MatrixTraversal col = of(new int[][] {{7}, {8}, {9}});
        MatrixTraversal one = of(new int[][] {{42}});

        assertThat(row.traverse("SPIRAL")).containsExactly(7, 8, 9);
        assertThat(row.traverse("SPIRAL_COUNTER")).containsExactly(7, 8, 9);
        assertThat(row.traverse("COLUMN_MAJOR")).containsExactly(7, 8, 9);
        assertThat(row.traverse("ZIGZAG")).containsExactly(7, 8, 9);
        assertThat(col.traverse("SPIRAL")).containsExactly(7, 8, 9);
        assertThat(col.traverse("SPIRAL_COUNTER")).containsExactly(7, 8, 9);
        assertThat(col.traverse("SNAKE")).containsExactly(7, 8, 9);
        assertThat(col.traverse("BOUSTROPHEDON_COLUMNS")).containsExactly(7, 8, 9);
        assertThat(col.traverse("DIAGONAL")).containsExactly(7, 8, 9);
        assertThat(col.traverse("ZIGZAG")).containsExactly(7, 8, 9);
        for (String name : BUILT_INS) {
            assertThat(one.traverse(name)).containsExactly(42);
        }
    }

    @Test
    void shouldVisitEveryCellExactlyOnceForEveryStrategyAndShape() {
        for (int rows = 1; rows <= 6; rows++) {
            for (int cols = 1; cols <= 6; cols++) {
                MatrixTraversal t = over(new Counting(rows, cols));
                for (String name : BUILT_INS) {
                    List<Integer> seen = t.traverse(name);
                    assertThat(seen).as(name + " " + rows + "x" + cols).hasSize(rows * cols);
                    assertThat(new HashSet<>(seen)).as(name + " " + rows + "x" + cols).hasSize(rows * cols);
                }
            }
        }
    }

    @Test
    void shouldRejectUnknownAndMisspelledStrategyNames() {
        MatrixTraversal t = of(G3);

        assertThatThrownBy(() -> t.traverse("NOPE")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.traverse("row_major")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.traverse(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.start("Spiral")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.pathSum("NOPE", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.kthValue("NOPE", 0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidMatrices() {
        of(G3); // the factory works for a valid matrix; now the invalid ones
        assertThatThrownBy(() -> Solution.create(MatrixSource.of(new int[0][]))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(MatrixSource.of(new int[][] {{}}))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(MatrixSource.of(new int[][] {{1, 2}, {3}})))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(new Counting(0, 5))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(new Counting(5, 0))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotSeeLaterChangesToTheArrayItWasBuiltFrom() {
        int[][] grid = {{1, 2}, {3, 4}};
        MatrixTraversal t = of(grid);
        grid[0][0] = 99;
        grid[1] = new int[] {7, 7};

        assertThat(t.traverse("ROW_MAJOR")).containsExactly(1, 2, 3, 4);
    }

    @Test
    void shouldIterateWithACursor() {
        MatrixTraversal t = of(G3);
        int c = t.start("SPIRAL");

        assertThat(c).isEqualTo(1);
        assertThat(t.hasNext(c)).isTrue();
        assertThat(t.remaining(c)).isEqualTo(9);
        assertThat(t.next(c)).contains(1);
        assertThat(t.next(c)).contains(2);
        assertThat(t.remaining(c)).isEqualTo(7);
        assertThat(drain(t, c)).containsExactly(3, 6, 9, 8, 7, 4, 5);
        assertThat(t.hasNext(c)).isFalse();
        assertThat(t.remaining(c)).isZero();
    }

    @Test
    void shouldReturnEmptyOnceExhaustedAndKeepDoingSo() {
        MatrixTraversal t = of(new int[][] {{5, 6}});
        int c = t.start("ROW_MAJOR");

        assertThat(drain(t, c)).containsExactly(5, 6);
        assertThat(t.next(c)).isEmpty();
        assertThat(t.next(c)).isEmpty();
        assertThat(t.hasNext(c)).isFalse();
        assertThat(t.remaining(c)).isZero();
    }

    @Test
    void shouldResetACursorToTheStartEvenWhenExhausted() {
        MatrixTraversal t = of(G3);
        int c = t.start("SNAKE");
        t.next(c);
        t.next(c);
        t.reset(c);

        assertThat(t.remaining(c)).isEqualTo(9);
        assertThat(drain(t, c)).containsExactly(1, 2, 3, 6, 5, 4, 7, 8, 9);
        t.reset(c);
        assertThat(t.hasNext(c)).isTrue();
        assertThat(t.next(c)).contains(1);
    }

    @Test
    void shouldKeepCursorsIndependentOfEachOther() {
        MatrixTraversal t = of(G3);
        int a = t.start("ROW_MAJOR");
        int b = t.start("COLUMN_MAJOR");
        int c = t.start("ROW_MAJOR");

        assertThat(List.of(a, b, c)).containsExactly(1, 2, 3);
        assertThat(t.next(a)).contains(1);
        assertThat(t.next(a)).contains(2);
        assertThat(t.next(b)).contains(1);
        assertThat(t.next(b)).contains(4);
        assertThat(t.remaining(c)).isEqualTo(9);
        t.reset(a);
        assertThat(t.remaining(b)).isEqualTo(7);
        assertThat(t.next(c)).contains(1);
    }

    @Test
    void shouldRejectUnknownAndClosedCursorsAndNeverReuseIds() {
        MatrixTraversal t = of(G3);
        int a = t.start("ROW_MAJOR");
        t.close(a);
        int b = t.start("ROW_MAJOR");

        assertThat(b).isEqualTo(2);
        assertThatThrownBy(() -> t.next(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.hasNext(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.remaining(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.reset(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.close(a)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.next(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.next(77)).isInstanceOf(IllegalArgumentException.class);
        assertThat(t.next(b)).contains(1);
    }

    @Test
    void shouldComputePathSumOfTheFirstKVisits() {
        MatrixTraversal t = of(G3);

        assertThat(t.pathSum("SPIRAL", 0)).isZero();
        assertThat(t.pathSum("SPIRAL", 1)).isEqualTo(1);
        assertThat(t.pathSum("SPIRAL", 4)).isEqualTo(1 + 2 + 3 + 6);
        assertThat(t.pathSum("ZIGZAG", 5)).isEqualTo(1 + 2 + 4 + 7 + 5);
        assertThat(t.pathSum("REVERSE_ROW_MAJOR", 3)).isEqualTo(9 + 8 + 7);
    }

    @Test
    void shouldClampPathSumToTheWholeMatrixAndRejectNegativeK() {
        MatrixTraversal t = of(G3);

        assertThat(t.pathSum("ROW_MAJOR", 9)).isEqualTo(45);
        assertThat(t.pathSum("ROW_MAJOR", 10)).isEqualTo(45);
        assertThat(t.pathSum("ROW_MAJOR", Long.MAX_VALUE)).isEqualTo(45);
        assertThatThrownBy(() -> t.pathSum("ROW_MAJOR", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSumInLongsWithoutOverflowingInt() {
        MatrixTraversal t = of(new int[][] {{Integer.MAX_VALUE, Integer.MAX_VALUE}, {Integer.MAX_VALUE, -5}});

        assertThat(t.pathSum("ROW_MAJOR", 3)).isEqualTo(3L * Integer.MAX_VALUE);
        assertThat(t.pathSum("ROW_MAJOR", 4)).isEqualTo(3L * Integer.MAX_VALUE - 5);
    }

    @Test
    void shouldReturnTheKthValueAndRejectOutOfRange() {
        MatrixTraversal t = of(G3);

        assertThat(t.kthValue("SPIRAL", 0)).isEqualTo(1);
        assertThat(t.kthValue("SPIRAL", 4)).isEqualTo(9);
        assertThat(t.kthValue("SPIRAL", 8)).isEqualTo(5);
        assertThat(t.kthValue("DIAGONAL", 3)).isEqualTo(3);
        assertThat(t.kthValue("BOUSTROPHEDON_COLUMNS", 4)).isEqualTo(5);
        assertThatThrownBy(() -> t.kthValue("SPIRAL", 9)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.kthValue("SPIRAL", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldUseARegisteredStrategyThroughTraverseCursorsPathSumAndKth() {
        MatrixTraversal t = of(G3);
        t.registerStrategy("REVERSE_COLUMN_MAJOR", REVERSE_COLUMN_MAJOR);

        assertThat(t.traverse("REVERSE_COLUMN_MAJOR")).containsExactly(9, 6, 3, 8, 5, 2, 7, 4, 1);
        int c = t.start("REVERSE_COLUMN_MAJOR");
        assertThat(t.next(c)).contains(9);
        assertThat(t.next(c)).contains(6);
        assertThat(t.remaining(c)).isEqualTo(7);
        assertThat(t.pathSum("REVERSE_COLUMN_MAJOR", 3)).isEqualTo(9 + 6 + 3);
        assertThat(t.kthValue("REVERSE_COLUMN_MAJOR", 4)).isEqualTo(5);
    }

    @Test
    void shouldHandOutTheMatrixShapeToARegisteredStrategy() {
        MatrixTraversal t = of(G23);
        List<String> shapes = new ArrayList<>();
        t.registerStrategy("SPY", (rows, cols, index) -> {
            shapes.add(rows + "x" + cols);
            return new int[] {(int) (index / cols), (int) (index % cols)};
        });

        assertThat(t.traverse("SPY")).containsExactly(1, 2, 3, 4, 5, 6);
        assertThat(shapes).isNotEmpty().containsOnly("2x3");
    }

    @Test
    void shouldRejectDuplicateAndBuiltInRegistrations() {
        MatrixTraversal t = of(G3);
        t.registerStrategy("MINE", REVERSE_COLUMN_MAJOR);

        assertThatThrownBy(() -> t.registerStrategy("MINE", REVERSE_COLUMN_MAJOR))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.registerStrategy("SPIRAL", REVERSE_COLUMN_MAJOR))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.registerStrategy(" ", REVERSE_COLUMN_MAJOR))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.registerStrategy(null, REVERSE_COLUMN_MAJOR))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.registerStrategy("OTHER", null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(t.traverse("SPIRAL")).containsExactly(1, 2, 3, 6, 9, 8, 7, 4, 5);
        t.registerStrategy("spiral", REVERSE_COLUMN_MAJOR); // names are case-sensitive: this is a new name
        assertThat(t.kthValue("spiral", 0)).isEqualTo(9);
    }

    @Test
    void shouldKeepRegisteredStrategiesPerServiceInstance() {
        MatrixTraversal a = of(G3);
        MatrixTraversal b = of(G3);
        a.registerStrategy("MINE", REVERSE_COLUMN_MAJOR);

        assertThatThrownBy(() -> b.traverse("MINE")).isInstanceOf(IllegalArgumentException.class);
        b.registerStrategy("MINE", REVERSE_COLUMN_MAJOR);
    }

    @Test
    void shouldFailTraverseNamingTheIndexOfAnOutOfRangePosition() {
        MatrixTraversal t = of(G3);
        t.registerStrategy("TOO_WIDE_AT_7", (rows, cols, index) ->
                index == 7 ? new int[] {0, cols} : new int[] {(int) (index / cols), (int) (index % cols)});
        t.registerStrategy("NEGATIVE_AT_4", (rows, cols, index) ->
                index == 4 ? new int[] {-1, 0} : new int[] {(int) (index / cols), (int) (index % cols)});

        assertThatThrownBy(() -> t.traverse("TOO_WIDE_AT_7"))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("7");
        assertThatThrownBy(() -> t.traverse("NEGATIVE_AT_4"))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("4");
    }

    @Test
    void shouldFailTraverseNamingTheIndexOfARepeatedPosition() {
        MatrixTraversal t = of(G3);
        t.registerStrategy("REPEATS", (rows, cols, index) -> {
            long i = index == 5 ? 2 : index;
            return new int[] {(int) (i / cols), (int) (i % cols)};
        });

        assertThatThrownBy(() -> t.traverse("REPEATS"))
                .isInstanceOf(IllegalStateException.class)
                .hasMessageContaining("5");
    }

    @Test
    void shouldFailCursorKthAndPathSumOnAnOutOfRangePositionWithoutCorruptingTheCursor() {
        MatrixTraversal t = of(G3);
        t.registerStrategy("BAD_AT_2", (rows, cols, index) ->
                index == 2 ? new int[] {9, 9} : new int[] {(int) (index / cols), (int) (index % cols)});
        int c = t.start("BAD_AT_2");

        assertThat(t.next(c)).contains(1);
        assertThat(t.next(c)).contains(2);
        assertThatThrownBy(() -> t.next(c)).isInstanceOf(IllegalStateException.class).hasMessageContaining("2");
        assertThat(t.remaining(c)).isEqualTo(7);
        assertThatThrownBy(() -> t.kthValue("BAD_AT_2", 2)).isInstanceOf(IllegalStateException.class);
        assertThat(t.kthValue("BAD_AT_2", 1)).isEqualTo(2);
        assertThatThrownBy(() -> t.pathSum("BAD_AT_2", 3)).isInstanceOf(IllegalStateException.class);
        assertThat(t.pathSum("BAD_AT_2", 2)).isEqualTo(3);
    }

    @Test
    void shouldStartAndTakeTenElementsOfAHugeMatrixWithAtMostHundredReads() {
        for (String name : BUILT_INS) {
            Counting huge = new Counting(20_000, 20_000);
            MatrixTraversal t = over(huge);
            int c = t.start(name);
            for (int i = 0; i < 10; i++) {
                assertThat(t.next(c)).isPresent();
            }
            assertThat(huge.gets.get()).as(name).isLessThanOrEqualTo(100);
            assertThat(t.remaining(c)).isEqualTo(400_000_000L - 10);
        }
    }

    @Test
    void shouldReadTheFirstValuesOfAHugeMatrixInTheRightOrder() {
        MatrixTraversal t = over(new Counting(20_000, 20_000));
        int spiral = t.start("SPIRAL");
        int counter = t.start("SPIRAL_COUNTER");
        int zig = t.start("ZIGZAG");

        List<Integer> first = new ArrayList<>();
        for (int i = 0; i < 4; i++) {
            first.add(t.next(spiral).get());
        }
        assertThat(first).containsExactly(0, 1, 2, 3);
        assertThat(t.next(counter)).contains(0);
        assertThat(t.next(counter)).contains(20_000);
        assertThat(t.next(zig)).contains(0);
        assertThat(t.next(zig)).contains(1);
        assertThat(t.next(zig)).contains(20_000);
        assertThat(t.next(zig)).contains(40_000);
    }

    @Test
    void shouldGiveEveryThreadItsOwnCursorWithTheFullSequence() {
        int rows = 40;
        int cols = 25;
        MatrixTraversal t = over(new Counting(rows, cols));
        List<String> names = new ArrayList<>(BUILT_INS);
        t.registerStrategy("REVERSE_COLUMN_MAJOR", REVERSE_COLUMN_MAJOR);
        names.add("REVERSE_COLUMN_MAJOR");

        List<List<Integer>> got = Concurrently.collect(16, i -> {
            int cursor = t.start(names.get(i % names.size()));
            return drain(t, cursor);
        });

        for (int i = 0; i < 16; i++) {
            assertThat(got.get(i)).as("thread " + i).isEqualTo(t.traverse(names.get(i % names.size())));
        }
    }

    @Test
    void shouldHandOutDistinctCursorIdsToConcurrentStarters() {
        MatrixTraversal t = of(G3);

        List<List<Integer>> ids = Concurrently.collect(16, i -> {
            List<Integer> mine = new ArrayList<>();
            for (int k = 0; k < 100; k++) {
                mine.add(t.start("ROW_MAJOR"));
            }
            return mine;
        });

        Set<Integer> all = new HashSet<>();
        ids.forEach(all::addAll);
        assertThat(all).hasSize(1_600);
        assertThat(all).allMatch(id -> id >= 1 && id <= 1_600);
    }
}

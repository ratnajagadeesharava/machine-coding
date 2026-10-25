package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SpreadsheetGrid;
import contract.StyleChange;
import contract.ValueType;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static String col(int c) {
        return c > 26 ? "" + (char) ('A' + (c - 1) / 26 - 1) + (char) ('A' + (c - 1) % 26) : "" + (char) ('A' + c - 1);
    }

    private static String addr(int r, int c) {
        return col(c) + r;
    }

    private static StyleChange bold(boolean b) {
        return new StyleChange(b, null, null, null, null, null);
    }

    /** Tiny reference model: sparse map of (row, col) -> cell, plain loops. */
    private static final class Model {
        record Cell(String text, Double number, boolean bold) {}

        Map<Long, Cell> cells = new HashMap<>();

        static long key(int r, int c) {
            return (long) r * 10_000 + c;
        }

        void put(int r, int c, Cell cell) {
            if (cell == null || (cell.text == null && cell.number == null && !cell.bold)) {
                cells.remove(key(r, c));
            } else {
                cells.put(key(r, c), cell);
            }
        }

        Cell get(int r, int c) {
            return cells.get(key(r, c));
        }

        void shift(boolean rows, int at, int delta) {
            Map<Long, Cell> next = new HashMap<>();
            for (var e : cells.entrySet()) {
                int r = (int) (e.getKey() / 10_000), c = (int) (e.getKey() % 10_000);
                int v = rows ? r : c;
                if (delta < 0 && v == at) {
                    continue;
                }
                if (v >= at) {
                    v += delta;
                }
                next.put(rows ? key(v, c) : key(r, v), e.getValue());
            }
            cells = next;
        }

        void copy(int r1, int c1, int r2, int c2, int dr, int dc, boolean move) {
            Cell[][] buf = new Cell[r2 - r1 + 1][c2 - c1 + 1];
            for (int r = r1; r <= r2; r++) {
                for (int c = c1; c <= c2; c++) {
                    buf[r - r1][c - c1] = get(r, c);
                }
            }
            if (move) {
                for (int r = r1; r <= r2; r++) {
                    for (int c = c1; c <= c2; c++) {
                        put(r, c, null);
                    }
                }
            }
            for (int r = r1; r <= r2; r++) {
                for (int c = c1; c <= c2; c++) {
                    put(dr + r - r1, dc + c - c1, buf[r - r1][c - c1]);
                }
            }
        }
    }

    @Test
    void shouldAgreeWithASimpleModelOverThousandsOfRandomOperations() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        Model m = new Model();
        long seed = 12345;
        java.util.function.IntUnaryOperator rnd = new java.util.function.IntUnaryOperator() {
            long s = seed;

            @Override
            public int applyAsInt(int bound) {
                s = s * 6364136223846793005L + 1442695040888963407L;
                return (int) ((s >>> 33) % bound);
            }
        };
        for (int step = 0; step < 4_000; step++) {
            int r = 1 + rnd.applyAsInt(15), c = 1 + rnd.applyAsInt(8);
            switch (rnd.applyAsInt(10)) {
                case 0, 1 -> {
                    double v = rnd.applyAsInt(100) - 50;
                    g.setNumber(addr(r, c), v);
                    Model.Cell old = m.get(r, c);
                    m.put(r, c, new Model.Cell(null, v == 0 ? 0.0 : v, old != null && old.bold()));
                }
                case 2 -> {
                    String t = "t" + rnd.applyAsInt(50);
                    g.setText(addr(r, c), t);
                    Model.Cell old = m.get(r, c);
                    m.put(r, c, new Model.Cell(t, null, old != null && old.bold()));
                }
                case 3 -> {
                    g.clear(addr(r, c));
                    m.put(r, c, null);
                }
                case 4 -> {
                    boolean b = rnd.applyAsInt(2) == 0;
                    g.setStyle(addr(r, c), bold(b));
                    Model.Cell old = m.get(r, c);
                    m.put(r, c, new Model.Cell(old == null ? null : old.text(), old == null ? null : old.number(), b));
                }
                case 5 -> {
                    g.insertRow(r);
                    m.shift(true, r, 1);
                }
                case 6 -> {
                    g.deleteRow(r);
                    m.shift(true, r, -1);
                }
                case 7 -> {
                    if (rnd.applyAsInt(2) == 0) {
                        g.insertColumn(col(c));
                        m.shift(false, c, 1);
                    } else {
                        g.deleteColumn(col(c));
                        m.shift(false, c, -1);
                    }
                }
                default -> {
                    int r2 = r + rnd.applyAsInt(4), c2 = c + rnd.applyAsInt(3);
                    int dr = 1 + rnd.applyAsInt(15), dc = 1 + rnd.applyAsInt(8);
                    boolean move = rnd.applyAsInt(2) == 0;
                    if (move) {
                        g.move(addr(r, c), addr(r2, c2), addr(dr, dc));
                    } else {
                        g.copy(addr(r, c), addr(r2, c2), addr(dr, dc));
                    }
                    m.copy(r, c, r2, c2, dr, dc, move);
                }
            }
            if (step % 25 == 0) {
                compare(g, m, step);
            }
        }
        compare(g, m, -1);
    }

    private static void compare(SpreadsheetGrid g, Model m, int step) {
        int maxR = 0, maxC = 0, minR = Integer.MAX_VALUE, minC = Integer.MAX_VALUE;
        for (int r = 1; r <= 300; r++) {
            for (int c = 1; c <= 60; c++) {
                Model.Cell e = m.get(r, c);
                String a = addr(r, c);
                String want = e == null || (e.text() == null && e.number() == null)
                        ? ""
                        : e.text() != null ? e.text() : String.valueOf(e.number().longValue());
                assertThat(g.getValue(a)).as("step %d cell %s", step, a).isEqualTo(want);
                assertThat(g.styleOf(a).startsWith("true|")).as("step %d bold %s", step, a).isEqualTo(e != null && e.bold());
                if (e != null) {
                    maxR = Math.max(maxR, r);
                    maxC = Math.max(maxC, c);
                    minR = Math.min(minR, r);
                    minC = Math.min(minC, c);
                }
            }
        }
        String range = maxR == 0 ? "" : addr(minR, minC) + ":" + addr(maxR, maxC);
        assertThat(g.usedRange()).as("step %d", step).isEqualTo(range);
        assertThat(g.rowCount()).isEqualTo(maxR);
        assertThat(g.columnCount()).isEqualTo(maxC);
    }

    @Test
    void shouldClearAHugeRangeInTimeProportionalToTheCellsPresent() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        for (int r = 1; r <= 2_000; r++) {
            for (int c = 1; c <= 100; c++) {
                g.setNumber(addr(r, c), 1);
            }
        }
        long start = System.nanoTime();
        for (int i = 0; i < 200; i++) {
            g.clearRange("ZZ999999", "ZZ1000000");
            g.clearRange("DA1", "ZZ1000000");
        }
        g.clearRange("A1", "ZZ1000000");
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).isLessThan(5_000);
        assertThat(g.usedRange()).isEmpty();
    }

    @Test
    void shouldSortFiftyThousandRowsStablyAndKeepStylesWithTheirRows() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        int n = 50_000;
        for (int r = 1; r <= n; r++) {
            int key = (int) ((r * 7919L) % 1000);
            if (r % 10 == 0) {
                g.setText("A" + r, "k" + (key % 100));
            } else if (r % 10 != 1) {
                g.setNumber("A" + r, key);
            }
            g.setNumber("B" + r, r);
            if (r % 3 == 0) {
                g.setStyle("B" + r, bold(true));
            }
        }

        g.sortRange("A1", "B" + n, "A", true);

        int previousKind = 0;
        double previousNumber = Double.NEGATIVE_INFINITY;
        int previousPayload = 0;
        for (int r = 1; r <= n; r++) {
            ValueType t = g.valueType("A" + r);
            int kind = t == ValueType.NUMBER ? 0 : t == ValueType.TEXT ? 1 : 2;
            assertThat(kind).isGreaterThanOrEqualTo(previousKind);
            int payload = Integer.parseInt(g.getValue("B" + r));
            assertThat(g.styleOf("B" + r).startsWith("true|")).isEqualTo(payload % 3 == 0);
            if (kind == 0) {
                double v = Double.parseDouble(g.getValue("A" + r));
                assertThat(v).isGreaterThanOrEqualTo(previousNumber);
                if (v == previousNumber) {
                    assertThat(payload).isGreaterThan(previousPayload);
                }
                previousNumber = v;
            } else if (kind == 2 && previousKind == 2) {
                assertThat(payload).isGreaterThan(previousPayload);
            }
            previousKind = kind;
            previousPayload = payload;
        }
    }

    @Test
    void shouldKeepAConsistentGridWhenReadersRunDuringStructureChanges() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        g.setText("A1", "anchor");
        g.setNumber("B2", 5);

        Concurrently.run(8, 60_000, t -> {
            if (t < 4) {
                for (int i = 0; i < 40; i++) {
                    g.insertRow(1);
                    g.insertColumn("A");
                }
            } else {
                for (int i = 0; i < 2_000; i++) {
                    String used = g.usedRange();
                    assertThat(used).isNotEmpty();
                    assertThat(g.rowCount()).isGreaterThanOrEqualTo(2);
                    g.getValue("C3");
                    g.distinctStyleCount();
                }
            }
        });

        assertThat(g.getValue(addr(161, 161))).isEqualTo("anchor");
        assertThat(g.getValue(addr(162, 162))).isEqualTo("5");
        assertThat(g.usedRange()).isEqualTo(addr(161, 161) + ":" + addr(162, 162));
    }

    @Test
    void shouldHandleTheGridCornersWhenStructureChanges() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        g.setText("ZZ1000000", "corner");

        g.insertRow(1);
        assertThat(g.usedRange()).isEmpty(); // pushed past the last row

        g.setText("ZZ1000000", "corner");
        g.deleteRow(1);
        assertThat(g.getValue("ZZ999999")).isEqualTo("corner");
        g.insertRow(1);
        assertThat(g.getValue("ZZ1000000")).isEqualTo("corner");

        g.deleteColumn("A");
        assertThat(g.getValue("ZY1000000")).isEqualTo("corner");
        g.insertColumn("ZY");
        assertThat(g.getValue("ZZ1000000")).isEqualTo("corner");
        g.insertColumn("ZZ");
        assertThat(g.usedRange()).isEmpty();

        g.setStyle("ZZ1", bold(true));
        g.deleteColumn("zz");
        assertThat(g.distinctStyleCount()).isZero();
        assertThat(g.usedRange()).isEmpty();
    }

    @Test
    void shouldFormatLargeAndTinyNumbersWithoutExponents() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        g.setNumber("A1", 1e20);
        g.setNumber("A2", -123456789012345678.0);
        g.setNumber("A3", 1.0E-10 / 3);
        g.setNumber("A4", 12345.678901234567);
        g.setNumber("A5", 0.99999999995);
        g.setNumber("A6", 999.9999999999);

        assertThat(g.getValue("A1")).isEqualTo("100000000000000000000");
        assertThat(g.getValue("A2")).isEqualTo("-123456789012345680");
        assertThat(g.getValue("A3")).isEqualTo("0");
        assertThat(g.getValue("A4")).isEqualTo("12345.6789012346");
        assertThat(g.getValue("A5")).isEqualTo("1");
        assertThat(g.getValue("A6")).isEqualTo("999.9999999999");

        g.setStyleRange("A1", "A6", new StyleChange(null, null, null, null, null, "PERCENT"));
        assertThat(g.getValue("A5")).isEqualTo("100%");
        assertThat(g.getValue("A1")).isEqualTo("10000000000000000000000%");
        g.setStyleRange("A1", "A6", new StyleChange(null, null, null, null, null, "TWO_DECIMALS"));
        assertThat(g.getValue("A6")).isEqualTo("1000.00");
        assertThat(g.getValue("A3")).isEqualTo("0.00");
    }

    @Test
    void shouldCopyAcrossStructureChangesWithoutSharingState() {
        SpreadsheetGrid g = Candidate.of(Solution::create);
        g.setNumber("A1", 1);
        g.setStyle("A1", new StyleChange(true, null, null, "blue", null, null));
        g.copy("A1", "A1", "B1");
        g.setStyle("A1", new StyleChange(false, null, null, null, null, null));
        g.setNumber("B1", 2);

        assertThat(g.styleOf("A1")).isEqualTo("false|false|11|blue|none|GENERAL");
        assertThat(g.styleOf("B1")).isEqualTo("true|false|11|blue|none|GENERAL");
        assertThat(g.distinctStyleCount()).isEqualTo(2);

        g.insertColumn("A");
        g.copy("B1", "C1", "A5");
        assertThat(g.getValue("A5")).isEqualTo("1");
        assertThat(g.getValue("B5")).isEqualTo("2");
        g.deleteColumn("B");
        assertThat(g.getValue("A5")).isEqualTo("1");
        assertThat(g.getValue("B5")).isEmpty();
        List<String> used = new ArrayList<>();
        used.add(g.usedRange());
        assertThat(used).containsExactly("A1:B5");
    }
}

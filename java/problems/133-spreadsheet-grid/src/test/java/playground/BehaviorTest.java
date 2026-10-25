package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.SpreadsheetGrid;
import contract.StyleChange;
import contract.ValueType;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String DEFAULT_STYLE = "false|false|11|black|none|GENERAL";

    private SpreadsheetGrid grid() {
        return Candidate.of(Solution::create);
    }

    private static StyleChange bold() {
        return new StyleChange(true, null, null, null, null, null);
    }

    private static StyleChange format(String f) {
        return new StyleChange(null, null, null, null, null, f);
    }

    private static void assertRejected(Runnable r) {
        assertThatThrownBy(r::run).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldStartEmptyWithDefaultStyle() {
        SpreadsheetGrid g = grid();

        assertThat(g.getValue("A1")).isEmpty();
        assertThat(g.valueType("A1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("ZZ1000000")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.usedRange()).isEmpty();
        assertThat(g.rowCount()).isZero();
        assertThat(g.columnCount()).isZero();
        assertThat(g.distinctStyleCount()).isZero();
    }

    @Test
    void shouldStoreNumbersAndTextWithCaseInsensitiveAddresses() {
        SpreadsheetGrid g = grid();
        g.setNumber("b3", 42);

        assertThat(g.getValue("B3")).isEqualTo("42");
        assertThat(g.valueType("b3")).isEqualTo(ValueType.NUMBER);

        g.setText("B3", "hello");
        assertThat(g.getValue("b3")).isEqualTo("hello");
        assertThat(g.valueType("B3")).isEqualTo(ValueType.TEXT);

        g.setText("C1", "42");
        assertThat(g.valueType("C1")).isEqualTo(ValueType.TEXT);
        assertThat(g.getValue("C1")).isEqualTo("42");
    }

    @Test
    void shouldRejectInvalidAddresses() {
        SpreadsheetGrid g = grid();
        for (String bad : new String[] {
            "", "A", "1", "A0", "A01", "AAA1", "A1000001", "A 1", "A1 ", " A1", "1A", "A-1", "A1.5", "$A$1", "A1:B2", null
        }) {
            assertRejected(() -> g.setNumber(bad, 1));
            assertRejected(() -> g.getValue(bad));
            assertRejected(() -> g.styleOf(bad));
        }
        g.setNumber("ZZ1000000", 1);
        g.setNumber("a1", 1);
        assertThat(g.rowCount()).isEqualTo(1_000_000);
        assertThat(g.columnCount()).isEqualTo(702);
    }

    @Test
    void shouldRejectBadValuesAndKeepTheOldOne() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 5);

        assertRejected(() -> g.setNumber("A1", Double.NaN));
        assertRejected(() -> g.setNumber("A1", Double.POSITIVE_INFINITY));
        assertRejected(() -> g.setNumber("A1", Double.NEGATIVE_INFINITY));
        assertRejected(() -> g.setText("A1", null));
        assertThat(g.getValue("A1")).isEqualTo("5");
    }

    @Test
    void shouldDisplayGeneralNumbersWithUpToTenDecimals() {
        SpreadsheetGrid g = grid();
        double[] in = {1.5, 2.0, 0.1 + 0.2, 1e15, 1.23456789012, 0.00000000004, 0.00000000005, -2.5, 1e-10, 123456789.125, -0.0, 100, -0.00000000001};
        String[] out = {"1.5", "2", "0.3", "1000000000000000", "1.2345678901", "0", "0.0000000001", "-2.5", "0.0000000001", "123456789.125", "0", "100", "0"};
        for (int i = 0; i < in.length; i++) {
            g.setNumber("A" + (i + 1), in[i]);
            assertThat(g.getValue("A" + (i + 1))).as("value %s", in[i]).isEqualTo(out[i]);
        }
    }

    @Test
    void shouldApplyIntegerFormatRoundingHalfAwayFromZero() {
        SpreadsheetGrid g = grid();
        g.setStyleRange("A1", "A6", format("INTEGER"));
        double[] in = {2.5, -2.5, 2.4, -0.4, 1234.5678, 0.5};
        String[] out = {"3", "-3", "2", "0", "1235", "1"};
        for (int i = 0; i < in.length; i++) {
            g.setNumber("A" + (i + 1), in[i]);
            assertThat(g.getValue("A" + (i + 1))).as("value %s", in[i]).isEqualTo(out[i]);
        }
    }

    @Test
    void shouldApplyTwoDecimalsAndPercentFormats() {
        SpreadsheetGrid g = grid();
        g.setStyleRange("A1", "A6", format("two_decimals"));
        g.setStyleRange("B1", "B7", format("PERCENT"));
        double[] two = {3, 1.005, 2.345, -0.001, -1.005, 1234.5};
        String[] twoOut = {"3.00", "1.01", "2.35", "0.00", "-1.01", "1234.50"};
        for (int i = 0; i < two.length; i++) {
            g.setNumber("A" + (i + 1), two[i]);
            assertThat(g.getValue("A" + (i + 1))).as("two decimals of %s", two[i]).isEqualTo(twoOut[i]);
        }
        double[] pct = {0.256, 0.5, 1, 0.005, -0.5, 0.001, 12};
        String[] pctOut = {"26%", "50%", "100%", "1%", "-50%", "0%", "1200%"};
        for (int i = 0; i < pct.length; i++) {
            g.setNumber("B" + (i + 1), pct[i]);
            assertThat(g.getValue("B" + (i + 1))).as("percent of %s", pct[i]).isEqualTo(pctOut[i]);
        }
    }

    @Test
    void shouldRenderStoredNumbersAgainWhenTheFormatChangesAndLeaveTextAlone() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 0.256);
        g.setText("A2", "0.256");

        g.setStyleRange("A1", "A2", format("PERCENT"));
        assertThat(g.getValue("A1")).isEqualTo("26%");
        assertThat(g.getValue("A2")).isEqualTo("0.256");

        g.setStyle("A1", format("GENERAL"));
        assertThat(g.getValue("A1")).isEqualTo("0.256");
    }

    @Test
    void shouldMergePartialStyleChangesAndNormaliseNames() {
        SpreadsheetGrid g = grid();
        g.setStyle("B2", bold());
        assertThat(g.styleOf("B2")).isEqualTo("true|false|11|black|none|GENERAL");

        g.setStyle("b2", new StyleChange(null, true, 14, "RED", "Light-Blue", "percent"));
        assertThat(g.styleOf("B2")).isEqualTo("true|true|14|red|light-blue|PERCENT");

        g.setStyle("B2", new StyleChange(false, null, null, null, "none", null));
        assertThat(g.styleOf("B2")).isEqualTo("false|true|14|red|none|PERCENT");

        g.setStyle("B2", StyleChange.none());
        assertThat(g.styleOf("B2")).isEqualTo("false|true|14|red|none|PERCENT");
        assertThat(g.styleOf("B3")).isEqualTo(DEFAULT_STYLE);
    }

    @Test
    void shouldRejectInvalidStyleChangesWithoutPartialEffects() {
        SpreadsheetGrid g = grid();
        g.setStyle("A1", bold());

        assertRejected(() -> g.setStyle("A1", new StyleChange(false, true, 7, null, null, null)));
        assertRejected(() -> g.setStyle("A1", new StyleChange(false, true, 73, null, null, null)));
        assertRejected(() -> g.setStyle("A1", new StyleChange(false, null, null, "dark red", null, null)));
        assertRejected(() -> g.setStyle("A1", new StyleChange(false, null, null, null, "a|b", null)));
        assertRejected(() -> g.setStyle("A1", new StyleChange(false, null, null, "", null, null)));
        assertRejected(() -> g.setStyle("A1", new StyleChange(false, null, null, null, null, "SCIENTIFIC")));
        assertRejected(() -> g.setStyle("A1", null));
        assertRejected(() -> g.setStyle("A0", bold()));
        assertRejected(() -> g.setStyleRange("A1", "B2", new StyleChange(true, true, 99, null, null, null)));

        assertThat(g.styleOf("A1")).isEqualTo("true|false|11|black|none|GENERAL");
        assertThat(g.styleOf("B2")).isEqualTo(DEFAULT_STYLE);
        g.setStyle("A1", new StyleChange(null, null, 8, null, null, null));
        g.setStyle("A1", new StyleChange(null, null, 72, null, null, null));
        assertThat(g.styleOf("A1")).isEqualTo("true|false|72|black|none|GENERAL");
    }

    @Test
    void shouldCountAStyledEmptyCellAsUsedUntilItBecomesDefaultAgain() {
        SpreadsheetGrid g = grid();
        g.setStyle("C3", bold());

        assertThat(g.getValue("C3")).isEmpty();
        assertThat(g.valueType("C3")).isEqualTo(ValueType.EMPTY);
        assertThat(g.usedRange()).isEqualTo("C3:C3");

        g.setStyle("C3", new StyleChange(false, null, null, null, null, null));
        assertThat(g.usedRange()).isEmpty();
        assertThat(g.distinctStyleCount()).isZero();
    }

    @Test
    void shouldClearValueAndStyleButEmptyTextKeepsTheStyle() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 7);
        g.setStyle("A1", bold());
        g.setNumber("A2", 8);
        g.setStyle("A2", bold());

        g.setText("A1", "");
        assertThat(g.valueType("A1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("A1")).startsWith("true|");

        g.clear("A2");
        assertThat(g.valueType("A2")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("A2")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.usedRange()).isEqualTo("A1:A1");
    }

    @Test
    void shouldCountDistinctNonDefaultStylesStillInUse() {
        SpreadsheetGrid g = grid();
        g.setStyleRange("A1", "A100", bold());
        assertThat(g.distinctStyleCount()).isEqualTo(1);

        g.setStyle("B1", new StyleChange(true, true, null, null, null, null));
        assertThat(g.distinctStyleCount()).isEqualTo(2);

        g.setStyle("C1", bold());
        assertThat(g.distinctStyleCount()).isEqualTo(2);

        g.clear("B1");
        assertThat(g.distinctStyleCount()).isEqualTo(1);

        g.setStyleRange("A1", "A100", new StyleChange(false, null, null, null, null, null));
        assertThat(g.distinctStyleCount()).isEqualTo(1); // C1 is still bold

        g.setStyle("C1", new StyleChange(false, null, null, null, null, null));
        assertThat(g.distinctStyleCount()).isZero();
    }

    @Test
    void shouldStyleEveryCellOfARectangleAndKeepValues() {
        SpreadsheetGrid g = grid();
        g.setNumber("B2", 1.5);
        g.setStyleRange("B2", "D4", new StyleChange(null, true, 20, null, "yellow", null));

        assertThat(g.styleOf("B2")).isEqualTo("false|true|20|black|yellow|GENERAL");
        assertThat(g.styleOf("D4")).isEqualTo("false|true|20|black|yellow|GENERAL");
        assertThat(g.styleOf("E4")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.styleOf("A1")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.getValue("B2")).isEqualTo("1.5");
        assertThat(g.usedRange()).isEqualTo("B2:D4");
        assertThat(g.distinctStyleCount()).isEqualTo(1);
    }

    @Test
    void shouldRejectReversedAndOversizedRanges() {
        SpreadsheetGrid g = grid();

        assertRejected(() -> g.setStyleRange("B2", "A2", bold()));
        assertRejected(() -> g.setStyleRange("B2", "B1", bold()));
        assertRejected(() -> g.setStyleRange("A1", "B500001", bold())); // 1,000,002 cells: over the limit
        assertRejected(() -> g.fillNumber("A1", "B500001", 1));
        assertRejected(() -> g.fillText("A1", "ZZ1000000", "x"));
        assertRejected(() -> g.clearRange("B2", "A1"));
        assertThat(g.usedRange()).isEmpty();
    }

    @Test
    void shouldFillValuesAndKeepStyles() {
        SpreadsheetGrid g = grid();
        g.setStyle("B2", bold());
        g.fillNumber("A1", "C3", 2.5);

        assertThat(g.getValue("A1")).isEqualTo("2.5");
        assertThat(g.getValue("C3")).isEqualTo("2.5");
        assertThat(g.getValue("D3")).isEmpty();
        assertThat(g.styleOf("B2")).startsWith("true|");

        g.fillText("B2", "C3", "x");
        assertThat(g.getValue("B2")).isEqualTo("x");
        assertThat(g.valueType("C2")).isEqualTo(ValueType.TEXT);
        assertThat(g.getValue("A1")).isEqualTo("2.5");

        g.fillText("A1", "C3", "");
        assertThat(g.usedRange()).isEqualTo("B2:B2"); // only the styled cell remains
        assertRejected(() -> g.fillNumber("A1", "A2", Double.NaN));
        assertRejected(() -> g.fillText("A1", "A2", null));
    }

    @Test
    void shouldClearOnlyTheRangeIncludingStyles() {
        SpreadsheetGrid g = grid();
        g.fillNumber("A1", "D4", 1);
        g.setStyleRange("A1", "D4", bold());

        g.clearRange("B2", "C3");

        assertThat(g.valueType("B2")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("C3")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.getValue("A1")).isEqualTo("1");
        assertThat(g.getValue("D4")).isEqualTo("1");
        assertThat(g.getValue("B1")).isEqualTo("1");
        assertThat(g.styleOf("B1")).startsWith("true|");
        assertThat(g.usedRange()).isEqualTo("A1:D4");
    }

    @Test
    void shouldCopyValuesAndStylesAndOverwriteTheDestination() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setStyle("A1", new StyleChange(true, null, null, "red", null, "TWO_DECIMALS"));
        g.setText("B1", "x");
        // A2 stays empty; the destination cells below hold data that must be wiped by the empty source cells
        g.fillNumber("E5", "F6", 9);
        g.setStyleRange("E5", "F6", new StyleChange(null, true, null, null, null, null));

        g.copy("A1", "B2", "E5");

        assertThat(g.getValue("E5")).isEqualTo("1.00");
        assertThat(g.styleOf("E5")).isEqualTo("true|false|11|red|none|TWO_DECIMALS");
        assertThat(g.getValue("F5")).isEqualTo("x");
        assertThat(g.valueType("E6")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("F6")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.getValue("A1")).isEqualTo("1.00");
        assertThat(g.getValue("B1")).isEqualTo("x");
    }

    @Test
    void shouldCopyOverlappingRangesAsIfThroughATemporaryBuffer() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("A2", 2);
        g.setNumber("A3", 3);
        g.copy("A1", "A3", "A2");
        assertThat(List.of(g.getValue("A1"), g.getValue("A2"), g.getValue("A3"), g.getValue("A4")))
                .containsExactly("1", "1", "2", "3");

        g.copy("A2", "A4", "A1");
        assertThat(List.of(g.getValue("A1"), g.getValue("A2"), g.getValue("A3"), g.getValue("A4")))
                .containsExactly("1", "2", "3", "3");

        SpreadsheetGrid h = grid();
        h.setText("A1", "a");
        h.setText("B1", "b");
        h.setText("A2", "c");
        h.setText("B2", "d");
        h.copy("A1", "B2", "B2");
        assertThat(List.of(h.getValue("B2"), h.getValue("C2"), h.getValue("B3"), h.getValue("C3")))
                .containsExactly("a", "b", "c", "d");
        assertThat(h.getValue("A2")).isEqualTo("c");
    }

    @Test
    void shouldRejectACopyThatLeavesTheGridAndChangeNothing() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("B1", 2);

        assertRejected(() -> g.copy("A1", "A2", "A1000000"));
        assertRejected(() -> g.copy("A1", "B1", "ZZ1"));
        assertRejected(() -> g.move("A1", "B1", "ZZ1"));
        assertRejected(() -> g.copy("B1", "A1", "C1"));
        assertRejected(() -> g.copy("A1", "A1", "bad"));

        assertThat(g.usedRange()).isEqualTo("A1:B1");
        assertThat(g.getValue("A1")).isEqualTo("1");
        g.copy("A1", "A2", "A999999");
    }

    @Test
    void shouldMoveCellsAndClearTheSourceOutsideTheDestination() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("B1", 2);
        g.setNumber("A2", 3);
        g.setNumber("B2", 4);
        g.setStyle("A1", bold());

        g.move("A1", "B2", "B2"); // destination B2:C3 overlaps the source in B2

        assertThat(g.getValue("B2")).isEqualTo("1");
        assertThat(g.styleOf("B2")).startsWith("true|");
        assertThat(g.getValue("C2")).isEqualTo("2");
        assertThat(g.getValue("B3")).isEqualTo("3");
        assertThat(g.getValue("C3")).isEqualTo("4");
        assertThat(g.valueType("A1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.styleOf("A1")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.valueType("B1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.valueType("A2")).isEqualTo(ValueType.EMPTY);
        assertThat(g.usedRange()).isEqualTo("B2:C3");
    }

    @Test
    void shouldInsertRowsShiftingValuesAndStyles() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("A2", 2);
        g.setNumber("B3", 3);
        g.setStyle("B3", bold());

        g.insertRow(2);

        assertThat(g.getValue("A1")).isEqualTo("1");
        assertThat(g.valueType("A2")).isEqualTo(ValueType.EMPTY);
        assertThat(g.getValue("A3")).isEqualTo("2");
        assertThat(g.getValue("B4")).isEqualTo("3");
        assertThat(g.styleOf("B4")).startsWith("true|");
        assertThat(g.styleOf("B3")).isEqualTo(DEFAULT_STYLE);
        assertThat(g.rowCount()).isEqualTo(4);
        assertRejected(() -> g.insertRow(0));
        assertRejected(() -> g.insertRow(1_000_001));
    }

    @Test
    void shouldDropCellsPushedPastTheLastRowOrColumn() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1000000", 1);
        g.setNumber("A999999", 2);
        g.setNumber("ZZ1", 3);
        g.setNumber("ZY2", 4);

        g.insertRow(1);
        assertThat(g.getValue("A1000000")).isEqualTo("2");
        assertThat(g.getValue("ZZ2")).isEqualTo("3");
        assertThat(g.rowCount()).isEqualTo(1_000_000);

        g.insertColumn("A");
        assertThat(g.getValue("ZZ2")).isEmpty();
        assertThat(g.getValue("ZZ3")).isEqualTo("4");
        assertThat(g.getValue("B1000000")).isEqualTo("2");
        assertThat(g.columnCount()).isEqualTo(702);
    }

    @Test
    void shouldDeleteRowsAndMakeTheirCellsVanish() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("A2", 2);
        g.setNumber("A3", 3);
        g.setNumber("C3", 33);

        g.deleteRow(2);

        assertThat(g.getValue("A1")).isEqualTo("1");
        assertThat(g.getValue("A2")).isEqualTo("3");
        assertThat(g.getValue("C2")).isEqualTo("33");
        assertThat(g.getValue("A3")).isEmpty();
        assertThat(g.usedRange()).isEqualTo("A1:C2");

        g.deleteRow(500); // deleting an empty row still closes the gap and changes nothing here
        assertThat(g.getValue("A2")).isEqualTo("3");
        assertRejected(() -> g.deleteRow(0));
        assertRejected(() -> g.deleteRow(1_000_001));
    }

    @Test
    void shouldInsertAndDeleteColumns() {
        SpreadsheetGrid g = grid();
        g.setNumber("A1", 1);
        g.setNumber("B1", 2);
        g.setNumber("C1", 3);
        g.setStyle("C1", bold());

        g.insertColumn("b");
        assertThat(g.getValue("A1")).isEqualTo("1");
        assertThat(g.valueType("B1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.getValue("C1")).isEqualTo("2");
        assertThat(g.getValue("D1")).isEqualTo("3");
        assertThat(g.styleOf("D1")).startsWith("true|");
        assertThat(g.columnCount()).isEqualTo(4);

        g.deleteColumn("A");
        assertThat(g.valueType("A1")).isEqualTo(ValueType.EMPTY);
        assertThat(g.getValue("B1")).isEqualTo("2");
        assertThat(g.getValue("C1")).isEqualTo("3");
        assertThat(g.usedRange()).isEqualTo("B1:C1");

        for (String bad : new String[] {"", "A1", "AAA", "1", "A$", null}) {
            assertRejected(() -> g.insertColumn(bad));
            assertRejected(() -> g.deleteColumn(bad));
        }
    }

    @Test
    void shouldReportTheUsedRangeAndCounts() {
        SpreadsheetGrid g = grid();
        g.setNumber("B2", 1);
        g.setText("D7", "x");

        assertThat(g.usedRange()).isEqualTo("B2:D7");
        assertThat(g.rowCount()).isEqualTo(7);
        assertThat(g.columnCount()).isEqualTo(4);

        g.setStyle("AA9", bold());
        assertThat(g.usedRange()).isEqualTo("B2:AA9");
        assertThat(g.columnCount()).isEqualTo(27);
        assertThat(g.rowCount()).isEqualTo(9);

        g.clearRange("A1", "ZZ1000000");
        assertThat(g.usedRange()).isEmpty();
        assertThat(g.rowCount()).isZero();
        assertThat(g.columnCount()).isZero();
    }

    private static String column(SpreadsheetGrid g, String col, int from, int to) {
        StringBuilder sb = new StringBuilder();
        for (int r = from; r <= to; r++) {
            sb.append(r == from ? "" : ",").append(g.getValue(col + r));
        }
        return sb.toString();
    }

    @Test
    void shouldSortNumbersBeforeTextBeforeEmptyAndKeepRowsTogether() {
        SpreadsheetGrid g = grid();
        // key column A, payload column B
        g.setText("A1", "banana");
        g.setText("B1", "r1");
        g.setNumber("A2", 10);
        g.setText("B2", "r2");
        g.setText("B3", "r3"); // empty key
        g.setText("A4", "Apple");
        g.setText("B4", "r4");
        g.setNumber("A5", -3.5);
        g.setText("B5", "r5");
        g.setStyle("B5", bold());
        g.setText("A6", "cherry");
        g.setText("B6", "r6");

        g.sortRange("A1", "B6", "A", true);

        assertThat(column(g, "A", 1, 6)).isEqualTo("-3.5,10,Apple,banana,cherry,");
        assertThat(column(g, "B", 1, 6)).isEqualTo("r5,r2,r4,r1,r6,r3");
        assertThat(g.styleOf("B1")).startsWith("true|");
        assertThat(g.styleOf("B5")).isEqualTo(DEFAULT_STYLE);
    }

    @Test
    void shouldSortStablyAndDescendWithinGroupsOnly() {
        SpreadsheetGrid g = grid();
        String[] keys = {"b", "A", "a", "2", "1", "", "B", "x"};
        for (int i = 0; i < keys.length; i++) {
            int row = i + 1;
            if (keys[i].equals("2") || keys[i].equals("1")) {
                g.setNumber("A" + row, Double.parseDouble(keys[i]));
            } else if (!keys[i].isEmpty()) {
                g.setText("A" + row, keys[i]);
            }
            g.setNumber("B" + row, row);
        }

        g.sortRange("A1", "B8", "A", true);
        // numbers 1,2 ; text a/A (stable: A before a), b/B (stable: b before B), x ; empty
        assertThat(column(g, "A", 1, 8)).isEqualTo("1,2,A,a,b,B,x,");
        assertThat(column(g, "B", 1, 8)).isEqualTo("5,4,2,3,1,7,8,6");

        g.sortRange("A1", "B8", "A", false);
        // numbers 2,1 ; text x, then b/B tie in current order (b,B), then A/a tie (A,a) ; empty last
        assertThat(column(g, "A", 1, 8)).isEqualTo("2,1,x,b,B,A,a,");
        assertThat(column(g, "B", 1, 8)).isEqualTo("4,5,8,1,7,2,3,6");
    }

    @Test
    void shouldSortOnlyTheGivenRowsAndColumnsAndRejectBadKeys() {
        SpreadsheetGrid g = grid();
        for (int r = 1; r <= 4; r++) {
            g.setNumber("B" + r, 5 - r);
            g.setNumber("A" + r, 100 + r);
            g.setNumber("C" + r, 200 + r);
        }

        g.sortRange("B2", "C4", "B", true);

        assertThat(column(g, "A", 1, 4)).isEqualTo("101,102,103,104");
        assertThat(column(g, "B", 1, 4)).isEqualTo("4,1,2,3");
        assertThat(column(g, "C", 1, 4)).isEqualTo("201,204,203,202");

        assertRejected(() -> g.sortRange("B2", "C4", "A", true));
        assertRejected(() -> g.sortRange("B2", "C4", "D", true));
        assertRejected(() -> g.sortRange("B2", "C4", "B1", true));
        assertRejected(() -> g.sortRange("C4", "B2", "B", true));
        assertThat(column(g, "B", 1, 4)).isEqualTo("4,1,2,3");
    }

    @Test
    void shouldKeepEveryConcurrentWriteToDistinctCells() {
        SpreadsheetGrid g = grid();
        int threads = 8, per = 1_000;

        Concurrently.run(threads, t -> {
            for (int i = 1; i <= per; i++) {
                g.setNumber(colName(t + 1) + i, t * 10_000 + i);
            }
        });

        for (int t = 0; t < threads; t++) {
            for (int i = 1; i <= per; i += 97) {
                assertThat(g.getValue(colName(t + 1) + i)).isEqualTo(String.valueOf(t * 10_000 + i));
            }
        }
        assertThat(g.rowCount()).isEqualTo(per);
        assertThat(g.columnCount()).isEqualTo(threads);
        assertThat(g.usedRange()).isEqualTo("A1:H1000");
    }

    @Test
    void shouldApplyConcurrentRowInsertionsWithoutLosingTheCell() {
        SpreadsheetGrid g = grid();
        g.setText("A1", "anchor");

        Concurrently.run(8, t -> {
            for (int i = 0; i < 50; i++) {
                g.insertRow(1);
            }
        });

        assertThat(g.getValue("A401")).isEqualTo("anchor");
        assertThat(g.usedRange()).isEqualTo("A401:A401");
    }

    @Test
    void shouldHandleTwoHundredThousandCellsAndAThousandStructureOperations() {
        SpreadsheetGrid g = grid();
        for (int r = 1; r <= 2_000; r++) {
            for (int c = 1; c <= 100; c++) {
                g.setNumber(colName(c) + r, r * 1000 + c);
            }
        }
        long start = System.nanoTime();
        for (int i = 0; i < 1_000; i++) {
            switch (i % 4) {
                case 0 -> g.insertRow(1_500 + i % 50);
                case 1 -> g.insertColumn(colName(80 + i % 10));
                case 2 -> g.deleteRow(1_800 + i % 50);
                default -> g.deleteColumn(colName(85 + i % 10));
            }
        }
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).as("1,000 structure operations on 200k cells (ms)").isLessThan(30_000);
        assertThat(g.getValue("A1")).isEqualTo("1001");
        assertThat(g.getValue("B1000")).isEqualTo("1000002");
        assertThat(g.usedRange()).startsWith("A1:");
    }

    private static String colName(int c) {
        return c > 26 ? "" + (char) ('A' + (c - 1) / 26 - 1) + (char) ('A' + (c - 1) % 26) : "" + (char) ('A' + c - 1);
    }
}

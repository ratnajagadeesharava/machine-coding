package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FormulaSheet;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private FormulaSheet sheet() {
        return Candidate.of(Solution::create);
    }

    /** Sets cell/input pairs: put(s, "A1", "5", "B1", "=A1+1"). */
    private static void put(FormulaSheet s, String... cellAndInput) {
        for (int i = 0; i < cellAndInput.length; i += 2) {
            s.set(cellAndInput[i], cellAndInput[i + 1]);
        }
    }

    private static String cellName(int index) {
        return "" + (char) ('A' + index % 26) + (index / 26 + 1);
    }

    /** Runs the body on a thread with the default stack size and rethrows what it threw. */
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
    void shouldStoreNumbersTextAndEmptyCells() {
        FormulaSheet s = sheet();
        put(s, "A1", "5", "B1", "hello", "C1", "3.50", "D1", "-0", "E1", "+7", "F1", "007", "G1", "-2.25");

        assertThat(s.get("A1")).isEqualTo("5");
        assertThat(s.get("B1")).isEqualTo("hello");
        assertThat(s.get("C1")).isEqualTo("3.5");
        assertThat(s.get("D1")).isEqualTo("0");
        assertThat(s.get("E1")).isEqualTo("7");
        assertThat(s.get("F1")).isEqualTo("7");
        assertThat(s.get("G1")).isEqualTo("-2.25");
        assertThat(s.get("H1")).isEmpty();
        assertThat(s.formulaOf("A1")).isEmpty();
    }

    @Test
    void shouldKeepInputThatIsNotAnExactNumberLiteralAsText() {
        FormulaSheet s = sheet();
        String[] texts = {"1e3", "5.", ".5", " 5", "5 ", "--5", "1,000", "1.2.3", "+", "-", "12abc", "A1+1", "'5"};
        for (int i = 0; i < texts.length; i++) {
            s.set(cellName(i), texts[i]);
        }
        for (int i = 0; i < texts.length; i++) {
            assertThat(s.get(cellName(i))).as(texts[i]).isEqualTo(texts[i]);
        }
        s.set("A2", "= not a formula");
        assertThat(s.get("A2")).isEqualTo("#ERR!");
        s.set("A3", " =1+1");
        assertThat(s.get("A3")).isEqualTo(" =1+1");
        assertThat(s.formulaOf("A3")).isEmpty();
    }

    @Test
    void shouldNormaliseAddressesToUpperCase() {
        FormulaSheet s = sheet();
        s.set("b2", "7");
        s.set("c3", "=b2*2");

        assertThat(s.get("B2")).isEqualTo("7");
        assertThat(s.get("c3")).isEqualTo("14");
        assertThat(s.usedCells()).containsExactly("B2", "C3");
        assertThat(s.dependencies("c3")).containsExactly("B2");
        assertThat(s.dependents("b2")).containsExactly("C3");
    }

    @Test
    void shouldRejectInvalidAddressesAndNullInput() {
        FormulaSheet s = sheet();
        s.set("A1", "1");
        String[] bad = {"", " ", "A0", "A101", "AA1", "1A", "A", "A 1", "A01", " A1", "A1 ", "@1", "A-1", "A1.5", "Z999", null};
        for (String address : bad) {
            assertThatThrownBy(() -> s.set(address, "1")).as("set " + address).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> s.get(address)).as("get " + address).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> s.formulaOf(address)).as("formulaOf " + address).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> s.dependencies(address)).as("dependencies " + address).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> s.dependents(address)).as("dependents " + address).isInstanceOf(IllegalArgumentException.class);
        }
        assertThatThrownBy(() -> s.set("A1", null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.get("A1")).isEqualTo("1");
        assertThat(s.usedCells()).containsExactly("A1");
        assertThat(s.get("Z100")).isEmpty();
        assertThat(s.get("a1")).isEqualTo("1");
    }

    @Test
    void shouldClearACellWithAnEmptyString() {
        FormulaSheet s = sheet();
        put(s, "A1", "5", "B1", "=A1+1");
        assertThat(s.get("B1")).isEqualTo("6");

        s.set("A1", "");
        assertThat(s.get("A1")).isEmpty();
        assertThat(s.get("B1")).isEqualTo("1");
        assertThat(s.usedCells()).containsExactly("B1");

        s.set("B1", "");
        assertThat(s.get("B1")).isEmpty();
        assertThat(s.formulaOf("B1")).isEmpty();
        assertThat(s.dependencies("B1")).isEmpty();
        assertThat(s.dependents("A1")).isEmpty();
        assertThat(s.usedCells()).isEmpty();
    }

    @Test
    void shouldRenderNumbersWithoutTrailingZerosAndAtMostTenDecimals() {
        FormulaSheet s = sheet();
        String[][] cases = {
            {"=1/3", "0.3333333333"}, {"=2/3", "0.6666666667"}, {"=1/8", "0.125"}, {"=10/4", "2.5"},
            {"=0.1+0.2", "0.3"}, {"=1/3*3", "1"}, {"=0-0", "0"}, {"=-0", "0"}, {"=0*-1", "0"},
            {"=0.00000000004", "0"}, {"=0.00000000006", "0.0000000001"}, {"=-0.00000000004", "0"},
            {"=100000*100000", "10000000000"}, {"=123456789012*10", "1234567890120"}, {"=-7/2", "-3.5"},
            {"=2.50", "2.5"}, {"=1.0", "1"},
        };
        for (String[] c : cases) {
            s.set("A1", c[0]);
            assertThat(s.get("A1")).as(c[0]).isEqualTo(c[1]);
        }
    }

    @Test
    void shouldEvaluateArithmeticWithPrecedenceAndLeftAssociativity() {
        FormulaSheet s = sheet();
        String[][] cases = {
            {"=1+2*3", "7"}, {"=(1+2)*3", "9"}, {"=10-4-3", "3"}, {"=100/10/5", "2"}, {"=2*3+4*5", "26"},
            {"=-3+5", "2"}, {"=--3", "3"}, {"=2*-3", "-6"}, {"=2--3", "5"}, {"=-(1+2)*2", "-6"},
            {"=((((7))))", "7"}, {"= 1 +  2 ", "3"}, {"=\t8 / 4\t", "2"}, {"=1.5*4", "6"}, {"=2-3", "-1"},
            {"=8/2*3", "12"}, {"=2*(3+4)-5", "9"},
        };
        for (String[] c : cases) {
            s.set("A1", c[0]);
            assertThat(s.get("A1")).as(c[0]).isEqualTo(c[1]);
        }
    }

    @Test
    void shouldReadCellsAndTreatEmptyCellsAsZero() {
        FormulaSheet s = sheet();
        put(s, "A1", "5", "A2", "=A1*2", "A3", "=A2+B9", "A4", "=B9", "A5", "=B9*7");

        assertThat(s.get("A2")).isEqualTo("10");
        assertThat(s.get("A3")).isEqualTo("10");
        assertThat(s.get("A4")).isEqualTo("0");
        assertThat(s.get("A5")).isEqualTo("0");
        assertThat(s.usedCells()).containsExactly("A1", "A2", "A3", "A4", "A5");
    }

    @Test
    void shouldSumARectangleInAnyCornerOrder() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "B1", "2", "A2", "3", "B2", "4", "A3", "5", "B3", "6", "C1", "100");
        String[] ranges = {"A1:B3", "B3:A1", "A3:B1", "B1:A3"};
        for (String r : ranges) {
            s.set("D1", "=SUM(" + r + ")");
            assertThat(s.get("D1")).as(r).isEqualTo("21");
        }
        s.set("D1", "=SUM(A2:B2)");
        assertThat(s.get("D1")).isEqualTo("7");
        s.set("D1", "=SUM(B2:B2)");
        assertThat(s.get("D1")).isEqualTo("4");
        s.set("D1", "=SUM(A1:A3)+SUM(B1:B3)*2");
        assertThat(s.get("D1")).isEqualTo("33");
        s.set("D1", "=sum(a1:b2)");
        assertThat(s.get("D1")).isEqualTo("10");
    }

    @Test
    void shouldSumAListOfRangesCellsAndNumbers() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "A2", "2", "B1", "10", "C5", "0.5");

        s.set("D1", "=SUM(A1:A2, B1, 10, -2.5)");
        assertThat(s.get("D1")).isEqualTo("20.5");
        s.set("D1", "=SUM(7)");
        assertThat(s.get("D1")).isEqualTo("7");
        s.set("D1", "= SUM ( A1 , A2 : B1 , C5 ) * 2");
        assertThat(s.get("D1")).isEqualTo("29");
        s.set("D1", "=SUM(A1,A1,A1)");
        assertThat(s.get("D1")).isEqualTo("3");
        assertThat(s.dependencies("D1")).containsExactly("A1");
    }

    @Test
    void shouldIgnoreTextAndEmptyCellsInSumButNotInArithmetic() {
        FormulaSheet s = sheet();
        put(s, "A1", "4", "A2", "words", "A3", "", "A4", "6", "B1", "=SUM(A1:A4)", "B2", "=SUM(A2)", "B3", "=SUM(A2:A3)");

        assertThat(s.get("B1")).isEqualTo("10");
        assertThat(s.get("B2")).isEqualTo("0");
        assertThat(s.get("B3")).isEqualTo("0");

        put(s, "C1", "=A1+A2", "C2", "=A2*2", "C3", "=-A2", "C4", "=SUM(A1:A4)+A2");
        assertThat(s.get("C1")).isEqualTo("#VALUE!");
        assertThat(s.get("C2")).isEqualTo("#VALUE!");
        assertThat(s.get("C3")).isEqualTo("#VALUE!");
        assertThat(s.get("C4")).isEqualTo("#VALUE!");
    }

    @Test
    void shouldCopyTextThroughABareReferenceButFailOnArithmetic() {
        FormulaSheet s = sheet();
        put(s, "A1", "hello", "B1", "=A1", "B2", "=(A1)", "B3", "=B1+1", "B4", "=SUM(B1:B2)", "B5", "=Z9");

        assertThat(s.get("B1")).isEqualTo("hello");
        assertThat(s.get("B2")).isEqualTo("hello");
        assertThat(s.get("B3")).isEqualTo("#VALUE!");
        assertThat(s.get("B4")).isEqualTo("0");
        assertThat(s.get("B5")).isEqualTo("0");
        s.set("A1", "12");
        assertThat(s.get("B1")).isEqualTo("12");
        assertThat(s.get("B3")).isEqualTo("13");
        assertThat(s.get("B4")).isEqualTo("24");
    }

    @Test
    void shouldReportDivisionByZeroAndPropagateErrors() {
        FormulaSheet s = sheet();
        put(s, "A1", "=1/0", "A2", "=0/0", "A3", "=1/B9", "A4", "=5/(2-2)", "A5", "=A1+1", "A6", "=SUM(A1:A2)",
                "A7", "=-A1", "A8", "=2*A5", "A9", "=1/(A1+1)");

        for (int i = 1; i <= 9; i++) {
            assertThat(s.get("A" + i)).as("A" + i).isEqualTo("#DIV/0!");
        }
        s.set("B1", "2");
        s.set("A1", "=1/B1");
        assertThat(s.get("A1")).isEqualTo("0.5");
        assertThat(s.get("A5")).isEqualTo("1.5");
        assertThat(s.get("A6")).isEqualTo("#DIV/0!");
        assertThat(s.get("A7")).isEqualTo("-0.5");
        assertThat(s.get("A9")).isEqualTo("0.6666666667");
    }

    @Test
    void shouldReportRefErrorForInvalidAddressesInsideFormulas() {
        FormulaSheet s = sheet();
        put(s, "B1", "5", "A1", "=Z101", "A2", "=AA1", "A3", "=A0+1", "A4", "=SUM(B1:Z101)", "A5", "=SUM(B1, AA1)",
                "A6", "=B1+A01", "A7", "=SUM(A0:B1)", "A8", "=ABC12");

        for (int i = 1; i <= 8; i++) {
            assertThat(s.get("A" + i)).as("A" + i).isEqualTo("#REF!");
        }
        assertThat(s.dependencies("A1")).isEmpty();
        assertThat(s.dependencies("A5")).containsExactly("B1");
        assertThat(s.dependencies("A6")).containsExactly("B1");
        assertThat(s.dependencies("A4")).isEmpty();
        assertThat(s.formulaOf("A2")).isEqualTo("=AA1");
    }

    @Test
    void shouldReportSyntaxErrors() {
        FormulaSheet s = sheet();
        String[] bad = {"=", "=1+", "=(1", "=1)", "=()", "=SUM()", "=SUM(A1:B2", "=SUM(A1,)", "=SUM(,A1)", "=A1:B2",
            "=foo", "=AVG(1)", "=1 2", "=A1 B1", "=1..2", "=.5", "=5.", "=+5", "=SUM(1+2)", "=SUM(A1+1)", "=A", "=1+*2",
            "=1**2", "=2 3", "=SUM(SUM(1))", "=SUM(A1:B2:C3)", "=SUM", "=SUM 1", "=1+2)", "=$A$1", "=A1;B1", "=\"x\"",
            "=SUM(--1)", "=1%"};
        for (String f : bad) {
            s.set("C3", f);
            assertThat(s.get("C3")).as(f).isEqualTo("#ERR!");
            assertThat(s.dependencies("C3")).as(f).isEmpty();
        }
        s.set("C3", "=a1 +  b2 ");
        assertThat(s.formulaOf("C3")).isEqualTo("=A1 +  B2 ");
        s.set("C3", "=1+");
        assertThat(s.formulaOf("C3")).isEqualTo("=1+");
        assertThat(s.usedCells()).containsExactly("C3");
    }

    @Test
    void shouldPropagateTheFirstErrorInRowMajorOrderAndThenArgumentOrder() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "B1", "=1/0", "A2", "=Z101", "B2", "5");

        s.set("D1", "=SUM(A1:B2)");
        assertThat(s.get("D1")).isEqualTo("#DIV/0!");
        s.set("D1", "=SUM(B2:A1)");
        assertThat(s.get("D1")).isEqualTo("#DIV/0!");
        s.set("D1", "=SUM(A2, B1)");
        assertThat(s.get("D1")).isEqualTo("#REF!");
        s.set("D1", "=SUM(B1, A2)");
        assertThat(s.get("D1")).isEqualTo("#DIV/0!");

        put(s, "E1", "text");
        s.set("D2", "=A2+B1");
        assertThat(s.get("D2")).isEqualTo("#REF!");
        s.set("D2", "=B1+A2");
        assertThat(s.get("D2")).isEqualTo("#DIV/0!");
        s.set("D2", "=E1+B1");
        assertThat(s.get("D2")).isEqualTo("#DIV/0!");
        s.set("D2", "=B1+E1");
        assertThat(s.get("D2")).isEqualTo("#DIV/0!");
        s.set("D2", "=E1/0");
        assertThat(s.get("D2")).isEqualTo("#VALUE!");
        s.set("D2", "=E1+1/0");
        assertThat(s.get("D2")).isEqualTo("#DIV/0!");
    }

    @Test
    void shouldRecomputeDependentsAfterAnyChange() {
        FormulaSheet s = sheet();
        put(s, "A1", "5", "A2", "7", "A3", "=A1+A2*2", "B1", "=SUM(A1:A3)");
        assertThat(s.get("A3")).isEqualTo("19");
        assertThat(s.get("B1")).isEqualTo("31");

        s.set("A1", "10");
        assertThat(s.get("A3")).isEqualTo("24");
        assertThat(s.get("B1")).isEqualTo("41");

        s.set("A2", "text");
        assertThat(s.get("A3")).isEqualTo("#VALUE!");
        assertThat(s.get("B1")).isEqualTo("#VALUE!");

        s.set("A2", "1");
        assertThat(s.get("A3")).isEqualTo("12");
        assertThat(s.get("B1")).isEqualTo("23");

        s.set("A3", "100");
        assertThat(s.get("B1")).isEqualTo("111");
        s.set("A3", "=A1");
        assertThat(s.get("B1")).isEqualTo("21");
    }

    @Test
    void shouldReportFormulaDependenciesAndDependents() {
        FormulaSheet s = sheet();
        put(s, "C1", "=a1 + B2 + SUM(A3:B4) + a1", "D5", "=C1*2", "A2", "=SUM(C1, B2)");

        assertThat(s.formulaOf("C1")).isEqualTo("=A1 + B2 + SUM(A3:B4) + A1");
        assertThat(s.formulaOf("D5")).isEqualTo("=C1*2");
        assertThat(s.formulaOf("A1")).isEmpty();
        assertThat(s.dependencies("C1")).containsExactly("A1", "B2", "A3", "B3", "A4", "B4");
        assertThat(s.dependencies("D5")).containsExactly("C1");
        assertThat(s.dependencies("A1")).isEmpty();
        assertThat(s.dependents("A1")).containsExactly("C1");
        assertThat(s.dependents("B2")).containsExactly("C1", "A2");
        assertThat(s.dependents("B3")).containsExactly("C1");
        assertThat(s.dependents("C1")).containsExactly("A2", "D5");
        assertThat(s.dependents("D5")).isEmpty();
        assertThat(s.dependents("Z100")).isEmpty();
    }

    @Test
    void shouldUpdateDependenciesWhenAFormulaIsReplaced() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "B1", "2", "C1", "=A1+1");
        assertThat(s.dependents("A1")).containsExactly("C1");

        s.set("C1", "=B1+1");
        assertThat(s.dependencies("C1")).containsExactly("B1");
        assertThat(s.dependents("A1")).isEmpty();
        assertThat(s.dependents("B1")).containsExactly("C1");
        long before = s.recomputeCount();
        s.set("A1", "50");
        assertThat(s.recomputeCount()).isEqualTo(before);
        assertThat(s.get("C1")).isEqualTo("3");

        s.set("C1", "7");
        assertThat(s.dependencies("C1")).isEmpty();
        assertThat(s.dependents("B1")).isEmpty();
        assertThat(s.formulaOf("C1")).isEmpty();
        s.set("C1", "=1/0");
        assertThat(s.dependencies("C1")).isEmpty();
    }

    @Test
    void shouldListUsedCellsRowByRow() {
        FormulaSheet s = sheet();
        put(s, "C2", "x", "A3", "1", "B1", "=A3", "Z100", "9", "A1", "t", "A2", "=Q9");
        assertThat(s.usedCells()).containsExactly("A1", "B1", "A2", "C2", "A3", "Z100");
        s.set("A2", "");
        assertThat(s.usedCells()).containsExactly("A1", "B1", "C2", "A3", "Z100");
    }

    @Test
    void shouldMarkEveryCellOnACycleAndEveryDependentOfIt() {
        FormulaSheet s = sheet();
        put(s, "A1", "=B1+1", "B1", "=A1+1", "C1", "=A1*2", "D1", "=SUM(C1:C2)", "E1", "=1+1");

        assertThat(s.get("A1")).isEqualTo("#CYCLE!");
        assertThat(s.get("B1")).isEqualTo("#CYCLE!");
        assertThat(s.get("C1")).isEqualTo("#CYCLE!");
        assertThat(s.get("D1")).isEqualTo("#CYCLE!");
        assertThat(s.get("E1")).isEqualTo("2");

        s.set("B1", "4");
        assertThat(s.get("A1")).isEqualTo("5");
        assertThat(s.get("C1")).isEqualTo("10");
        assertThat(s.get("D1")).isEqualTo("10");
    }

    @Test
    void shouldDetectSelfAndRangeAndLongCycles() {
        FormulaSheet s = sheet();
        s.set("A1", "=A1");
        assertThat(s.get("A1")).isEqualTo("#CYCLE!");
        s.set("A1", "=1+A1*2");
        assertThat(s.get("A1")).isEqualTo("#CYCLE!");
        s.set("A1", "3");
        assertThat(s.get("A1")).isEqualTo("3");

        s.set("A2", "=SUM(A1:A3)");
        assertThat(s.get("A2")).isEqualTo("#CYCLE!");
        s.set("A2", "=SUM(A1:A1)");
        assertThat(s.get("A2")).isEqualTo("3");

        put(s, "B1", "=B2", "B2", "=B3", "B3", "=B4", "B4", "=B1");
        for (String c : List.of("B1", "B2", "B3", "B4")) {
            assertThat(s.get(c)).isEqualTo("#CYCLE!");
        }
        s.set("B3", "8");
        assertThat(s.get("B1")).isEqualTo("8");
        assertThat(s.get("B2")).isEqualTo("8");
        assertThat(s.get("B4")).isEqualTo("8");
    }

    @Test
    void shouldNotRecomputeAnythingWhenAnUnrelatedCellChanges() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "B1", "=A1+1", "C1", "=SUM(A1:B1)");
        long before = s.recomputeCount();

        s.set("F9", "123");
        s.set("F10", "text");
        s.set("G9", "");
        s.set("C2", "5");

        assertThat(s.recomputeCount()).isEqualTo(before);
        assertThat(s.get("C1")).isEqualTo("3");
    }

    @Test
    void shouldCountEachFormulaEvaluationAtSetTime() {
        FormulaSheet s = sheet();
        assertThat(s.recomputeCount()).isZero();
        s.set("A1", "1");
        s.set("A2", "text");
        assertThat(s.recomputeCount()).isZero();
        s.set("B1", "=A1+1");
        assertThat(s.recomputeCount()).isEqualTo(1);
        s.set("B2", "=SUM(A1:A2)");
        assertThat(s.recomputeCount()).isEqualTo(2);
        s.set("B3", "=1/0");
        assertThat(s.recomputeCount()).isEqualTo(3);
        s.set("B1", "=A1+2");
        assertThat(s.recomputeCount()).isEqualTo(4);
        assertThatThrownBy(() -> s.set("a0", "=1")).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.recomputeCount()).isEqualTo(4);
    }

    @Test
    void shouldEvaluateEachDependentOfADiamondExactlyOnce() {
        FormulaSheet s = sheet();
        put(s, "A1", "1", "B1", "=A1+1", "C1", "=A1+2", "D1", "=B1+C1");
        assertThat(s.get("D1")).isEqualTo("5");
        assertThat(s.recomputeCount()).isEqualTo(3);

        s.set("A1", "10");
        assertThat(s.get("D1")).isEqualTo("23");
        assertThat(s.recomputeCount()).isEqualTo(6);

        put(s, "E1", "=D1+B1+A1", "F1", "=SUM(A1:E1)");
        long before = s.recomputeCount();
        s.set("A1", "0");
        assertThat(s.recomputeCount() - before).isEqualTo(5);
        assertThat(s.get("F1")).isEqualTo("10");
    }

    @Test
    void shouldEvaluateAChainOfOneHundredDependentsExactlyOncePerChange() {
        FormulaSheet s = sheet();
        s.set("A1", "1");
        for (int r = 1; r <= 100; r++) {
            s.set("B" + r, r == 1 ? "=A1+1" : "=B" + (r - 1) + "+1");
        }
        assertThat(s.get("B100")).isEqualTo("101");
        long before = s.recomputeCount();

        s.set("A1", "50");
        assertThat(s.recomputeCount() - before).isEqualTo(100);
        assertThat(s.get("B100")).isEqualTo("150");

        before = s.recomputeCount();
        s.set("B50", "=A1+1000");
        assertThat(s.recomputeCount() - before).isEqualTo(51);
        assertThat(s.get("B100")).isEqualTo("1100");

        before = s.recomputeCount();
        s.set("A1", "");
        assertThat(s.recomputeCount() - before).isEqualTo(100);
        assertThat(s.get("B100")).isEqualTo("1050");
        assertThat(s.get("B49")).isEqualTo("49");
    }

    @Test
    void shouldCountCycleMembersOnlyWhenTheyAreActuallyEvaluated() {
        FormulaSheet s = sheet();
        s.set("A1", "=B1+1");
        long start = s.recomputeCount();
        assertThat(start).isEqualTo(1);

        s.set("B1", "=A1+1");
        assertThat(s.recomputeCount()).isEqualTo(start);

        s.set("C1", "=A1*2");
        assertThat(s.recomputeCount()).isEqualTo(start + 1);

        s.set("B1", "4");
        assertThat(s.recomputeCount()).isEqualTo(start + 3);
        assertThat(s.get("C1")).isEqualTo("10");
    }

    @Test
    void shouldSurviveAChainThroughAllTwentySixHundredCellsOnADefaultStack() {
        FormulaSheet s = sheet();
        onDefaultStack(() -> {
            s.set("A1", "1");
            for (int i = 1; i < 2600; i++) {
                s.set(cellName(i), "=" + cellName(i - 1) + "+1");
            }
            assertThat(s.get("Z100")).isEqualTo("2600");
            long before = s.recomputeCount();
            long start = System.nanoTime();
            for (int k = 1; k <= 100; k++) {
                s.set("A1", String.valueOf(k * 10));
                assertThat(s.get("Z100")).isEqualTo(String.valueOf(k * 10 + 2599));
            }
            assertThat(s.recomputeCount() - before).isEqualTo(100L * 2599);
            assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(20_000);
        });
    }

    @Test
    void shouldKeepAllCellsConsistentUnderConcurrentSetsAndGets() {
        FormulaSheet s = sheet();
        Concurrently.run(8, 60_000, t -> {
            char col = (char) ('A' + t);
            s.set(col + "1", "1");
            for (int r = 2; r <= 100; r++) {
                s.set(col + String.valueOf(r), "=" + col + (r - 1) + "+1");
                s.get(col + String.valueOf(r));
                s.usedCells();
            }
        });
        for (int t = 0; t < 8; t++) {
            assertThat(s.get((char) ('A' + t) + "100")).isEqualTo("100");
        }
        assertThat(s.recomputeCount()).isEqualTo(8L * 99);
        assertThat(s.usedCells()).hasSize(800);
    }

    @Test
    void shouldNeverExposeAHalfUpdatedChainToReaders() {
        FormulaSheet s = sheet();
        s.set("A1", "1000");
        for (int r = 1; r <= 99; r++) {
            s.set("B" + r, r == 1 ? "=A1+1" : "=B" + (r - 1) + "+1");
        }
        List<List<String>> seen = Concurrently.collect(6, 60_000, t -> {
            List<String> out = new ArrayList<>();
            if (t < 2) {
                for (int i = 0; i < 300; i++) {
                    s.set("A1", i % 2 == 0 ? "2000" : "1000");
                }
            } else {
                for (int i = 0; i < 2_000; i++) {
                    out.add(s.get("B99"));
                }
            }
            return out;
        });
        for (List<String> reads : seen) {
            for (String v : reads) {
                assertThat(v).isIn("1099", "2099");
            }
        }
        assertThat(s.get("B99")).isIn("1099", "2099");
        assertThat(s.get("B99")).isEqualTo(String.valueOf(Integer.parseInt(s.get("A1")) + 99));
    }
}

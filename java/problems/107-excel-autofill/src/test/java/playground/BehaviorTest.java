package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AutofillService;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private AutofillService sheet() {
        return Candidate.of(Solution::create);
    }

    private static List<String> seeds(String... s) {
        return Arrays.asList(s);
    }

    @Test
    void shouldContinueAnIncreasingIntegerSeries() {
        assertThat(sheet().autofill(seeds("1", "3"), 4)).containsExactly("5", "7", "9", "11");
    }

    @Test
    void shouldContinueADecreasingSeriesThroughZeroIntoNegatives() {
        assertThat(sheet().autofill(seeds("5", "3"), 4)).containsExactly("1", "-1", "-3", "-5");
    }

    @Test
    void shouldContinueSeriesWhoseSeedsAreNegative() {
        assertThat(sheet().autofill(seeds("-10", "-7"), 3)).containsExactly("-4", "-1", "2");
    }

    @Test
    void shouldContinueUsingTheStepSharedByAllSeeds() {
        assertThat(sheet().autofill(seeds("10", "20", "30"), 2)).containsExactly("40", "50");
    }

    @Test
    void shouldRepeatAConstantSeries() {
        assertThat(sheet().autofill(seeds("4", "4"), 3)).containsExactly("4", "4", "4");
    }

    @Test
    void shouldAddDecimalsExactly() {
        assertThat(sheet().autofill(seeds("0.1", "0.2"), 3)).containsExactly("0.3", "0.4", "0.5");
    }

    @Test
    void shouldDropTrailingZerosAndNeverUseScientificNotation() {
        AutofillService s = sheet();

        assertThat(s.autofill(seeds("0.5", "1.0"), 3)).containsExactly("1.5", "2", "2.5");
        assertThat(s.autofill(seeds("1000000", "2000000"), 2)).containsExactly("3000000", "4000000");
        assertThat(s.autofill(seeds("0.0000001", "0.0000002"), 2)).containsExactly("0.0000003", "0.0000004");
        assertThat(s.autofill(seeds("1", "0.5"), 3)).containsExactly("0", "-0.5", "-1");
    }

    @Test
    void shouldStepByOneForASingleNumericSeed() {
        AutofillService s = sheet();

        assertThat(s.autofill(seeds("7"), 3)).containsExactly("8", "9", "10");
        assertThat(s.autofill(seeds("-1"), 3)).containsExactly("0", "1", "2");
        assertThat(s.autofill(seeds("1.5"), 2)).containsExactly("2.5", "3.5");
    }

    @Test
    void shouldContinueAnUppercaseLetterSeries() {
        assertThat(sheet().autofill(seeds("A", "B"), 3)).containsExactly("C", "D", "E");
    }

    @Test
    void shouldPreserveLowercase() {
        assertThat(sheet().autofill(seeds("a", "b"), 2)).containsExactly("c", "d");
    }

    @Test
    void shouldWrapPastZIntoTwoLetterLabels() {
        AutofillService s = sheet();

        assertThat(s.autofill(seeds("X", "Y"), 4)).containsExactly("Z", "AA", "AB", "AC");
        assertThat(s.autofill(seeds("Z"), 2)).containsExactly("AA", "AB");
        assertThat(s.autofill(seeds("z"), 2)).containsExactly("aa", "ab");
        assertThat(s.autofill(seeds("AZ"), 2)).containsExactly("BA", "BB");
    }

    @Test
    void shouldContinueALetterSeriesWithAStep() {
        assertThat(sheet().autofill(seeds("A", "C"), 3)).containsExactly("E", "G", "I");
    }

    @Test
    void shouldStepByOneForASingleLetterSeed() {
        assertThat(sheet().autofill(seeds("M"), 2)).containsExactly("N", "O");
    }

    @Test
    void shouldStopADecreasingLetterSeriesBeforeA() {
        AutofillService s = sheet();

        assertThat(s.autofill(seeds("D", "C"), 5)).containsExactly("B", "A");
        assertThat(s.autofill(seeds("c", "b"), 1)).containsExactly("a");
        assertThat(s.autofill(seeds("B", "A"), 3)).isEmpty();
    }

    @Test
    void shouldReturnExactlyCountValuesAndNothingForZero() {
        AutofillService s = sheet();

        assertThat(s.autofill(seeds("1", "2"), 0)).isEmpty();
        assertThat(s.autofill(seeds("A", "B"), 0)).isEmpty();
        List<String> many = s.autofill(seeds("1", "2"), 1000);
        assertThat(many).hasSize(1000);
        assertThat(many.get(0)).isEqualTo("3");
        assertThat(many.get(999)).isEqualTo("1002");
    }

    @Test
    void shouldRejectMixedKindsOfSeeds() {
        AutofillService s = sheet();

        assertThatThrownBy(() -> s.autofill(seeds("1", "A"), 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(seeds("A", "1"), 2)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectSeedsThatAreNotAnArithmeticProgression() {
        AutofillService s = sheet();

        assertThatThrownBy(() -> s.autofill(seeds("1", "2", "4"), 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(seeds("A", "B", "D"), 2)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectMixedCaseLetters() {
        AutofillService s = sheet();

        assertThatThrownBy(() -> s.autofill(seeds("A", "b"), 2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(seeds("Ab"), 2)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectMalformedNumbers() {
        AutofillService s = sheet();

        for (String bad : List.of("1e3", "+1", ".5", "1.", " 1", "1,000", "0x10", "--1", "1.2.3", "-")) {
            assertThatThrownBy(() -> s.autofill(seeds(bad), 1)).as(bad).isInstanceOf(IllegalArgumentException.class);
        }
    }

    @Test
    void shouldRejectTextThatIsNeitherNumberNorLetters() {
        AutofillService s = sheet();

        for (String bad : List.of("", " ", "Jan", "A1", "?", "A B", "é")) {
            assertThatThrownBy(() -> s.autofill(seeds(bad), 1)).as(bad).isInstanceOf(IllegalArgumentException.class);
        }
    }

    @Test
    void shouldRejectInvalidArguments() {
        AutofillService s = sheet();

        assertThatThrownBy(() -> s.autofill(null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(List.of(), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(seeds("1", null), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.autofill(seeds("1", "2"), -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldValidateSeedsEvenWhenCountIsZero() {
        AutofillService s = sheet();

        assertThatThrownBy(() -> s.autofill(seeds("1", "A"), 0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotModifyTheSeedList() {
        List<String> given = new ArrayList<>(seeds("1", "3"));

        sheet().autofill(given, 3);

        assertThat(given).containsExactly("1", "3");
    }

    @Test
    void shouldGiveTheSameAnswerWhenCalledFromManyThreads() {
        AutofillService s = sheet();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                assertThat(s.autofill(seeds("" + t, "" + (t + 2)), 3))
                        .containsExactly("" + (t + 4), "" + (t + 6), "" + (t + 8));
                assertThat(s.autofill(seeds("Y", "Z"), 2)).containsExactly("AA", "AB");
            }
        });
    }
}

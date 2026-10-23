package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AutofillService;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static List<String> seeds(String... s) {
        return Arrays.asList(s);
    }

    private final AutofillService sheet = Candidate.of(Solution::create);

    @Test
    void shouldTreatLetterLabelsAsBijectiveBaseTwentySix() {
        assertThat(sheet.autofill(seeds("Z", "AA"), 3)).containsExactly("AB", "AC", "AD");
        assertThat(sheet.autofill(seeds("ZY", "ZZ"), 3)).containsExactly("AAA", "AAB", "AAC");
        assertThat(sheet.autofill(seeds("zz"), 2)).containsExactly("aaa", "aab");
        assertThat(sheet.autofill(seeds("Y", "AA"), 3)).containsExactly("AC", "AE", "AG");
    }

    @Test
    void shouldShrinkLabelsWhenADecreasingLetterSeriesCrossesALengthBoundary() {
        assertThat(sheet.autofill(seeds("AB", "AA"), 4)).containsExactly("Z", "Y", "X", "W");
        assertThat(sheet.autofill(seeds("BA", "AZ"), 2)).containsExactly("AY", "AX");
    }

    @Test
    void shouldStopImmediatelyWhenTheFirstStepLeavesTheAlphabet() {
        assertThat(sheet.autofill(seeds("C", "A"), 3)).isEmpty();
        assertThat(sheet.autofill(seeds("e", "c"), 5)).containsExactly("a");
    }

    @Test
    void shouldRepeatALetterWhenTheStepIsZero() {
        assertThat(sheet.autofill(seeds("B", "B", "B"), 2)).containsExactly("B", "B");
    }

    @Test
    void shouldKeepDecimalArithmeticExactAcrossScalesAndSigns() {
        assertThat(sheet.autofill(seeds("0.1", "0.2", "0.3"), 2)).containsExactly("0.4", "0.5");
        assertThat(sheet.autofill(seeds("1.10", "1.20"), 2)).containsExactly("1.3", "1.4");
        assertThat(sheet.autofill(seeds("1", "1.5", "2.0"), 2)).containsExactly("2.5", "3");
        assertThat(sheet.autofill(seeds("0.5", "0.25"), 3)).containsExactly("0", "-0.25", "-0.5");
        assertThat(sheet.autofill(seeds("-0.5", "0.5"), 2)).containsExactly("1.5", "2.5");
        assertThat(sheet.autofill(seeds("0.001", "0.002"), 2)).containsExactly("0.003", "0.004");
    }

    @Test
    void shouldNeverEmitNegativeZero() {
        assertThat(sheet.autofill(seeds("-2", "-1"), 3)).containsExactly("0", "1", "2");
        assertThat(sheet.autofill(seeds("-0.5", "-0.25"), 3)).containsExactly("0", "0.25", "0.5");
        assertThat(sheet.autofill(seeds("-0"), 2)).containsExactly("1", "2");
    }

    @Test
    void shouldAcceptLeadingZerosButNotPreservePadding() {
        assertThat(sheet.autofill(seeds("007", "008"), 3)).containsExactly("9", "10", "11");
        assertThat(sheet.autofill(seeds("00.50", "01.00"), 2)).containsExactly("1.5", "2");
    }

    @Test
    void shouldAcceptFifteenDigitsAndRejectSixteen() {
        assertThat(sheet.autofill(seeds("999999999999998", "999999999999999"), 2))
                .containsExactly("1000000000000000", "1000000000000001");
        assertThat(sheet.autofill(seeds("0.12345678901234"), 1)).containsExactly("1.12345678901234");
        assertThatThrownBy(() -> sheet.autofill(seeds("1234567890123456"), 1))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> sheet.autofill(seeds("0.1234567890123456"), 1))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAcceptTenLettersAndRejectEleven() {
        assertThat(sheet.autofill(seeds("AAAAAAAAAA"), 1)).containsExactly("AAAAAAAAAB");
        assertThatThrownBy(() -> sheet.autofill(seeds("AAAAAAAAAAA"), 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectAnyBadSeedEvenWhenItIsLastOrTheCountIsZero() {
        assertThatThrownBy(() -> sheet.autofill(seeds("1", "2", "3", "x"), 0))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> sheet.autofill(seeds("1", "2", "3", "5"), 0))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> sheet.autofill(seeds("1", "2", "2"), 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> sheet.autofill(seeds("a", "B"), 0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCheckEveryPairNotJustTheFirstTwoSeeds() {
        assertThatThrownBy(() -> sheet.autofill(seeds("2", "4", "6", "9"), 1))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> sheet.autofill(seeds("A", "C", "E", "H"), 1))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldHandleLargeStepsAndLargeCounts() {
        assertThat(sheet.autofill(seeds("1000000000000", "2000000000000"), 2))
                .containsExactly("3000000000000", "4000000000000");
        List<String> big = sheet.autofill(seeds("1", "2"), 100_000);
        assertThat(big).hasSize(100_000);
        assertThat(big.get(99_999)).isEqualTo("100002");
        List<String> letters = sheet.autofill(seeds("A"), 703);
        assertThat(letters.get(25)).isEqualTo("AA");
        assertThat(letters.get(701)).isEqualTo("AAA");
    }

    @Test
    void shouldStayCorrectWhenManyThreadsMixKinds() {
        Concurrently.run(8, t -> {
            for (int i = 0; i < 5_000; i++) {
                assertThat(sheet.autofill(seeds("a", "c"), 2)).containsExactly("e", "g");
                assertThat(sheet.autofill(seeds("0.1", "0.2"), 1)).containsExactly("0.3");
                assertThatThrownBy(() -> sheet.autofill(seeds("1", "x"), 1))
                        .isInstanceOf(IllegalArgumentException.class);
            }
        });
    }
}

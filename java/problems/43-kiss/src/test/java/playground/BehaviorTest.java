package playground;

import static org.assertj.core.api.Assertions.assertThat;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import solution.LateFees;

class BehaviorTest {

    private static final Path SOURCE_DIR = Path.of("src/main/java/solution");

    @Test
    void shouldNotChargeForOnTimeOrEarlyReturns() {
        assertThat(LateFees.fee(0, false)).isZero();
        assertThat(LateFees.fee(-3, false)).isZero();
        assertThat(LateFees.fee(-3, true)).isZero();
    }

    @Test
    void shouldChargeTwentyFiveCentsPerDayForStandardMembers() {
        assertThat(LateFees.fee(1, false)).isEqualTo(25);
        assertThat(LateFees.fee(10, false)).isEqualTo(250);
    }

    @Test
    void shouldChargeTenCentsPerDayForPremiumMembers() {
        assertThat(LateFees.fee(1, true)).isEqualTo(10);
        assertThat(LateFees.fee(30, true)).isEqualTo(300);
    }

    @Test
    void shouldCapTheFee() {
        assertThat(LateFees.fee(40, false)).isEqualTo(1000);
        assertThat(LateFees.fee(41, false)).isEqualTo(1000);
        assertThat(LateFees.fee(50, true)).isEqualTo(500);
        assertThat(LateFees.fee(51, true)).isEqualTo(500);
    }

    @Test
    void shouldNotOverflowForAbsurdlyLateReturns() {
        assertThat(LateFees.fee(Integer.MAX_VALUE, false)).isEqualTo(1000);
        assertThat(LateFees.fee(Integer.MAX_VALUE, true)).isEqualTo(500);
    }

    @Test
    void shouldFitInOneSmallClassWithNoMachinery() throws IOException {
        assertThat(LateFees.fee(2, false)).isEqualTo(50);

        List<Path> files;
        try (Stream<Path> s = Files.list(SOURCE_DIR)) {
            files = s.toList();
        }
        assertThat(files).as("KISS: one class is enough").hasSize(1);

        List<String> code = Files.readAllLines(files.get(0)).stream()
                .map(String::strip)
                .filter(l -> !l.isEmpty() && !l.startsWith("//") && !l.startsWith("/*") && !l.startsWith("*"))
                .toList();
        assertThat(code.size()).as("code lines (budget 25)").isLessThanOrEqualTo(25);
        String joined = String.join("\n", code);
        for (String banned : List.of("interface ", "abstract ", " extends ", " implements ", "enum ", "record ", "switch")) {
            assertThat(joined).as("over-engineering smell: " + banned.strip()).doesNotContain(banned);
        }
    }
}

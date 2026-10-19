package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.UrlShortener;
import java.util.HashSet;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandleAMillionUrlsWithShortCodes() {
        UrlShortener s = Candidate.of(Solution::create);
        Set<String> codes = new HashSet<>();

        for (int i = 0; i < 1_000_000; i++) {
            String code = s.shorten("https://example.com/" + i);
            assertThat(code.length()).isLessThanOrEqualTo(6);
            codes.add(code);
        }

        assertThat(codes).hasSize(1_000_000);
    }

    @Test
    void shouldNotLetAliasesCollideWithGeneratedCodesUnderLoad() {
        UrlShortener s = Candidate.of(Solution::create);

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 2_000; i++) {
                String alias = "alias-" + t + "-" + i;
                s.shorten("https://example.com/alias/" + t + "/" + i, alias);
                s.shorten("https://example.com/gen/" + t + "/" + i);
            }
        });

        for (int t = 0; t < 8; t++) {
            assertThat(s.resolve("alias-" + t + "-5")).hasValue("https://example.com/alias/" + t + "/5");
        }
    }
}

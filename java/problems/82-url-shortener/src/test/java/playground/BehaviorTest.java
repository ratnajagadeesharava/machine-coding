package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.UrlShortener;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private UrlShortener shortener() {
        return Candidate.of(Solution::create);
    }

    private static String url(int i) {
        return "https://example.com/page/" + i;
    }

    @Test
    void shouldResolveACodeBackToItsUrl() {
        UrlShortener s = shortener();

        String code = s.shorten("https://example.com/a?b=c#d");

        assertThat(code).matches("[0-9A-Za-z]+");
        assertThat(s.resolve(code)).hasValue("https://example.com/a?b=c#d");
    }

    @Test
    void shouldReturnTheSameCodeForTheSameUrlAndDifferentCodesForDifferentUrls() {
        UrlShortener s = shortener();

        String a1 = s.shorten(url(1));
        String a2 = s.shorten(url(1));
        String b = s.shorten(url(2));

        assertThat(a1).isEqualTo(a2);
        assertThat(b).isNotEqualTo(a1);
    }

    @Test
    void shouldRejectInvalidUrls() {
        UrlShortener s = shortener();

        assertThatThrownBy(() -> s.shorten(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("   ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("example.com/a")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("ftp://example.com/a")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldKeepCodesShort() {
        UrlShortener s = shortener();

        for (int i = 0; i < 10_000; i++) {
            assertThat(s.shorten(url(i)).length()).isLessThanOrEqualTo(6);
        }
    }

    @Test
    void shouldReturnEmptyForUnknownCodes() {
        UrlShortener s = shortener();

        assertThat(s.resolve("nope")).isEmpty();
        assertThatThrownBy(() -> s.clicks("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSupportCustomAliases() {
        UrlShortener s = shortener();

        assertThat(s.shorten(url(1), "my-link_1")).isEqualTo("my-link_1");
        assertThat(s.shorten(url(1), "my-link_1")).isEqualTo("my-link_1");
        assertThat(s.resolve("my-link_1")).hasValue(url(1));
    }

    @Test
    void shouldRejectBadOrTakenAliases() {
        UrlShortener s = shortener();
        s.shorten(url(1), "taken");
        String generated = s.shorten(url(2));

        assertThatThrownBy(() -> s.shorten(url(3), "taken")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten(url(3), generated)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten(url(3), "ab")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten(url(3), "has space")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten(url(3), "x".repeat(21))).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.resolve("taken")).hasValue(url(1));
    }

    @Test
    void shouldCountOnlySuccessfulResolves() {
        UrlShortener s = shortener();
        String code = s.shorten(url(1));

        for (int i = 0; i < 5; i++) {
            s.resolve(code);
        }
        s.resolve("unknown");

        assertThat(s.clicks(code)).isEqualTo(5);
    }

    @Test
    void shouldDeleteCodesAndNeverReissueThem() {
        UrlShortener s = shortener();
        String code = s.shorten(url(1));

        assertThat(s.delete(code)).isTrue();
        assertThat(s.delete(code)).isFalse();

        assertThat(s.resolve(code)).isEmpty();
        assertThatThrownBy(() -> s.clicks(code)).isInstanceOf(IllegalArgumentException.class);
        Set<String> later = new HashSet<>();
        for (int i = 10; i < 500; i++) {
            later.add(s.shorten(url(i)));
        }
        assertThat(later).doesNotContain(code);
        assertThat(s.resolve(s.shorten(url(1)))).hasValue(url(1));
    }

    @Test
    void shouldGiveEveryUrlItsOwnCodeUnderConcurrentShortening() {
        UrlShortener s = shortener();

        List<List<String>> codes = Concurrently.collect(16, t -> {
            List<String> mine = new java.util.ArrayList<>();
            for (int i = 0; i < 2_000; i++) {
                mine.add(s.shorten(url(t * 2_000 + i)));
            }
            return mine;
        });

        Set<String> all = new HashSet<>();
        for (List<String> mine : codes) {
            all.addAll(mine);
        }
        assertThat(all).hasSize(32_000);
        for (int t = 0; t < 16; t += 5) {
            for (int i = 0; i < 2_000; i += 97) {
                assertThat(s.resolve(codes.get(t).get(i))).hasValue(url(t * 2_000 + i));
            }
        }
    }

    @Test
    void shouldGiveOneCodeWhenManyThreadsShortenTheSameUrl() {
        UrlShortener s = shortener();

        List<String> codes = Concurrently.collect(16, t -> s.shorten("https://example.com/hot"));

        assertThat(new HashSet<>(codes)).hasSize(1);
    }

    @Test
    void shouldLetExactlyOneThreadWinAnAlias() {
        UrlShortener s = shortener();

        List<Boolean> wins = Concurrently.collect(16, t -> {
            try {
                s.shorten(url(t), "contested");
                return true;
            } catch (IllegalArgumentException e) {
                return false;
            }
        });

        assertThat(wins.stream().filter(w -> w).count()).isEqualTo(1);
    }

    @Test
    void shouldCountEveryClickUnderConcurrency() {
        UrlShortener s = shortener();
        String code = s.shorten(url(1));

        Concurrently.run(8, t -> {
            for (int i = 0; i < 1_000; i++) {
                s.resolve(code);
            }
        });

        assertThat(s.clicks(code)).isEqualTo(8_000);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            UrlShortener s = shortener();
            AtomicInteger sameUrlCodes = new AtomicInteger();
            Set<String> codes = java.util.concurrent.ConcurrentHashMap.newKeySet();

            Concurrently.run(8, t -> {
                for (int i = 0; i < 50; i++) {
                    codes.add(s.shorten(url(i)));
                }
                sameUrlCodes.incrementAndGet();
            });

            assertThat(codes).as("run %d", run).hasSize(50);
        }
    }
}

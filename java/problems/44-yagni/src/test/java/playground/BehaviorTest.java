package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.io.IOException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import solution.UrlShortener;

class BehaviorTest {

    private static final Path SOURCE_DIR = Path.of("src/main/java/solution");

    @Test
    void shouldHandOutSequentialBase36Codes() {
        UrlShortener s = new UrlShortener();

        assertThat(s.shorten("https://example.com/a")).isEqualTo("1");
        assertThat(s.shorten("https://example.com/b")).isEqualTo("2");
        for (int i = 3; i <= 10; i++) {
            s.shorten("http://x.io/" + i);
        }
        assertThat(s.shorten("http://x.io/last")).isEqualTo("b");
    }

    @Test
    void shouldReturnTheSameCodeForTheSameUrl() {
        UrlShortener s = new UrlShortener();

        String first = s.shorten("https://example.com/page");
        s.shorten("https://example.com/other");

        assertThat(s.shorten("https://example.com/page")).isEqualTo(first);
        assertThat(s.shorten("  https://example.com/page  ")).isEqualTo(first);
    }

    @Test
    void shouldResolveCodesBackToTheOriginalUrl() {
        UrlShortener s = new UrlShortener();
        String code = s.shorten("  https://example.com/long/path?q=1  ");

        assertThat(s.resolve(code)).hasValue("https://example.com/long/path?q=1");
        assertThat(s.resolve("zzz")).isEmpty();
        assertThat(s.resolve("")).isEmpty();
        assertThat(s.resolve(null)).isEmpty();
    }

    @Test
    void shouldRejectThingsThatAreNotHttpUrls() {
        UrlShortener s = new UrlShortener();

        assertThatThrownBy(() -> s.shorten("ftp://example.com")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("example.com")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("https://")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.shorten(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(s.shorten("https://ok.example")).isEqualTo("1");
    }

    @Test
    void shouldExposeOnlyWhatWasAskedFor() {
        assertThat(new UrlShortener().shorten("https://example.com")).isEqualTo("1");

        assertThat(Arrays.stream(UrlShortener.class.getDeclaredMethods())
                .filter(m -> Modifier.isPublic(m.getModifiers()) && !m.isSynthetic())
                .map(Method::getName))
                .as("public methods (YAGNI: no delete / stats / alias / expiry)")
                .containsExactlyInAnyOrder("shorten", "resolve");
        assertThat(UrlShortener.class.getConstructors()).hasSize(1);
        assertThat(Arrays.stream(UrlShortener.class.getDeclaredFields()).filter(f -> Modifier.isPublic(f.getModifiers())))
                .isEmpty();
    }

    @Test
    void shouldNotBuildMachineryNobodyAskedFor() throws IOException {
        assertThat(new UrlShortener().shorten("https://example.com")).isEqualTo("1");

        List<Path> files;
        try (Stream<Path> s = Files.list(SOURCE_DIR)) {
            files = s.toList();
        }
        assertThat(files).as("one class is enough").hasSize(1);
        String code = String.join("\n", Files.readAllLines(files.get(0)).stream()
                .map(String::strip)
                .filter(l -> !l.isEmpty() && !l.startsWith("//") && !l.startsWith("/*") && !l.startsWith("*"))
                .toList());
        for (String speculative : List.of("interface ", "abstract ", " implements ", "synchronized", "volatile",
                "ConcurrentHashMap", "Atomic", "Executor", "Thread", "Instant", "Duration", "Random")) {
            assertThat(code).as("nobody asked for: " + speculative.strip()).doesNotContain(speculative);
        }
    }
}

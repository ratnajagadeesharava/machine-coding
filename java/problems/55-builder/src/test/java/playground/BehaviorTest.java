package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import org.junit.jupiter.api.Test;
import solution.HttpRequest;

class BehaviorTest {

    @Test
    void shouldApplyDefaults() {
        HttpRequest r = HttpRequest.builder().url("https://api.test/items").build();

        assertThat(r.method()).isEqualTo("GET");
        assertThat(r.url()).isEqualTo("https://api.test/items");
        assertThat(r.timeoutMillis()).isEqualTo(5000);
        assertThat(r.headers()).isEmpty();
        assertThat(r.body()).isEmpty();
    }

    @Test
    void shouldBuildAFullRequestStepByStep() {
        HttpRequest r = HttpRequest.builder()
                .method("post")
                .url("http://api.test/items")
                .header("Content-Type", "application/json")
                .header("X-Trace", "abc")
                .body("{\"a\":1}")
                .timeoutMillis(250)
                .build();

        assertThat(r.method()).isEqualTo("POST");
        assertThat(r.headers()).containsExactly(
                java.util.Map.entry("content-type", "application/json"), java.util.Map.entry("x-trace", "abc"));
        assertThat(r.header("CONTENT-TYPE")).hasValue("application/json");
        assertThat(r.header("missing")).isEmpty();
        assertThat(r.body()).hasValue("{\"a\":1}");
        assertThat(r.timeoutMillis()).isEqualTo(250);
    }

    @Test
    void shouldAppendEncodedQueryParametersInOrder() {
        HttpRequest r = HttpRequest.builder()
                .url("https://api.test/items")
                .query("page", "2")
                .query("q", "a b&c")
                .query("name", "café")
                .build();

        assertThat(r.url()).isEqualTo("https://api.test/items?page=2&q=a%20b%26c&name=caf%C3%A9");
    }

    @Test
    void shouldContinueAnExistingQueryString() {
        HttpRequest r = HttpRequest.builder().url("https://api.test/items?sort=asc").query("page", "2").build();

        assertThat(r.url()).isEqualTo("https://api.test/items?sort=asc&page=2");
    }

    @Test
    void shouldReplaceARepeatedHeaderInPlaceIgnoringCase() {
        HttpRequest r = HttpRequest.builder()
                .url("https://api.test")
                .header("A", "1")
                .header("B", "2")
                .header("a", "3")
                .build();

        assertThat(r.headers()).containsExactly(java.util.Map.entry("a", "3"), java.util.Map.entry("b", "2"));
    }

    @Test
    void shouldRejectBadArgumentsImmediately() {
        HttpRequest.Builder b = HttpRequest.builder();

        assertThatThrownBy(() -> b.method("FETCH")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.method(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.url("ftp://x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.url("https://")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.header(" ", "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.header("n", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.query("", "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.timeoutMillis(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.timeoutMillis(60_001)).isInstanceOf(IllegalArgumentException.class);
        assertThat(b.url("https://ok.test").timeoutMillis(60_000).build().timeoutMillis()).isEqualTo(60_000);
    }

    @Test
    void shouldRefuseToBuildAnIncompleteOrInconsistentRequest() {
        assertThatThrownBy(() -> HttpRequest.builder().build()).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> HttpRequest.builder().url("https://x.test").body("data").build())
                .isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> HttpRequest.builder().method("DELETE").url("https://x.test").body("data").build())
                .isInstanceOf(IllegalStateException.class);
        assertThat(HttpRequest.builder().method("PUT").url("https://x.test").body("data").build().body()).hasValue("data");
    }

    @Test
    void shouldLetABuilderBeReusedWithoutAffectingEarlierRequests() {
        HttpRequest.Builder b = HttpRequest.builder().url("https://api.test/a").header("x-one", "1");
        HttpRequest first = b.build();

        b.header("x-two", "2").query("page", "9").url("https://api.test/b");
        HttpRequest second = b.build();

        assertThat(first.url()).isEqualTo("https://api.test/a");
        assertThat(first.headers()).containsOnlyKeys("x-one");
        assertThat(second.url()).isEqualTo("https://api.test/b?page=9");
        assertThat(second.headers()).containsOnlyKeys("x-one", "x-two");
        assertThat(first).isNotSameAs(second);
    }

    @Test
    void shouldBeImmutable() {
        HttpRequest r = HttpRequest.builder().url("https://api.test").header("a", "1").build();

        assertThat(r.headers()).containsOnlyKeys("a");
        assertThatThrownBy(() -> r.headers().put("b", "2")).isInstanceOf(UnsupportedOperationException.class);
        assertThat(r.headers()).hasSize(1);
        assertThat(HttpRequest.class.getConstructors()).as("no public constructor").isEmpty();
        assertThat(java.util.Arrays.stream(HttpRequest.class.getDeclaredMethods())
                .filter(m -> java.lang.reflect.Modifier.isPublic(m.getModifiers()))
                .map(java.lang.reflect.Method::getName))
                .noneMatch(n -> n.startsWith("set"));
    }
}

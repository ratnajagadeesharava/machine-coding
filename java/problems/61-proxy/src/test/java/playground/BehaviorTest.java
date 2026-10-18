package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.AccessPolicy;
import contract.DocumentStore;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.AccessControlStoreProxy;
import solution.CachingStoreProxy;
import solution.LazyStoreProxy;

class BehaviorTest {

    /** The "real" subject, written in the test: counts every call that reaches it. */
    private static final class CountingStore implements DocumentStore {
        final Map<String, String> documents = new HashMap<>();
        final List<String> requests = new ArrayList<>();

        public Optional<String> fetch(String id) {
            requests.add(id);
            return Optional.ofNullable(documents.get(id));
        }
    }

    private static CountingStore store(String... idAndText) {
        CountingStore s = new CountingStore();
        for (int i = 0; i < idAndText.length; i += 2) {
            s.documents.put(idAndText[i], idAndText[i + 1]);
        }
        return s;
    }

    @Test
    void shouldAskTheRealStoreOnlyOncePerDocument() {
        CountingStore real = store("a", "alpha", "b", "beta");
        CachingStoreProxy cache = new CachingStoreProxy(real);

        assertThat(cache.fetch("a")).hasValue("alpha");
        assertThat(cache.fetch("a")).hasValue("alpha");
        assertThat(cache.fetch("b")).hasValue("beta");
        assertThat(cache.fetch("a")).hasValue("alpha");

        assertThat(real.requests).containsExactly("a", "b");
        assertThat(cache.cachedCount()).isEqualTo(2);
    }

    @Test
    void shouldNotCacheMissesBecauseTheDocumentMayAppearLater() {
        CountingStore real = store();
        CachingStoreProxy cache = new CachingStoreProxy(real);

        assertThat(cache.fetch("late")).isEmpty();
        assertThat(cache.fetch("late")).isEmpty();
        real.documents.put("late", "now here");

        assertThat(cache.fetch("late")).hasValue("now here");
        assertThat(real.requests).containsExactly("late", "late", "late");
        assertThat(cache.cachedCount()).isEqualTo(1);
    }

    @Test
    void shouldForgetADocumentWhenInvalidated() {
        CountingStore real = store("a", "v1");
        CachingStoreProxy cache = new CachingStoreProxy(real);
        cache.fetch("a");
        real.documents.put("a", "v2");

        assertThat(cache.fetch("a")).hasValue("v1");
        cache.invalidate("a");
        cache.invalidate("never-cached");

        assertThat(cache.cachedCount()).isZero();
        assertThat(cache.fetch("a")).hasValue("v2");
    }

    @Test
    void shouldBuildTheRealStoreOnlyWhenFirstNeededAndOnlyOnce() {
        List<String> built = new ArrayList<>();
        CountingStore real = store("a", "alpha");
        LazyStoreProxy lazy = new LazyStoreProxy(() -> {
            built.add("built");
            return real;
        });

        assertThat(built).isEmpty();
        assertThat(lazy.fetch("a")).hasValue("alpha");
        assertThat(lazy.fetch("zzz")).isEmpty();
        assertThat(lazy.fetch("a")).hasValue("alpha");

        assertThat(built).hasSize(1);
        assertThat(real.requests).containsExactly("a", "zzz", "a");
    }

    @Test
    void shouldCheckThePolicyBeforeEveryFetchAndNeverTouchTheRealStoreWhenDenied() {
        CountingStore real = store("secret", "s3cr3t", "public", "hello");
        List<String> asked = new ArrayList<>();
        AccessPolicy policy = (user, id) -> {
            asked.add(user + ":" + id);
            return id.startsWith("public");
        };
        DocumentStore guarded = new AccessControlStoreProxy(real, policy, "ann");

        assertThat(guarded.fetch("public")).hasValue("hello");
        assertThatThrownBy(() -> guarded.fetch("secret")).isInstanceOf(SecurityException.class);

        assertThat(asked).containsExactly("ann:public", "ann:secret");
        assertThat(real.requests).containsExactly("public");
    }

    @Test
    void shouldRejectBlankIdsBeforeAnythingElseHappens() {
        CountingStore real = store();
        List<String> asked = new ArrayList<>();
        DocumentStore guarded = new AccessControlStoreProxy(real, (u, id) -> asked.add(id), "ann");
        DocumentStore cache = new CachingStoreProxy(real);

        assertThatThrownBy(() -> guarded.fetch(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> guarded.fetch(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> cache.fetch("")).isInstanceOf(IllegalArgumentException.class);

        assertThat(asked).isEmpty();
        assertThat(real.requests).isEmpty();
    }

    @Test
    void shouldStackProxiesBecauseEachOneIsAlsoADocumentStore() {
        CountingStore real = store("secret", "s3cr3t", "public", "hello");
        List<String> built = new ArrayList<>();
        AccessPolicy policy = (user, id) -> id.equals("public");
        CachingStoreProxy stack = new CachingStoreProxy(
                new AccessControlStoreProxy(new LazyStoreProxy(() -> { built.add("x"); return real; }), policy, "bob"));

        assertThat(built).isEmpty();
        assertThatThrownBy(() -> stack.fetch("secret")).isInstanceOf(SecurityException.class);
        assertThat(built).isEmpty();
        assertThat(stack.fetch("public")).hasValue("hello");
        assertThat(stack.fetch("public")).hasValue("hello");

        assertThat(built).hasSize(1);
        assertThat(real.requests).containsExactly("public");
        assertThat(stack.cachedCount()).isEqualTo(1);
    }

    @Test
    void shouldBeSubstitutableForTheRealSubject() {
        for (Class<?> proxy : List.of(CachingStoreProxy.class, LazyStoreProxy.class, AccessControlStoreProxy.class)) {
            assertThat(DocumentStore.class.isAssignableFrom(proxy)).as(proxy.getSimpleName()).isTrue();
        }
        DocumentStore asPlainStore = new CachingStoreProxy(store("a", "alpha"));
        assertThat(asPlainStore.fetch("a")).hasValue("alpha");
    }
}

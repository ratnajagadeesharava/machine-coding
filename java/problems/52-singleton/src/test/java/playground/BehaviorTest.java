package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Constructor;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Set;
import java.util.concurrent.CyclicBarrier;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import solution.AppConfig;

class BehaviorTest {

    @BeforeEach
    void freshStart() {
        AppConfig.resetForTests();
    }

    @Test
    void shouldAlwaysReturnTheSameInstance() {
        AppConfig a = AppConfig.getInstance();
        AppConfig b = AppConfig.getInstance();

        assertThat(a).isSameAs(b);
        assertThat(AppConfig.instancesCreated()).isEqualTo(1);
    }

    @Test
    void shouldShareStateBetweenEveryCaller() {
        AppConfig.getInstance().set("mode", "fast");

        assertThat(AppConfig.getInstance().get("mode")).hasValue("fast");
        assertThat(AppConfig.getInstance().get("missing")).isEmpty();
        assertThatThrownBy(() -> AppConfig.getInstance().get(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> AppConfig.getInstance().set(null, "x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCreateTheInstanceLazily() {
        assertThat(AppConfig.instancesCreated()).isZero();

        AppConfig.getInstance();
        assertThat(AppConfig.instancesCreated()).isEqualTo(1);
        AppConfig.getInstance();
        assertThat(AppConfig.instancesCreated()).isEqualTo(1);
    }

    @Test
    void shouldCreateOnlyOneInstanceEvenWhenManyThreadsAskAtTheSameTime() throws Exception {
        int threads = 16;
        for (int round = 0; round < 300; round++) {
            AppConfig.resetForTests();
            CyclicBarrier startTogether = new CyclicBarrier(threads);
            Set<AppConfig> seen = Collections.newSetFromMap(new IdentityHashMap<>());
            List<Thread> workers = new ArrayList<>();
            for (int i = 0; i < threads; i++) {
                Thread t = new Thread(() -> {
                    try {
                        startTogether.await();
                    } catch (Exception e) {
                        throw new IllegalStateException(e);
                    }
                    AppConfig instance = AppConfig.getInstance();
                    synchronized (seen) {
                        seen.add(instance);
                    }
                });
                workers.add(t);
                t.start();
            }
            for (Thread t : workers) {
                t.join();
            }

            assertThat(seen).as("distinct instances seen in round " + round).hasSize(1);
            assertThat(AppConfig.instancesCreated()).as("constructor runs in round " + round).isEqualTo(1);
        }
    }

    @Test
    void shouldKeepTheStateConsistentUnderConcurrentWrites() throws Exception {
        int threads = 32;
        List<Thread> workers = new ArrayList<>();
        for (int i = 0; i < threads; i++) {
            int id = i;
            Thread t = new Thread(() -> {
                for (int k = 0; k < 100; k++) {
                    AppConfig.getInstance().set("key-" + id + "-" + k, "v" + id);
                }
            });
            workers.add(t);
            t.start();
        }
        for (Thread t : workers) {
            t.join();
        }

        for (int i = 0; i < threads; i++) {
            for (int k = 0; k < 100; k++) {
                assertThat(AppConfig.getInstance().get("key-" + i + "-" + k)).hasValue("v" + i);
            }
        }
    }

    @Test
    void shouldHaveNoWayToConstructItFromOutside() {
        assertThat(Modifier.isFinal(AppConfig.class.getModifiers())).isTrue();
        for (Constructor<?> c : AppConfig.class.getDeclaredConstructors()) {
            assertThat(Modifier.isPrivate(c.getModifiers())).as("constructor must be private").isTrue();
        }
        assertThat(AppConfig.getInstance()).isNotNull();
    }

    @Test
    void shouldGiveAFreshInstanceAfterAReset() {
        AppConfig first = AppConfig.getInstance();
        first.set("a", "1");

        AppConfig.resetForTests();
        AppConfig second = AppConfig.getInstance();

        assertThat(second).isNotSameAs(first);
        assertThat(second.get("a")).isEmpty();
        assertThat(AppConfig.instancesCreated()).isEqualTo(1);
    }
}

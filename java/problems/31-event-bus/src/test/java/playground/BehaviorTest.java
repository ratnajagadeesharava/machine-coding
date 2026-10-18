package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.EventBusApi;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.Eventually;

class BehaviorTest {
    private static final long LONG = 10_000;

    private EventBusApi bus() {
        return Candidate.of(Solution::create);
    }

    private static void await(CountDownLatch latch) {
        try {
            if (!latch.await(LONG, TimeUnit.MILLISECONDS)) {
                throw new AssertionError("gate was never opened");
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }

    private static Consumer<String> into(List<String> sink) {
        return sink::add;
    }

    @Test
    void shouldDeliverEachEventToEverySubscriberOfTheTopicOnly() {
        EventBusApi bus = bus();
        List<String> s1 = new CopyOnWriteArrayList<>();
        List<String> s2 = new CopyOnWriteArrayList<>();
        List<String> other = new CopyOnWriteArrayList<>();
        bus.subscribe("orders", into(s1));
        bus.subscribe("orders", into(s2));
        bus.subscribe("billing", into(other));

        bus.publish("orders", "o-1");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(s1).containsExactly("o-1");
        assertThat(s2).containsExactly("o-1");
        assertThat(other).isEmpty();
    }

    @Test
    void shouldTreatTheSameHandlerSubscribedTwiceAsTwoSubscriptions() {
        EventBusApi bus = bus();
        List<String> sink = new CopyOnWriteArrayList<>();
        Consumer<String> handler = into(sink);

        String first = bus.subscribe("t", handler);
        String second = bus.subscribe("t", handler);
        bus.publish("t", "e");

        assertThat(first).isNotEmpty().isNotEqualTo(second);
        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).containsExactly("e", "e");
    }

    @Test
    void shouldBeIdleWhenNothingWasPublishedAndDiscardEventsWithoutSubscribers() {
        EventBusApi bus = bus();
        assertThat(bus.awaitIdle(0)).isTrue();

        bus.publish("nobody", "lost");
        assertThat(bus.awaitIdle(LONG)).isTrue();

        List<String> sink = new CopyOnWriteArrayList<>();
        bus.subscribe("nobody", into(sink));
        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).isEmpty();
    }

    @Test
    void shouldNotDeliverEventsPublishedBeforeTheSubscription() {
        EventBusApi bus = bus();
        bus.publish("t", "e1");
        assertThat(bus.awaitIdle(LONG)).isTrue();

        List<String> sink = new CopyOnWriteArrayList<>();
        bus.subscribe("t", into(sink));
        bus.publish("t", "e2");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).containsExactly("e2");
    }

    @Test
    void shouldKeepPublishOrderPerSubscriberForOnePublisher() {
        EventBusApi bus = bus();
        List<List<String>> sinks = List.of(new CopyOnWriteArrayList<>(), new CopyOnWriteArrayList<>(), new CopyOnWriteArrayList<>());
        sinks.forEach(s -> bus.subscribe("t", into(s)));
        List<String> expected = new ArrayList<>();

        for (int i = 0; i < 500; i++) {
            expected.add("e" + i);
            bus.publish("t", "e" + i);
        }

        assertThat(bus.awaitIdle(LONG)).isTrue();
        sinks.forEach(s -> assertThat(s).containsExactlyElementsOf(expected));
    }

    @Test
    void shouldKeepEachPublishersOrderWhenSeveralThreadsPublish() {
        EventBusApi bus = bus();
        List<List<String>> sinks = List.of(new CopyOnWriteArrayList<>(), new CopyOnWriteArrayList<>());
        sinks.forEach(s -> bus.subscribe("t", into(s)));

        Concurrently.run(4, t -> {
            for (int i = 0; i < 100; i++) {
                bus.publish("t", t + ":" + i);
            }
        });

        assertThat(bus.awaitIdle(LONG)).isTrue();
        for (List<String> sink : sinks) {
            assertThat(sink).hasSize(400).doesNotHaveDuplicates();
            for (int t = 0; t < 4; t++) {
                int publisher = t;
                List<Integer> seen = sink.stream()
                        .filter(e -> e.startsWith(publisher + ":"))
                        .map(e -> Integer.parseInt(e.substring(2)))
                        .toList();
                assertThat(seen).isSorted().hasSize(100);
            }
        }
    }

    @Test
    void shouldNeverRunOneSubscriptionsHandlerConcurrentlyWithItself() {
        EventBusApi bus = bus();
        AtomicInteger active = new AtomicInteger();
        AtomicBoolean overlapped = new AtomicBoolean();
        AtomicInteger handled = new AtomicInteger();
        bus.subscribe("t", e -> {
            if (active.incrementAndGet() > 1) {
                overlapped.set(true);
            }
            Thread.yield();
            active.decrementAndGet();
            handled.incrementAndGet();
        });

        Concurrently.run(4, t -> {
            for (int i = 0; i < 50; i++) {
                bus.publish("t", "e");
            }
        });

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(handled).hasValue(200);
        assertThat(overlapped).isFalse();
    }

    @Test
    void shouldReturnFromPublishAndReportBusyWhileAHandlerIsBlocked() {
        EventBusApi bus = bus();
        CountDownLatch gate = new CountDownLatch(1);
        List<String> sink = new CopyOnWriteArrayList<>();
        bus.subscribe("t", e -> {
            await(gate);
            sink.add(e);
        });

        Concurrently.collect(1, LONG, i -> {
            for (int n = 0; n < 50; n++) {
                bus.publish("t", "e" + n);
            }
            return null;
        });

        assertThat(bus.awaitIdle(100)).isFalse();
        gate.countDown();
        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).hasSize(50).first().isEqualTo("e0");
        assertThat(sink).last().isEqualTo("e49");
    }

    @Test
    void shouldNotLetASlowHandlerBlockOtherSubscribersOrTopics() {
        EventBusApi bus = bus();
        CountDownLatch gate = new CountDownLatch(1);
        List<String> slow = new CopyOnWriteArrayList<>();
        List<String> sameTopic = new CopyOnWriteArrayList<>();
        List<String> otherTopic = new CopyOnWriteArrayList<>();
        bus.subscribe("a", e -> {
            await(gate);
            slow.add(e);
        });
        bus.subscribe("a", into(sameTopic));
        bus.subscribe("b", into(otherTopic));

        for (int i = 0; i < 20; i++) {
            bus.publish("a", "a" + i);
            bus.publish("b", "b" + i);
        }

        Eventually.assertTrue(() -> sameTopic.size() == 20 && otherTopic.size() == 20, LONG,
                "other subscribers keep receiving while one handler is blocked");
        assertThat(slow).isEmpty();
        gate.countDown();
        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(slow).hasSize(20);
    }

    @Test
    void shouldIsolateHandlerFailures() {
        EventBusApi bus = bus();
        AtomicInteger failures = new AtomicInteger();
        List<String> good = new CopyOnWriteArrayList<>();
        bus.subscribe("t", e -> {
            failures.incrementAndGet();
            throw new IllegalStateException("handler blew up on " + e);
        });
        bus.subscribe("t", into(good));

        bus.publish("t", "e1");
        bus.publish("t", "e2");
        bus.publish("t", "e3");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(good).containsExactly("e1", "e2", "e3");
        assertThat(failures).hasValue(3);
    }

    @Test
    void shouldLetHandlersPublishWithoutDeadlock() {
        EventBusApi bus = bus();
        List<String> chained = new CopyOnWriteArrayList<>();
        bus.subscribe("a", e -> bus.publish("b", e + "!"));
        bus.subscribe("b", e -> bus.publish("c", e + "!"));
        bus.subscribe("c", into(chained));
        List<Integer> countdown = new CopyOnWriteArrayList<>();
        bus.subscribe("count", e -> {
            int n = Integer.parseInt(e);
            countdown.add(n);
            if (n > 0) {
                bus.publish("count", String.valueOf(n - 1));
            }
        });

        bus.publish("a", "x");
        bus.publish("count", "200");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(chained).containsExactly("x!!");
        assertThat(countdown).hasSize(201).first().isEqualTo(200);
        assertThat(countdown).isSortedAccordingTo((x, y) -> y - x);
    }

    @Test
    void shouldStopDeliveriesAfterUnsubscribe() {
        EventBusApi bus = bus();
        List<String> sink = new CopyOnWriteArrayList<>();
        String id = bus.subscribe("t", into(sink));
        bus.publish("t", "e1");
        assertThat(bus.awaitIdle(LONG)).isTrue();

        assertThat(bus.unsubscribe(id)).isTrue();
        bus.publish("t", "e2");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).containsExactly("e1");
        assertThat(bus.unsubscribe(id)).isFalse();
        assertThat(bus.unsubscribe("no-such-id")).isFalse();
    }

    @Test
    void shouldDiscardPendingEventsOfAnUnsubscribedSubscription() {
        EventBusApi bus = bus();
        CountDownLatch started = new CountDownLatch(1);
        CountDownLatch gate = new CountDownLatch(1);
        List<String> sink = new CopyOnWriteArrayList<>();
        String id = bus.subscribe("t", e -> {
            started.countDown();
            await(gate);
            sink.add(e);
        });
        bus.publish("t", "e1");
        await(started);
        bus.publish("t", "e2");
        bus.publish("t", "e3");

        assertThat(bus.unsubscribe(id)).isTrue();
        gate.countDown();

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).containsExactly("e1");
    }

    @Test
    void shouldAllowAHandlerToUnsubscribeItself() {
        EventBusApi bus = bus();
        AtomicReference<String> id = new AtomicReference<>();
        AtomicInteger calls = new AtomicInteger();
        id.set(bus.subscribe("t", e -> {
            calls.incrementAndGet();
            bus.unsubscribe(id.get());
        }));

        bus.publish("t", "e1");
        bus.publish("t", "e2");
        bus.publish("t", "e3");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(calls).hasValue(1);
    }

    @Test
    void shouldDiscardEventsPublishedAfterCloseButDeliverEarlierOnes() {
        EventBusApi bus = bus();
        List<String> sink = new CopyOnWriteArrayList<>();
        bus.subscribe("t", into(sink));
        bus.publish("t", "before");

        bus.close();
        bus.publish("t", "after");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(sink).containsExactly("before");
    }

    @Test
    void shouldStayConsistentWhileSubscriptionsComeAndGoDuringPublishing() {
        EventBusApi bus = bus();
        List<String> stable = new CopyOnWriteArrayList<>();
        bus.subscribe("t", into(stable));

        Concurrently.run(8, t -> {
            if (t < 4) {
                for (int i = 0; i < 200; i++) {
                    bus.publish("t", t + ":" + i);
                }
            } else {
                for (int i = 0; i < 100; i++) {
                    String id = bus.subscribe("t", e -> { });
                    bus.unsubscribe(id);
                }
            }
        });

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(stable).hasSize(800).doesNotHaveDuplicates();
    }
}

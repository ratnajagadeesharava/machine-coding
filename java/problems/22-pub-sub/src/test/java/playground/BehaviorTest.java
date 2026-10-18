package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.PubSubApi;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private PubSubApi broker() {
        return Candidate.of(Solution::create);
    }

    private static List<String> drain(PubSubApi api, String subscriber, String topic) {
        List<String> all = new ArrayList<>();
        while (true) {
            List<String> batch = api.poll(subscriber, topic, 100);
            if (batch.isEmpty()) {
                return all;
            }
            all.addAll(batch);
        }
    }

    /** Messages look like "publisher:sequence"; each publisher's sequence numbers must be strictly increasing. */
    private static void assertPerPublisherOrder(List<String> messages) {
        java.util.Map<String, Integer> last = new java.util.HashMap<>();
        for (String m : messages) {
            String[] parts = m.split(":");
            int seq = Integer.parseInt(parts[1]);
            Integer previous = last.put(parts[0], seq);
            assertThat(previous == null || previous < seq).as("order of %s after %s", m, previous).isTrue();
        }
    }

    @Test
    void shouldCreateTopicsOnlyOnce() {
        PubSubApi api = broker();

        assertThat(api.createTopic("news")).isTrue();
        assertThat(api.createTopic("news")).isFalse();
        assertThat(api.createTopic("")).isFalse();
        assertThat(api.createTopic("   ")).isFalse();
        assertThat(api.createTopic("sports")).isTrue();
    }

    @Test
    void shouldFanOutToEverySubscriberAndReportTheCount() {
        PubSubApi api = broker();
        api.createTopic("news");
        api.subscribe("news", "ann");
        assertThat(api.publish("news", "m1")).isEqualTo(1);
        api.subscribe("news", "bob");

        assertThat(api.publish("news", "m2")).isEqualTo(2);

        assertThat(api.poll("ann", "news", 10)).containsExactly("m1", "m2");
        assertThat(api.poll("bob", "news", 10)).containsExactly("m2");
    }

    @Test
    void shouldReportMinusOneForUnknownTopicAndZeroWithoutSubscribers() {
        PubSubApi api = broker();
        api.createTopic("empty");

        assertThat(api.publish("nope", "x")).isEqualTo(-1);
        assertThat(api.publish("empty", "x")).isZero();
    }

    @Test
    void shouldNotDeliverMessagesPublishedBeforeSubscribing() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.publish("t", "old");

        api.subscribe("t", "late");
        api.publish("t", "new");

        assertThat(api.poll("late", "t", 10)).containsExactly("new");
    }

    @Test
    void shouldReturnOldestFirstAndConsumeWhatWasPolled() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        for (int i = 1; i <= 5; i++) {
            api.publish("t", "m" + i);
        }

        assertThat(api.poll("s", "t", 2)).containsExactly("m1", "m2");
        assertThat(api.poll("s", "t", 2)).containsExactly("m3", "m4");
        assertThat(api.poll("s", "t", 2)).containsExactly("m5");
        assertThat(api.poll("s", "t", 2)).isEmpty();
    }

    @Test
    void shouldReturnNothingAndConsumeNothingForNonPositiveMax() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        api.publish("t", "m1");

        assertThat(api.poll("s", "t", 0)).isEmpty();
        assertThat(api.poll("s", "t", -3)).isEmpty();

        assertThat(api.poll("s", "t", 5)).containsExactly("m1");
    }

    @Test
    void shouldReturnNothingForUnknownTopicOrNonSubscriber() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        api.publish("t", "m1");

        assertThat(api.poll("s", "nope", 5)).isEmpty();
        assertThat(api.poll("stranger", "t", 5)).isEmpty();
        assertThat(api.poll("s", "t", 5)).containsExactly("m1");
    }

    @Test
    void shouldRefuseInvalidSubscriptions() {
        PubSubApi api = broker();
        api.createTopic("t");

        assertThat(api.subscribe("nope", "s")).isFalse();
        assertThat(api.subscribe("t", "")).isFalse();
        assertThat(api.subscribe("t", "s")).isTrue();
        assertThat(api.subscribe("t", "s")).isFalse();
        assertThat(api.publish("t", "m")).isEqualTo(1);
    }

    @Test
    void shouldStopDeliveringAndDiscardPendingMessagesOnUnsubscribe() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        api.publish("t", "m1");

        assertThat(api.unsubscribe("t", "s")).isTrue();
        assertThat(api.unsubscribe("t", "s")).isFalse();
        assertThat(api.unsubscribe("nope", "s")).isFalse();

        assertThat(api.publish("t", "m2")).isZero();
        assertThat(api.poll("s", "t", 10)).isEmpty();
    }

    @Test
    void shouldKeepSubscribersQueuesIndependent() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "a");
        api.subscribe("t", "b");
        api.publish("t", "m1");
        api.publish("t", "m2");

        assertThat(api.poll("a", "t", 10)).containsExactly("m1", "m2");

        assertThat(api.poll("b", "t", 1)).containsExactly("m1");
        assertThat(api.poll("b", "t", 1)).containsExactly("m2");
    }

    @Test
    void shouldKeepTopicsSeparateForTheSameSubscriber() {
        PubSubApi api = broker();
        api.createTopic("x");
        api.createTopic("y");
        api.subscribe("x", "s");
        api.subscribe("y", "s");

        api.publish("x", "fromX");
        api.publish("y", "fromY");

        assertThat(api.poll("s", "y", 10)).containsExactly("fromY");
        assertThat(api.poll("s", "x", 10)).containsExactly("fromX");
    }

    @Test
    void shouldReturnACopyThatLaterPollsDoNotAffect() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        api.publish("t", "m1");

        List<String> first = api.poll("s", "t", 10);
        api.publish("t", "m2");
        api.poll("s", "t", 10);

        assertThat(first).containsExactly("m1");
    }

    @Test
    void shouldDeliverEverythingInPerPublisherOrderToAStableSubscriber() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "s");
        int publishers = 8;
        int perPublisher = 500;

        Concurrently.run(publishers, p -> {
            for (int i = 0; i < perPublisher; i++) {
                api.publish("t", p + ":" + i);
            }
        });

        List<String> received = drain(api, "s", "t");
        assertThat(received).hasSize(publishers * perPublisher);
        assertThat(new HashSet<>(received)).hasSize(publishers * perPublisher);
        assertPerPublisherOrder(received);
    }

    @Test
    void shouldGiveEverySubscriberTheSameMessagesUnderConcurrentPublishing() {
        PubSubApi api = broker();
        api.createTopic("t");
        for (int s = 0; s < 5; s++) {
            api.subscribe("t", "s" + s);
        }

        Concurrently.run(6, p -> {
            for (int i = 0; i < 200; i++) {
                assertThat(api.publish("t", p + ":" + i)).isEqualTo(5);
            }
        });

        for (int s = 0; s < 5; s++) {
            List<String> received = drain(api, "s" + s, "t");
            assertThat(received).hasSize(1200);
            assertPerPublisherOrder(received);
        }
    }

    @Test
    void shouldHandOutEachMessageOnceWhenSeveralThreadsPollTheSameSubscriber() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "shared");
        int total = 4 * 500;
        AtomicInteger consumed = new AtomicInteger();

        List<List<String>> perThread = Concurrently.collect(8, i -> {
            List<String> mine = new ArrayList<>();
            if (i < 4) {
                for (int n = 0; n < 500; n++) {
                    api.publish("t", i + ":" + n);
                }
                return mine;
            }
            long deadline = System.nanoTime() + Duration.ofSeconds(20).toNanos();
            while (consumed.get() < total && System.nanoTime() < deadline) {
                List<String> batch = api.poll("shared", "t", 7);
                consumed.addAndGet(batch.size());
                mine.addAll(batch);
            }
            return mine;
        });

        List<String> all = new ArrayList<>();
        for (List<String> mine : perThread) {
            assertPerPublisherOrder(mine);
            all.addAll(mine);
        }
        assertThat(all).hasSize(total);
        assertThat(new HashSet<>(all)).hasSize(total);
    }

    @Test
    void shouldStayConsistentWhileSubscribersComeAndGoDuringPublishing() {
        PubSubApi api = broker();
        api.createTopic("t");
        api.subscribe("t", "stable");
        ConcurrentLinkedQueue<Integer> counts = new ConcurrentLinkedQueue<>();

        List<List<String>> received = Concurrently.collect(8, i -> {
            List<String> mine = new ArrayList<>();
            if (i < 4) {
                for (int n = 0; n < 300; n++) {
                    counts.add(api.publish("t", i + ":" + n));
                }
                return mine;
            }
            String me = "churn" + i;
            for (int round = 0; round < 100; round++) {
                api.subscribe("t", me);
                mine.addAll(api.poll(me, "t", 50));
                api.unsubscribe("t", me);
            }
            return mine;
        });

        assertThat(counts).hasSize(1200).allMatch(c -> c >= 1 && c <= 5);
        List<String> stable = drain(api, "stable", "t");
        assertThat(stable).hasSize(1200);
        assertThat(new HashSet<>(stable)).hasSize(1200);
        assertPerPublisherOrder(stable);
        for (List<String> mine : received) {
            assertThat(new HashSet<>(mine)).hasSize(mine.size());
            assertPerPublisherOrder(mine);
        }
    }

    @Test
    void shouldAllowOnlyOneWinnerWhenCreatingOrSubscribingConcurrently() {
        PubSubApi api = broker();

        List<Boolean> created = Concurrently.collect(16, i -> api.createTopic("same"));
        assertThat(created.stream().filter(b -> b).count()).isEqualTo(1);

        List<Boolean> subscribed = Concurrently.collect(16, i -> api.subscribe("same", "dup"));
        assertThat(subscribed.stream().filter(b -> b).count()).isEqualTo(1);
        assertThat(api.publish("same", "m")).isEqualTo(1);
    }

    @Test
    void shouldHandleManyMessagesAndSubscribersQuickly() {
        PubSubApi api = broker();
        api.createTopic("t");
        for (int s = 0; s < 10; s++) {
            api.subscribe("t", "s" + s);
        }

        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(10), () -> {
            for (int i = 0; i < 50_000; i++) {
                api.publish("t", "m" + i);
            }
            for (int s = 0; s < 10; s++) {
                int count = 0;
                while (true) {
                    List<String> batch = api.poll("s" + s, "t", 500);
                    if (batch.isEmpty()) {
                        break;
                    }
                    count += batch.size();
                }
                assertThat(count).isEqualTo(50_000);
            }
        });
    }
}

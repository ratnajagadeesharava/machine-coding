package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.EventBusApi;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {
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

    @Test
    void aSubscriptionCreatedWhileAnEventIsStillPendingMustNotReceiveIt() {
        EventBusApi bus = bus();
        CountDownLatch started = new CountDownLatch(1);
        CountDownLatch gate = new CountDownLatch(1);
        List<String> a = new CopyOnWriteArrayList<>();
        List<String> b = new CopyOnWriteArrayList<>();
        bus.subscribe("t", e -> {
            started.countDown();
            await(gate);
            a.add(e);
        });
        bus.publish("t", "e1");
        await(started);
        bus.publish("t", "e2");

        bus.subscribe("t", b::add);
        bus.publish("t", "e3");
        gate.countDown();

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(a).containsExactly("e1", "e2", "e3");
        assertThat(b).containsExactly("e3");
    }

    @Test
    void anAlreadyRunningHandlerFinishesAfterUnsubscribe() {
        EventBusApi bus = bus();
        CountDownLatch started = new CountDownLatch(1);
        CountDownLatch gate = new CountDownLatch(1);
        List<String> finished = new CopyOnWriteArrayList<>();
        String id = bus.subscribe("t", e -> {
            started.countDown();
            await(gate);
            finished.add(e);
        });
        bus.publish("t", "e1");
        await(started);

        assertThat(bus.unsubscribe(id)).isTrue();
        assertThat(bus.awaitIdle(100)).isFalse(); // the running invocation still counts as work
        gate.countDown();

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(finished).containsExactly("e1");
    }

    @Test
    void aHandlerThatPublishesAndThenFailsStillGetsItsPublishedEventDelivered() {
        EventBusApi bus = bus();
        List<String> out = new CopyOnWriteArrayList<>();
        bus.subscribe("in", e -> {
            bus.publish("out", e + "-out");
            throw new IllegalStateException("after publishing");
        });
        bus.subscribe("out", out::add);

        bus.publish("in", "x");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(out).containsExactly("x-out");
    }

    @Test
    void closeIsIdempotentAndLaterSubscriptionsNeverReceive() {
        EventBusApi bus = bus();
        List<String> late = new CopyOnWriteArrayList<>();

        bus.close();
        bus.close();
        String id = bus.subscribe("t", late::add);
        bus.publish("t", "e");

        assertThat(id).isNotEmpty();
        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(late).isEmpty();
    }

    @Test
    void topicsAreCaseSensitiveAndEmptyEventsAreEvents() {
        EventBusApi bus = bus();
        List<String> lower = new CopyOnWriteArrayList<>();
        List<String> upper = new CopyOnWriteArrayList<>();
        bus.subscribe("orders", lower::add);
        bus.subscribe("Orders", upper::add);

        bus.publish("orders", "");
        bus.publish("Orders", "  spaced  ");

        assertThat(bus.awaitIdle(LONG)).isTrue();
        assertThat(lower).containsExactly("");
        assertThat(upper).containsExactly("  spaced  ");
    }
}

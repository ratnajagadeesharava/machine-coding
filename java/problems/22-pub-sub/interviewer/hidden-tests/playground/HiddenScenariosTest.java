package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.PubSubApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private PubSubApi broker() {
        PubSubApi api = Candidate.of(Solution::create);
        api.createTopic("t");
        return api;
    }

    @Test
    void resubscribingShouldNotBringBackMessagesFromBeforeUnsubscribe() {
        PubSubApi api = broker();
        api.subscribe("t", "s");
        api.publish("t", "m1");
        api.unsubscribe("t", "s");
        api.publish("t", "m2");

        assertThat(api.subscribe("t", "s")).isTrue();
        api.publish("t", "m3");

        assertThat(api.poll("s", "t", 10)).containsExactly("m3");
    }

    @Test
    void messagesPublishedWithoutSubscribersShouldNotBeKeptForLaterSubscribers() {
        PubSubApi api = broker();
        assertThat(api.publish("t", "lost")).isZero();

        api.subscribe("t", "s");

        assertThat(api.poll("s", "t", 10)).isEmpty();
    }

    @Test
    void partialPollShouldLeaveRemainderAheadOfNewMessages() {
        PubSubApi api = broker();
        api.subscribe("t", "s");
        api.publish("t", "m1");
        api.publish("t", "m2");
        api.publish("t", "m3");

        assertThat(api.poll("s", "t", 1)).containsExactly("m1");
        api.publish("t", "m4");

        assertThat(api.poll("s", "t", 10)).containsExactly("m2", "m3", "m4");
    }

    @Test
    void sameTextPublishedTwiceShouldBeDeliveredTwice() {
        PubSubApi api = broker();
        api.subscribe("t", "s");

        api.publish("t", "ping");
        api.publish("t", "ping");

        assertThat(api.poll("s", "t", 10)).containsExactly("ping", "ping");
    }

    @Test
    void unsubscribedSubscriberShouldNotCountInFanOutAndOthersKeepTheirMessages() {
        PubSubApi api = broker();
        api.subscribe("t", "a");
        api.subscribe("t", "b");
        api.publish("t", "m1");
        api.unsubscribe("t", "a");

        assertThat(api.publish("t", "m2")).isEqualTo(1);

        assertThat(api.poll("b", "t", 10)).containsExactly("m1", "m2");
        assertThat(api.poll("a", "t", 10)).isEmpty();
    }
}

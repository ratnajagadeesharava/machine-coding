package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Subscription;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.PriceFeed;

class BehaviorTest {

    private final List<String> log = new ArrayList<>();

    @Test
    void shouldNotifyObserversInSubscriptionOrder() {
        PriceFeed feed = new PriceFeed();
        feed.subscribe((s, p) -> log.add("first " + s + "=" + p));
        feed.subscribe((s, p) -> log.add("second " + s + "=" + p));
        feed.subscribe((s, p) -> log.add("third " + s + "=" + p));

        int failures = feed.publish("ACME", 1250);

        assertThat(failures).isZero();
        assertThat(log).containsExactly("first ACME=1250", "second ACME=1250", "third ACME=1250");
    }

    @Test
    void shouldDeliverOnlyTheRequestedSymbolToFilteredSubscribers() {
        PriceFeed feed = new PriceFeed();
        feed.subscribe("ACME", (s, p) -> log.add("acme-only " + p));
        feed.subscribe((s, p) -> log.add("all " + s));

        feed.publish("ACME", 100);
        feed.publish("INIT", 200);

        assertThat(log).containsExactly("acme-only 100", "all ACME", "all INIT");
    }

    @Test
    void shouldStopNotifyingAfterCancelAndAllowCancellingTwice() {
        PriceFeed feed = new PriceFeed();
        Subscription sub = feed.subscribe((s, p) -> log.add("a " + p));
        feed.subscribe((s, p) -> log.add("b " + p));
        feed.publish("X", 1);

        sub.cancel();
        sub.cancel();
        feed.publish("X", 2);

        assertThat(sub.isActive()).isFalse();
        assertThat(feed.subscriberCount()).isEqualTo(1);
        assertThat(log).containsExactly("a 1", "b 1", "b 2");
    }

    @Test
    void shouldNotNotifyObserversThatSubscribeDuringANotification() {
        PriceFeed feed = new PriceFeed();
        feed.subscribe((s, p) -> {
            log.add("early " + p);
            feed.subscribe((s2, p2) -> log.add("late " + p2));
        });

        feed.publish("X", 1);
        feed.publish("X", 2);

        assertThat(log).containsExactly("early 1", "early 2", "late 2");
    }

    @Test
    void shouldSkipAnObserverThatIsCancelledBeforeItsTurnEvenDuringThePublish() {
        PriceFeed feed = new PriceFeed();
        Subscription[] second = new Subscription[1];
        feed.subscribe((s, p) -> {
            log.add("first " + p);
            second[0].cancel();
        });
        second[0] = feed.subscribe((s, p) -> log.add("second " + p));
        feed.subscribe((s, p) -> log.add("third " + p));

        feed.publish("X", 1);

        assertThat(log).containsExactly("first 1", "third 1");
    }

    @Test
    void shouldKeepNotifyingTheOthersWhenOneObserverThrows() {
        PriceFeed feed = new PriceFeed();
        feed.subscribe((s, p) -> log.add("ok-1"));
        feed.subscribe((s, p) -> {
            throw new IllegalStateException("boom");
        });
        feed.subscribe((s, p) -> {
            throw new UnsupportedOperationException("boom too");
        });
        feed.subscribe((s, p) -> log.add("ok-2"));

        int failures = feed.publish("X", 5);

        assertThat(failures).isEqualTo(2);
        assertThat(log).containsExactly("ok-1", "ok-2");
        assertThat(feed.subscriberCount()).isEqualTo(4);
    }

    @Test
    void shouldNotifyTwiceWhenTheSameObserverSubscribesTwice() {
        PriceFeed feed = new PriceFeed();
        contract.PriceObserver observer = (s, p) -> log.add("hit " + p);
        feed.subscribe(observer);
        Subscription second = feed.subscribe(observer);

        feed.publish("X", 1);
        second.cancel();
        feed.publish("X", 2);

        assertThat(log).containsExactly("hit 1", "hit 1", "hit 2");
    }

    @Test
    void shouldHandlePublishingWithNoSubscribersAndRejectBadInput() {
        PriceFeed feed = new PriceFeed();

        assertThat(feed.publish("X", 1)).isZero();
        assertThatThrownBy(() -> feed.subscribe(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> feed.subscribe("X", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> feed.subscribe(" ", (s, p) -> { })).isInstanceOf(IllegalArgumentException.class);
        feed.subscribe((s, p) -> log.add("called"));
        assertThatThrownBy(() -> feed.publish(" ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> feed.publish("X", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(log).isEmpty();
    }
}

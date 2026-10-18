package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.MessageSink;
import java.lang.reflect.Constructor;
import java.time.Clock;
import java.time.Instant;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.CountingSink;
import solution.RedactingSink;
import solution.RetryingSink;
import solution.TimestampingSink;

class BehaviorTest {

    private static final Clock CLOCK = Clock.fixed(Instant.ofEpochMilli(1_700_000_000_000L), ZoneOffset.UTC);

    /** The innermost component, written in the test: records what finally arrives. Can fail the first N sends. */
    private static final class Recorder implements MessageSink {
        final List<String> received = new ArrayList<>();
        int attempts;
        int failuresLeft;

        Recorder(int failures) { this.failuresLeft = failures; }

        public void send(String message) {
            attempts++;
            if (failuresLeft > 0) {
                failuresLeft--;
                throw new IllegalStateException("network down #" + attempts);
            }
            received.add(message);
        }
    }

    @Test
    void shouldPrefixTheClockTime() {
        Recorder rec = new Recorder(0);

        new TimestampingSink(rec, CLOCK).send("hello");

        assertThat(rec.received).containsExactly("[t=1700000000000] hello");
    }

    @Test
    void shouldRedactLongDigitRunsOnly() {
        Recorder rec = new Recorder(0);

        new RedactingSink(rec).send("card 4111111111111111 exp 1226 room 101 floor 7");

        assertThat(rec.received).containsExactly("card **** exp **** room 101 floor 7");
    }

    @Test
    void shouldRetryUntilTheNextSinkSucceeds() {
        Recorder rec = new Recorder(2);

        new RetryingSink(rec, 3).send("important");

        assertThat(rec.attempts).isEqualTo(3);
        assertThat(rec.received).containsExactly("important");
    }

    @Test
    void shouldGiveUpAfterTheMaximumNumberOfAttemptsAndRethrowTheLastFailure() {
        Recorder rec = new Recorder(5);
        MessageSink sink = new RetryingSink(rec, 2);

        assertThatThrownBy(() -> sink.send("x")).isInstanceOf(IllegalStateException.class).hasMessage("network down #2");

        assertThat(rec.attempts).isEqualTo(2);
        assertThat(rec.received).isEmpty();
    }

    @Test
    void shouldCountOnlyDeliveredMessages() {
        Recorder rec = new Recorder(1);
        CountingSink counting = new CountingSink(rec);

        assertThatThrownBy(() -> counting.send("lost")).isInstanceOf(IllegalStateException.class);
        counting.send("a");
        counting.send("b");

        assertThat(counting.count()).isEqualTo(2);
        assertThat(rec.received).containsExactly("a", "b");
    }

    @Test
    void shouldLetTheOrderOfWrappingChangeTheResult() {
        Recorder redactFirst = new Recorder(0);
        Recorder timestampFirst = new Recorder(0);

        new RedactingSink(new TimestampingSink(redactFirst, CLOCK)).send("pin 123456");
        new TimestampingSink(new RedactingSink(timestampFirst), CLOCK).send("pin 123456");

        assertThat(redactFirst.received).containsExactly("[t=1700000000000] pin ****");
        assertThat(timestampFirst.received).containsExactly("[t=****] pin ****");
    }

    @Test
    void shouldStackManyDecoratorsAroundOneComponent() {
        Recorder rec = new Recorder(1);
        CountingSink counting = new CountingSink(new RetryingSink(new RedactingSink(new TimestampingSink(rec, CLOCK)), 3));

        counting.send("otp 998877");
        counting.send("ok");

        assertThat(counting.count()).isEqualTo(2);
        assertThat(rec.attempts).isEqualTo(3);
        assertThat(rec.received).containsExactly("[t=1700000000000] otp ****", "[t=1700000000000] ok");
    }

    @Test
    void shouldBeInterchangeableWithTheComponentItWraps() throws Exception {
        for (Class<?> type : List.of(TimestampingSink.class, RedactingSink.class, RetryingSink.class, CountingSink.class)) {
            assertThat(MessageSink.class.isAssignableFrom(type)).as(type.getSimpleName()).isTrue();
            assertThat(type.getConstructors()).hasSize(1);
            Constructor<?> c = type.getConstructors()[0];
            assertThat(c.getParameterTypes()[0]).isEqualTo(MessageSink.class);
        }
        assertThatThrownBy(() -> new RedactingSink(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new CountingSink(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new RetryingSink(new Recorder(0), 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new TimestampingSink(new Recorder(0), null)).isInstanceOf(IllegalArgumentException.class);
    }
}

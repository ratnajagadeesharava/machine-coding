package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.List;
import java.util.stream.Stream;
import org.junit.jupiter.api.Test;
import solution.Ticket;

class BehaviorTest {

    private static final Instant START = Instant.parse("2026-05-01T09:00:00Z");

    private static final class FakeClock extends Clock {
        Instant now = START;

        public ZoneId getZone() { return ZoneOffset.UTC; }
        public Clock withZone(ZoneId zone) { return this; }
        public Instant instant() { return now; }
    }

    private final FakeClock clock = new FakeClock();

    private Ticket ticket() {
        return new Ticket("T-1", clock);
    }

    private Ticket inProgress() {
        Ticket t = ticket();
        t.assign("ann");
        return t;
    }

    private Ticket onHold() {
        Ticket t = inProgress();
        t.hold("waiting for customer");
        return t;
    }

    private Ticket resolved() {
        Ticket t = inProgress();
        t.resolve("fixed");
        return t;
    }

    private Ticket closed() {
        Ticket t = resolved();
        t.close();
        return t;
    }

    @Test
    void shouldStartNewAndUnassigned() {
        Ticket t = ticket();

        assertThat(t.id()).isEqualTo("T-1");
        assertThat(t.state()).isEqualTo("NEW");
        assertThat(t.assignedTo()).isEmpty();
        assertThat(t.comments()).isEmpty();
    }

    @Test
    void shouldMoveThroughTheHappyPath() {
        Ticket t = ticket();

        t.assign("ann");
        assertThat(t.state()).isEqualTo("IN_PROGRESS");
        assertThat(t.assignedTo()).hasValue("ann");
        t.hold("waiting for customer");
        assertThat(t.state()).isEqualTo("ON_HOLD");
        t.resume();
        assertThat(t.state()).isEqualTo("IN_PROGRESS");
        t.resolve("restarted the server");
        assertThat(t.state()).isEqualTo("RESOLVED");
        t.close();

        assertThat(t.state()).isEqualTo("CLOSED");
        assertThat(t.comments()).containsExactly("HOLD: waiting for customer", "RESOLVED: restarted the server");
    }

    @Test
    void shouldAllowReassigningWhileWorkIsOngoingWithoutChangingState() {
        Ticket progress = inProgress();
        progress.assign("bob");
        Ticket hold = onHold();
        hold.assign("cy");

        assertThat(progress.state()).isEqualTo("IN_PROGRESS");
        assertThat(progress.assignedTo()).hasValue("bob");
        assertThat(hold.state()).isEqualTo("ON_HOLD");
        assertThat(hold.assignedTo()).hasValue("cy");
    }

    @Test
    void shouldReopenAResolvedTicketAtMostTwice() {
        Ticket t = resolved();

        t.reopen();
        assertThat(t.state()).isEqualTo("IN_PROGRESS");
        t.resolve("again");
        t.reopen();
        t.resolve("and again");

        assertThatThrownBy(t::reopen).isInstanceOf(IllegalStateException.class);
        assertThat(t.state()).isEqualTo("RESOLVED");
        assertThat(t.comments()).containsExactly(
                "RESOLVED: fixed", "REOPENED", "RESOLVED: again", "REOPENED", "RESOLVED: and again");
    }

    @Test
    void shouldRejectOperationsThatAreNotAllowedInTheCurrentState() {
        assertThatThrownBy(() -> ticket().resolve("x")).isInstanceOf(IllegalStateException.class)
                .hasMessage("cannot resolve a ticket in state NEW");
        assertThatThrownBy(() -> ticket().hold("x")).hasMessage("cannot hold a ticket in state NEW");
        assertThatThrownBy(() -> ticket().close()).hasMessage("cannot close a ticket in state NEW");
        assertThatThrownBy(() -> inProgress().resume()).hasMessage("cannot resume a ticket in state IN_PROGRESS");
        assertThatThrownBy(() -> inProgress().reopen()).hasMessage("cannot reopen a ticket in state IN_PROGRESS");
        assertThatThrownBy(() -> onHold().resolve("x")).hasMessage("cannot resolve a ticket in state ON_HOLD");
        assertThatThrownBy(() -> onHold().hold("x")).hasMessage("cannot hold a ticket in state ON_HOLD");
        assertThatThrownBy(() -> resolved().assign("x")).hasMessage("cannot assign a ticket in state RESOLVED");
        assertThatThrownBy(() -> resolved().hold("x")).hasMessage("cannot hold a ticket in state RESOLVED");
    }

    @Test
    void shouldTreatClosedAsTerminal() {
        Ticket t = closed();

        assertThat(t.state()).isEqualTo("CLOSED");
        assertThatThrownBy(() -> t.assign("x")).hasMessage("cannot assign a ticket in state CLOSED");
        assertThatThrownBy(() -> t.hold("x")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(t::resume).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> t.resolve("x")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(t::reopen).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(t::close).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> t.comment("late")).hasMessage("cannot comment a ticket in state CLOSED");
        assertThat(t.state()).isEqualTo("CLOSED");
    }

    @Test
    void shouldAllowCommentsInEveryStateExceptClosedInTheOrderTheyWereAdded() {
        Ticket t = ticket();

        t.comment("one");
        t.assign("ann");
        t.comment("two");
        t.hold("why");
        t.comment("three");
        t.resume();
        t.resolve("done");
        t.comment("four");

        assertThat(t.comments()).containsExactly("one", "two", "HOLD: why", "three", "RESOLVED: done", "four");
        assertThatThrownBy(() -> t.comments().add("hack")).isInstanceOf(UnsupportedOperationException.class);
    }

    @Test
    void shouldCloseResolvedTicketsAfterSeventyTwoHoursOnly() {
        Ticket stale = resolved();
        Ticket inProgress = inProgress();
        Ticket fresh = ticket();

        clock.now = START.plus(Duration.ofHours(71)).plus(Duration.ofMinutes(59));
        assertThat(stale.autoCloseIfStale()).isFalse();
        assertThat(stale.state()).isEqualTo("RESOLVED");

        clock.now = START.plus(Duration.ofHours(72));
        assertThat(stale.autoCloseIfStale()).isTrue();
        assertThat(stale.state()).isEqualTo("CLOSED");
        assertThat(stale.autoCloseIfStale()).isFalse();
        assertThat(inProgress.autoCloseIfStale()).isFalse();
        assertThat(fresh.autoCloseIfStale()).isFalse();
        assertThat(inProgress.state()).isEqualTo("IN_PROGRESS");
    }

    @Test
    void shouldCheckArgumentsBeforeStateAndLeaveTheTicketUntouchedOnFailure() {
        Ticket t = ticket();

        assertThatThrownBy(() -> t.assign(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.assign(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.comment("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> inProgress().hold(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> inProgress().resolve("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Ticket(" ", clock)).isInstanceOf(IllegalArgumentException.class);

        assertThat(t.state()).isEqualTo("NEW");
        assertThat(t.assignedTo()).isEmpty();
        assertThat(t.comments()).isEmpty();
    }

    @Test
    void shouldBeBuiltFromStateObjectsNotFromSwitchingOnAStateValue() throws IOException {
        assertThat(ticket().state()).isEqualTo("NEW");

        Path dir = Path.of("src/main/java/solution");
        List<Path> files;
        try (Stream<Path> s = Files.list(dir)) {
            files = s.filter(p -> p.toString().endsWith(".java")).toList();
        }
        assertThat(files.size()).as("Ticket plus one class per state: at least 6 source files").isGreaterThanOrEqualTo(6);

        StringBuilder code = new StringBuilder();
        for (Path file : files) {
            for (String line : Files.readAllLines(file)) {
                String t = line.strip();
                if (!t.isEmpty() && !t.startsWith("//") && !t.startsWith("/*") && !t.startsWith("*")) {
                    code.append(t).append('\n');
                }
            }
        }
        for (String smell : List.of("switch", "state ==", "state.equals", ".ordinal()", "getClass()")) {
            assertThat(code.toString()).as("state-switching smell: " + smell).doesNotContain(smell);
        }
    }
}

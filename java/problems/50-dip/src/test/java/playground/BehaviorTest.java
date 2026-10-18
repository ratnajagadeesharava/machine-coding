package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Appointment;
import contract.AppointmentRepository;
import contract.MessageSender;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.ReminderService;

class BehaviorTest {

    private static final Instant NOW = Instant.parse("2026-03-10T08:00:00Z");

    private static final class FakeClock extends Clock {
        Instant now;

        FakeClock(Instant now) { this.now = now; }

        public ZoneId getZone() { return ZoneOffset.UTC; }
        public Clock withZone(ZoneId zone) { return this; }
        public Instant instant() { return now; }
    }

    private static final class FakeSender implements MessageSender {
        final List<String> sent = new ArrayList<>();
        final Set<String> failFor = new HashSet<>();
        final Set<String> throwFor = new HashSet<>();

        public boolean send(String phone, String text) {
            if (throwFor.contains(phone)) throw new IllegalStateException("gateway down");
            sent.add(phone + " | " + text);
            return !failFor.contains(phone);
        }
    }

    private static final class FakeRepository implements AppointmentRepository {
        final List<Appointment> all = new ArrayList<>();
        final Set<String> reminded = new HashSet<>();
        final List<Instant[]> queries = new ArrayList<>();

        public List<Appointment> findBetween(Instant from, Instant to) {
            queries.add(new Instant[] {from, to});
            return all.stream()
                    .filter(a -> !reminded.contains(a.id()))
                    .filter(a -> !a.time().isBefore(from) && !a.time().isAfter(to))
                    .toList();
        }

        public void markReminded(String id) { reminded.add(id); }
    }

    private final FakeSender sender = new FakeSender();
    private final FakeRepository repo = new FakeRepository();
    private final FakeClock clock = new FakeClock(NOW);

    private ReminderService service() {
        return new ReminderService(sender, repo, clock);
    }

    private void appointment(String id, String phone, Duration fromNow) {
        repo.all.add(new Appointment(id, phone, NOW.plus(fromNow)));
    }

    @Test
    void shouldAskTheRepositoryForTheNextTwentyFourHoursUsingTheInjectedClock() {
        service().sendDueReminders();

        assertThat(repo.queries).hasSize(1);
        assertThat(repo.queries.get(0)[0]).isEqualTo(NOW);
        assertThat(repo.queries.get(0)[1]).isEqualTo(NOW.plus(Duration.ofHours(24)));
    }

    @Test
    void shouldSendTheMessageToTheRightPhoneAndMarkTheAppointment() {
        appointment("a1", "+100", Duration.ofHours(3));

        int sent = service().sendDueReminders();

        assertThat(sent).isEqualTo(1);
        assertThat(sender.sent).containsExactly("+100 | Reminder: appointment at 2026-03-10T11:00:00Z");
        assertThat(repo.reminded).containsExactly("a1");
    }

    @Test
    void shouldNotMarkAnAppointmentWhenTheSenderRefusesAndShouldCarryOn() {
        appointment("a1", "+100", Duration.ofHours(1));
        appointment("a2", "+200", Duration.ofHours(2));
        appointment("a3", "+300", Duration.ofHours(3));
        sender.failFor.add("+200");

        int sent = service().sendDueReminders();

        assertThat(sent).isEqualTo(2);
        assertThat(repo.reminded).containsExactlyInAnyOrder("a1", "a3");
    }

    @Test
    void shouldSurviveASenderThatThrows() {
        appointment("a1", "+100", Duration.ofHours(1));
        appointment("a2", "+200", Duration.ofHours(2));
        sender.throwFor.add("+100");

        int sent = service().sendDueReminders();

        assertThat(sent).isEqualTo(1);
        assertThat(repo.reminded).containsExactly("a2");
    }

    @Test
    void shouldDoNothingWhenNothingIsDue() {
        appointment("late", "+100", Duration.ofHours(30));
        appointment("past", "+200", Duration.ofMinutes(-5));

        assertThat(service().sendDueReminders()).isZero();
        assertThat(sender.sent).isEmpty();
    }

    @Test
    void shouldRetryOnlyWhatFailedOnTheNextRun() {
        appointment("a1", "+100", Duration.ofHours(1));
        appointment("a2", "+200", Duration.ofHours(2));
        sender.failFor.add("+200");
        ReminderService service = service();
        service.sendDueReminders();
        sender.sent.clear();
        sender.failFor.clear();

        int second = service.sendDueReminders();

        assertThat(second).isEqualTo(1);
        assertThat(sender.sent).hasSize(1).first().asString().startsWith("+200");
    }

    @Test
    void shouldFollowTheClockWhenTimePasses() {
        appointment("tomorrow", "+100", Duration.ofHours(30));
        ReminderService service = service();
        assertThat(service.sendDueReminders()).isZero();

        clock.now = NOW.plus(Duration.ofHours(10));

        assertThat(service.sendDueReminders()).isEqualTo(1);
        assertThat(repo.queries.get(1)[0]).isEqualTo(NOW.plus(Duration.ofHours(10)));
    }

    @Test
    void shouldDependOnAbstractionsOnlyThroughItsConstructor() {
        assertThat(ReminderService.class.getConstructors()).hasSize(1);
        assertThat(ReminderService.class.getConstructors()[0].getParameterTypes())
                .containsExactly(MessageSender.class, AppointmentRepository.class, Clock.class);
        assertThat(service().sendDueReminders()).isZero();
    }
}

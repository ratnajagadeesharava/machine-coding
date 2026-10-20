package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CalendarApi;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private CalendarApi cal() {
        return Candidate.of(Solution::create);
    }

    @Test
    void shouldScheduleAndListAnAgendaSortedByStart() {
        CalendarApi c = cal();
        c.schedule("ann", "standup", 600, 630, List.of("bob"));
        c.schedule("ann", "lunch", 720, 780, List.of());
        c.schedule("bob", "1:1", 540, 570, List.of("ann"));

        assertThat(c.agenda("ann", 0, 2_000)).containsExactly("540-570:1:1", "600-630:standup", "720-780:lunch");
        assertThat(c.agenda("bob", 0, 2_000)).containsExactly("540-570:1:1", "600-630:standup");
        assertThat(c.agenda("cy", 0, 2_000)).isEmpty();
    }

    @Test
    void shouldIncludeEventsThatOverlapTheAgendaWindow() {
        CalendarApi c = cal();
        c.schedule("ann", "a", 100, 200, List.of());
        c.schedule("ann", "b", 200, 300, List.of());

        assertThat(c.agenda("ann", 150, 200)).containsExactly("100-200:a");
        assertThat(c.agenda("ann", 200, 250)).containsExactly("200-300:b");
        assertThat(c.agenda("ann", 300, 400)).isEmpty();
    }

    @Test
    void shouldAllowEventsThatOnlyTouch() {
        CalendarApi c = cal();
        c.schedule("ann", "a", 100, 200, List.of("bob"));

        c.schedule("bob", "b", 200, 300, List.of("ann"));
        c.schedule("ann", "c", 0, 100, List.of());

        assertThat(c.agenda("ann", 0, 1_000)).hasSize(3);
    }

    @Test
    void shouldRefuseAnEventThatConflictsForAnyParticipantAndCreateNothing() {
        CalendarApi c = cal();
        c.schedule("bob", "busy", 100, 200, List.of());

        assertThatThrownBy(() -> c.schedule("ann", "meeting", 150, 250, List.of("bob", "cy")))
                .isInstanceOf(IllegalStateException.class);

        assertThat(c.agenda("ann", 0, 1_000)).isEmpty();
        assertThat(c.agenda("cy", 0, 1_000)).isEmpty();
    }

    @Test
    void shouldRejectInvalidEvents() {
        CalendarApi c = cal();

        assertThatThrownBy(() -> c.schedule("ann", "x", 100, 100, List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.schedule("ann", "x", 200, 100, List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.schedule("ann", " ", 100, 200, List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.schedule("ann", "x", 100, 200, List.of("bob", "bob"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.schedule("ann", "x", 100, 200, List.of("ann"))).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCancelOnlyAsTheOrganizerAndFreeEverybodysSlot() {
        CalendarApi c = cal();
        String id = c.schedule("ann", "sync", 100, 200, List.of("bob"));

        assertThatThrownBy(() -> c.cancel("bob", id)).isInstanceOf(SecurityException.class);
        assertThatThrownBy(() -> c.cancel("ann", "no-such-id")).isInstanceOf(IllegalArgumentException.class);
        c.cancel("ann", id);

        assertThat(c.agenda("bob", 0, 1_000)).isEmpty();
        c.schedule("bob", "other", 100, 200, List.of("ann"));
    }

    @Test
    void shouldLetAnAttendeeDeclineWithoutAffectingOthers() {
        CalendarApi c = cal();
        String id = c.schedule("ann", "sync", 100, 200, List.of("bob", "cy"));

        c.decline("bob", id);

        assertThat(c.agenda("bob", 0, 1_000)).isEmpty();
        assertThat(c.agenda("cy", 0, 1_000)).containsExactly("100-200:sync");
        assertThatThrownBy(() -> c.decline("ann", id)).isInstanceOf(SecurityException.class);
        c.schedule("bob", "free now", 100, 200, List.of());
    }

    @Test
    void shouldScheduleRecurringEventsAllOrNothing() {
        CalendarApi c = cal();
        c.schedule("bob", "blocker", 10_000 + 2 * 1_440, 10_060 + 2 * 1_440, List.of()); // hits the 3rd occurrence

        assertThatThrownBy(() -> c.scheduleRecurring("ann", "daily", 10_000, 10_030, List.of("bob"), 1_440, 5))
                .isInstanceOf(IllegalStateException.class);
        assertThat(c.agenda("ann", 0, 100_000)).isEmpty();

        List<String> ids = c.scheduleRecurring("ann", "daily", 10_000, 10_030, List.of("cy"), 1_440, 5);
        assertThat(ids).hasSize(5).doesNotHaveDuplicates();
        assertThat(c.agenda("cy", 0, 100_000)).hasSize(5);
        assertThat(c.agenda("ann", 0, 100_000).get(1)).isEqualTo((10_000 + 1_440) + "-" + (10_030 + 1_440) + ":daily");
    }

    @Test
    void shouldFindCommonFreeSlots() {
        CalendarApi c = cal();
        c.schedule("ann", "a", 60, 120, List.of());
        c.schedule("bob", "b", 100, 180, List.of());
        c.schedule("bob", "c", 240, 260, List.of());

        assertThat(c.commonFreeSlots(List.of("ann", "bob"), 0, 300, 30)).containsExactly("0-60", "180-240", "260-300");
        assertThat(c.commonFreeSlots(List.of("ann", "bob"), 0, 300, 60)).containsExactly("0-60", "180-240");
        assertThat(c.commonFreeSlots(List.of("ann"), 0, 300, 100)).containsExactly("120-300");
        assertThat(c.commonFreeSlots(List.of("ann", "bob"), 0, 300, 200)).isEmpty();
    }

    @Test
    void shouldLetExactlyOneOfTwoCrossingMeetingsWin() {
        for (int round = 0; round < 50; round++) {
            CalendarApi c = cal();

            List<Boolean> results = Concurrently.collect(2, t -> {
                try {
                    c.schedule(t == 0 ? "ann" : "bob", "m" + t, 100, 200, List.of(t == 0 ? "bob" : "ann"));
                    return true;
                } catch (IllegalStateException e) {
                    return false;
                }
            });

            assertThat(results.stream().filter(r -> r).count()).as("round %d", round).isEqualTo(1);
        }
    }

    @Test
    void shouldNeverDoubleBookAnyoneUnderConcurrentScheduling() {
        CalendarApi c = cal();
        String[] people = {"p0", "p1", "p2", "p3", "p4", "p5"};
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(12, t -> {
            java.util.Random r = new java.util.Random(t);
            for (int i = 0; i < 200; i++) {
                int a = r.nextInt(people.length);
                int b = (a + 1 + r.nextInt(people.length - 1)) % people.length;
                long start = r.nextInt(20) * 30L;
                try {
                    c.schedule(people[a], "m", start, start + 30, List.of(people[b]));
                    accepted.incrementAndGet();
                } catch (IllegalStateException e) {
                    // conflict: fine
                }
            }
        });

        int entries = 0;
        for (String p : people) {
            List<String> agenda = c.agenda(p, 0, 10_000);
            entries += agenda.size();
            for (int i = 1; i < agenda.size(); i++) {
                long prevEnd = Long.parseLong(agenda.get(i - 1).split(":")[0].split("-")[1]);
                long start = Long.parseLong(agenda.get(i).split(":")[0].split("-")[0]);
                assertThat(start).as("%s double booked", p).isGreaterThanOrEqualTo(prevEnd);
            }
        }
        assertThat(entries).as("each accepted meeting has exactly two participants").isEqualTo(accepted.get() * 2);
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            CalendarApi c = cal();
            AtomicInteger wins = new AtomicInteger();

            Concurrently.run(6, t -> {
                try {
                    c.schedule("p" + t, "m", 0, 60, List.of("p" + ((t + 1) % 6)));
                    wins.incrementAndGet();
                } catch (IllegalStateException e) {
                    // lost the race
                }
            });

            assertThat(wins.get()).as("run %d: at most 3 disjoint pairs fit in a ring of 6", run).isBetween(1, 3);
            int total = 0;
            for (int t = 0; t < 6; t++) {
                total += c.agenda("p" + t, 0, 100).size();
            }
            assertThat(total).isEqualTo(wins.get() * 2);
        }
    }
}

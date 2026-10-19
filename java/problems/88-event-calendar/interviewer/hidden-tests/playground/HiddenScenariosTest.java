package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CalendarApi;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandleAYearOfDailyMeetings() {
        CalendarApi c = Candidate.of(Solution::create);
        List<String> ids = c.scheduleRecurring("ann", "daily", 0, 30, List.of("bob"), 1_440, 365);

        assertThat(ids).hasSize(365);
        assertThat(c.agenda("bob", 0, 365 * 1_440L)).hasSize(365);
    }

    @Test
    void shouldNotDeadlockWithLargeCrossingGroups() {
        CalendarApi c = Candidate.of(Solution::create);
        List<String> everyone = new ArrayList<>();
        for (int i = 0; i < 30; i++) {
            everyone.add("u" + i);
        }

        Concurrently.run(30, 60_000, t -> {
            List<String> others = new ArrayList<>(everyone);
            others.remove("u" + t);
            for (int i = 0; i < 100; i++) {
                try {
                    c.schedule("u" + t, "all", i * 60L, i * 60L + 30, others);
                } catch (IllegalStateException e) {
                    // someone else got that slot
                }
            }
        });

        assertThat(c.agenda("u0", 0, 10_000).size()).isEqualTo(100);
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ChatApi;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHandleALargeGroupWithManyMessages() {
        ChatApi c = Candidate.of(Solution::create);
        List<String> members = new ArrayList<>();
        for (int i = 0; i < 200; i++) {
            c.register("u" + i);
            members.add("u" + i);
            c.connect("u" + i);
        }
        c.register("admin");
        c.createGroup("big", "admin", members);

        Concurrently.run(10, 60_000, t -> {
            for (int i = 0; i < 100; i++) {
                c.sendGroup("u" + t, "big", "m" + t + "-" + i);
            }
        });

        assertThat(c.history("u199", "group:big")).hasSize(1_000);
        assertThat(c.unreadCount("u199", "group:big")).isEqualTo(1_000);
    }

    @Test
    void shouldKeepUnreadCountsConsistentWhileReading() {
        ChatApi c = Candidate.of(Solution::create);
        c.register("ann");
        c.register("bob");

        Concurrently.run(3, 60_000, t -> {
            for (int i = 0; i < 500; i++) {
                if (t == 2) {
                    c.read("bob", "dm:ann:bob");
                } else {
                    c.sendDirect("ann", "bob", "x");
                }
            }
        });
        c.read("bob", "dm:ann:bob");

        assertThat(c.unreadCount("bob", "dm:ann:bob")).isZero();
        assertThat(c.history("bob", "dm:ann:bob")).hasSize(1_000);
    }
}

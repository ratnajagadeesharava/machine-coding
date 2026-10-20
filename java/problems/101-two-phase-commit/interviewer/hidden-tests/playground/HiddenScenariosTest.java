package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.CoordinatorNode;
import contract.DurableLog;
import contract.Outcome;
import contract.ParticipantNode;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class HiddenScenariosTest {

    private static final class MemoryLog implements DurableLog {
        private final List<String> records = new ArrayList<>();

        @Override
        public void append(String record) {
            records.add(record);
        }

        @Override
        public List<String> records() {
            return new ArrayList<>(records);
        }
    }

    private static final List<String> PARTICIPANTS = List.of("A", "B", "C");

    private static final class Env {
        final SimNet net;
        final CoordinatorNode coordinator;
        final Map<String, ParticipantNode> participants = new HashMap<>();

        Env(long seed) {
            net = new SimNet(seed);
            MemoryLog klog = new MemoryLog();
            coordinator = Candidate.of(() -> Solution.coordinator("K", PARTICIPANTS, net.sender("K"), klog));
            net.register("K", coordinator::onMessage, coordinator::onTick);
            for (String p : PARTICIPANTS) {
                ParticipantNode node = Candidate.of(() -> Solution.participant(p, "K", net.sender(p), new MemoryLog()));
                participants.put(p, node);
                net.register(p, node::onMessage, node::onTick);
            }
        }
    }

    @Test
    void shouldCommitTwoHundredIndependentTransactionsRunningAtTheSameTime() {
        Env e = new Env(1);
        for (int i = 0; i < 200; i++) {
            e.coordinator.begin("t" + i, Map.of("A", Map.of("a" + i, "v"), "B", Map.of("b" + i, "v"), "C", Map.of("c" + i, "v")));
        }

        e.net.run(3_000);

        for (int i = 0; i < 200; i++) {
            assertThat(e.coordinator.outcome("t" + i)).isEqualTo(Outcome.COMMITTED);
            assertThat(e.participants.get("C").get("c" + i)).hasValue("v");
        }
    }

    @Test
    void shouldLetAtMostOneOfManyConflictingTransactionsCommitAtATime() {
        Env e = new Env(2);
        e.net.setLatency(5, 5);
        for (int i = 0; i < 50; i++) {
            e.coordinator.begin("t" + i, Map.of("A", Map.of("hot", "from-" + i), "B", Map.of("hot", "from-" + i)));
            e.net.run(7);
        }
        e.net.run(3_000);

        int committed = 0;
        String winner = null;
        for (int i = 0; i < 50; i++) {
            if (e.coordinator.outcome("t" + i) == Outcome.COMMITTED) {
                committed++;
                winner = "from-" + i;
            }
        }
        assertThat(committed).isGreaterThanOrEqualTo(1);
        assertThat(e.participants.get("A").get("hot")).isEqualTo(e.participants.get("B").get("hot"));
        assertThat(e.participants.get("A").get("hot")).hasValue(winner); // the last committed one wins on both
    }
}

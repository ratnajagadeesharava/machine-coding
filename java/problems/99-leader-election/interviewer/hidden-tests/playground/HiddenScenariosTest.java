package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ElectionNode;
import contract.Role;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class HiddenScenariosTest {

    @Test
    void shouldElectWithinAFewTimeoutsInAnElevenNodeClusterWithJitteryLatency() {
        SimNet net = new SimNet(5);
        net.setLatency(1, 40);
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < 11; i++) {
            ids.add("n" + i);
        }
        List<ElectionNode> nodes = new ArrayList<>();
        for (String id : ids) {
            List<String> peers = new ArrayList<>(ids);
            peers.remove(id);
            ElectionNode n = Candidate.of(() -> Solution.create(id, peers, net.sender(id), id.hashCode()));
            nodes.add(n);
            net.register(id, n::onMessage, n::onTick);
        }

        boolean elected = net.runUntil(() -> nodes.stream().filter(n -> n.role() == Role.LEADER).count() == 1, 5_000);

        assertThat(elected).isTrue();
    }

    @Test
    void shouldSurviveTheLeaderCrashingTenTimesInARow() {
        SimNet net = new SimNet(9);
        List<String> ids = List.of("a", "b", "c", "d", "e");
        java.util.Map<String, ElectionNode> nodes = new java.util.HashMap<>();
        for (String id : ids) {
            List<String> peers = new ArrayList<>(ids);
            peers.remove(id);
            ElectionNode n = Candidate.of(() -> Solution.create(id, peers, net.sender(id), id.hashCode() * 7L));
            nodes.put(id, n);
            net.register(id, n::onMessage, n::onTick);
        }
        long lastTerm = 0;

        for (int round = 0; round < 10; round++) {
            net.runUntil(() -> nodes.entrySet().stream().filter(e -> !net.isCrashed(e.getKey()) && e.getValue().role() == Role.LEADER).count() == 1, 2_000);
            String leader = nodes.entrySet().stream().filter(e -> !net.isCrashed(e.getKey()) && e.getValue().role() == Role.LEADER).findFirst().orElseThrow().getKey();
            assertThat(nodes.get(leader).term()).isGreaterThan(lastTerm - 1);
            lastTerm = nodes.get(leader).term();
            net.crash(leader);
            net.run(800); // long enough for the others to elect a new leader
            net.recover(leader); // it comes back as a follower of the next leader
            net.run(400);
        }

        assertThat(lastTerm).isGreaterThanOrEqualTo(10);
    }
}

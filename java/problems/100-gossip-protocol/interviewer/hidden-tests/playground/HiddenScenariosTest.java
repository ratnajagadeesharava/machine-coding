package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.GossipNode;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class HiddenScenariosTest {

    private List<GossipNode> build(SimNet net, int n, long seed) {
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < n; i++) {
            ids.add("n" + i);
        }
        List<GossipNode> nodes = new ArrayList<>();
        for (String id : ids) {
            List<String> peers = new ArrayList<>(ids);
            peers.remove(id);
            GossipNode node = Candidate.of(() -> Solution.create(id, peers, net.sender(id), seed + id.hashCode()));
            nodes.add(node);
            net.register(id, node::onMessage, node::onTick);
        }
        return nodes;
    }

    @Test
    void shouldSpreadAWriteToHundredNodesInOrderLogN() {
        SimNet net = new SimNet(3);
        List<GossipNode> nodes = build(net, 100, 7);
        nodes.get(0).put("k", "v");

        boolean spread = net.runUntil(() -> nodes.stream().allMatch(n -> n.get("k").equals(Optional.of("v"))), 2_500);

        assertThat(spread).isTrue();
    }

    @Test
    void shouldDetectTenSimultaneousCrashesInAHundredNodeCluster() {
        SimNet net = new SimNet(4);
        List<GossipNode> nodes = build(net, 100, 9);
        net.run(1_000);
        for (int i = 0; i < 10; i++) {
            net.crash("n" + i);
        }

        net.run(6_000);

        for (int i = 10; i < 100; i++) {
            assertThat(nodes.get(i).aliveMembers()).hasSize(90);
        }
    }
}

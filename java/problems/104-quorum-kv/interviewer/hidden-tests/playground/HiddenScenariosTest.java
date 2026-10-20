package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.KvNode;
import contract.Reply;
import contract.Reply.Status;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class HiddenScenariosTest {

    private static final List<String> IDS = List.of("n1", "n2", "n3", "n4", "n5");

    private static Map<String, KvNode> cluster(SimNet net) {
        Map<String, KvNode> nodes = new HashMap<>();
        for (String id : IDS) {
            KvNode n = Candidate.of(() -> Solution.node(id, IDS, net.sender(id)));
            nodes.put(id, n);
            net.register(id, n::onMessage, n::onTick);
        }
        return nodes;
    }

    private static Reply await(SimNet net, KvNode via, String rid) {
        net.runUntil(() -> via.reply(rid).status() != Status.PENDING, 3_000);
        return via.reply(rid);
    }

    @Test
    void shouldUseAMajorityOfFiveNotOfThree() {
        SimNet net = new SimNet(1);
        Map<String, KvNode> nodes = cluster(net);
        net.crash("n3");
        net.crash("n4");
        nodes.get("n1").put("a", "k", "v");
        assertThat(await(net, nodes.get("n1"), "a").status()).isEqualTo(Status.OK);

        net.crash("n5");
        nodes.get("n1").put("b", "k", "w");
        assertThat(await(net, nodes.get("n1"), "b").status()).isEqualTo(Status.FAILED);
    }

    @Test
    void shouldStayConsistentWithRotatingCrashesAndPartitions() {
        for (long seed = 0; seed < 60; seed++) {
            SimNet net = new SimNet(seed);
            Map<String, KvNode> nodes = cluster(net);
            Random rnd = new Random(seed);
            net.setDropRate(0.1);
            net.setDuplicateRate(0.1);
            String lastOk = null;
            for (int op = 0; op < 40; op++) {
                net.heal();
                for (String id : IDS) net.recover(id);
                net.run(2); // a recovered node needs a tick to learn the time
                int mode = rnd.nextInt(3);
                if (mode == 1) {
                    net.crash(IDS.get(rnd.nextInt(5)));
                    net.crash(IDS.get(rnd.nextInt(5)));
                } else if (mode == 2) {
                    net.partition(List.of("n1", "n2", "n3"), List.of("n4", "n5"));
                }
                String via = IDS.get(rnd.nextInt(5));
                if (net.isCrashed(via)) continue;
                String rid = "r" + op;
                KvNode node = nodes.get(via);
                if (rnd.nextBoolean()) {
                    node.put(rid, "k", "v" + op);
                    if (await(net, node, rid).status() == Status.OK) lastOk = "v" + op;
                    else lastOk = null; // a failed put may or may not surface later: stop asserting until the next OK put
                } else {
                    node.get(rid, "k");
                    Reply r = await(net, node, rid);
                    if (r.status() == Status.OK && lastOk != null) assertThat(r.value()).as("seed %d op %d", seed, op).isEqualTo(lastOk);
                }
                net.run(rnd.nextInt(30));
            }
        }
    }

    @Test
    void shouldConvergeEveryReplicaAfterTheNetworkHeals() {
        SimNet net = new SimNet(5);
        Map<String, KvNode> nodes = cluster(net);
        net.partition(List.of("n1", "n2", "n3"), List.of("n4", "n5"));
        nodes.get("n1").put("p", "k", "during-partition");
        assertThat(await(net, nodes.get("n1"), "p").status()).isEqualTo(Status.OK);
        net.heal();

        nodes.get("n4").get("g", "k");
        assertThat(await(net, nodes.get("n4"), "g").value()).isEqualTo("during-partition");
        net.run(500);

        for (String id : IDS) assertThat(nodes.get(id).local("k")).as(id).hasValue("during-partition");
    }
}

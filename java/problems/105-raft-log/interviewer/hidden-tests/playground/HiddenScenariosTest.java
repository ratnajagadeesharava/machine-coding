package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.RaftNode;
import contract.Role;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class HiddenScenariosTest {

    private static Map<String, RaftNode> cluster(SimNet net, List<String> ids, long seed) {
        Map<String, RaftNode> nodes = new HashMap<>();
        for (String id : ids) {
            List<String> peers = new ArrayList<>(ids);
            peers.remove(id);
            RaftNode n = Candidate.of(() -> Solution.create(id, peers, net.sender(id), seed * 7 + id.hashCode()));
            nodes.put(id, n);
            net.register(id, n::onMessage, n::onTick);
        }
        return nodes;
    }

    private static String leader(SimNet net, Map<String, RaftNode> nodes) {
        String found = null;
        for (var e : nodes.entrySet()) {
            if (!net.isCrashed(e.getKey()) && e.getValue().role() == Role.LEADER) {
                if (found != null) return null;
                found = e.getKey();
            }
        }
        return found;
    }

    @Test
    void shouldReplicateAHundredCommandsQuicklyThroughAFiveNodeCluster() {
        SimNet net = new SimNet(1);
        List<String> ids = List.of("a", "b", "c", "d", "e");
        Map<String, RaftNode> nodes = cluster(net, ids, 1);
        net.runUntil(() -> leader(net, nodes) != null, 2_000);
        RaftNode l = nodes.get(leader(net, nodes));
        List<String> expected = new ArrayList<>();

        for (int i = 0; i < 100; i++) {
            assertThat(l.propose("cmd" + i)).isPresent();
            expected.add("cmd" + i);
        }

        assertThat(net.runUntil(() -> ids.stream().allMatch(i -> nodes.get(i).committed().equals(expected)), 1_000)).isTrue();
    }

    @Test
    void shouldSurviveARollingRestartOfEveryNodeWithoutLosingCommittedCommands() {
        SimNet net = new SimNet(2);
        List<String> ids = List.of("a", "b", "c");
        Map<String, RaftNode> nodes = cluster(net, ids, 2);
        List<String> expected = new ArrayList<>();
        for (int round = 0; round < 6; round++) {
            net.runUntil(() -> leader(net, nodes) != null, 3_000);
            RaftNode l = nodes.get(leader(net, nodes));
            String cmd = "r" + round;
            if (l.propose(cmd).isPresent()) {
                int target = expected.size() + 1;
                if (net.runUntil(() -> l.committed().size() >= target, 1_000)) expected.add(cmd);
            }
            String victim = ids.get(round % 3);
            net.crash(victim);
            net.run(600);
            net.recover(victim);
            net.run(400);
        }

        net.run(2_000);

        for (String id : ids) assertThat(nodes.get(id).committed()).as(id).startsWith(expected.toArray(new String[0]));
    }

    @Test
    void shouldKeepSafetyWithFiveNodesAndHeavyFaults() {
        for (long seed = 0; seed < 60; seed++) {
            SimNet net = new SimNet(seed + 1_000);
            List<String> ids = List.of("a", "b", "c", "d", "e");
            Map<String, RaftNode> nodes = cluster(net, ids, seed);
            Random rnd = new Random(seed);
            Map<Long, String> leaderOfTerm = new HashMap<>();
            Map<String, List<String>> before = new HashMap<>();
            ids.forEach(i -> before.put(i, List.of()));
            List<String> longest = new ArrayList<>();
            List<String> violations = new ArrayList<>();
            net.observe(() -> {
                for (String id : ids) {
                    RaftNode n = nodes.get(id);
                    if (n.role() == Role.LEADER) {
                        String prev = leaderOfTerm.putIfAbsent(n.term(), id);
                        if (prev != null && !prev.equals(id)) violations.add("two leaders in term " + n.term());
                    }
                    List<String> now = n.committed();
                    List<String> old = before.get(id);
                    if (now.size() < old.size() || !now.subList(0, old.size()).equals(old)) violations.add(id + " rewrote its committed entries");
                    before.put(id, now);
                    int common = Math.min(now.size(), longest.size());
                    if (!now.subList(0, common).equals(longest.subList(0, common))) violations.add(id + " diverged");
                    if (now.size() > longest.size()) {
                        longest.clear();
                        longest.addAll(now);
                    }
                }
            });
            net.setDropRate(0.1);
            net.setDuplicateRate(0.2);
            net.setLatency(1, 60);
            for (int round = 0; round < 40; round++) {
                switch (rnd.nextInt(6)) {
                    case 0 -> net.crash(ids.get(rnd.nextInt(5)));
                    case 1 -> ids.forEach(net::recover);
                    case 2 -> net.partition(List.of("a", "b"), List.of("c", "d", "e"));
                    case 3 -> net.partition(List.of("a", "c", "e"), List.of("b", "d"));
                    case 4 -> net.heal();
                    default -> { }
                }
                for (String id : ids) if (!net.isCrashed(id)) nodes.get(id).propose("s" + seed + "-" + round + id);
                net.run(30 + rnd.nextInt(300));
            }
            assertThat(violations).as("seed %d", seed).isEmpty();
        }
    }
}

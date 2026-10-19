package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.KvNode;
import contract.Reply;
import contract.Reply.Status;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.SimNet;

class BehaviorTest {

    /** Three replicas n1, n2, n3. */
    private static final class Cluster {
        final SimNet net;
        final List<String> ids = List.of("n1", "n2", "n3");
        final Map<String, KvNode> nodes = new HashMap<>();
        int requests;

        Cluster(long seed) {
            net = new SimNet(seed);
            for (String id : ids) {
                KvNode node = Candidate.of(() -> Solution.node(id, ids, net.sender(id)));
                nodes.put(id, node);
                net.register(id, node::onMessage, node::onTick);
            }
        }

        KvNode node(String id) {
            return nodes.get(id);
        }

        String start(String via, boolean put, String key, String value) {
            String rid = "r" + (++requests);
            if (put) nodes.get(via).put(rid, key, value);
            else nodes.get(via).get(rid, key);
            return rid;
        }

        Reply finish(String via, String rid) {
            net.runUntil(() -> nodes.get(via).reply(rid).status() != Status.PENDING, 3_000);
            return nodes.get(via).reply(rid);
        }

        Reply put(String via, String key, String value) {
            return finish(via, start(via, true, key, value));
        }

        Reply get(String via, String key) {
            return finish(via, start(via, false, key, null));
        }
    }

    @Test
    void shouldReadWhatWasWrittenFromAnyNode() {
        Cluster c = new Cluster(1);

        assertThat(c.put("n1", "k", "v1").status()).isEqualTo(Status.OK);

        for (String via : c.ids) {
            Reply r = c.get(via, "k");
            assertThat(r.status()).as(via).isEqualTo(Status.OK);
            assertThat(r.value()).as(via).isEqualTo("v1");
        }
    }

    @Test
    void shouldReportNotFoundForAnUnknownKey() {
        Cluster c = new Cluster(1);

        assertThat(c.get("n2", "missing").status()).isEqualTo(Status.NOT_FOUND);
    }

    @Test
    void shouldStayPendingUntilAMajorityHasAnswered() {
        Cluster c = new Cluster(1);
        c.net.crash("n2");
        c.net.crash("n3");

        String rid = c.start("n1", true, "k", "v");
        c.net.run(500);

        assertThat(c.node("n1").reply(rid).status()).isEqualTo(Status.PENDING);
    }

    @Test
    void shouldFailWithinASecondWhenNoMajorityIsReachable() {
        Cluster c = new Cluster(1);
        c.net.crash("n2");
        c.net.crash("n3");

        String put = c.start("n1", true, "k", "v");
        String get = c.start("n1", false, "k", null);
        c.net.run(1_100);

        assertThat(c.node("n1").reply(put).status()).isEqualTo(Status.FAILED);
        assertThat(c.node("n1").reply(get).status()).isEqualTo(Status.FAILED);
    }

    @Test
    void shouldKeepWorkingWithOneNodeDown() {
        Cluster c = new Cluster(2);
        c.net.crash("n3");

        assertThat(c.put("n1", "k", "v1").status()).isEqualTo(Status.OK);
        Reply r = c.get("n2", "k");

        assertThat(r.status()).isEqualTo(Status.OK);
        assertThat(r.value()).isEqualTo("v1");
    }

    @Test
    void shouldReadTheLatestWriteEvenWhenADifferentNodeIsDownThanDuringTheWrite() {
        Cluster c = new Cluster(3);
        c.net.crash("n3");
        c.put("n1", "k", "old");
        c.net.run(10);
        c.put("n1", "k", "new");
        c.net.recover("n3");
        c.net.crash("n1");

        Reply r = c.get("n3", "k");

        assertThat(r.status()).isEqualTo(Status.OK);
        assertThat(r.value()).isEqualTo("new");
    }

    @Test
    void shouldRepairAStaleReplicaOnRead() {
        Cluster c = new Cluster(4);
        c.net.crash("n3");
        c.put("n1", "k", "v1");
        c.net.recover("n3");
        assertThat(c.node("n3").local("k")).isEmpty();

        assertThat(c.get("n1", "k").value()).isEqualTo("v1");
        c.net.run(300);

        assertThat(c.node("n3").local("k")).hasValue("v1");
    }

    @Test
    void shouldRepairAReplicaWhoseAnswerArrivesAfterTheReplyWasDecided() {
        Cluster c = new Cluster(5);
        c.net.crash("n3");
        c.put("n1", "k", "v1");
        c.net.recover("n3");
        c.net.setLatency(1, 1);

        c.get("n1", "k");
        c.net.run(500);

        assertThat(c.node("n3").local("k")).hasValue("v1");
    }

    @Test
    void shouldKeepResendingAWriteSoEveryReplicaGetsItDespiteLoss() {
        Cluster c = new Cluster(6);
        c.net.setDropRate(0.4);

        Reply r = c.put("n1", "k", "v1");
        c.net.run(900);

        assertThat(r.status()).isEqualTo(Status.OK);
        assertThat(c.node("n2").local("k")).hasValue("v1");
        assertThat(c.node("n3").local("k")).hasValue("v1");
    }

    @Test
    void shouldNotWriteOnTheMinorityPartitionButOnTheMajority() {
        Cluster c = new Cluster(7);
        c.net.partition(List.of("n1"), List.of("n2", "n3"));

        assertThat(c.put("n1", "k", "lonely").status()).isEqualTo(Status.FAILED);
        assertThat(c.put("n2", "k", "majority").status()).isEqualTo(Status.OK);
        c.net.heal();
        c.net.run(200);

        assertThat(c.get("n1", "k").value()).isEqualTo("majority");
    }

    @Test
    void shouldLetTheNewerWriteWin() {
        Cluster c = new Cluster(8);
        c.put("n1", "k", "first");
        c.net.run(5);
        c.put("n2", "k", "second");
        c.net.run(5);
        c.put("n3", "k", "third");

        for (String via : c.ids) {
            assertThat(c.get(via, "k").value()).as(via).isEqualTo("third");
        }
    }

    @Test
    void shouldConvergeOnTheLargerNodeIdForWritesInTheSameMillisecond() {
        Cluster c = new Cluster(9);

        String a = c.start("n1", true, "k", "from-n1");
        String b = c.start("n3", true, "k", "from-n3");
        c.finish("n1", a);
        c.finish("n3", b);
        c.net.run(1_200);

        for (String id : c.ids) {
            assertThat(c.node(id).local("k")).as(id).hasValue("from-n3");
        }
    }

    @Test
    void shouldIgnoreARequestIdThatWasAlreadyUsed() {
        Cluster c = new Cluster(10);
        String rid = c.start("n1", true, "k", "v1");
        c.finish("n1", rid);

        c.node("n1").put(rid, "k", "other");
        c.net.run(300);

        assertThat(c.node("n1").reply(rid).status()).isEqualTo(Status.OK);
        assertThat(c.get("n2", "k").value()).isEqualTo("v1");
    }

    @Test
    void shouldReportPendingForAnUnknownRequestId() {
        Cluster c = new Cluster(11);

        assertThat(c.node("n1").reply("nope").status()).isEqualTo(Status.PENDING);
    }

    @Test
    void shouldSurviveDuplicatedMessages() {
        Cluster c = new Cluster(12);
        c.net.setDuplicateRate(0.5);

        c.put("n1", "k", "v1");
        c.put("n2", "k", "v2");
        Reply r = c.get("n3", "k");

        assertThat(r.value()).isEqualTo("v2");
    }

    @Test
    void shouldAlwaysReadTheLastAcknowledgedWriteUnderRandomFaults() {
        for (long seed = 0; seed < 100; seed++) {
            Cluster c = new Cluster(seed);
            Random rnd = new Random(seed * 31 + 7);
            c.net.setDropRate(0.1);
            Map<String, String> lastOk = new HashMap<>();
            Map<String, List<String>> maybe = new HashMap<>(); // failed puts since the last OK: may or may not be visible
            String down = null;
            for (int op = 0; op < 40; op++) {
                if (rnd.nextInt(4) == 0) {
                    if (down != null) c.net.recover(down);
                    down = rnd.nextBoolean() ? null : c.ids.get(rnd.nextInt(3));
                    if (down != null) c.net.crash(down);
                }
                List<String> up = new ArrayList<>(c.ids);
                if (down != null) up.remove(down);
                String via = up.get(rnd.nextInt(up.size()));
                String key = "k" + rnd.nextInt(3);
                c.net.run(1 + rnd.nextInt(20));
                if (rnd.nextBoolean()) {
                    String value = "v" + op;
                    Reply r = c.put(via, key, value);
                    assertThat(r.status()).as("seed %d op %d put", seed, op).isEqualTo(Status.OK);
                    lastOk.put(key, value);
                    maybe.remove(key);
                } else {
                    Reply r = c.get(via, key);
                    assertThat(r.status()).as("seed %d op %d get", seed, op).isIn(Status.OK, Status.NOT_FOUND);
                    if (lastOk.containsKey(key)) {
                        assertThat(r.value()).as("seed %d op %d get %s", seed, op, key).isEqualTo(lastOk.get(key));
                    } else {
                        assertThat(r.status()).as("seed %d op %d get %s", seed, op, key).isEqualTo(Status.NOT_FOUND);
                    }
                }
            }
        }
    }
}

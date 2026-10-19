package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

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

class BehaviorTest {

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

    /** A coordinator "K" and participants "A", "B", "C" in the simulation. Restarting a node builds a NEW instance on the SAME log. */
    private static final class Cluster {
        final SimNet net;
        final List<String> participantIds = List.of("A", "B", "C");
        final Map<String, MemoryLog> logs = new HashMap<>();
        final Map<String, ParticipantNode> participants = new HashMap<>();
        CoordinatorNode coordinator;

        Cluster(long seed) {
            net = new SimNet(seed);
            logs.put("K", new MemoryLog());
            startCoordinator();
            for (String p : participantIds) {
                logs.put(p, new MemoryLog());
                startParticipant(p);
            }
        }

        void startCoordinator() {
            coordinator = Candidate.of(() -> Solution.coordinator("K", participantIds, net.sender("K"), logs.get("K")));
            net.register("K", coordinator::onMessage, coordinator::onTick);
        }

        void startParticipant(String p) {
            ParticipantNode node = Candidate.of(() -> Solution.participant(p, "K", net.sender(p), logs.get(p)));
            participants.put(p, node);
            net.register(p, node::onMessage, node::onTick);
        }

        void restartCoordinator() {
            startCoordinator();
        }

        void restartParticipant(String p) {
            startParticipant(p);
        }

        Outcome await(String tx, long maxMs) {
            net.runUntil(() -> coordinator.outcome(tx) != Outcome.PENDING, maxMs);
            return coordinator.outcome(tx);
        }

        boolean quiet() {
            return participants.values().stream().allMatch(p -> p.inDoubt().isEmpty());
        }
    }

    private static Map<String, Map<String, String>> writes(Object... participantsAndPairs) {
        Map<String, Map<String, String>> m = new HashMap<>();
        for (int i = 0; i < participantsAndPairs.length; i += 3) {
            m.computeIfAbsent((String) participantsAndPairs[i], k -> new HashMap<>()).put((String) participantsAndPairs[i + 1], (String) participantsAndPairs[i + 2]);
        }
        return m;
    }

    @Test
    void shouldCommitAcrossParticipantsWhenEveryoneVotesYes() {
        Cluster c = new Cluster(1);

        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));

        assertThat(c.await("t1", 1_000)).isEqualTo(Outcome.COMMITTED);
        c.net.run(300);
        assertThat(c.participants.get("A").get("x")).hasValue("1");
        assertThat(c.participants.get("B").get("y")).hasValue("2");
        assertThat(c.participants.get("C").get("x")).isEmpty();
        assertThat(c.participants.get("A").isLocked("x")).isFalse();
        assertThat(c.quiet()).isTrue();
    }

    @Test
    void shouldAbortEverywhereWhenOneParticipantVotesNo() {
        Cluster c = new Cluster(2);

        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "REJECT", "C", "z", "3"));

        assertThat(c.await("t1", 1_000)).isEqualTo(Outcome.ABORTED);
        c.net.run(500);
        assertThat(c.participants.get("A").get("x")).isEmpty();
        assertThat(c.participants.get("C").get("z")).isEmpty();
        assertThat(c.participants.get("A").isLocked("x")).as("the YES voters release their locks").isFalse();
        assertThat(c.participants.get("C").isLocked("z")).isFalse();
        assertThat(c.quiet()).isTrue();
    }

    @Test
    void shouldHoldLocksBetweenVoteAndDecisionAndRefuseAConflictingTransaction() {
        Cluster c = new Cluster(3);
        c.net.setLatency(40, 40); // long enough to observe the window
        c.coordinator.begin("t1", writes("A", "k", "from-t1"));
        c.net.run(60); // PREPARE has arrived, the YES is on its way

        assertThat(c.participants.get("A").isLocked("k")).isTrue();
        assertThat(c.participants.get("A").inDoubt()).containsExactly("t1");
        c.coordinator.begin("t2", writes("A", "k", "from-t2"));

        assertThat(c.await("t1", 1_000)).isEqualTo(Outcome.COMMITTED);
        assertThat(c.await("t2", 1_000)).isEqualTo(Outcome.ABORTED);
        c.net.run(500);
        assertThat(c.participants.get("A").get("k")).hasValue("from-t1");
    }

    @Test
    void shouldAllowASecondTransactionOnTheSameKeyOnceTheFirstIsDone() {
        Cluster c = new Cluster(4);
        c.coordinator.begin("t1", writes("A", "k", "one"));
        c.await("t1", 1_000);
        c.net.run(300);

        c.coordinator.begin("t2", writes("A", "k", "two"));

        assertThat(c.await("t2", 1_000)).isEqualTo(Outcome.COMMITTED);
        c.net.run(300);
        assertThat(c.participants.get("A").get("k")).hasValue("two");
    }

    @Test
    void shouldAbortAfterHalfASecondWhenAParticipantNeverAnswers() {
        Cluster c = new Cluster(5);
        c.net.crash("B");

        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));

        assertThat(c.await("t1", 400)).as("not before the 500 ms vote timeout").isEqualTo(Outcome.PENDING);
        assertThat(c.await("t1", 400)).isEqualTo(Outcome.ABORTED);
        c.net.run(500);
        assertThat(c.participants.get("A").isLocked("x")).as("A voted YES and must be told to release").isFalse();
    }

    @Test
    void shouldTellTheLateParticipantTheOutcomeWhenItComesBack() {
        Cluster c = new Cluster(6);
        c.net.crash("B");
        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));
        c.await("t1", 1_000);
        c.net.run(1_000);

        c.net.recover("B");
        c.net.run(1_000);

        assertThat(c.participants.get("B").get("y")).as("aborted: nothing applied").isEmpty();
        assertThat(c.participants.get("B").isLocked("y")).isFalse();
        assertThat(c.participants.get("B").inDoubt()).isEmpty();
    }

    @Test
    void shouldKeepAYesVotersTransactionInDoubtUntilItLearnsTheCommit() {
        Cluster c = new Cluster(7);
        c.net.setLatency(10, 10);
        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));
        c.net.runUntil(() -> c.participants.get("B").inDoubt().contains("t1"), 500); // B has voted YES
        c.net.crash("B");

        assertThat(c.await("t1", 1_000)).isEqualTo(Outcome.COMMITTED);
        c.net.run(1_000);
        assertThat(c.participants.get("A").get("x")).hasValue("1");
        assertThat(c.participants.get("B").inDoubt()).as("B is still in doubt and may not decide alone").containsExactly("t1");
        assertThat(c.participants.get("B").isLocked("y")).isTrue();

        c.net.recover("B");
        c.net.run(1_000);

        assertThat(c.participants.get("B").get("y")).hasValue("2");
        assertThat(c.participants.get("B").inDoubt()).isEmpty();
    }

    @Test
    void shouldRebuildAParticipantsLocksAndDoubtFromItsLogAfterARestartAndFinishTheCommit() {
        Cluster c = new Cluster(8);
        c.net.setLatency(10, 10);
        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));
        c.net.runUntil(() -> c.participants.get("B").inDoubt().contains("t1"), 500);
        c.net.crash("B");
        c.await("t1", 1_000);
        c.net.run(500);

        c.restartParticipant("B"); // memory lost, log kept
        assertThat(c.participants.get("B").inDoubt()).containsExactly("t1");
        assertThat(c.participants.get("B").isLocked("y")).isTrue();
        c.net.run(1_000);

        assertThat(c.participants.get("B").get("y")).hasValue("2");
        assertThat(c.quiet()).isTrue();
    }

    @Test
    void shouldBlockParticipantsWhileTheCoordinatorIsGoneAndFinishWhenItReturns() {
        Cluster c = new Cluster(9);
        c.net.setLatency(10, 10);
        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));
        // the moment the commit decision is logged, the coordinator dies before the decision can be delivered
        c.net.runUntil(() -> c.coordinator.outcome("t1") == Outcome.COMMITTED, 1_000);
        c.net.partition(List.of("K"), List.of("A", "B", "C"));
        c.net.crash("K");
        c.net.run(2_000);

        assertThat(c.participants.get("A").inDoubt()).as("2PC blocks: A cannot decide without K").containsExactly("t1");
        assertThat(c.participants.get("A").isLocked("x")).isTrue();
        assertThat(c.participants.get("A").get("x")).isEmpty();

        c.net.heal();
        c.restartCoordinator(); // new instance, same log
        assertThat(c.coordinator.outcome("t1")).as("the decision survived in the log").isEqualTo(Outcome.COMMITTED);
        c.net.run(1_000);

        assertThat(c.participants.get("A").get("x")).hasValue("1");
        assertThat(c.participants.get("B").get("y")).hasValue("2");
        assertThat(c.quiet()).isTrue();
    }

    @Test
    void shouldPresumeAbortWhenTheCoordinatorRestartsWithoutHavingDecided() {
        Cluster c = new Cluster(10);
        c.net.setLatency(10, 10);
        c.coordinator.begin("t1", writes("A", "x", "1", "B", "y", "2"));
        c.net.run(15); // PREPAREs delivered, no decision yet
        c.net.crash("K");
        c.net.run(100);

        c.restartCoordinator();

        assertThat(c.coordinator.outcome("t1")).isEqualTo(Outcome.ABORTED);
        c.net.run(1_000);
        assertThat(c.quiet()).isTrue();
        assertThat(c.participants.get("A").get("x")).isEmpty();
        assertThat(c.participants.get("A").isLocked("x")).isFalse();
    }

    @Test
    void shouldAnswerAnInquiryAboutAnUnknownTransactionWithAbort() {
        Cluster c = new Cluster(11);
        c.coordinator.begin("t1", writes("A", "x", "1"));
        c.await("t1", 1_000);
        // a brand-new coordinator with an EMPTY log knows nothing about t1 -> a participant asking must hear ABORT
        c.logs.put("K", new MemoryLog());
        c.restartCoordinator();

        // A already committed t1, so nothing changes; the point is that nothing breaks and nobody blocks
        c.net.run(1_000);
        assertThat(c.participants.get("A").get("x")).hasValue("1");
        assertThatThrownBy(() -> c.coordinator.outcome("t1")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidBeginCalls() {
        Cluster c = new Cluster(12);
        c.coordinator.begin("t1", writes("A", "x", "1"));

        assertThatThrownBy(() -> c.coordinator.begin("t1", writes("A", "y", "1"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.coordinator.begin("t2", new HashMap<>())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.coordinator.begin("t3", writes("Z", "y", "1"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.coordinator.outcome("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldDecideEveryTransactionConsistentlyOnALossyNetwork() {
        Cluster c = new Cluster(13);
        c.net.setDropRate(0.2);
        List<String> txs = new ArrayList<>();
        for (int i = 0; i < 30; i++) {
            String tx = "tx" + i;
            txs.add(tx);
            c.coordinator.begin(tx, writes("A", "a" + i, "v" + i, "B", "b" + i, "v" + i, "C", "c" + i, "v" + i));
            c.net.run(40);
        }
        c.net.run(3_000);

        int committed = 0;
        for (int i = 0; i < 30; i++) {
            Outcome o = c.coordinator.outcome("tx" + i);
            assertThat(o).as("tx%d decided", i).isNotEqualTo(Outcome.PENDING);
            boolean a = c.participants.get("A").get("a" + i).isPresent();
            boolean b = c.participants.get("B").get("b" + i).isPresent();
            boolean cc = c.participants.get("C").get("c" + i).isPresent();
            assertThat(a == b && b == cc).as("tx%d atomic", i).isTrue();
            assertThat(a).as("tx%d outcome matches the data", i).isEqualTo(o == Outcome.COMMITTED);
            committed += a ? 1 : 0;
        }
        assertThat(c.quiet()).isTrue();
        assertThat(committed).as("most transactions still make it through").isGreaterThan(5);
    }

    @Test
    void shouldStayAtomicOver100RandomCrashAndPartitionSchedules() {
        for (int seed = 0; seed < 100; seed++) {
            Cluster c = new Cluster(2_000 + seed);
            java.util.Random r = new java.util.Random(seed);
            c.net.setLatency(1, 15);
            for (int i = 0; i < 6; i++) {
                c.coordinator.begin("tx" + i, writes("A", "a" + i, "v", "B", "b" + i, i % 3 == 0 ? "REJECT" : "v", "C", "c" + i, "v"));
                c.net.run(r.nextInt(150));
                String victim = List.of("A", "B", "C", "K").get(r.nextInt(4));
                switch (r.nextInt(5)) {
                    case 0 -> c.net.crash(victim);
                    case 1 -> {
                        c.net.recover(victim);
                    }
                    case 2 -> {
                        if (victim.equals("K")) {
                            c.net.crash("K");
                            c.restartCoordinator();
                            c.net.crash("K");
                        } else {
                            c.net.crash(victim);
                            c.restartParticipant(victim);
                            c.net.crash(victim);
                        }
                    }
                    case 3 -> c.net.partition(List.of(victim), List.of("A", "B", "C", "K").stream().filter(x -> !x.equals(victim)).toList());
                    default -> c.net.heal();
                }
            }
            c.net.heal();
            for (String id : List.of("A", "B", "C", "K")) {
                c.net.recover(id);
            }
            c.net.run(5_000);

            for (int i = 0; i < 6; i++) {
                Outcome o = c.coordinator.outcome("tx" + i);
                assertThat(o).as("seed %d tx%d decided", seed, i).isNotEqualTo(Outcome.PENDING);
                boolean a = c.participants.get("A").get("a" + i).isPresent();
                boolean b = c.participants.get("B").get("b" + i).isPresent();
                boolean cc = c.participants.get("C").get("c" + i).isPresent();
                assertThat(a == b && b == cc).as("seed %d tx%d atomic (%s %s %s)", seed, i, a, b, cc).isTrue();
                assertThat(a).as("seed %d tx%d matches outcome %s", seed, i, o).isEqualTo(o == Outcome.COMMITTED);
                if (i % 3 == 0) {
                    assertThat(o).as("a REJECT vote can never commit").isEqualTo(Outcome.ABORTED);
                }
            }
            assertThat(c.quiet()).as("seed %d: nobody left in doubt", seed).isTrue();
        }
    }
}

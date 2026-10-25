package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FantasyLeaderboard;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static List<String> squad(FantasyLeaderboard f, String prefix) {
        String[] roles = {"KEEPER", "BATTER", "BATTER", "BATTER", "BATTER", "BOWLER", "BOWLER", "BOWLER",
            "ALL_ROUNDER", "ALL_ROUNDER", "ALL_ROUNDER"};
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < 11; i++) {
            f.addPlayer(prefix + (i + 1), "Player", roles[i]);
            ids.add(prefix + (i + 1));
        }
        return ids;
    }

    @Test
    void shouldReturnToATieAfterACorrectionAndRestoreCreationOrder() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        f.createTeam("A", "o", squad(f, "a"), "a1", "a2");
        f.createTeam("B", "o", squad(f, "b"), "b1", "b2");
        f.recordEvent("b3", "WICKET", 1);
        assertThat(f.leaderboard(2)).containsExactly("1:B:50", "2:A:0");

        f.correctEvent("b3", "WICKET", 1);

        assertThat(f.leaderboard(2)).containsExactly("1:A:0", "1:B:0");
        assertThat(f.rankOf("B")).isEqualTo(1);
    }

    @Test
    void shouldSkipRanksAfterAThreeWayTie() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        for (String p : new String[] {"a", "b", "c", "d", "e"}) {
            f.createTeam("T" + p, "o", squad(f, p), p + "1", p + "2");
        }
        f.recordEvent("a3", "RUN", 10);
        f.recordEvent("b3", "RUN", 10);
        f.recordEvent("c3", "RUN", 10);
        f.recordEvent("d3", "RUN", 4);

        assertThat(f.leaderboard(5)).containsExactly("1:Ta:20", "1:Tb:20", "1:Tc:20", "4:Td:8", "5:Te:0");
        assertThat(f.leaderboard(2)).containsExactly("1:Ta:20", "1:Tb:20");
        assertThat(f.rankOf("Te")).isEqualTo(5);
    }

    @Test
    void shouldScoreVeryLargeCountsWithoutOverflow() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        f.createTeam("T", "o", squad(f, "a"), "a1", "a2");

        f.recordEvent("a1", "WICKET", 2_000_000_000);
        f.recordEvent("a1", "WICKET", 2_000_000_000);

        assertThat(f.playerPoints("a1")).isEqualTo(100_000_000_000L);
        assertThat(f.teamScore("T")).isEqualTo(400_000_000_000L);
        f.correctEvent("a1", "WICKET", 2_000_000_000);
        assertThat(f.teamScore("T")).isEqualTo(200_000_000_000L);
    }

    @Test
    void shouldAcceptExactlyOneOfSixteenConcurrentCreationsWithTheSameTeamId() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        List<List<String>> squads = new ArrayList<>();
        for (int i = 0; i < 16; i++) {
            squads.add(squad(f, "s" + i + "_"));
        }
        AtomicInteger ok = new AtomicInteger();

        Concurrently.run(16, t -> {
            try {
                f.createTeam("same", "owner" + t, squads.get(t), squads.get(t).get(0), squads.get(t).get(1));
                ok.incrementAndGet();
            } catch (IllegalArgumentException expected) {
                // lost the race
            }
        });

        assertThat(ok.get()).isEqualTo(1);
        assertThat(f.leaderboard(5)).hasSize(1);
    }

    @Test
    void shouldKeepSnapshotsValidWhileTeamsAreCreatedAndEventsRecordedConcurrently() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        List<String> shared = squad(f, "s");
        AtomicBoolean done = new AtomicBoolean();
        List<String> problems = java.util.Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(16, 60_000, t -> {
            if (t < 4) {
                while (!done.get()) {
                    List<String> lines = f.leaderboard(1_000);
                    java.util.Set<String> seen = new java.util.HashSet<>();
                    long prev = Long.MAX_VALUE;
                    for (String line : lines) {
                        String[] p = line.split(":");
                        long score = Long.parseLong(p[2]);
                        if (!seen.add(p[1])) {
                            problems.add("duplicate " + p[1]);
                        }
                        if (score > prev) {
                            problems.add("order broken at " + line);
                        }
                        prev = score;
                    }
                }
                return;
            }
            if (t < 12) {
                for (int i = 0; i < 100; i++) {
                    f.createTeam("T" + (t * 100 + i), "o", shared, "s1", "s2");
                }
                return;
            }
            for (int i = 0; i < 20_000; i++) {
                f.recordEvent("s" + (1 + i % 11), "RUN", 1);
            }
            if (t == 12) {
                done.set(true);
            }
        });
        done.set(true);

        assertThat(problems).isEmpty();
        assertThat(f.leaderboard(2_000)).hasSize(800);
        long expected = f.teamScore("T400");
        for (int i = 400; i < 1200; i++) {
            if (i / 100 >= 4 && i / 100 < 12) {
                assertThat(f.teamScore("T" + i)).isEqualTo(expected);
            }
        }
        assertThat(f.leaderboard(800)).allMatch(line -> line.startsWith("1:"));
    }

    @Test
    void shouldMatchRankOfWithTheLeaderboardForEveryTeam() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        int n = 60;
        for (int i = 0; i < n; i++) {
            f.createTeam("T" + i, "o", squad(f, "p" + i + "_"), "p" + i + "_1", "p" + i + "_2");
        }
        for (int i = 0; i < n; i++) {
            f.recordEvent("p" + i + "_3", "RUN", 1 + (i * 7) % 10);
        }

        List<String> lines = f.leaderboard(n);

        assertThat(lines).hasSize(n);
        for (String line : lines) {
            String[] p = line.split(":");
            assertThat(f.rankOf(p[1])).isEqualTo(Integer.parseInt(p[0]));
            assertThat(f.teamScore(p[1])).isEqualTo(Long.parseLong(p[2]));
        }
    }

    @Test
    void shouldNotLoseEventsWhenRecordingAndCorrectingTheSamePlayerFromManyThreads() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        List<String> s = squad(f, "a");
        f.createTeam("T", "o", s, "a1", "a2");
        f.createTeam("U", "o", s, "a3", "a1");
        f.recordEvent("a5", "CATCH", 16 * 500);

        Concurrently.run(16, t -> {
            for (int i = 0; i < 500; i++) {
                f.correctEvent("a5", "CATCH", 1);
                f.recordEvent("a5", "STUMPING", 1);
            }
        });

        assertThat(f.playerPoints("a5")).isEqualTo(16L * 500 * 12);
        assertThat(f.teamScore("T")).isEqualTo(2L * 16 * 500 * 12);
        assertThatThrownBy(() -> f.correctEvent("a5", "CATCH", 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRankTwoHundredThousandEventsOverManyTeamsWithoutSlowdownAcrossThreads() {
        FantasyLeaderboard f = Candidate.of(Solution::create);
        int players = 2_000;
        String[] roles = {"KEEPER", "BATTER", "BATTER", "BOWLER", "ALL_ROUNDER"};
        for (int i = 0; i < players; i++) {
            f.addPlayer("P" + i, "Player", roles[i % 5]);
        }
        for (int i = 0; i < 20_000; i++) {
            List<String> ids = new ArrayList<>();
            for (int j = 0; j < 11; j++) {
                ids.add("P" + ((i * 3 + j * 437) % players));
            }
            f.createTeam("T" + i, "o", ids, ids.get(0), ids.get(1));
        }

        Concurrently.run(16, 120_000, t -> {
            for (int i = 0; i < 12_500; i++) {
                f.recordEvent("P" + ((t * 131 + i * 17) % players), "RUN", 1);
                if (i % 250 == 0) {
                    assertThat(f.leaderboard(10)).hasSize(10);
                }
            }
        });

        long total = 0;
        for (int i = 0; i < players; i++) {
            total += f.playerPoints("P" + i);
        }
        assertThat(total).isEqualTo(200_000);
    }
}

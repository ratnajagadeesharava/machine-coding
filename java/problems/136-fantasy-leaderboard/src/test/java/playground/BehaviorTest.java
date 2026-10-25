package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.FantasyLeaderboard;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private FantasyLeaderboard board() {
        return Candidate.of(Solution::create);
    }

    /** Registers prefix1..prefix11: 1 KEEPER, 4 BATTER (2-5), 3 BOWLER (6-8), 3 ALL_ROUNDER (9-11). */
    private static List<String> squad(FantasyLeaderboard f, String prefix) {
        String[] roles = {"KEEPER", "BATTER", "BATTER", "BATTER", "BATTER", "BOWLER", "BOWLER", "BOWLER",
            "ALL_ROUNDER", "ALL_ROUNDER", "ALL_ROUNDER"};
        List<String> ids = new ArrayList<>();
        for (int i = 0; i < 11; i++) {
            String id = prefix + (i + 1);
            f.addPlayer(id, "Player " + id, roles[i]);
            ids.add(id);
        }
        return ids;
    }

    /** Team with captain = first player of the squad and vice-captain = second. */
    private static void team(FantasyLeaderboard f, String teamId, List<String> squad) {
        f.createTeam(teamId, "owner-" + teamId, squad, squad.get(0), squad.get(1));
    }

    @Test
    void shouldStartEveryTeamAtZeroAndListThemInCreationOrder() {
        FantasyLeaderboard f = board();
        team(f, "T1", squad(f, "a"));
        team(f, "T2", squad(f, "b"));
        team(f, "T3", squad(f, "c"));

        assertThat(f.leaderboard(10)).containsExactly("1:T1:0", "1:T2:0", "1:T3:0");
        assertThat(f.teamScore("T2")).isZero();
        assertThat(f.rankOf("T3")).isEqualTo(1);
    }

    @Test
    void shouldScoreEveryEventTypeWithTheFixedTable() {
        FantasyLeaderboard f = board();
        squad(f, "a");
        f.recordEvent("a3", "RUN", 10);
        f.recordEvent("a3", "FOUR_BONUS", 2);
        f.recordEvent("a3", "SIX_BONUS", 3);
        f.recordEvent("a3", "WICKET", 2);
        f.recordEvent("a3", "CATCH", 1);
        f.recordEvent("a3", "STUMPING", 1);
        f.recordEvent("a3", "MAIDEN", 1);
        f.recordEvent("a3", "DUCK", 1);

        // 10 + 2 + 6 + 50 + 8 + 12 + 12 - 2
        assertThat(f.playerPoints("a3")).isEqualTo(98);
        assertThat(f.playerPoints("a4")).isZero();
    }

    @Test
    void shouldDoubleTheCaptainAndPayTheViceCaptainOneAndAHalf() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        team(f, "T", s); // captain a1, vice-captain a2

        f.recordEvent("a1", "RUN", 50);
        assertThat(f.teamScore("T")).isEqualTo(200); // 50 points x2 = 100 points = 200 half points
        f.recordEvent("a2", "CATCH", 1);
        assertThat(f.teamScore("T")).isEqualTo(224); // + 8 x1.5 = 12 points = 24 half points
        f.recordEvent("a3", "WICKET", 1);
        assertThat(f.teamScore("T")).isEqualTo(274); // + 25 points = 50 half points
        assertThat(f.leaderboard(1)).containsExactly("1:T:274");
    }

    @Test
    void shouldGiveHalfPointsForAnOddViceCaptainTotal() {
        FantasyLeaderboard f = board();
        team(f, "T", squad(f, "a"));

        f.recordEvent("a2", "RUN", 1); // 1 x1.5 = 1.5 points

        assertThat(f.teamScore("T")).isEqualTo(3);
    }

    @Test
    void shouldUpdateEveryTeamContainingThePlayerImmediately() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        f.createTeam("captainTeam", "x", s, "a1", "a2");
        f.createTeam("viceTeam", "y", s, "a2", "a1");
        f.createTeam("plainTeam", "z", s, "a3", "a4");

        f.recordEvent("a1", "WICKET", 1); // 25 points

        assertThat(f.teamScore("captainTeam")).isEqualTo(100);
        assertThat(f.teamScore("viceTeam")).isEqualTo(75);
        assertThat(f.teamScore("plainTeam")).isEqualTo(50);
        assertThat(f.leaderboard(3)).containsExactly("1:captainTeam:100", "2:viceTeam:75", "3:plainTeam:50");
    }

    @Test
    void shouldOnlyChangeTeamsThatContainThePlayer() {
        FantasyLeaderboard f = board();
        team(f, "A", squad(f, "a"));
        team(f, "B", squad(f, "b"));

        f.recordEvent("b5", "MAIDEN", 1);

        assertThat(f.teamScore("A")).isZero();
        assertThat(f.teamScore("B")).isEqualTo(24);
        assertThat(f.leaderboard(2)).containsExactly("1:B:24", "2:A:0");
    }

    @Test
    void shouldCountEventsRecordedBeforeATeamWasCreated() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        f.recordEvent("a1", "RUN", 10);
        f.recordEvent("a5", "RUN", 10);

        team(f, "late", s);

        assertThat(f.teamScore("late")).isEqualTo(60); // 10 x2 + 10 for a5 as half points: 40 + 20
    }

    @Test
    void shouldBreakTiesByTeamCreationOrder() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        List<String> t = squad(f, "b");
        team(f, "first", s);
        team(f, "second", t);

        f.recordEvent("b3", "RUN", 5);
        assertThat(f.leaderboard(2)).containsExactly("1:second:10", "2:first:0");

        f.recordEvent("a3", "RUN", 5);
        assertThat(f.leaderboard(2)).containsExactly("1:first:10", "1:second:10");
    }

    @Test
    void shouldUseCompetitionRankingForTies() {
        FantasyLeaderboard f = board();
        List<String> top = squad(f, "a");
        List<String> mid1 = squad(f, "b");
        List<String> mid2 = squad(f, "c");
        List<String> low = squad(f, "d");
        team(f, "low", low);
        team(f, "mid1", mid1);
        team(f, "top", top);
        team(f, "mid2", mid2);
        f.recordEvent("a3", "RUN", 100);
        f.recordEvent("b3", "RUN", 50);
        f.recordEvent("c3", "RUN", 50);

        assertThat(f.leaderboard(4)).containsExactly("1:top:200", "2:mid1:100", "2:mid2:100", "4:low:0");
        assertThat(f.rankOf("top")).isEqualTo(1);
        assertThat(f.rankOf("mid1")).isEqualTo(2);
        assertThat(f.rankOf("mid2")).isEqualTo(2);
        assertThat(f.rankOf("low")).isEqualTo(4);
    }

    @Test
    void shouldReturnOnlyTheTopKAndKeepTheRankOfATiedCut() {
        FantasyLeaderboard f = board();
        team(f, "A", squad(f, "a"));
        team(f, "B", squad(f, "b"));
        team(f, "C", squad(f, "c"));
        f.recordEvent("c3", "RUN", 1);

        assertThat(f.leaderboard(0)).isEmpty();
        assertThat(f.leaderboard(1)).containsExactly("1:C:2");
        assertThat(f.leaderboard(2)).containsExactly("1:C:2", "2:A:0");
        assertThat(f.leaderboard(100)).containsExactly("1:C:2", "2:A:0", "2:B:0");
        assertThat(board().leaderboard(5)).isEmpty();
    }

    @Test
    void shouldRejectANegativeK() {
        FantasyLeaderboard f = board();

        assertThatThrownBy(() -> f.leaderboard(-1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldLetADuckMakeTeamScoresNegative() {
        FantasyLeaderboard f = board();
        team(f, "T", squad(f, "a"));
        team(f, "U", squad(f, "b"));

        f.recordEvent("a1", "DUCK", 1);

        assertThat(f.playerPoints("a1")).isEqualTo(-2);
        assertThat(f.teamScore("T")).isEqualTo(-8);
        assertThat(f.leaderboard(2)).containsExactly("1:U:0", "2:T:-8");
    }

    @Test
    void shouldReverseEventsWithCorrectEvent() {
        FantasyLeaderboard f = board();
        team(f, "T", squad(f, "a"));
        team(f, "U", squad(f, "b"));
        f.recordEvent("a1", "WICKET", 2);
        f.recordEvent("b3", "RUN", 10);

        f.correctEvent("a1", "WICKET", 1);

        assertThat(f.playerPoints("a1")).isEqualTo(25);
        assertThat(f.teamScore("T")).isEqualTo(100);
        assertThat(f.leaderboard(2)).containsExactly("1:T:100", "2:U:20");

        f.correctEvent("a1", "WICKET", 1);
        assertThat(f.teamScore("T")).isZero();
        assertThat(f.leaderboard(2)).containsExactly("1:U:20", "2:T:0");
    }

    @Test
    void shouldRejectACorrectionBeyondTheRecordedCountAndChangeNothing() {
        FantasyLeaderboard f = board();
        team(f, "T", squad(f, "a"));
        f.recordEvent("a1", "RUN", 5);
        f.recordEvent("a1", "WICKET", 1);

        assertThatThrownBy(() -> f.correctEvent("a1", "RUN", 6)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.correctEvent("a1", "CATCH", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.correctEvent("a3", "RUN", 1)).isInstanceOf(IllegalArgumentException.class);

        assertThat(f.playerPoints("a1")).isEqualTo(30);
        assertThat(f.teamScore("T")).isEqualTo(120);
        f.correctEvent("a1", "RUN", 5);
        assertThatThrownBy(() -> f.correctEvent("a1", "RUN", 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldTrackTheCorrectionLimitPerEventTypeAndKeepAllowanceAfterRecordingAgain() {
        FantasyLeaderboard f = board();
        squad(f, "a");
        f.recordEvent("a1", "RUN", 3);
        f.correctEvent("a1", "RUN", 3);
        f.recordEvent("a1", "RUN", 2);

        f.correctEvent("a1", "RUN", 2);

        assertThat(f.playerPoints("a1")).isZero();
        assertThatThrownBy(() -> f.correctEvent("a1", "RUN", 1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidEvents() {
        FantasyLeaderboard f = board();
        squad(f, "a");

        assertThatThrownBy(() -> f.recordEvent("nobody", "RUN", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.recordEvent("a1", "HAT_TRICK", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.recordEvent("a1", "run", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.recordEvent("a1", null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.recordEvent("a1", "RUN", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.recordEvent("a1", "RUN", -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.correctEvent("a1", "RUN", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(f.playerPoints("a1")).isZero();
    }

    @Test
    void shouldRejectBadPlayerRegistrations() {
        FantasyLeaderboard f = board();
        f.addPlayer("p1", "One", "KEEPER");

        assertThatThrownBy(() -> f.addPlayer("p1", "Again", "BATTER")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer(" ", "Name", "BATTER")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer("p2", "", "BATTER")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer("p2", "Name", "COACH")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer("p2", "Name", "batter")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer("p2", "Name", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addPlayer(null, "Name", "BATTER")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectTeamsThatDoNotHaveExactlyElevenDistinctRegisteredPlayers() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        f.addPlayer("extra", "Extra", "BOWLER");

        List<String> ten = new ArrayList<>(s.subList(0, 10));
        List<String> twelve = new ArrayList<>(s);
        twelve.add("extra");
        List<String> duplicate = new ArrayList<>(s.subList(0, 10));
        duplicate.add("a2");
        List<String> unknown = new ArrayList<>(s.subList(0, 10));
        unknown.add("ghost");

        assertThatThrownBy(() -> f.createTeam("t", "o", ten, "a1", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", twelve, "a1", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", duplicate, "a1", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", unknown, "a1", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", List.of(), "a1", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThat(f.leaderboard(5)).isEmpty();
    }

    @Test
    void shouldRejectBadCaptainChoices() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        f.addPlayer("outsider", "Outsider", "BOWLER");

        assertThatThrownBy(() -> f.createTeam("t", "o", s, "outsider", "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", s, "a1", "outsider")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", s, "a1", "a1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("t", "o", s, null, "a2")).isInstanceOf(IllegalArgumentException.class);
        assertThat(f.leaderboard(5)).isEmpty();
    }

    @Test
    void shouldEnforceTheRoleRules() {
        FantasyLeaderboard f = board();
        for (int i = 1; i <= 7; i++) {
            f.addPlayer("bat" + i, "Bat " + i, "BATTER");
        }
        for (int i = 1; i <= 5; i++) {
            f.addPlayer("bowl" + i, "Bowl " + i, "BOWLER");
        }
        f.addPlayer("keep", "Keeper", "KEEPER");
        f.addPlayer("all", "All", "ALL_ROUNDER");

        List<String> noKeeper = List.of("bat1", "bat2", "bat3", "bat4", "bat5", "bowl1", "bowl2", "bowl3", "bowl4", "bowl5", "all");
        List<String> sevenBatters = List.of("bat1", "bat2", "bat3", "bat4", "bat5", "bat6", "bat7", "bowl1", "bowl2", "keep", "all");
        List<String> sixBatters = List.of("bat1", "bat2", "bat3", "bat4", "bat5", "bat6", "bowl1", "bowl2", "bowl3", "keep", "all");

        assertThatThrownBy(() -> f.createTeam("noKeeper", "o", noKeeper, "bat1", "bat2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("seven", "o", sevenBatters, "bat1", "bat2")).isInstanceOf(IllegalArgumentException.class);
        f.createTeam("six", "o", sixBatters, "bat1", "keep");

        assertThat(f.leaderboard(5)).containsExactly("1:six:0");
    }

    @Test
    void shouldRejectDuplicateTeamIdsAndBlankInputsWithoutDisturbingTheExistingTeam() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        List<String> t = squad(f, "b");
        team(f, "T", s);
        f.recordEvent("a1", "RUN", 1);

        assertThatThrownBy(() -> team(f, "T", t)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam(" ", "o", t, "b1", "b2")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.createTeam("U", "", t, "b1", "b2")).isInstanceOf(IllegalArgumentException.class);

        assertThat(f.leaderboard(5)).containsExactly("1:T:4");
        team(f, "U", t); // a failed attempt must not have used up the id
        assertThat(f.leaderboard(5)).containsExactly("1:T:4", "2:U:0");
    }

    @Test
    void shouldRejectUnknownTeamsAndPlayersInQueries() {
        FantasyLeaderboard f = board();

        assertThatThrownBy(() -> f.rankOf("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.teamScore("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.playerPoints("nope")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldKeepASortedDuplicateFreeLeaderboardWhileSixteenThreadsRecordEvents() {
        FantasyLeaderboard f = board();
        int teams = 40;
        for (int i = 0; i < teams; i++) {
            team(f, "T" + i, squad(f, "p" + i + "_"));
        }
        // two teams that always hold the same squad must never be seen apart
        List<String> shared = squad(f, "s");
        f.createTeam("twinA", "o", shared, "s1", "s2");
        f.createTeam("twinB", "o", shared, "s1", "s2");
        AtomicBoolean done = new AtomicBoolean();
        List<String> problems = java.util.Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(16, 60_000, t -> {
            if (t < 4) {
                while (!done.get()) {
                    check(f.leaderboard(teams + 2), teams + 2, problems);
                }
                return;
            }
            for (int i = 0; i < 5_000; i++) {
                int n = (t * 31 + i) % teams;
                f.recordEvent("p" + n + "_" + (1 + i % 11), "RUN", 1 + i % 5);
                f.recordEvent("s" + (1 + i % 11), "RUN", 1);
            }
            if (t == 4) {
                done.set(true);
            }
        });
        done.set(true);

        assertThat(problems).isEmpty();
        assertThat(f.teamScore("twinA")).isEqualTo(f.teamScore("twinB"));
    }

    private static void check(List<String> lines, int expectedSize, List<String> problems) {
        if (lines.size() != expectedSize) {
            problems.add("size " + lines.size());
            return;
        }
        java.util.Set<String> seen = new java.util.HashSet<>();
        long prev = Long.MAX_VALUE;
        long twinA = Long.MIN_VALUE;
        long twinB = Long.MAX_VALUE;
        for (int i = 0; i < lines.size(); i++) {
            String[] p = lines.get(i).split(":");
            long score = Long.parseLong(p[2]);
            if (!seen.add(p[1])) {
                problems.add("duplicate " + p[1]);
            }
            if (score > prev) {
                problems.add("unsorted at " + i);
            }
            prev = score;
            if (p[1].equals("twinA")) {
                twinA = score;
            }
            if (p[1].equals("twinB")) {
                twinB = score;
            }
        }
        if (twinA != twinB) {
            problems.add("twins differ " + twinA + " vs " + twinB);
        }
    }

    @Test
    void shouldEndWithTheSequentialScoresAfterConcurrentEventsAndCorrections() {
        FantasyLeaderboard f = board();
        List<String> s = squad(f, "a");
        f.createTeam("T", "o", s, "a1", "a2");
        f.createTeam("U", "o", s, "a2", "a3");

        Concurrently.run(16, t -> {
            for (int i = 0; i < 1_000; i++) {
                String player = s.get((t + i) % 11);
                f.recordEvent(player, "WICKET", 2);
                f.recordEvent(player, "RUN", 3);
                f.correctEvent(player, "RUN", 1);
            }
        });

        long totalPoints = 0;
        for (String p : s) {
            totalPoints += f.playerPoints(p);
        }
        assertThat(totalPoints).isEqualTo(16L * 1_000 * (50 + 2));
        long expectedT = 0;
        long expectedU = 0;
        for (int i = 0; i < 11; i++) {
            long pts = f.playerPoints(s.get(i));
            expectedT += pts * (i == 0 ? 4 : i == 1 ? 3 : 2);
            expectedU += pts * (i == 1 ? 4 : i == 2 ? 3 : 2);
        }
        assertThat(f.teamScore("T")).isEqualTo(expectedT);
        assertThat(f.teamScore("U")).isEqualTo(expectedU);
        assertThat(f.leaderboard(2)).hasSize(2);
    }

    @Test
    void shouldHandleFiftyThousandTeamsTwoHundredThousandEventsAndThousandLeaderboardCalls() {
        FantasyLeaderboard f = board();
        int players = 5_000;
        String[] roles = {"KEEPER", "BATTER", "BATTER", "BOWLER", "ALL_ROUNDER"};
        for (int i = 0; i < players; i++) {
            f.addPlayer("P" + i, "Player " + i, roles[i % 5]);
        }
        int teams = 50_000;
        for (int i = 0; i < teams; i++) {
            List<String> ids = new ArrayList<>();
            for (int j = 0; j < 11; j++) {
                ids.add("P" + ((i * 7 + j * 437) % players));
            }
            f.createTeam("T" + i, "owner" + (i % 100), ids, ids.get(0), ids.get(1));
        }
        long[] points = new long[players];
        long seed = 12345;
        List<String> last = null;
        long start = System.nanoTime();
        for (int e = 0; e < 200_000; e++) {
            seed = seed * 6364136223846793005L + 1442695040888963407L;
            int p = (int) ((seed >>> 33) % players);
            int count = 1 + (int) ((seed >>> 20) & 3);
            f.recordEvent("P" + p, "RUN", count);
            points[p] += count;
            if (e % 200 == 0) {
                last = f.leaderboard(10);
            }
        }
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(last).hasSize(10);
        assertThat(elapsedMs).as("200k events + 1k leaderboard calls").isLessThan(60_000);
        long[] expected = new long[teams];
        for (int i = 0; i < teams; i++) {
            for (int j = 0; j < 11; j++) {
                int p = (i * 7 + j * 437) % players;
                expected[i] += points[p] * (j == 0 ? 4 : j == 1 ? 3 : 2);
            }
        }
        for (int i : new int[] {0, 1, 777, 25_000, 49_999}) {
            assertThat(f.teamScore("T" + i)).isEqualTo(expected[i]);
        }
        long best = Long.MIN_VALUE;
        for (long v : expected) {
            best = Math.max(best, v);
        }
        assertThat(f.leaderboard(1).get(0)).endsWith(":" + best);
    }
}

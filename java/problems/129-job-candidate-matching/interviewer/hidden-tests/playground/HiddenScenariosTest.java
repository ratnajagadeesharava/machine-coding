package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.JobMatching;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static Set<String> s(String... skills) {
        return new LinkedHashSet<>(Arrays.asList(skills));
    }

    @Test
    void shouldNotOverflowTheSalaryBonusForHugeSalaries() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s(), s(), 0, "Pune", Long.MAX_VALUE);
        m.registerCandidate("half", s(), 0, "Pune", Long.MAX_VALUE / 2, false);
        m.registerCandidate("overHalf", s(), 0, "Pune", Long.MAX_VALUE / 2 + 1, false);
        m.registerCandidate("max", s(), 0, "Pune", Long.MAX_VALUE, false);

        assertThat(m.matchScore("J", "half")).isEqualTo(70);
        assertThat(m.matchScore("J", "overHalf")).isEqualTo(60);
        assertThat(m.matchScore("J", "max")).isEqualTo(60);
    }

    @Test
    void shouldBreakTiesByYearsDescendingThenIdAsStringsNotNumbers() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s(), s(), 0, "Pune", 100);
        // years 10..14 all hit the same capped-or-not scores differently, so use equal scores via years >= 10
        for (String id : List.of("c9", "c10", "c2", "c1")) {
            m.registerCandidate(id, s(), 15, "Pune", 10, false);
        }
        m.registerCandidate("older", s(), 30, "Pune", 10, false); // same score (capped), more years
        m.registerCandidate("Cap", s(), 15, "Pune", 10, false);

        assertThat(m.topCandidates("J", 10)).containsExactly("older", "Cap", "c1", "c10", "c2", "c9");
    }

    @Test
    void shouldMatchABruteForceOracleOnRandomData() {
        JobMatching m = Candidate.of(Solution::create);
        Random rnd = new Random(42);
        String[] pool = {"java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s"};
        String[] cities = {"Pune", "Delhi", "Goa"};
        int[] years = new int[300];
        List<String> candIds = new ArrayList<>();
        for (int c = 0; c < 300; c++) {
            Set<String> sk = new HashSet<>();
            for (int i = 0; i < 1 + rnd.nextInt(6); i++) {
                sk.add(pool[rnd.nextInt(pool.length)]);
            }
            years[c] = rnd.nextInt(15);
            m.registerCandidate("c" + c, sk, years[c], cities[rnd.nextInt(3)], 10 + rnd.nextInt(90), rnd.nextInt(4) == 0);
            candIds.add("c" + c);
        }
        for (int j = 0; j < 40; j++) {
            Set<String> req = new HashSet<>(List.of(pool[rnd.nextInt(pool.length)]));
            Set<String> nice = new HashSet<>();
            for (int i = 0; i < 4; i++) {
                nice.add(pool[rnd.nextInt(pool.length)].toUpperCase());
            }
            m.postJob("j" + j, "t", req, nice, rnd.nextInt(8), cities[rnd.nextInt(3)], 30 + rnd.nextInt(80));
        }

        for (int j = 0; j < 40; j++) {
            String job = "j" + j;
            List<String> expected = new ArrayList<>(candIds);
            expected.removeIf(c -> m.matchScore(job, c) == 0);
            expected.sort((a, b) -> {
                int x = Integer.compare(m.matchScore(job, b), m.matchScore(job, a));
                if (x != 0) {
                    return x;
                }
                x = Integer.compare(years[Integer.parseInt(b.substring(1))], years[Integer.parseInt(a.substring(1))]);
                return x != 0 ? x : a.compareTo(b);
            });
            assertThat(m.topCandidates(job, 1_000)).as(job).containsExactlyElementsOf(expected);
            assertThat(m.topCandidates(job, 5)).as(job).containsExactlyElementsOf(
                    expected.subList(0, Math.min(5, expected.size())));
        }
        for (int c = 0; c < 300; c += 7) {
            String cid = "c" + c;
            List<String> expected = new ArrayList<>();
            for (int j = 0; j < 40; j++) {
                if (m.matchScore("j" + j, cid) > 0) {
                    expected.add("j" + j);
                }
            }
            expected.sort((a, b) -> {
                int x = Integer.compare(m.matchScore(b, cid), m.matchScore(a, cid));
                return x != 0 ? x : a.compareTo(b);
            });
            assertThat(m.topJobs(cid, 100)).as(cid).containsExactlyElementsOf(expected);
        }
    }

    @Test
    void shouldNeverAcceptAnApplicationAfterCloseJobReturned() {
        for (int round = 0; round < 20; round++) {
            JobMatching m = Candidate.of(Solution::create);
            m.postJob("J", "t", s("java"), s(), 0, "Pune", 100);
            for (int i = 0; i < 200; i++) {
                m.registerCandidate("c" + i, s("java"), 1, "Pune", 10, false);
            }
            ConcurrentLinkedQueue<String> accepted = new ConcurrentLinkedQueue<>();
            AtomicBoolean closed = new AtomicBoolean();
            AtomicBoolean lateAccept = new AtomicBoolean();

            Concurrently.run(9, t -> {
                if (t == 8) {
                    m.closeJob("J");
                    closed.set(true);
                    return;
                }
                for (int i = t; i < 200; i += 8) {
                    boolean closedBefore = closed.get();
                    try {
                        m.apply("c" + i, "J");
                        accepted.add("c" + i);
                        if (closedBefore) {
                            lateAccept.set(true);
                        }
                    } catch (IllegalStateException expected) {
                        // job already closed
                    }
                }
            });

            assertThat(lateAccept).isFalse();
            assertThat(m.applications("J")).containsExactlyInAnyOrderElementsOf(accepted);
        }
    }

    @Test
    void shouldStayConsistentWhileCandidatesAreUpdatedAndRanked() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s("java"), s("go"), 1, "Pune", 100);
        for (int i = 0; i < 100; i++) {
            m.registerCandidate("c" + i, s("java"), 3, "Pune", 10, false);
        }

        Concurrently.run(8, 60_000, t -> {
            if (t < 4) {
                for (int i = 0; i < 3_000; i++) {
                    String id = "c" + ((i * 4 + t) % 100);
                    if (i % 2 == 0) {
                        m.updateCandidate(id, s("python"), 3, "Pune", 10, false);
                    } else {
                        m.updateCandidate(id, s("java", "go"), 3, "Pune", 10, false);
                    }
                }
            } else {
                for (int i = 0; i < 1_500; i++) {
                    List<String> top = m.topCandidates("J", 10);
                    assertThat(top).hasSizeLessThanOrEqualTo(10);
                    assertThat(new HashSet<>(top)).hasSameSizeAs(top);
                    m.topJobs("c" + (i % 100), 5);
                }
            }
        });

        // every candidate's last update was applied, whichever thread did it last
        List<String> top = m.topCandidates("J", 1_000);
        for (int i = 0; i < 100; i++) {
            assertThat(top.contains("c" + i)).isEqualTo(m.matchScore("J", "c" + i) > 0);
        }
        m.updateCandidate("c0", s("java", "go"), 3, "Pune", 10, false);
        assertThat(m.topCandidates("J", 1)).containsExactly("c0");
    }

    @Test
    void shouldHandleTheScaleScenarioWithConcurrentProfileUpdates() {
        JobMatching m = Candidate.of(Solution::create);
        String[] skills = {"java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s", "aws", "react"};
        String[] cities = {"Pune", "Delhi", "Mumbai", "Goa"};
        for (int j = 0; j < 500; j++) {
            m.postJob("job" + j, "t", s(skills[j % 10]), s(skills[(j + 2) % 10], skills[(j + 5) % 10]),
                    j % 8, cities[j % 4], 50 + (j % 50));
        }
        for (int c = 0; c < 5_000; c++) {
            m.registerCandidate("cand" + c, s(skills[c % 10], skills[(c / 10) % 10], skills[(c * 7) % 10]),
                    c % 12, cities[(c / 3) % 4], 10 + (c % 60), c % 5 == 0);
        }

        long start = System.nanoTime();
        Concurrently.run(5, 60_000, t -> {
            if (t == 4) {
                for (int i = 0; i < 2_000; i++) {
                    m.updateCandidate("cand" + (i * 2), s(skills[i % 10], "go"), i % 12, cities[i % 4], 20, true);
                }
                return;
            }
            for (int i = 0; i < 250; i++) {
                assertThat(m.topCandidates("job" + ((i * 4 + t) % 500), 10)).hasSizeLessThanOrEqualTo(10);
            }
        });
        for (int i = 0; i < 1_000; i++) {
            m.topCandidates("job" + (i % 500), 10);
        }
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).as("scale run took %d ms", ms).isLessThan(8_000);
    }

    @Test
    void shouldCountDuplicateAndMixedCaseSkillsOnceAfterNormalisation() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s("Java", "JAVA", " java "), s("Go", "gO", "go ", "RUST", "rust"), 0, "Pune", 100);
        m.registerCandidate("c", s("GO", "go", "java"), 0, "Pune", 90, false);

        assertThat(m.matchScore("J", "c")).isEqualTo(50 + 10 + 0 + 10 + 0);
    }

    @Test
    void shouldKeepApplicationsOfARejectedWithdrawAndAllowReapplyAcrossJobs() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s("java"), s(), 0, "Pune", 100);
        m.postJob("K", "t", s("java"), s(), 0, "Pune", 100);
        m.registerCandidate("a", s("java"), 1, "Pune", 10, false);
        m.registerCandidate("b", s("java"), 1, "Pune", 10, false);
        m.apply("a", "J");
        m.apply("b", "J");
        m.apply("a", "K");

        assertThatThrownBy(() -> m.withdraw("b", "K")).isInstanceOf(IllegalStateException.class);
        m.withdraw("a", "K");

        assertThat(m.applications("J")).containsExactly("a", "b");
        assertThat(m.applications("K")).isEmpty();
    }

    @Test
    void shouldRejectApplyOnAClosedJobEvenForAnIneligibleCandidateWithoutChangingAnything() {
        JobMatching m = Candidate.of(Solution::create);
        m.postJob("J", "t", s("java"), s(), 0, "Pune", 100);
        m.registerCandidate("a", s("java"), 1, "Pune", 10, false);
        m.registerCandidate("x", s("go"), 1, "Pune", 10, false);
        m.apply("a", "J");
        m.closeJob("J");

        assertThatThrownBy(() -> m.apply("x", "J")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> m.apply("a", "J")).isInstanceOf(IllegalStateException.class);
        assertThat(m.applications("J")).containsExactly("a");
    }
}

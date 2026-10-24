package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.JobMatching;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private JobMatching svc() {
        return Candidate.of(Solution::create);
    }

    private static Set<String> s(String... skills) {
        return new LinkedHashSet<>(Arrays.asList(skills));
    }

    /** A job needing java, minYears 2, Pune, max 100, no nice-to-have skills. */
    private static void simpleJob(JobMatching m, String id) {
        m.postJob(id, "Engineer", s("java"), s(), 2, "Pune", 100);
    }

    private static void cand(JobMatching m, String id, int years, String loc, long salary, boolean remote, String... skills) {
        m.registerCandidate(id, s(skills), years, loc, salary, remote);
    }

    private JobMatching backendExample() {
        JobMatching m = svc();
        m.postJob("backend", "Backend Engineer", s("java", "sql"), s("kafka", "redis", "docker"), 3, "Pune", 100);
        cand(m, "alice", 6, "Pune", 40, false, "Java", "SQL", "Kafka", "Redis");
        cand(m, "bob", 10, "Delhi", 60, true, "java", "sql", "docker", "kafka", "redis");
        cand(m, "xena", 5, "Delhi", 40, true, "java", "sql", "kafka", "redis", "docker");
        cand(m, "zed", 5, "Delhi", 40, true, "java", "sql", "kafka", "redis", "docker");
        cand(m, "erin", 20, "pune", 50, false, " JAVA ", " Sql ");
        cand(m, "carol", 8, "Pune", 10, false, "java");
        cand(m, "dave", 9, "Mumbai", 10, false, "java", "sql");
        cand(m, "frank", 2, "Pune", 10, false, "java", "sql");
        cand(m, "gina", 4, "Pune", 101, false, "java", "sql");
        return m;
    }

    @Test
    void shouldScoreTheWorkedExampleExactly() {
        JobMatching m = backendExample();

        assertThat(m.matchScore("backend", "alice")).isEqualTo(96);
        assertThat(m.matchScore("backend", "bob")).isEqualTo(94);
        assertThat(m.matchScore("backend", "xena")).isEqualTo(94);
        assertThat(m.matchScore("backend", "zed")).isEqualTo(94);
        assertThat(m.matchScore("backend", "erin")).isEqualTo(90);
        for (String c : List.of("carol", "dave", "frank", "gina")) {
            assertThat(m.matchScore("backend", c)).as(c).isZero();
        }
    }

    @Test
    void shouldRankCandidatesByScoreThenYearsThenId() {
        JobMatching m = backendExample();

        assertThat(m.topCandidates("backend", 10)).containsExactly("alice", "bob", "xena", "zed", "erin");
        assertThat(m.topCandidates("backend", 3)).containsExactly("alice", "bob", "xena");
        assertThat(m.topCandidates("backend", 1)).containsExactly("alice");
    }

    @Test
    void shouldRankJobsByScoreThenJobIdAndSkipIneligibleOnes() {
        JobMatching m = svc();
        m.registerCandidate("c1", s("java", "docker"), 2, "Pune", 40, true);
        m.postJob("A", "a", s("java"), s(), 0, "Pune", 100);
        m.postJob("B", "b", s("java"), s("docker"), 2, "Delhi", 80);
        m.postJob("B2", "b2", s("java"), s("docker"), 2, "Delhi", 80);
        m.postJob("D", "d", s("java"), s(), 2, "Delhi", 80);
        m.postJob("C", "c", s("java", "go"), s(), 0, "Pune", 100);

        assertThat(m.matchScore("A", "c1")).isEqualTo(74);
        assertThat(m.matchScore("B", "c1")).isEqualTo(70);
        assertThat(m.matchScore("D", "c1")).isEqualTo(60);
        assertThat(m.matchScore("C", "c1")).isZero();
        assertThat(m.topJobs("c1", 10)).containsExactly("A", "B", "B2", "D");
        assertThat(m.topJobs("c1", 2)).containsExactly("A", "B");

        m.closeJob("B");

        assertThat(m.topJobs("c1", 10)).containsExactly("A", "B2", "D");
        assertThat(m.matchScore("B", "c1")).isZero();
    }

    @Test
    void shouldFollowTheApplicationLifecycleOfTheWorkedExample() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "a", 2, "Pune", 50, false, "java");
        cand(m, "b", 5, "Pune", 50, false, "java");
        cand(m, "c", 5, "Pune", 50, false, "go");
        assertThat(m.matchScore("J", "a")).isEqualTo(70);
        assertThat(m.matchScore("J", "b")).isEqualTo(76);

        m.apply("a", "J");
        m.apply("b", "J");
        assertThat(m.applications("J")).containsExactly("a", "b");
        assertThatThrownBy(() -> m.apply("a", "J")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> m.apply("c", "J")).isInstanceOf(IllegalStateException.class);

        m.withdraw("a", "J");
        m.apply("a", "J");
        assertThat(m.applications("J")).containsExactly("b", "a");

        m.updateCandidate("a", s("go"), 2, "Pune", 50, false);
        assertThat(m.matchScore("J", "a")).isZero();
        assertThat(m.topCandidates("J", 5)).containsExactly("b");
        assertThat(m.applications("J")).containsExactly("b", "a");

        m.closeJob("J");
        assertThat(m.topCandidates("J", 5)).isEmpty();
        assertThat(m.matchScore("J", "b")).isZero();
        assertThat(m.applications("J")).containsExactly("b", "a");
        assertThatThrownBy(() -> m.apply("b", "J")).isInstanceOf(IllegalStateException.class);
        assertThat(m.topJobs("b", 5)).isEmpty();
    }

    @Test
    void shouldRequireEveryRequiredSkill() {
        JobMatching m = svc();
        m.postJob("J", "t", s("java", "sql"), s(), 0, "Pune", 100);
        cand(m, "one", 1, "Pune", 10, false, "java");
        cand(m, "both", 1, "Pune", 10, false, "java", "sql", "extra");

        assertThat(m.matchScore("J", "one")).isZero();
        assertThat(m.matchScore("J", "both")).isPositive();
    }

    @Test
    void shouldTreatYearsAndSalaryBoundariesAsEligible() {
        JobMatching m = svc();
        simpleJob(m, "J"); // minYears 2, max 100
        cand(m, "edge", 2, "Pune", 100, false, "java");
        cand(m, "lessYears", 1, "Pune", 10, false, "java");
        cand(m, "pricey", 5, "Pune", 101, false, "java");

        assertThat(m.matchScore("J", "edge")).isEqualTo(50 + 0 + 0 + 10 + 0);
        assertThat(m.matchScore("J", "lessYears")).isZero();
        assertThat(m.matchScore("J", "pricey")).isZero();
    }

    @Test
    void shouldMatchLocationCaseInsensitivelyAndLetRemoteCandidatesIn() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "local", 2, "  PUNE ", 90, false, "java");
        cand(m, "elsewhere", 2, "Delhi", 90, false, "java");
        cand(m, "remote", 2, "Delhi", 90, true, "java");
        cand(m, "remoteLocal", 2, "Pune", 90, true, "java");

        assertThat(m.matchScore("J", "local")).isEqualTo(60);
        assertThat(m.matchScore("J", "elsewhere")).isZero();
        assertThat(m.matchScore("J", "remote")).isEqualTo(50);
        assertThat(m.matchScore("J", "remoteLocal")).isEqualTo(60);
    }

    @Test
    void shouldCompareSkillsCaseInsensitivelyAfterTrimming() {
        JobMatching m = svc();
        m.postJob("J", "t", s(" JAVA", "Spring Boot"), s("  KaFkA "), 0, "Pune", 100);
        cand(m, "c", 0, "Pune", 90, false, "java ", "spring boot", " kafka", "KAFKA");

        assertThat(m.matchScore("J", "c")).isEqualTo(50 + 10 + 0 + 10 + 0);
    }

    @Test
    void shouldCapTheNiceToHaveTermAtFiveSkills() {
        JobMatching m = svc();
        m.postJob("J", "t", s(), s("a", "b", "c", "d", "e", "f", "g"), 0, "Pune", 100);
        cand(m, "four", 0, "Pune", 90, false, "a", "b", "c", "d");
        cand(m, "five", 0, "Pune", 90, false, "a", "b", "c", "d", "e");
        cand(m, "seven", 0, "Pune", 90, false, "a", "b", "c", "d", "e", "f", "g");

        assertThat(m.matchScore("J", "four")).isEqualTo(50 + 40 + 10);
        assertThat(m.matchScore("J", "five")).isEqualTo(50 + 50 + 10);
        assertThat(m.matchScore("J", "seven")).isEqualTo(50 + 50 + 10);
    }

    @Test
    void shouldCapTheExperienceTermAtTwenty() {
        JobMatching m = svc();
        m.postJob("J", "t", s(), s(), 1, "Pune", 100);
        cand(m, "y1", 1, "Pune", 90, false);
        cand(m, "y4", 4, "Pune", 90, false);
        cand(m, "y11", 11, "Pune", 90, false);
        cand(m, "y12", 12, "Pune", 90, false);
        cand(m, "y40", 40, "Pune", 90, false);

        assertThat(m.matchScore("J", "y1")).isEqualTo(60);
        assertThat(m.matchScore("J", "y4")).isEqualTo(66);
        assertThat(m.matchScore("J", "y11")).isEqualTo(80);
        assertThat(m.matchScore("J", "y12")).isEqualTo(80);
        assertThat(m.matchScore("J", "y40")).isEqualTo(80);
    }

    @Test
    void shouldGiveTheSalaryBonusOnlyWhenTwiceTheExpectationFits() {
        JobMatching m = svc();
        m.postJob("J", "t", s(), s(), 0, "Pune", 100);
        cand(m, "half", 0, "Pune", 50, false);
        cand(m, "justOver", 0, "Pune", 51, false);
        cand(m, "free", 0, "Pune", 0, false);
        cand(m, "all", 0, "Pune", 100, false);

        assertThat(m.matchScore("J", "half")).isEqualTo(70);
        assertThat(m.matchScore("J", "justOver")).isEqualTo(60);
        assertThat(m.matchScore("J", "free")).isEqualTo(70);
        assertThat(m.matchScore("J", "all")).isEqualTo(60);
    }

    @Test
    void shouldCountASkillListedInBothSetsAsRequiredOnly() {
        JobMatching m = svc();
        m.postJob("J", "t", s("java"), s("JAVA", "go"), 0, "Pune", 100);
        cand(m, "javaOnly", 0, "Pune", 90, false, "java");
        cand(m, "javaAndGo", 0, "Pune", 90, false, "java", "go");

        assertThat(m.matchScore("J", "javaOnly")).isEqualTo(60);
        assertThat(m.matchScore("J", "javaAndGo")).isEqualTo(70);
    }

    @Test
    void shouldOpenAJobWithoutRequiredSkillsToCandidatesWithoutSkills() {
        JobMatching m = svc();
        m.postJob("J", "t", s(), s(), 0, "Pune", 100);
        cand(m, "bare", 0, "Pune", 0, false);

        assertThat(m.matchScore("J", "bare")).isEqualTo(70);
        assertThat(m.topCandidates("J", 5)).containsExactly("bare");
    }

    @Test
    void shouldReturnFewerThanKWhenFewAreEligibleAndRejectBadK() {
        JobMatching m = backendExample();

        assertThat(m.topCandidates("backend", 100)).hasSize(5);
        assertThatThrownBy(() -> m.topCandidates("backend", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.topJobs("alice", -1)).isInstanceOf(IllegalArgumentException.class);
        m.postJob("empty", "t", s("cobol"), s(), 0, "Pune", 1);
        assertThat(m.topCandidates("empty", 3)).isEmpty();
    }

    @Test
    void shouldRejectDuplicateIdsButKeepSeparateIdSpaces() {
        JobMatching m = svc();
        simpleJob(m, "x");
        cand(m, "x", 3, "Pune", 10, false, "java");

        assertThatThrownBy(() -> simpleJob(m, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> cand(m, "x", 1, "Pune", 1, false)).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.matchScore("x", "x")).isPositive();
        cand(m, "X", 3, "Pune", 10, false, "java");
        assertThat(m.topCandidates("x", 5)).containsExactly("X", "x");
    }

    @Test
    void shouldRejectUnknownIdsEverywhere() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c", 2, "Pune", 10, false, "java");

        assertThatThrownBy(() -> m.matchScore("nope", "c")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.matchScore("J", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.topCandidates("nope", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.topJobs("nope", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.apply("nope", "J")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.apply("c", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.withdraw("c", "nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.withdraw("nope", "J")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.applications("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.closeJob("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.updateCandidate("nope", s(), 1, "Pune", 1, false))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidInputWithoutCreatingAnything() {
        JobMatching m = svc();
        Class<IllegalArgumentException> bad = IllegalArgumentException.class;

        assertThatThrownBy(() -> m.postJob(" ", "t", s(), s(), 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob(null, "t", s(), s(), 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", " ", s(), s(), 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", null, s(), 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", s(), null, 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", s("java", " "), s(), 0, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", s(), s(), -1, "Pune", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", s(), s(), 0, "", 1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.postJob("J", "t", s(), s(), 0, "Pune", -1)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("", s(), 1, "Pune", 1, false)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("c", null, 1, "Pune", 1, false)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("c", s("a", null), 1, "Pune", 1, false)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("c", s(), -1, "Pune", 1, false)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("c", s(), 1, null, 1, false)).isInstanceOf(bad);
        assertThatThrownBy(() -> m.registerCandidate("c", s(), 1, "Pune", -5, false)).isInstanceOf(bad);

        // nothing was created by the failed calls: both ids are still free
        simpleJob(m, "J");
        cand(m, "c", 2, "Pune", 10, false, "java");
        assertThat(m.matchScore("J", "c")).isPositive();
    }

    @Test
    void shouldAcceptZeroYearsAndZeroSalary() {
        JobMatching m = svc();
        m.postJob("J", "t", s(), s(), 0, "Pune", 0);
        cand(m, "c", 0, "Pune", 0, false);

        assertThat(m.matchScore("J", "c")).isEqualTo(70);
    }

    @Test
    void shouldApplyInOrderAndRejectRepeatsAndIneligibleCandidates() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c3", 3, "Pune", 10, false, "java");
        cand(m, "c1", 3, "Pune", 10, false, "java");
        cand(m, "c2", 3, "Pune", 10, false, "java");
        cand(m, "nope", 3, "Pune", 10, false, "go");

        m.apply("c3", "J");
        m.apply("c1", "J");
        m.apply("c2", "J");

        assertThat(m.applications("J")).containsExactly("c3", "c1", "c2");
        assertThatThrownBy(() -> m.apply("c1", "J")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> m.apply("nope", "J")).isInstanceOf(IllegalStateException.class);
        assertThat(m.applications("J")).containsExactly("c3", "c1", "c2");
    }

    @Test
    void shouldReturnACopyOfTheApplications() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c", 3, "Pune", 10, false, "java");
        m.apply("c", "J");

        List<String> before = m.applications("J");
        try {
            before.add("intruder");
        } catch (UnsupportedOperationException ignored) {
            // an unmodifiable list is fine too
        }

        assertThat(m.applications("J")).containsExactly("c");
    }

    @Test
    void shouldWithdrawAndPutAReapplicationAtTheEnd() {
        JobMatching m = svc();
        simpleJob(m, "J");
        for (String c : List.of("a", "b", "c")) {
            cand(m, c, 3, "Pune", 10, false, "java");
            m.apply(c, "J");
        }

        m.withdraw("a", "J");
        assertThat(m.applications("J")).containsExactly("b", "c");
        assertThatThrownBy(() -> m.withdraw("a", "J")).isInstanceOf(IllegalStateException.class);
        m.apply("a", "J");

        assertThat(m.applications("J")).containsExactly("b", "c", "a");
    }

    @Test
    void shouldExcludeAClosedJobFromMatchingButKeepItsApplications() {
        JobMatching m = svc();
        simpleJob(m, "J");
        simpleJob(m, "K");
        cand(m, "c", 3, "Pune", 10, false, "java");
        cand(m, "d", 3, "Pune", 10, false, "java");
        m.apply("c", "J");

        m.closeJob("J");

        assertThat(m.topCandidates("J", 5)).isEmpty();
        assertThat(m.topJobs("c", 5)).containsExactly("K");
        assertThat(m.matchScore("J", "c")).isZero();
        assertThat(m.applications("J")).containsExactly("c");
        assertThatThrownBy(() -> m.apply("d", "J")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> m.closeJob("J")).isInstanceOf(IllegalStateException.class);
        m.withdraw("c", "J");
        assertThat(m.applications("J")).isEmpty();
    }

    @Test
    void shouldReflectCandidateUpdatesInTheNextQueries() {
        JobMatching m = svc();
        m.postJob("J", "t", s("java"), s("go"), 3, "Pune", 100);
        cand(m, "c", 1, "Delhi", 200, false, "python");
        assertThat(m.topCandidates("J", 5)).isEmpty();
        assertThat(m.topJobs("c", 5)).isEmpty();

        m.updateCandidate("c", s("Java", "GO"), 5, "PUNE", 40, false);

        assertThat(m.topCandidates("J", 5)).containsExactly("c");
        assertThat(m.topJobs("c", 5)).containsExactly("J");
        assertThat(m.matchScore("J", "c")).isEqualTo(50 + 10 + 4 + 10 + 10);

        m.updateCandidate("c", s("java"), 5, "Delhi", 40, true);

        assertThat(m.matchScore("J", "c")).isEqualTo(50 + 0 + 4 + 0 + 10);
    }

    @Test
    void shouldLeaveACandidateUntouchedWhenAnUpdateIsRejected() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c", 3, "Pune", 10, false, "java");

        assertThatThrownBy(() -> m.updateCandidate("c", s("go", " "), 9, "Pune", 10, false))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.updateCandidate("c", s("go"), -1, "Pune", 10, false))
                .isInstanceOf(IllegalArgumentException.class);

        assertThat(m.matchScore("J", "c")).isEqualTo(50 + 0 + 2 + 10 + 10);
    }

    @Test
    void shouldLetAnIneligibleCandidateKeepAnEarlierApplicationButNotApplyAgain() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c", 3, "Pune", 10, false, "java");
        m.apply("c", "J");

        m.updateCandidate("c", s("java"), 1, "Pune", 10, false);

        assertThat(m.applications("J")).containsExactly("c");
        m.withdraw("c", "J");
        assertThatThrownBy(() -> m.apply("c", "J")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldSeeNewJobsAndCandidatesInTheNextRanking() {
        JobMatching m = svc();
        cand(m, "c", 3, "Pune", 10, false, "java");
        assertThat(m.topJobs("c", 5)).isEmpty();

        simpleJob(m, "J");
        assertThat(m.topJobs("c", 5)).containsExactly("J");
        assertThat(m.topCandidates("J", 5)).containsExactly("c");
        cand(m, "d", 9, "Pune", 10, false, "java");
        assertThat(m.topCandidates("J", 5)).containsExactly("d", "c");
    }

    @Test
    void shouldLetExactlyOneOfManyIdenticalAppliesSucceed() {
        JobMatching m = svc();
        simpleJob(m, "J");
        cand(m, "c", 3, "Pune", 10, false, "java");
        AtomicInteger ok = new AtomicInteger();

        Concurrently.run(16, t -> {
            try {
                m.apply("c", "J");
                ok.incrementAndGet();
            } catch (IllegalStateException expected) {
                // lost the race
            }
        });

        assertThat(ok.get()).isEqualTo(1);
        assertThat(m.applications("J")).containsExactly("c");
    }

    @Test
    void shouldLetExactlyOneOfManyIdenticalRegistrationsSucceed() {
        JobMatching m = svc();
        AtomicInteger ok = new AtomicInteger();

        Concurrently.run(16, t -> {
            try {
                m.registerCandidate("same", s("java"), t, "Pune", 10, false);
                ok.incrementAndGet();
            } catch (IllegalArgumentException expected) {
                // lost the race
            }
        });

        assertThat(ok.get()).isEqualTo(1);
    }

    @Test
    void shouldKeepEveryApplicationWhenManyCandidatesApplyAtOnce() {
        JobMatching m = svc();
        simpleJob(m, "J");
        for (int i = 0; i < 400; i++) {
            cand(m, "c" + i, 3, "Pune", 10, false, "java");
        }

        Concurrently.run(8, t -> {
            for (int i = t; i < 400; i += 8) {
                m.apply("c" + i, "J");
            }
        });

        List<String> apps = m.applications("J");
        assertThat(apps).hasSize(400);
        assertThat(new HashSet<>(apps)).hasSize(400);
    }

    @Test
    void shouldRankFiveThousandCandidatesAgainstFiveHundredJobsQuickly() {
        JobMatching m = svc();
        String[] skills = {"java", "sql", "go", "rust", "kafka", "redis", "docker", "k8s", "aws", "react"};
        String[] cities = {"Pune", "Delhi", "Mumbai", "Goa"};
        for (int j = 0; j < 500; j++) {
            m.postJob("job" + j, "t" + j, s(skills[j % 10], skills[(j * 3 + 1) % 10]),
                    s(skills[(j + 2) % 10], skills[(j + 5) % 10]), j % 8, cities[j % 4], 50 + (j % 50));
        }
        for (int c = 0; c < 5_000; c++) {
            m.registerCandidate("cand" + c, s(skills[c % 10], skills[(c / 10) % 10], skills[(c / 100) % 10],
                    skills[(c * 7) % 10]), c % 12, cities[(c / 3) % 4], 10 + (c % 60), c % 5 == 0);
        }

        long start = System.nanoTime();
        int nonEmpty = 0;
        for (int i = 0; i < 1_000; i++) {
            List<String> top = m.topCandidates("job" + (i % 500), 10);
            assertThat(top.size()).isLessThanOrEqualTo(10);
            if (!top.isEmpty()) {
                nonEmpty++;
            }
        }
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(nonEmpty).isGreaterThan(900);
        assertThat(ms).as("1,000 topCandidates calls took %d ms", ms).isLessThan(10_000);
    }
}

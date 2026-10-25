package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.CityMap;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static CityMap map(String... names) {
        CityMap m = Candidate.of(Solution::create);
        for (String n : names) {
            m.addLocation(n);
        }
        return m;
    }

    /** A-B 4, A-C 1, C-B 2, B-D 5, C-D 8 ; E isolated. */
    private static CityMap town() {
        CityMap m = map("A", "B", "C", "D", "E");
        m.addRoad("A", "B", 4);
        m.addRoad("A", "C", 1);
        m.addRoad("C", "B", 2);
        m.addRoad("B", "D", 5);
        m.addRoad("C", "D", 8);
        return m;
    }

    @Test
    void shouldRejectBadLocationNames() {
        CityMap m = map("A");
        assertThatThrownBy(() -> m.addLocation("A")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addLocation(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addLocation("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addLocation(null)).isInstanceOf(IllegalArgumentException.class);
        m.addLocation("a"); // case-sensitive: a different location
        assertThat(m.withinRadius("a", 10)).containsExactly("a");
    }

    @Test
    void shouldRejectBadRoads() {
        CityMap m = map("A", "B");
        assertThatThrownBy(() -> m.addRoad("A", "Z", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addRoad("Z", "A", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addRoad("A", "A", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addRoad("A", "B", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addRoad("A", "B", -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.addRoad(null, "B", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThat(m.shortestDistance("A", "B")).isEqualTo(-1); // nothing was added by the failed calls
    }

    @Test
    void shouldMeasureDistanceToItselfAsZero() {
        CityMap m = town();
        assertThat(m.shortestDistance("E", "E")).isZero();
        assertThat(m.shortestDistance("A", "A")).isZero();
    }

    @Test
    void shouldFindTheShortestDistanceNotTheFewestRoads() {
        CityMap m = town();
        assertThat(m.shortestDistance("A", "D")).isEqualTo(8); // A-C-B-D, not A-B-D (9) or A-C-D (9)
        assertThat(m.shortestDistance("D", "A")).isEqualTo(8); // roads are undirected
        assertThat(m.shortestDistance("A", "B")).isEqualTo(3);
    }

    @Test
    void shouldReportUnreachableAsMinusOneAndEmptyPath() {
        CityMap m = town();
        assertThat(m.shortestDistance("A", "E")).isEqualTo(-1);
        assertThat(m.shortestDistance("E", "A")).isEqualTo(-1);
        assertThat(m.shortestPath("A", "E")).isEmpty();
    }

    @Test
    void shouldRejectUnknownLocationsInQueries() {
        CityMap m = town();
        assertThatThrownBy(() -> m.shortestDistance("A", "Z")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.shortestDistance("Z", "A")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.shortestPath("A", "Z")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.nearestLandmarks("Z", "cafe", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.withinRadius("Z", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.removeRoad("A", "Z")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.tagLandmark("Z", "cafe")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.untagLandmark("Z", "cafe")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldUseTheShortestOfParallelRoads() {
        CityMap m = map("A", "B");
        m.addRoad("A", "B", 10);
        m.addRoad("B", "A", 7);
        m.addRoad("A", "B", 9);
        assertThat(m.shortestDistance("A", "B")).isEqualTo(7);
    }

    @Test
    void shouldRemoveEveryRoadBetweenAPairAndCountThem() {
        CityMap m = map("A", "B", "C");
        m.addRoad("A", "B", 10);
        m.addRoad("B", "A", 7);
        m.addRoad("B", "C", 1);

        assertThat(m.removeRoad("A", "B")).isEqualTo(2);
        assertThat(m.shortestDistance("A", "B")).isEqualTo(-1);
        assertThat(m.shortestDistance("B", "C")).isEqualTo(1);
        assertThat(m.removeRoad("B", "A")).isZero();
        assertThat(m.removeRoad("A", "C")).isZero();
    }

    @Test
    void shouldSeeRoadChangesImmediately() {
        CityMap m = town();
        assertThat(m.shortestDistance("A", "D")).isEqualTo(8);

        m.addRoad("A", "D", 3);
        assertThat(m.shortestDistance("A", "D")).isEqualTo(3);
        assertThat(m.shortestPath("A", "D")).containsExactly("A", "D");

        m.removeRoad("A", "D");
        assertThat(m.shortestDistance("A", "D")).isEqualTo(8);

        m.removeRoad("C", "B");
        assertThat(m.shortestDistance("A", "D")).isEqualTo(9);

        m.addRoad("D", "E", 2);
        assertThat(m.shortestDistance("A", "E")).isEqualTo(11);
    }

    @Test
    void shouldHandleDistancesBeyondIntRange() {
        CityMap m = map("A", "B", "C", "D");
        m.addRoad("A", "B", 2_000_000_000L);
        m.addRoad("B", "C", 2_000_000_000L);
        m.addRoad("C", "D", 2_000_000_000L);
        assertThat(m.shortestDistance("A", "D")).isEqualTo(6_000_000_000L);
        assertThat(m.shortestPath("A", "D")).containsExactly("A", "B", "C", "D");
        assertThat(m.withinRadius("A", 5_999_999_999L)).containsExactly("A", "B", "C");
        assertThat(m.withinRadius("A", 6_000_000_000L)).containsExactly("A", "B", "C", "D");
    }

    @Test
    void shouldReturnTheRouteNamesInOrder() {
        CityMap m = town();
        assertThat(m.shortestPath("A", "D")).containsExactly("A", "C", "B", "D");
        assertThat(m.shortestPath("D", "A")).containsExactly("D", "B", "C", "A");
        assertThat(m.shortestPath("E", "E")).containsExactly("E");
    }

    @Test
    void shouldBreakEqualLengthRoutesBySmallestNameSequence() {
        CityMap m = map("S", "T", "X", "Y", "Z");
        m.addRoad("S", "Z", 1);
        m.addRoad("S", "Y", 1);
        m.addRoad("S", "X", 1);
        m.addRoad("Z", "T", 1);
        m.addRoad("Y", "T", 1);
        m.addRoad("X", "T", 1);
        assertThat(m.shortestPath("S", "T")).containsExactly("S", "X", "T");
    }

    @Test
    void shouldCompareTieRoutesElementByElementAndNeverTradeLengthForNames() {
        CityMap m = map("S", "T", "a", "b", "c", "d");
        // S-a-c-T and S-b-d-T both cost 3; S-a-T costs 5 and must lose despite having the smallest first name.
        m.addRoad("S", "a", 1);
        m.addRoad("a", "c", 1);
        m.addRoad("c", "T", 1);
        m.addRoad("S", "b", 1);
        m.addRoad("b", "d", 1);
        m.addRoad("d", "T", 1);
        m.addRoad("a", "T", 4);
        assertThat(m.shortestPath("S", "T")).containsExactly("S", "a", "c", "T");

        m.addRoad("a", "d", 1); // now S-a-d-T (3) ties too; "c" < "d" keeps S-a-c-T
        assertThat(m.shortestPath("S", "T")).containsExactly("S", "a", "c", "T");
        m.removeRoad("a", "c"); // S-a-d-T (3) and S-b-d-T (3) remain: "a" < "b"
        assertThat(m.shortestPath("S", "T")).containsExactly("S", "a", "d", "T");
    }

    @Test
    void shouldOrderNearestLandmarksByDistanceThenName() {
        CityMap m = map("S", "p", "q", "r", "s");
        m.addRoad("S", "p", 5);
        m.addRoad("S", "q", 3);
        m.addRoad("S", "r", 5);
        m.addRoad("S", "s", 9);
        for (String n : List.of("p", "q", "r", "s")) {
            m.tagLandmark(n, "cafe");
        }
        assertThat(m.nearestLandmarks("S", "cafe", 10)).containsExactly("q:3", "p:5", "r:5", "s:9");
        assertThat(m.nearestLandmarks("S", "cafe", 3)).containsExactly("q:3", "p:5", "r:5");
        assertThat(m.nearestLandmarks("S", "cafe", 1)).containsExactly("q:3");
    }

    @Test
    void shouldCountTheSourceItselfAtDistanceZero() {
        CityMap m = town();
        m.tagLandmark("A", "hospital");
        m.tagLandmark("D", "hospital");
        assertThat(m.nearestLandmarks("A", "hospital", 5)).containsExactly("A:0", "D:8");
        assertThat(m.nearestLandmarks("D", "hospital", 1)).containsExactly("D:0");
    }

    @Test
    void shouldSkipUnreachableLandmarksAndUnknownTypes() {
        CityMap m = town();
        m.tagLandmark("E", "cafe");
        assertThat(m.nearestLandmarks("A", "cafe", 3)).isEmpty();
        assertThat(m.nearestLandmarks("A", "museum", 3)).isEmpty();
        assertThat(m.nearestLandmarks("E", "cafe", 3)).containsExactly("E:0");
    }

    @Test
    void shouldTreatTypesAsCaseInsensitiveTrimmedAndAllowSeveralPerLocation() {
        CityMap m = town();
        m.tagLandmark("B", "  Cafe ");
        m.tagLandmark("B", "HOSPITAL");
        m.tagLandmark("B", "cafe"); // harmless repeat
        assertThat(m.nearestLandmarks("A", "CAFE", 5)).containsExactly("B:3");
        assertThat(m.nearestLandmarks("A", " hospital", 5)).containsExactly("B:3");

        assertThat(m.untagLandmark("B", "Cafe")).isTrue();
        assertThat(m.untagLandmark("B", "cafe")).isFalse();
        assertThat(m.nearestLandmarks("A", "cafe", 5)).isEmpty();
        assertThat(m.nearestLandmarks("A", "hospital", 5)).containsExactly("B:3");
    }

    @Test
    void shouldRejectBadKAndBlankTypes() {
        CityMap m = town();
        m.tagLandmark("B", "cafe");
        assertThatThrownBy(() -> m.nearestLandmarks("A", "cafe", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.nearestLandmarks("A", "cafe", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.nearestLandmarks("A", " ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.nearestLandmarks("A", null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.tagLandmark("B", "")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.untagLandmark("B", "  ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSeeTagAndRoadChangesInNearestLandmarksImmediately() {
        CityMap m = town();
        m.tagLandmark("D", "cafe");
        assertThat(m.nearestLandmarks("A", "cafe", 2)).containsExactly("D:8");

        m.tagLandmark("C", "cafe");
        assertThat(m.nearestLandmarks("A", "cafe", 2)).containsExactly("C:1", "D:8");

        m.removeRoad("A", "C");
        assertThat(m.nearestLandmarks("A", "cafe", 2)).containsExactly("C:6", "D:9");

        m.untagLandmark("C", "cafe");
        m.addRoad("A", "E", 1);
        m.tagLandmark("E", "cafe");
        assertThat(m.nearestLandmarks("A", "cafe", 2)).containsExactly("E:1", "D:9");
    }

    @Test
    void shouldListLocationsWithinRadiusSortedByNameWithBoundaryIncluded() {
        CityMap m = town();
        assertThat(m.withinRadius("A", 0)).containsExactly("A");
        assertThat(m.withinRadius("A", 2)).containsExactly("A", "C");
        assertThat(m.withinRadius("A", 3)).containsExactly("A", "B", "C");
        assertThat(m.withinRadius("A", 100)).containsExactly("A", "B", "C", "D");
        assertThat(m.withinRadius("E", 100)).containsExactly("E");
        assertThatThrownBy(() -> m.withinRadius("A", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldAnswerThreeThousandQueriesOnA150By150Grid() {
        int n = 150;
        CityMap m = Candidate.of(Solution::create);
        for (int r = 0; r < n; r++) {
            for (int c = 0; c < n; c++) {
                m.addLocation(r + "," + c);
            }
        }
        for (int r = 0; r < n; r++) {
            for (int c = 0; c < n; c++) {
                if (c + 1 < n) {
                    m.addRoad(r + "," + c, r + "," + (c + 1), 7);
                }
                if (r + 1 < n) {
                    m.addRoad(r + "," + c, (r + 1) + "," + c, 7);
                }
            }
        }
        long start = System.nanoTime();
        long seed = 12345;
        for (int q = 0; q < 3_000; q++) {
            seed = seed * 6364136223846793005L + 1442695040888963407L;
            int r1 = (int) ((seed >>> 33) % n);
            int c1 = (int) ((seed >>> 20) % n);
            int r2 = (int) ((seed >>> 45) % n);
            int c2 = (int) ((seed >>> 11) % n);
            long expected = 7L * (Math.abs(r1 - r2) + Math.abs(c1 - c2));
            assertThat(m.shortestDistance(r1 + "," + c1, r2 + "," + c2)).isEqualTo(expected);
        }
        assertThat((System.nanoTime() - start) / 1_000_000).as("elapsed ms").isLessThan(90_000);
    }

    @Test
    void shouldGiveConsistentAnswersToQueriesRunningDuringUpdates() {
        CityMap m = map("A", "B", "M");
        m.addRoad("A", "M", 2_500);
        m.addRoad("M", "B", 2_500);
        m.tagLandmark("B", "cafe");

        Concurrently.run(5, 60_000, t -> {
            if (t == 0) {
                for (long d = 1_000; d >= 1; d--) {
                    m.addRoad("A", "B", d);
                }
                return;
            }
            long last = Long.MAX_VALUE;
            for (int i = 0; i < 3_000; i++) {
                long d = m.shortestDistance("A", "B");
                assertThat(d).isBetween(1L, 5_000L);
                assertThat(d).as("only shorter roads are ever added").isLessThanOrEqualTo(last);
                last = d;
                List<String> path = m.shortestPath("A", "B");
                assertThat(path.get(0)).isEqualTo("A");
                assertThat(path.get(path.size() - 1)).isEqualTo("B");
                assertThat(m.nearestLandmarks("A", "cafe", 1)).hasSize(1);
            }
        });
        assertThat(m.shortestDistance("A", "B")).isEqualTo(1);
        assertThat(m.shortestPath("A", "B")).containsExactly("A", "B");
    }

    @Test
    void shouldNotLoseConcurrentLocationsRoadsAndTags() {
        CityMap m = map("hub");
        int threads = 8;
        int per = 200;
        Concurrently.run(threads, 60_000, t -> {
            for (int i = 0; i < per; i++) {
                String name = "n" + t + "_" + i;
                m.addLocation(name);
                m.addRoad("hub", name, i + 1);
                m.tagLandmark(name, i % 2 == 0 ? "cafe" : "park");
                m.shortestDistance("hub", name);
            }
        });
        assertThat(m.withinRadius("hub", 1_000_000)).hasSize(threads * per + 1);
        assertThat(m.nearestLandmarks("hub", "cafe", 1_000_000)).hasSize(threads * per / 2);
        assertThat(m.shortestDistance("n3_7", "n5_9")).isEqualTo(8 + 10);
    }
}

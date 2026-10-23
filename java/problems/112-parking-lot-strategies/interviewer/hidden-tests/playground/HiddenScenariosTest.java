package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ParkingLot;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final String TWO = "TWO_WHEELER";
    private static final String FOUR = "FOUR_WHEELER";
    private static final String LOWEST = "LOWEST_INDEX_SPOT";
    private static final String MOST_FREE = "MOST_FREE_FLOOR";

    private static String park(ParkingLot l, String type, String reg) {
        return l.park(type, reg).orElseThrow(() -> new AssertionError("expected a spot for " + reg));
    }

    @Test
    void shouldFollowTheMostFreeFloorRuleThroughAFullFillAndPartialDrain() {
        // floors with 2, 4, 3 two-wheeler spots: global 0-1 | 2-5 | 6-8
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {2, 4, 3}, new int[] {0, 0, 0}, MOST_FREE));
        String[] expected = {"1:2", "1:3", "2:6", "0:0", "1:4", "2:7", "0:1", "1:5", "2:8"};
        String[] tickets = new String[expected.length];
        for (int i = 0; i < expected.length; i++) {
            tickets[i] = park(l, TWO, "v" + i);
            assertThat(l.findByTicket(tickets[i])).as("vehicle %d", i).contains(expected[i]);
        }
        assertThat(l.park(TWO, "extra")).isEmpty();

        // free 1:2, 2:6 and 0:1: each floor has exactly 1 free -> floor 0 first, then 1, then 2
        l.unpark(tickets[0]);
        l.unpark(tickets[2]);
        l.unpark(tickets[6]);
        assertThat(l.findByTicket(park(l, TWO, "n1"))).contains("0:1");
        assertThat(l.findByTicket(park(l, TWO, "n2"))).contains("1:2");
        assertThat(l.findByTicket(park(l, TWO, "n3"))).contains("2:6");
    }

    @Test
    void shouldFillHolesInGlobalOrderAfterSwitchingBackToLowestIndex() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {3, 3}, new int[] {0, 0}, MOST_FREE));
        List<String> t = new ArrayList<>();
        for (int i = 0; i < 6; i++) {
            t.add(park(l, TWO, "v" + i)); // alternates floors: 0:0 1:3 0:1 1:4 0:2 1:5
        }
        assertThat(l.findByTicket(t.get(1))).contains("1:3");
        l.unpark(t.get(5)); // 1:5
        l.unpark(t.get(2)); // 0:1
        l.unpark(t.get(3)); // 1:4

        l.setStrategy(LOWEST);

        assertThat(l.findByTicket(park(l, TWO, "a"))).contains("0:1");
        assertThat(l.findByTicket(park(l, TWO, "b"))).contains("1:4");
        assertThat(l.findByTicket(park(l, TWO, "c"))).contains("1:5");
    }

    @Test
    void shouldSurviveAnOverFullLotThenFullyEmptyIt() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {2, 2}, new int[] {2, 2}, LOWEST));
        List<String> tickets = new ArrayList<>();
        for (int i = 0; i < 20; i++) {
            Optional<String> t = l.park(i % 2 == 0 ? TWO : FOUR, "r" + i);
            t.ifPresent(tickets::add);
        }
        assertThat(tickets).hasSize(8);

        tickets.forEach(t -> assertThat(l.unpark(t)).isTrue());

        assertThat(l.freeSpots(TWO)).isEqualTo(4);
        assertThat(l.freeSpots(FOUR)).isEqualTo(4);
        // "r2" was turned away earlier (lot full for two-wheelers) and must be accepted now
        assertThat(l.findByRegistration("r2")).isEmpty();
        assertThat(l.park(TWO, "r2")).isPresent();
    }

    @Test
    void shouldMatchABruteForceModelOverARandomWorkloadForBothStrategies() {
        int[] two = {3, 0, 4, 2};
        int[] four = {1, 3, 0, 2};
        for (String strategy : List.of(LOWEST, MOST_FREE)) {
            ParkingLot l = Candidate.of(() -> Solution.create(two, four, strategy));
            Model m = new Model(two, four, strategy);
            long seed = 12345;
            Map<String, String> active = new HashMap<>(); // reg -> ticket
            for (int step = 0; step < 4_000; step++) {
                seed = seed * 6364136223846793005L + 1442695040888963407L;
                int r = (int) ((seed >>> 33) & 0x7fffffff);
                if (r % 5 == 0 && !active.isEmpty()) {
                    String reg = new ArrayList<>(new java.util.TreeSet<>(active.keySet())).get((r / 5) % active.size());
                    assertThat(l.unpark(active.remove(reg))).isTrue();
                    m.leave(reg);
                } else if (r % 97 == 0) {
                    String s = (r / 97) % 2 == 0 ? LOWEST : MOST_FREE;
                    l.setStrategy(s);
                    m.strategy = s;
                } else {
                    String type = (r / 3) % 2 == 0 ? TWO : FOUR;
                    String reg = "R" + (r % 40);
                    if (active.containsKey(reg)) {
                        assertThatThrownBy(() -> l.park(type, reg)).isInstanceOf(IllegalStateException.class);
                        continue;
                    }
                    Optional<String> ticket = l.park(type, reg);
                    String expect = m.park(type, reg);
                    assertThat(ticket.isPresent()).as("step %d", step).isEqualTo(expect != null);
                    if (ticket.isPresent()) {
                        assertThat(l.findByTicket(ticket.get())).as("step %d", step).contains(expect);
                        assertThat(l.findByRegistration(reg)).contains(expect);
                        active.put(reg, ticket.get());
                    }
                }
                if (step % 50 == 0) {
                    assertThat(l.freeSpots(TWO)).isEqualTo(m.free(0));
                    assertThat(l.freeSpots(FOUR)).isEqualTo(m.free(1));
                    for (int f = 0; f < two.length; f++) {
                        assertThat(l.freeSpotsOnFloor(f, TWO)).isEqualTo(m.freeOn(0, f));
                        assertThat(l.freeSpotsOnFloor(f, FOUR)).isEqualTo(m.freeOn(1, f));
                    }
                }
            }
        }
    }

    @Test
    void shouldHandleALargeLotQuicklyUnderBothStrategies() {
        int floors = 60;
        int[] two = new int[floors];
        int[] four = new int[floors];
        java.util.Arrays.fill(two, 100);
        java.util.Arrays.fill(four, 100);
        for (String strategy : List.of(LOWEST, MOST_FREE)) {
            ParkingLot l = Candidate.of(() -> Solution.create(two, four, strategy));
            org.junit.jupiter.api.Assertions.assertTimeoutPreemptively(Duration.ofSeconds(20), () -> {
                List<String> tickets = new ArrayList<>();
                for (int i = 0; i < floors * 100; i++) {
                    tickets.add(park(l, TWO, "t" + i));
                }
                assertThat(l.park(TWO, "overflow")).isEmpty();
                for (int i = 0; i < tickets.size(); i += 2) {
                    assertThat(l.unpark(tickets.get(i))).isTrue();
                }
                for (int i = 0; i < tickets.size() / 2; i++) {
                    park(l, TWO, "u" + i);
                }
                assertThat(l.freeSpots(TWO)).isZero();
                assertThat(l.freeSpots(FOUR)).isEqualTo(floors * 100);
            });
        }
    }

    @Test
    void shouldNeverShareOneSpotBetweenTwoLiveTicketsUnderChurn() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {3, 3, 3}, new int[] {2, 2, 2}, MOST_FREE));
        ConcurrentHashMap<String, String> live = new ConcurrentHashMap<>(); // spot -> reg
        AtomicInteger conflicts = new AtomicInteger();

        Concurrently.run(12, 60_000, t -> {
            for (int i = 0; i < 2_000; i++) {
                String reg = "t" + t + "-" + i;
                String type = i % 3 == 0 ? FOUR : TWO;
                Optional<String> ticket = l.park(type, reg);
                if (ticket.isPresent()) {
                    String spot = l.findByTicket(ticket.get()).orElseThrow();
                    if (live.putIfAbsent(spot, reg) != null) {
                        conflicts.incrementAndGet();
                    }
                    live.remove(spot, reg);
                    l.unpark(ticket.get());
                }
            }
        });

        assertThat(conflicts.get()).isZero();
        assertThat(l.freeSpots(TWO)).isEqualTo(9);
        assertThat(l.freeSpots(FOUR)).isEqualTo(6);
    }

    @Test
    void shouldAllowStrategySwitchesWhileThreadsPark() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {20, 20}, new int[] {0, 0}, LOWEST));

        List<Integer> parked = Concurrently.collect(9, 60_000, t -> {
            if (t == 8) {
                for (int i = 0; i < 2_000; i++) {
                    l.setStrategy(i % 2 == 0 ? MOST_FREE : LOWEST);
                }
                return 0;
            }
            int ok = 0;
            for (int i = 0; i < 10; i++) {
                if (l.park(TWO, "p" + t + "-" + i).isPresent()) {
                    ok++;
                }
            }
            return ok;
        });

        assertThat(parked.stream().mapToInt(Integer::intValue).sum()).isEqualTo(40 - l.freeSpots(TWO));
        assertThat(l.freeSpots(TWO)).isZero().isEqualTo(0);
    }

    @Test
    void shouldKeepGoodStateAfterEveryRejectedCall() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {1}, new int[] {1}, LOWEST));
        String t = park(l, TWO, "A");

        assertThatThrownBy(() -> l.park(TWO, "A")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> l.park("BUS", "B")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.park(FOUR, " ")).isInstanceOf(IllegalArgumentException.class);

        assertThat(l.freeSpots(FOUR)).isEqualTo(1);
        assertThat(l.findByTicket(t)).contains("0:0");
        assertThat(l.findByRegistration("B")).isEmpty();
        assertThat(park(l, FOUR, "B")).isNotEqualTo(t);
    }

    /** Brute-force reference used only to cross-check the implementation. */
    private static final class Model {
        final int floors;
        final TreeSet<Integer>[][] free;
        final Map<String, int[]> parked = new HashMap<>();
        String strategy;

        @SuppressWarnings("unchecked")
        Model(int[] two, int[] four, String strategy) {
            floors = two.length;
            this.strategy = strategy;
            free = new TreeSet[2][floors];
            int idx = 0;
            for (int f = 0; f < floors; f++) {
                free[0][f] = new TreeSet<>();
                free[1][f] = new TreeSet<>();
                for (int i = 0; i < two[f]; i++) {
                    free[0][f].add(idx++);
                }
                for (int i = 0; i < four[f]; i++) {
                    free[1][f].add(idx++);
                }
            }
        }

        String park(String type, String reg) {
            int t = type.equals(TWO) ? 0 : 1;
            int best = -1;
            for (int f = 0; f < floors; f++) {
                if (free[t][f].isEmpty()) {
                    continue;
                }
                if (best < 0) {
                    best = f;
                } else if (strategy.equals(LOWEST) ? free[t][f].first() < free[t][best].first()
                        : free[t][f].size() > free[t][best].size()) {
                    best = f;
                }
            }
            if (best < 0) {
                return null;
            }
            int spot = free[t][best].pollFirst();
            parked.put(reg, new int[] {t, best, spot});
            return best + ":" + spot;
        }

        void leave(String reg) {
            int[] p = parked.remove(reg);
            free[p[0]][p[1]].add(p[2]);
        }

        int free(int t) {
            int n = 0;
            for (int f = 0; f < floors; f++) {
                n += free[t][f].size();
            }
            return n;
        }

        int freeOn(int t, int f) {
            return free[t][f].size();
        }
    }
}

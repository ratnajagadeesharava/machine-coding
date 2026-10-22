package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ParkingLot;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String TWO = "TWO_WHEELER";
    private static final String FOUR = "FOUR_WHEELER";
    private static final String LOWEST = "LOWEST_INDEX_SPOT";
    private static final String MOST_FREE = "MOST_FREE_FLOOR";

    /**
     * Standard lot: floor 0 has 2 two-wheeler + 1 four-wheeler spots (global 0,1 | 2),
     * floor 1 has 1 two-wheeler + 2 four-wheeler spots (global 3 | 4,5).
     */
    private ParkingLot lot(String strategy) {
        return Candidate.of(() -> Solution.create(new int[] {2, 1}, new int[] {1, 2}, strategy));
    }

    private static String park(ParkingLot l, String type, String reg) {
        return l.park(type, reg).orElseThrow(() -> new AssertionError("expected a spot for " + reg));
    }

    @Test
    void shouldParkTwoWheelersInTheLowestIndexedSpotsFirst() {
        ParkingLot l = lot(LOWEST);

        assertThat(l.findByTicket(park(l, TWO, "A"))).contains("0:0");
        assertThat(l.findByTicket(park(l, TWO, "B"))).contains("0:1");
        assertThat(l.findByTicket(park(l, TWO, "C"))).contains("1:3");
    }

    @Test
    void shouldParkFourWheelersInTheLowestIndexedFourWheelerSpotsFirst() {
        ParkingLot l = lot(LOWEST);

        assertThat(l.findByTicket(park(l, FOUR, "A"))).contains("0:2");
        assertThat(l.findByTicket(park(l, FOUR, "B"))).contains("1:4");
        assertThat(l.findByTicket(park(l, FOUR, "C"))).contains("1:5");
    }

    @Test
    void shouldNeverPutAVehicleInASpotOfTheOtherType() {
        ParkingLot l = lot(LOWEST);
        park(l, TWO, "A");
        park(l, TWO, "B");
        park(l, TWO, "C");

        assertThat(l.park(TWO, "D")).isEmpty();
        assertThat(l.freeSpots(FOUR)).isEqualTo(3);
        assertThat(l.park(FOUR, "E")).isPresent();
    }

    @Test
    void shouldReturnEmptyWhenTheLotIsFullAndChangeNothing() {
        ParkingLot l = lot(LOWEST);
        for (int i = 0; i < 3; i++) {
            park(l, FOUR, "F" + i);
        }

        assertThat(l.park(FOUR, "overflow")).isEmpty();
        assertThat(l.findByRegistration("overflow")).isEmpty();
        assertThat(l.freeSpots(FOUR)).isZero();
        assertThat(l.freeSpots(TWO)).isEqualTo(3);
    }

    @Test
    void shouldAllowAVehicleThatWasTurnedAwayToParkOnceASpotIsFree() {
        ParkingLot l = lot(LOWEST);
        String t1 = park(l, FOUR, "F1");
        park(l, FOUR, "F2");
        park(l, FOUR, "F3");
        assertThat(l.park(FOUR, "late")).isEmpty();

        l.unpark(t1);

        assertThat(l.park(FOUR, "late")).isPresent();
    }

    @Test
    void shouldRejectTheSameRegistrationWhileItIsParked() {
        ParkingLot l = lot(LOWEST);
        park(l, TWO, "KA-01");

        assertThatThrownBy(() -> l.park(TWO, "KA-01")).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> l.park(FOUR, "KA-01")).isInstanceOf(IllegalStateException.class);
        assertThat(l.freeSpots(TWO)).isEqualTo(2);
        assertThat(l.freeSpots(FOUR)).isEqualTo(3);
    }

    @Test
    void shouldRejectADuplicateRegistrationEvenWhenTheLotIsFull() {
        ParkingLot l = lot(LOWEST);
        park(l, FOUR, "F1");
        park(l, FOUR, "F2");
        park(l, FOUR, "F3");

        assertThatThrownBy(() -> l.park(FOUR, "F1")).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldAcceptTheSameRegistrationAgainAfterItLeft() {
        ParkingLot l = lot(LOWEST);
        String first = park(l, TWO, "KA-01");
        assertThat(l.unpark(first)).isTrue();

        String second = park(l, TWO, "KA-01");

        assertThat(second).isNotEqualTo(first);
        assertThat(l.findByRegistration("KA-01")).contains("0:0");
    }

    @Test
    void shouldTreatRegistrationNumbersAsExactCaseSensitiveText() {
        ParkingLot l = lot(LOWEST);
        park(l, TWO, "ka-01");

        assertThat(l.park(TWO, "KA-01")).isPresent();
        assertThat(l.findByRegistration("Ka-01")).isEmpty();
    }

    @Test
    void shouldUnparkATicketOnlyOnce() {
        ParkingLot l = lot(LOWEST);
        String t = park(l, TWO, "A");

        assertThat(l.unpark(t)).isTrue();
        assertThat(l.unpark(t)).isFalse();
        assertThat(l.freeSpots(TWO)).isEqualTo(3);
    }

    @Test
    void shouldReturnFalseForAnUnknownTicket() {
        ParkingLot l = lot(LOWEST);
        park(l, TWO, "A");

        assertThat(l.unpark("no-such-ticket")).isFalse();
        assertThat(l.freeSpots(TWO)).isEqualTo(2);
    }

    @Test
    void shouldHandOutTheFreedSpotToTheNextVehicle() {
        ParkingLot l = lot(LOWEST);
        String a = park(l, TWO, "A");
        park(l, TWO, "B");
        park(l, TWO, "C");
        l.unpark(a);

        assertThat(l.findByTicket(park(l, TWO, "D"))).contains("0:0");
    }

    @Test
    void shouldFindVehiclesByRegistrationAndByTicket() {
        ParkingLot l = lot(LOWEST);
        String t = park(l, FOUR, "KA-09");

        assertThat(l.findByRegistration("KA-09")).contains("0:2");
        assertThat(l.findByTicket(t)).contains("0:2");
        assertThat(l.findByRegistration("other")).isEmpty();
        assertThat(l.findByTicket("other")).isEmpty();
    }

    @Test
    void shouldNotFindAVehicleAfterItWasUnparked() {
        ParkingLot l = lot(LOWEST);
        String t = park(l, FOUR, "KA-09");
        l.unpark(t);

        assertThat(l.findByRegistration("KA-09")).isEmpty();
        assertThat(l.findByTicket(t)).isEmpty();
    }

    @Test
    void shouldReportFreeSpotsInTotalAndPerFloor() {
        ParkingLot l = lot(LOWEST);
        assertThat(l.freeSpots(TWO)).isEqualTo(3);
        assertThat(l.freeSpots(FOUR)).isEqualTo(3);
        assertThat(l.freeSpotsOnFloor(0, TWO)).isEqualTo(2);
        assertThat(l.freeSpotsOnFloor(0, FOUR)).isEqualTo(1);
        assertThat(l.freeSpotsOnFloor(1, TWO)).isEqualTo(1);
        assertThat(l.freeSpotsOnFloor(1, FOUR)).isEqualTo(2);

        String a = park(l, TWO, "A"); // 0:0
        park(l, FOUR, "B"); // 0:2
        park(l, FOUR, "C"); // 1:4

        assertThat(l.freeSpots(TWO)).isEqualTo(2);
        assertThat(l.freeSpots(FOUR)).isEqualTo(1);
        assertThat(l.freeSpotsOnFloor(0, TWO)).isEqualTo(1);
        assertThat(l.freeSpotsOnFloor(0, FOUR)).isZero();
        assertThat(l.freeSpotsOnFloor(1, FOUR)).isEqualTo(1);

        l.unpark(a);
        assertThat(l.freeSpotsOnFloor(0, TWO)).isEqualTo(2);
        assertThat(l.freeSpots(TWO)).isEqualTo(3);
    }

    @Test
    void shouldNumberSpotsGloballyFloorByFloorWithTwoWheelersFirstOnEachFloor() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {0, 3, 1}, new int[] {2, 0, 2}, LOWEST));

        assertThat(l.findByTicket(park(l, FOUR, "a"))).contains("0:0");
        assertThat(l.findByTicket(park(l, FOUR, "b"))).contains("0:1");
        assertThat(l.findByTicket(park(l, TWO, "c"))).contains("1:2");
        assertThat(l.findByTicket(park(l, FOUR, "d"))).contains("2:6");
        assertThat(l.findByTicket(park(l, TWO, "e"))).contains("1:3");
        assertThat(l.findByTicket(park(l, TWO, "f"))).contains("1:4");
        assertThat(l.findByTicket(park(l, TWO, "g"))).contains("2:5");
        assertThat(l.findByTicket(park(l, FOUR, "h"))).contains("2:7");
    }

    @Test
    void shouldPreferTheFloorWithTheMostFreeSpotsForTheVehicleType() {
        ParkingLot l = lot(MOST_FREE);

        assertThat(l.findByTicket(park(l, FOUR, "A"))).contains("1:4"); // floor 1 has 2 free, floor 0 has 1
        assertThat(l.findByTicket(park(l, FOUR, "B"))).contains("0:2"); // tie 1-1: lowest floor
        assertThat(l.findByTicket(park(l, FOUR, "C"))).contains("1:5");
        assertThat(l.park(FOUR, "D")).isEmpty();
    }

    @Test
    void shouldBreakFloorTiesByLowestFloorThenLowestSpot() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {2, 2, 2}, new int[] {0, 0, 0}, MOST_FREE));

        assertThat(l.findByTicket(park(l, TWO, "a"))).contains("0:0");
        assertThat(l.findByTicket(park(l, TWO, "b"))).contains("1:2");
        assertThat(l.findByTicket(park(l, TWO, "c"))).contains("2:4");
        assertThat(l.findByTicket(park(l, TWO, "d"))).contains("0:1");
        assertThat(l.findByTicket(park(l, TWO, "e"))).contains("1:3");
        assertThat(l.findByTicket(park(l, TWO, "f"))).contains("2:5");
    }

    @Test
    void shouldCountOnlyFreeSpotsOfTheVehicleTypeWhenChoosingTheFloor() {
        // floor 0: 1 two + 5 four, floor 1: 3 two + 1 four
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {1, 3}, new int[] {5, 1}, MOST_FREE));

        assertThat(l.findByTicket(park(l, TWO, "t"))).contains("1:6"); // floor 1 has most free two-wheeler spots
        assertThat(l.findByTicket(park(l, FOUR, "f"))).contains("0:1"); // floor 0 has most free four-wheeler spots
    }

    @Test
    void shouldReuseAFreedSpotUnderMostFreeFloorWhenItsFloorIsTheEmptiest() {
        ParkingLot l = lot(MOST_FREE);
        String a = park(l, TWO, "A"); // floor 0 (2 free vs 1) -> 0:0
        park(l, TWO, "B"); // tie 1-1 -> 0:1
        park(l, TWO, "C"); // floor 1 -> 1:3
        l.unpark(a); // floor 0 has 1 free, floor 1 has 0

        assertThat(l.findByTicket(park(l, TWO, "D"))).contains("0:0");
    }

    @Test
    void shouldApplyANewStrategyOnlyToLaterParksAndLeaveParkedVehiclesAlone() {
        ParkingLot l = lot(LOWEST);
        String a = park(l, FOUR, "A"); // 0:2
        assertThat(l.findByTicket(a)).contains("0:2");

        l.setStrategy(MOST_FREE);

        assertThat(l.findByTicket(a)).contains("0:2");
        assertThat(l.findByTicket(park(l, FOUR, "B"))).contains("1:4"); // floor 0 has 0 free, floor 1 has 2
        l.setStrategy(LOWEST);
        assertThat(l.findByTicket(park(l, FOUR, "C"))).contains("1:5");
    }

    @Test
    void shouldGiveEveryTicketAUniqueIdThatIsNeverReused() {
        ParkingLot l = lot(LOWEST);
        Set<String> seen = new HashSet<>();
        for (int i = 0; i < 200; i++) {
            String t = park(l, TWO, "R" + i);
            assertThat(t).isNotBlank();
            assertThat(seen.add(t)).as("ticket %s reused", t).isTrue();
            assertThat(l.unpark(t)).isTrue();
        }
    }

    @Test
    void shouldNotLetAnOldTicketReleaseTheSpotOfALaterVehicle() {
        ParkingLot l = lot(LOWEST);
        String old = park(l, TWO, "A");
        l.unpark(old);
        String fresh = park(l, TWO, "B"); // takes the same spot

        assertThat(l.unpark(old)).isFalse();
        assertThat(l.findByTicket(fresh)).contains("0:0");
        assertThat(l.findByRegistration("B")).contains("0:0");
    }

    @Test
    void shouldRejectInvalidInput() {
        ParkingLot l = lot(LOWEST);
        park(l, TWO, "A");

        assertThatThrownBy(() -> l.park("TRUCK", "X")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.park(null, "X")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.park(TWO, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.park(TWO, "  ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.unpark(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.unpark(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.findByRegistration("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.findByTicket(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.freeSpots("BUS")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.freeSpotsOnFloor(2, TWO)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.freeSpotsOnFloor(-1, TWO)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.freeSpotsOnFloor(0, "BUS")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> l.setStrategy("RANDOM")).isInstanceOf(IllegalArgumentException.class);
        assertThat(l.freeSpots(TWO)).isEqualTo(2); // nothing above changed the state
    }

    @Test
    void shouldKeepTheCurrentStrategyWhenAnUnknownOneIsRejected() {
        ParkingLot l = lot(MOST_FREE);

        assertThatThrownBy(() -> l.setStrategy("nope")).isInstanceOf(IllegalArgumentException.class);

        assertThat(l.findByTicket(park(l, FOUR, "A"))).contains("1:4"); // still most-free-floor
    }

    @Test
    void shouldRejectInvalidLotConfiguration() {
        lot(LOWEST); // skips while unimplemented
        assertThatThrownBy(() -> Solution.create(new int[] {1}, new int[] {1, 1}, LOWEST))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(new int[] {}, new int[] {}, LOWEST))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(new int[] {-1}, new int[] {1}, LOWEST))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(new int[] {1}, new int[] {1}, "RANDOM"))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldSupportALotWithNoSpotsOfOneType() {
        ParkingLot l = Candidate.of(() -> Solution.create(new int[] {0, 0}, new int[] {1, 1}, LOWEST));

        assertThat(l.freeSpots(TWO)).isZero();
        assertThat(l.park(TWO, "bike")).isEmpty();
        assertThat(l.freeSpots(FOUR)).isEqualTo(2);
    }

    @Test
    void shouldNeverDoubleBookASpotWhenManyThreadsParkAtOnce() {
        for (String strategy : List.of(LOWEST, MOST_FREE)) {
            ParkingLot l = Candidate.of(() -> Solution.create(new int[] {10, 10, 10}, new int[] {0, 0, 0}, strategy));

            List<Optional<String>> tickets = Concurrently.collect(64, t -> l.park(TWO, "V" + t));

            List<String> issued = new ArrayList<>();
            tickets.forEach(o -> o.ifPresent(issued::add));
            Set<String> spots = new HashSet<>();
            for (String id : issued) {
                spots.add(l.findByTicket(id).orElseThrow());
            }
            assertThat(issued).hasSize(30).doesNotHaveDuplicates();
            assertThat(spots).hasSize(30);
            assertThat(l.freeSpots(TWO)).isZero();
        }
    }

    @Test
    void shouldAdmitExactlyOneOfManyThreadsParkingTheSameRegistration() {
        ParkingLot l = lot(LOWEST);

        List<Boolean> parked = Concurrently.collect(16, t -> {
            try {
                return l.park(TWO, "SAME").isPresent();
            } catch (IllegalStateException e) {
                return false;
            }
        });

        assertThat(parked.stream().filter(b -> b).count()).isEqualTo(1);
        assertThat(l.freeSpots(TWO)).isEqualTo(2);
    }

    @Test
    void shouldKeepCountsConsistentWhileThreadsParkAndLeave() {
        ParkingLot l = lot(MOST_FREE);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 500; i++) {
                Optional<String> ticket = l.park(i % 2 == 0 ? TWO : FOUR, "t" + t + "-" + i);
                ticket.ifPresent(id -> assertThat(l.unpark(id)).isTrue());
            }
        });

        assertThat(l.freeSpots(TWO)).isEqualTo(3);
        assertThat(l.freeSpots(FOUR)).isEqualTo(3);
        assertThat(l.freeSpotsOnFloor(0, TWO) + l.freeSpotsOnFloor(1, TWO)).isEqualTo(3);
    }
}

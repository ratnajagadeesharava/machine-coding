package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LibraryApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class HiddenScenariosTest {
    private static final long HOUR = 3_600_000L;
    private static final long DAY = 24 * HOUR;

    private final FakeClock clock = new FakeClock(DAY);

    private LibraryApi library() {
        LibraryApi lib = Candidate.of(() -> Solution.create(clock));
        lib.addMember("alice");
        lib.addMember("bob");
        return lib;
    }

    @Test
    void shouldNotFineWithinTheFirstOverdueDay() {
        LibraryApi lib = library();
        lib.addBook("1", "T", 1);
        String loan = lib.checkout("alice", "1").orElseThrow();
        clock.advance(14 * DAY + 23 * HOUR + 59 * 60_000L);
        assertThat(lib.returnLoan(loan)).hasValue(0L);
    }

    @Test
    void shouldFineExactlyOneDayLate() {
        LibraryApi lib = library();
        lib.addBook("1", "T", 1);
        String loan = lib.checkout("alice", "1").orElseThrow();
        clock.advance(15 * DAY);
        assertThat(lib.returnLoan(loan)).hasValue(50L);
    }

    @Test
    void addingCopiesShouldMakeTitleAvailableAgain() {
        LibraryApi lib = library();
        lib.addBook("1", "T", 1);
        lib.checkout("alice", "1");
        assertThat(lib.checkout("bob", "1")).isEmpty();
        lib.addBook("1", "T", 1);
        assertThat(lib.availableCopies("1")).isEqualTo(1);
        assertThat(lib.checkout("bob", "1")).isPresent();
    }

    @Test
    void shouldRefuseReservationByUnknownMemberOrForUnknownBook() {
        LibraryApi lib = library();
        lib.addBook("1", "T", 1);
        lib.checkout("alice", "1");
        assertThat(lib.reserve("ghost", "1")).isFalse();
        assertThat(lib.reserve("bob", "404")).isFalse();
    }

    @Test
    void multipleHeldCopiesShouldServeTheFirstSeveralReservers() {
        LibraryApi lib = library();
        lib.addMember("carol");
        lib.addMember("dave");
        lib.addBook("1", "T", 2);
        String a = lib.checkout("alice", "1").orElseThrow();
        String b = lib.checkout("bob", "1").orElseThrow();
        lib.reserve("carol", "1");
        lib.reserve("dave", "1");
        lib.returnLoan(a);
        lib.returnLoan(b);
        assertThat(lib.checkout("dave", "1")).isPresent();
        assertThat(lib.checkout("carol", "1")).isPresent();
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.LibraryApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.FakeClock;

class BehaviorTest {
    private static final long HOUR = 3_600_000L;
    private static final long DAY = 24 * HOUR;

    private final FakeClock clock = new FakeClock(10 * DAY);

    private LibraryApi library() {
        LibraryApi lib = Candidate.of(() -> Solution.create(clock));
        lib.addMember("alice");
        lib.addMember("bob");
        lib.addMember("carol");
        return lib;
    }

    @Test
    void shouldCheckOutAvailableBook() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 2);

        assertThat(lib.checkout("alice", "111")).isPresent();
        assertThat(lib.availableCopies("111")).isEqualTo(1);
    }

    @Test
    void shouldRefuseUnknownMemberOrBook() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);

        assertThat(lib.checkout("nobody", "111")).isEmpty();
        assertThat(lib.checkout("alice", "999")).isEmpty();
        assertThat(lib.availableCopies("111")).isEqualTo(1);
        assertThat(lib.availableCopies("999")).isZero();
    }

    @Test
    void shouldRefuseWhenNoCopiesAvailable() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        lib.checkout("alice", "111");

        assertThat(lib.checkout("bob", "111")).isEmpty();
    }

    @Test
    void shouldLimitMemberToThreeActiveLoans() {
        LibraryApi lib = library();
        for (int i = 1; i <= 4; i++) {
            lib.addBook("B" + i, "Book " + i, 1);
        }

        assertThat(lib.checkout("alice", "B1")).isPresent();
        assertThat(lib.checkout("alice", "B2")).isPresent();
        assertThat(lib.checkout("alice", "B3")).isPresent();
        assertThat(lib.checkout("alice", "B4")).isEmpty();
        assertThat(lib.availableCopies("B4")).isEqualTo(1);
    }

    @Test
    void shouldNotAllowSameMemberToBorrowSameBookTwice() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 3);
        lib.checkout("alice", "111");

        assertThat(lib.checkout("alice", "111")).isEmpty();
        assertThat(lib.availableCopies("111")).isEqualTo(2);
    }

    @Test
    void shouldRestoreAvailabilityWhenBookIsReturned() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        String loan = lib.checkout("alice", "111").orElseThrow();

        assertThat(lib.returnLoan(loan)).isPresent();

        assertThat(lib.availableCopies("111")).isEqualTo(1);
        assertThat(lib.checkout("bob", "111")).isPresent();
    }

    @Test
    void shouldSetDueDateFourteenDaysAfterCheckout() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        long start = clock.millis();
        String loan = lib.checkout("alice", "111").orElseThrow();

        assertThat(lib.dueAt(loan)).hasValue(start + 14 * DAY);
        assertThat(lib.dueAt("nope")).isEmpty();
    }

    @Test
    void shouldChargeFinePerFullOverdueDay() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        String loan = lib.checkout("alice", "111").orElseThrow();

        clock.advance(16 * DAY);

        assertThat(lib.returnLoan(loan)).hasValue(100L);
    }

    @Test
    void shouldNotChargeFineWhenReturnedOnDueDate() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        String loan = lib.checkout("alice", "111").orElseThrow();

        clock.advance(14 * DAY);

        assertThat(lib.returnLoan(loan)).hasValue(0L);
    }

    @Test
    void shouldRejectReturningTheSameLoanTwice() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 2);
        String loan = lib.checkout("alice", "111").orElseThrow();

        assertThat(lib.returnLoan(loan)).isPresent();
        assertThat(lib.returnLoan(loan)).isEmpty();
        assertThat(lib.returnLoan("unknown")).isEmpty();
        assertThat(lib.availableCopies("111")).isEqualTo(2);
    }

    @Test
    void shouldHoldReturnedCopyForReservingMember() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        String loan = lib.checkout("alice", "111").orElseThrow();
        assertThat(lib.reserve("bob", "111")).isTrue();

        lib.returnLoan(loan);

        assertThat(lib.availableCopies("111")).isZero();
        assertThat(lib.checkout("carol", "111")).isEmpty();
        assertThat(lib.checkout("bob", "111")).isPresent();
    }

    @Test
    void shouldServeReservationsInFirstComeFirstServedOrder() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        String first = lib.checkout("alice", "111").orElseThrow();
        assertThat(lib.reserve("bob", "111")).isTrue();
        assertThat(lib.reserve("carol", "111")).isTrue();

        lib.returnLoan(first);

        assertThat(lib.checkout("carol", "111")).isEmpty();
        String bobsLoan = lib.checkout("bob", "111").orElseThrow();
        lib.returnLoan(bobsLoan);
        assertThat(lib.checkout("carol", "111")).isPresent();
    }

    @Test
    void shouldRefuseReservationWhenCopyIsAvailable() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);

        assertThat(lib.reserve("bob", "111")).isFalse();
    }

    @Test
    void shouldRefuseDuplicateReservationOrReservationByBorrower() {
        LibraryApi lib = library();
        lib.addBook("111", "Clean Code", 1);
        lib.checkout("alice", "111");

        assertThat(lib.reserve("alice", "111")).isFalse(); // already has it
        assertThat(lib.reserve("bob", "111")).isTrue();
        assertThat(lib.reserve("bob", "111")).isFalse();   // already queued
    }

    @Test
    void shouldSearchTitlesCaseInsensitivelySortedByIsbn() {
        LibraryApi lib = library();
        lib.addBook("300", "The Pragmatic Programmer", 1);
        lib.addBook("100", "Clean Code", 1);
        lib.addBook("200", "Clean Architecture", 1);

        assertThat(lib.search("CLEAN")).containsExactly("100", "200");
        assertThat(lib.search("pragmatic")).containsExactly("300");
        assertThat(lib.search("zzz")).isEmpty();
        assertThat(lib.search("")).containsExactly("100", "200", "300");
    }

    @Test
    void shouldFreeMemberLimitWhenLoanIsReturned() {
        LibraryApi lib = library();
        for (int i = 1; i <= 4; i++) {
            lib.addBook("B" + i, "Book " + i, 1);
        }
        String l1 = lib.checkout("alice", "B1").orElseThrow();
        lib.checkout("alice", "B2");
        lib.checkout("alice", "B3");

        lib.returnLoan(l1);

        assertThat(lib.checkout("alice", "B4")).isPresent();
    }
}

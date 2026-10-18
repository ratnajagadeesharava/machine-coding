package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface LibraryApi {

    /** Adds a book, or adds copies if the ISBN already exists. */
    void addBook(String isbn, String title, int copies);

    void addMember(String memberId);

    /** Returns the loan id, or empty if refused. */
    Optional<String> checkout(String memberId, String isbn);

    /** Closes the loan and returns the fine (0 if on time), or empty if unknown/already returned. */
    Optional<Long> returnLoan(String loanId);

    /** Due time in epoch millis. */
    Optional<Long> dueAt(String loanId);

    /** Queues the member for the book. Returns true only if queued. */
    boolean reserve(String memberId, String isbn);

    int availableCopies(String isbn);

    /** ISBNs whose title contains the query (case-insensitive), sorted ascending. */
    List<String> search(String query);
}

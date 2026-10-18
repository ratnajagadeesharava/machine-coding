package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/**
 * One link of the chain. A handler either DECIDES a request (approve or reject) or passes it to the next link.
 * The chain is built by linking handlers with {@link #setNext}; the sender only talks to the first one.
 */
public abstract class ApprovalHandler {

    /**
     * Links the next handler and returns it, so a chain can be written {@code a.setNext(b).setNext(c)}. Replaces any
     * earlier next. Null -> IllegalArgumentException.
     */
    public ApprovalHandler setNext(ApprovalHandler next) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Asks {@link #decide}; if it returns a decision, that is the answer and the rest of the chain is NOT asked.
     * Otherwise the request goes to the next handler. At the end of the chain with no decision the answer is
     * {@code "REJECTED:no approver for <amountCents>"}. Null request -> IllegalArgumentException.
     * Final: subclasses decide, they never change how the chain is walked.
     */
    public final String handle(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The decision text, or empty to pass the request on. */
    protected abstract Optional<String> decide(ExpenseRequest request);
}

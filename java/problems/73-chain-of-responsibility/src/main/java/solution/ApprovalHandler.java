package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/** One link of an approval chain. */
public abstract class ApprovalHandler {

    /** Links the next handler and returns it. */
    public ApprovalHandler setNext(ApprovalHandler next) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Handles the request, returning the decision text. */
    public final String handle(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The decision text, or empty if this handler does not decide. */
    protected abstract Optional<String> decide(ExpenseRequest request);
}

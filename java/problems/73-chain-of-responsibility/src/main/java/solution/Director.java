package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/** Approves ({@code "APPROVED:director"}) amounts up to 5000000 cents inclusive. Passes the rest on. */
public class Director extends ApprovalHandler {

    @Override
    protected Optional<String> decide(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

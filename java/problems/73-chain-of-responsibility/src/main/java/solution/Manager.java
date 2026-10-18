package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/** Approves ({@code "APPROVED:manager"}) amounts up to 500000 cents inclusive. Passes the rest on. */
public class Manager extends ApprovalHandler {

    @Override
    protected Optional<String> decide(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

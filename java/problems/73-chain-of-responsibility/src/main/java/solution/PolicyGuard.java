package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/** Rejects (decision {@code "REJECTED:policy violation"}) any request with amount <= 0 or category "gambling" (any case). Passes everything else on. */
public class PolicyGuard extends ApprovalHandler {

    @Override
    protected Optional<String> decide(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

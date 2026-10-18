package solution;

import contract.ExpenseRequest;
import java.util.Optional;

/** Approves ({@code "APPROVED:team lead"}) amounts up to 50000 cents inclusive - except category "travel" (any case), which always goes up. Passes the rest on. */
public class TeamLead extends ApprovalHandler {

    @Override
    protected Optional<String> decide(ExpenseRequest request) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

package solution;

import contract.InvoiceLine;
import contract.InvoiceRenderer;
import contract.InvoiceRepository;
import contract.TotalsCalculator;
import java.util.List;
import java.util.Optional;

/** Coordinates the three collaborators. It does no maths, no layout and no storage itself. */
public class InvoiceService {

    public InvoiceService(TotalsCalculator calculator, InvoiceRenderer renderer, InvoiceRepository repository) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Blank id -> IllegalArgumentException (nothing is called). Otherwise: calculate -> render -> save, in that
     * order, and return the rendered text. If the calculator throws, the exception propagates and neither the
     * renderer nor the repository is touched.
     */
    public String issue(String invoiceId, List<InvoiceLine> lines, int taxBps) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> lookup(String invoiceId) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

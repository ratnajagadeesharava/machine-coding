package solution;

import contract.InvoiceLine;
import contract.InvoiceTotals;
import contract.TotalsCalculator;
import java.util.List;

/**
 * subtotal = sum of quantity * unitCents; tax = subtotal * taxBps / 10000 rounded half up; total = subtotal + tax.
 * Empty lines, a quantity <= 0, a negative unit price or a negative taxBps -> IllegalArgumentException.
 */
public class StandardCalculator implements TotalsCalculator {

    @Override
    public InvoiceTotals calculate(List<InvoiceLine> lines, int taxBps) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

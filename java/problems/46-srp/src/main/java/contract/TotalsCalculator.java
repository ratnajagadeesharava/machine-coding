package contract;

import java.util.List;

/** Provided contract. */
public interface TotalsCalculator {

    InvoiceTotals calculate(List<InvoiceLine> lines, int taxBps);
}

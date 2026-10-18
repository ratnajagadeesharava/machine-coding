package contract;

import java.util.List;

/** Provided contract: knows the maths, nothing about text or storage. */
public interface TotalsCalculator {

    InvoiceTotals calculate(List<InvoiceLine> lines, int taxBps);
}

package contract;

import java.util.List;

/** Provided contract. */
public interface InvoiceRenderer {

    String render(String invoiceId, List<InvoiceLine> lines, InvoiceTotals totals);
}

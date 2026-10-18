package contract;

import java.util.List;

/** Provided contract: knows the layout, nothing about maths or storage. */
public interface InvoiceRenderer {

    String render(String invoiceId, List<InvoiceLine> lines, InvoiceTotals totals);
}

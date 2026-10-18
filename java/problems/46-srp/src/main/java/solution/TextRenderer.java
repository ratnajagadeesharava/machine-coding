package solution;

import contract.InvoiceLine;
import contract.InvoiceRenderer;
import contract.InvoiceTotals;
import java.util.List;

/**
 * Lays out an invoice exactly like this (lines joined with "\n", no trailing newline):
 * <pre>
 * INVOICE A-1
 * Widget x2 @ 5.00 = 10.00
 * Gadget x1 @ 2.50 = 2.50
 * SUBTOTAL 12.50
 * TAX 2.25
 * TOTAL 14.75
 * </pre>
 * It prints the totals it is GIVEN. It must not add up anything itself (except each line's quantity * price).
 */
public class TextRenderer implements InvoiceRenderer {

    @Override
    public String render(String invoiceId, List<InvoiceLine> lines, InvoiceTotals totals) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

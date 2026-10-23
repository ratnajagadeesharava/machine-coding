package solution;

import contract.PricingRules;
import java.util.List;

/**
 * Produces the three customer documents.
 *
 * <pre>
 * quote(lines)               "QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00"
 * invoice("INV-1", lines)    "INVOICE #INV-1 net=50.00 discount=0.00 tax=9.00 total=59.00"
 * creditNote("INV-1", lines) "CREDIT NOTE #INV-1 net=50.00 discount=0.00 tax=9.00 total=-59.00"
 * </pre>
 * net = sum of lines; total = net - discount + tax; a credit note shows the total negated.
 * Money prints as major.minor with two digits and a leading '-' when negative.
 * null / empty lines, a negative line, or (for invoice and credit note) a blank number -> IllegalArgumentException.
 */
public class Documents {

    public Documents(PricingRules rules) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String quote(List<Long> lineCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String invoice(String number, List<Long> lineCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String creditNote(String invoiceNumber, List<Long> lineCents) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

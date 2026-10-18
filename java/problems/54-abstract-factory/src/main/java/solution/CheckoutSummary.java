package solution;

import contract.RegionKit;

/** The client. It knows only the abstract kit, never a concrete region. */
public final class CheckoutSummary {
    private CheckoutSummary() {}

    /**
     * Returns {@code "<region>: net <net>, tax <tax>, total <total>, postal <valid|invalid>"} where every amount is
     * written by the kit's price formatter, tax comes from the kit's tax calculator (on the net amount),
     * total = net + tax, and the postal code is judged by the kit's address validator.
     * Example (US): {@code "US: net $100.00, tax $8.00, total $108.00, postal valid"}.
     */
    public static String summarize(RegionKit kit, long netCents, String postalCode) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

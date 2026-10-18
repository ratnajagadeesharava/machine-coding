package contract;

/** Provided product interface. Amounts are in cents. */
public interface TaxCalculator {

    /** Tax due on a net amount (>= 0), rounded half up to whole cents. Negative -> IllegalArgumentException. */
    long tax(long netCents);
}

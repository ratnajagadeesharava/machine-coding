package contract;

/** Provided product interface. */
public interface PriceFormatter {

    /** Human-readable price for an amount (>= 0) in cents. Negative -> IllegalArgumentException. */
    String format(long cents);
}

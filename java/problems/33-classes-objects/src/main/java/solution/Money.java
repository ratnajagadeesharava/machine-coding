package solution;

import java.util.ArrayList;
import java.util.Locale;
import java.util.Objects;

/**
 * Implement this class. Keep the public signatures: the tests compile against them.
 */
public final class Money implements Comparable<Money> {
    private final String currency;
    private final long minorUnits;

    private Money(String currency, long minorUnits) {
        this.currency = currency;
        this.minorUnits = minorUnits;
    }

    private static void ValidateCurrency(String currency) {
        if (currency == null) {
            throw new IllegalArgumentException("currency cannot be null");
        }
        if (currency.length() != 3) {
            throw new IllegalArgumentException("currency length should be 3 characters");
        }
        if (!currency.toUpperCase(Locale.ROOT).equals(currency)) {
            throw new IllegalArgumentException("currency should be in capital letters");
        }
    }

    /**
     * currency must be exactly 3 upper-case letters, otherwise IllegalArgumentException.
     */
    public static Money of(String currency, long minorUnits) {
        ValidateCurrency(currency);
        return new Money(currency, minorUnits);
    }

    public static Money zero(String currency) {
        ValidateCurrency(currency);
        return new Money(currency, 0);
    }

    public String currency() {
        return this.currency;
    }

    public long minorUnits() {
        return this.minorUnits;
    }

    public Money plus(Money other) {
        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException("currency should sane");
        }
        long a = this.minorUnits;
        long b = other.minorUnits;
        long r = a + b;
        if (((a ^ r) & (b ^ r)) < 0) {
            throw new ArithmeticException("overflow");
        }

        return new Money(this.currency, this.minorUnits + other.minorUnits);
    }

    public Money minus(Money other) {

        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException("currency should sane");
        }
        return new Money(this.currency, this.minorUnits - other.minorUnits);
    }

    public Money times(int factor) {
        long a = this.minorUnits;
        long b = factor;                // int widened to long
        long r = a * b;
        if (a != 0 && r / a != b) {
            throw new ArithmeticException("long overflow");
        }

        return new Money(this.currency, r);
    }

    public boolean isNegative() {
        return this.minorUnits < 0;
    }

    @Override
    public int compareTo(Money other) {
        if (!this.currency.equals(other.currency)) {
            throw new IllegalArgumentException("currency should be in capital letters");
        }
        return Long.compare(this.minorUnits, other.minorUnits());
    }

    @Override
    public int hashCode() {
        return Objects.hash(this.currency, this.minorUnits);
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) return true;
        if (obj == null || obj.getClass() != getClass()) return false;
        if (!(obj instanceof Money)) return false;
        Money other = (Money) obj;
        return this.currency.equals(other.currency) && this.minorUnits == other.minorUnits;
    }

    @Override
    public String toString() {
        long total = Math.abs(this.minorUnits / 100);
        long decimal = Math.abs(this.minorUnits % 100);
        String result = this.currency + " " + (this.minorUnits < 0 ? "-" + total : total) + "." + (decimal < 10 ? "0" + decimal : decimal);
        return result;
    }
    // equals, hashCode and toString are part of the exercise too.
}

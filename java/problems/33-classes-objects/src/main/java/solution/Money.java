package solution;

/** Implement this class. Keep the public signatures: the tests compile against them. */
public final class Money implements Comparable<Money> {

    /** currency must be exactly 3 upper-case letters, otherwise IllegalArgumentException. */
    public static Money of(String currency, long minorUnits) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public static Money zero(String currency) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String currency() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long minorUnits() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Money plus(Money other) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Money minus(Money other) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Money times(int factor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean isNegative() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public int compareTo(Money other) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    // equals, hashCode and toString are part of the exercise too.
}

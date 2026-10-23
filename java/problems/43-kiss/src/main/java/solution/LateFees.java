package solution;

/** Late fee calculator. */
public final class LateFees {
    private LateFees() {}

    /**
     * Late fee in cents for a returned library item.
     * daysLate <= 0 -> 0. Otherwise 25 cents per day (premium members: 10 cents per day),
     * capped at 1000 cents (premium members: 500 cents).
     */
    public static long fee(int daysLate, boolean premium) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

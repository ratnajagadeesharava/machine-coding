package solution;

/**
 * Keep it simple: the whole solution is this one small class.
 */
public final class LateFees {
    private LateFees() {
    }

    /**
     * Late fee in cents for a returned library item.
     * daysLate <= 0 -> 0. Otherwise 25 cents per day (premium members: 10 cents per day),
     * capped at 1000 cents (premium members: 500 cents).
     */
    public static long fee(int daysLate, boolean premium) {
        if (daysLate <= 0)
            return 0;

        if (!premium) {
        long value = (long)daysLate *25;
            return Math.min(value, 1000);
        } else {
            long value = (long)daysLate *10;
            return Math.min(value, 500);
        }
    }
}

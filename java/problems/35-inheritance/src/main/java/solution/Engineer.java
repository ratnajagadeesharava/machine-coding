package solution;

import java.util.logging.Level;

/**
 * Pay = base * (100 + 20 * (level - 1)) / 100 (integer division). Level must be 1..3. Role "Engineer".
 */
public class Engineer extends Employee {
    private final int level;

    public Engineer(String name, long baseMonthly, int level) {
        super(name, baseMonthly);
        if (level < 1 || level > 3) {
            throw new IllegalArgumentException("Illegal level");
        }
        this.level = level;

    }

    @Override
    public long monthlyPay() {
        double multiplier = (100.0D + 20D * ((double) level - 1)) / 100.0D;
        long pay = (long) (super.monthlyPay() * multiplier);
        return pay;
    }

    @Override
    public String role() {
        return "Engineer";
    }
}

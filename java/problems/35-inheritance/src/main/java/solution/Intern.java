package solution;

/**
 * Pay = base / 2 (integer division). Role "Intern".
 */
public class Intern extends Employee {

    public Intern(String name, long baseMonthly) {
        super(name, baseMonthly);

    }

    @Override
    public long monthlyPay() {
        return super.monthlyPay() / 2;
    }

    @Override
    public String role() {
        return "Intern";
    }
}

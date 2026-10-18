package solution;

/** The base class. Subclasses reuse it instead of copying it. */
public class Employee {

    /** name must not be blank and baseMonthly must not be negative, otherwise IllegalArgumentException. */
    public Employee(String name, long baseMonthly) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public long monthlyPay() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Role label used by describe(): "Employee" here, overridden by subclasses. */
    protected String role() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** "{role}: {name} earns {monthlyPay}" — written ONCE, here. Subclasses must not override it. */
    public String describe() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

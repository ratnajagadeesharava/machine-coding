package solution;

/**
 * The base class. Subclasses reuse it instead of copying it.
 */
public class Employee {
    private final String name;
    private long baseMonthly;

    /**
     * name must not be blank and baseMonthly must not be negative, otherwise IllegalArgumentException.
     */
    public Employee(String name, long baseMonthly) {
        if (name == null || name.trim().isEmpty()) {
            throw new IllegalArgumentException("name should not be empty");
        }

        if (baseMonthly < 0) {
            throw new IllegalArgumentException("salary should be positive");
        }
        this.name = name;
        this.baseMonthly = baseMonthly;

    }

    public String name() {
        return this.name;
    }

    public long monthlyPay() {
        return this.baseMonthly;
    }

    /**
     * Role label used by describe(): "Employee" here, overridden by subclasses.
     */
    protected String role() {
        return "Employee";
    }

    /**
     * "{role}: {name} earns {monthlyPay}" — written ONCE, here. Subclasses must not override it.
     */
    public String describe() {
        return role() + ": " + name + " earns " + monthlyPay();
    }
}

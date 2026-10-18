package solution;

import java.util.List;

/** Pay = own base pay + (sum of the reports' monthly pay) / 20. Role "Manager". Reports may be managers too. */
public class Manager extends Employee {

    public Manager(String name, long baseMonthly, List<Employee> reports) {
        super(name, baseMonthly);
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

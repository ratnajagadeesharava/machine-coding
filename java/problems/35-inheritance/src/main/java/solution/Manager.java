package solution;

import java.util.ArrayList;
import java.util.List;

/**
 * Pay = own base pay + (sum of the reports' monthly pay) / 20. Role "Manager". Reports may be managers too.
 */
public class Manager extends Employee {
    private List<Employee> reports;

    public Manager(String name, long baseMonthly, List<Employee> reports) {
        super(name, baseMonthly);
        this.reports =List.copyOf(reports);
    }

    @Override
    public long monthlyPay() {
        long avgPay = this.reports.stream().map((employee -> employee.monthlyPay())).reduce(0L, (sum, p) -> sum + p);
        return super.monthlyPay() + (avgPay) / 20;
    }

    @Override
    protected String role() {
        return "Manager";
    }
}

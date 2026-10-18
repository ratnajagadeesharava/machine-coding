package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Employee;
import solution.Engineer;
import solution.Intern;
import solution.Manager;

class BehaviorTest {

    @Test
    void shouldPayTheBaseSalaryForAPlainEmployee() {
        Employee e = new Employee("Zed", 1000);

        assertThat(e.name()).isEqualTo("Zed");
        assertThat(e.monthlyPay()).isEqualTo(1000);
        assertThat(e.describe()).isEqualTo("Employee: Zed earns 1000");
    }

    @Test
    void shouldRaiseEngineerPayByTwentyPercentPerLevel() {
        assertThat(new Engineer("A", 4000, 1).monthlyPay()).isEqualTo(4000);
        assertThat(new Engineer("B", 4000, 2).monthlyPay()).isEqualTo(4800);
        assertThat(new Engineer("C", 4000, 3).monthlyPay()).isEqualTo(5600);
    }

    @Test
    void shouldPayInternsHalf() {
        assertThat(new Intern("Ivy", 1001).monthlyPay()).isEqualTo(500);
    }

    @Test
    void shouldAddFivePercentOfTheReportsPayToAManager() {
        Employee eng = new Engineer("E", 4000, 2);   // 4800
        Employee intern = new Intern("I", 1000);     // 500
        Manager m = new Manager("M", 5000, List.of(eng, intern));

        assertThat(m.monthlyPay()).isEqualTo(5000 + 5300 / 20);
    }

    @Test
    void shouldLetManagersManageManagers() {
        Manager team = new Manager("Lead", 5000, List.of(new Engineer("E", 4000, 2), new Intern("I", 1000)));
        Manager dept = new Manager("Head", 8000, List.of(team));

        assertThat(dept.monthlyPay()).isEqualTo(8000 + team.monthlyPay() / 20);
    }

    @Test
    void shouldUseSubclassesWhereABaseEmployeeIsExpected() {
        List<Employee> staff = List.of(
                new Employee("P", 1000), new Engineer("E", 1000, 3), new Intern("I", 1000), new Manager("M", 1000, List.of()));

        assertThat(staff).extracting(Employee::monthlyPay).containsExactly(1000L, 1400L, 500L, 1000L);
        assertThat(staff).extracting(Employee::describe).containsExactly(
                "Employee: P earns 1000", "Engineer: E earns 1400", "Intern: I earns 500", "Manager: M earns 1000");
    }

    @Test
    void shouldInheritDescribeInsteadOfCopyingIt() throws Exception {
        for (Class<?> sub : List.of(Engineer.class, Intern.class, Manager.class)) {
            assertThat(sub.getSuperclass()).isEqualTo(Employee.class);
            assertThat(sub.getDeclaredMethod("role")).as(sub.getSimpleName() + " overrides role()").isNotNull();
            assertThat(Arrays.stream(sub.getDeclaredMethods()).map(Method::getName))
                    .as(sub.getSimpleName() + " overrides").doesNotContain("describe");
            assertThat(sub.getMethod("describe").getDeclaringClass()).isEqualTo(Employee.class);
        }
    }

    @Test
    void shouldRejectInvalidInput() {
        assertThatThrownBy(() -> new Employee(" ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Employee("A", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Engineer("A", 1, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Engineer("A", 1, 4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Intern("", 1)).isInstanceOf(IllegalArgumentException.class);
    }
}

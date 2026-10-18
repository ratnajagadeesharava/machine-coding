package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.AtmApi;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private AtmApi atm() {
        AtmApi atm = Candidate.of(Solution::create);
        atm.addAccount("c", "1234", 5000);
        atm.loadCash(100, 20);
        atm.loadCash(10, 20);
        return atm;
    }

    @Test
    void thirdWrongAfterTwoWrongSeparatedBySuccessShouldNotLockUntilThreeInARow() {
        AtmApi atm = atm();
        atm.authenticate("c", "x");
        atm.authenticate("c", "x");
        atm.authenticate("c", "1234");
        atm.authenticate("c", "x");
        atm.authenticate("c", "x");
        atm.authenticate("c", "x");
        assertThat(atm.authenticate("c", "1234")).isEmpty();
    }

    @Test
    void refusedWithdrawalsShouldNotConsumeSessionLimit() {
        AtmApi atm = atm();
        String s = atm.authenticate("c", "1234").orElseThrow();
        assertThat(atm.withdraw(s, 505)).isEmpty();
        assertThat(atm.withdraw(s, 600)).isEmpty();
        assertThat(atm.withdraw(s, 500)).isPresent();
    }

    @Test
    void limitIsInclusiveAtFiveHundred() {
        AtmApi atm = atm();
        String s = atm.authenticate("c", "1234").orElseThrow();
        assertThat(atm.withdraw(s, 490)).isPresent();
        assertThat(atm.withdraw(s, 20)).isEmpty();
        assertThat(atm.withdraw(s, 10)).isPresent();
    }

    @Test
    void shouldPreferFewestNotes() {
        AtmApi atm = atm();
        String s = atm.authenticate("c", "1234").orElseThrow();
        assertThat(atm.withdraw(s, 130)).hasValue(Map.of(100, 1, 10, 3));
    }
}

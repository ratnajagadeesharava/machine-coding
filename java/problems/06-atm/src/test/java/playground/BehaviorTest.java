package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.AtmApi;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private AtmApi atm() {
        AtmApi atm = Candidate.of(Solution::create);
        atm.addAccount("card-1", "1234", 2000);
        atm.loadCash(100, 10);
        atm.loadCash(50, 10);
        atm.loadCash(20, 10);
        atm.loadCash(10, 10);
        return atm;
    }

    private static int total(Map<Integer, Integer> notes) {
        return notes.entrySet().stream().mapToInt(e -> e.getKey() * e.getValue()).sum();
    }

    @Test
    void shouldAuthenticateWithCorrectPin() {
        AtmApi atm = atm();

        assertThat(atm.authenticate("card-1", "1234")).isPresent();
    }

    @Test
    void shouldRejectWrongPinAndUnknownCard() {
        AtmApi atm = atm();

        assertThat(atm.authenticate("card-1", "0000")).isEmpty();
        assertThat(atm.authenticate("nope", "1234")).isEmpty();
    }

    @Test
    void shouldLockCardAfterThreeConsecutiveWrongPins() {
        AtmApi atm = atm();

        atm.authenticate("card-1", "1");
        atm.authenticate("card-1", "2");
        atm.authenticate("card-1", "3");

        assertThat(atm.authenticate("card-1", "1234")).isEmpty();
    }

    @Test
    void shouldResetFailureCountAfterSuccessfulLogin() {
        AtmApi atm = atm();

        atm.authenticate("card-1", "1");
        atm.authenticate("card-1", "2");
        assertThat(atm.authenticate("card-1", "1234")).isPresent();
        atm.authenticate("card-1", "3");
        atm.authenticate("card-1", "4");

        assertThat(atm.authenticate("card-1", "1234")).isPresent();
    }

    @Test
    void shouldReportBalance() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.balance(s)).hasValue(2000L);
    }

    @Test
    void shouldDispenseNotesAndDebitAccount() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        Map<Integer, Integer> notes = atm.withdraw(s, 180).orElseThrow();

        assertThat(total(notes)).isEqualTo(180);
        assertThat(notes.values().stream().mapToInt(Integer::intValue).sum()).isEqualTo(4); // 100+50+20+10
        assertThat(atm.balance(s)).hasValue(1820L);
        assertThat(atm.cashInMachine()).isEqualTo(1000 + 500 + 200 + 100 - 180);
    }

    @Test
    void shouldRefuseAmountsThatAreNotPositiveMultiplesOfTen() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.withdraw(s, 0)).isEmpty();
        assertThat(atm.withdraw(s, -10)).isEmpty();
        assertThat(atm.withdraw(s, 15)).isEmpty();
        assertThat(atm.balance(s)).hasValue(2000L);
    }

    @Test
    void shouldRefuseWhenAccountBalanceIsInsufficient() {
        AtmApi atm = Candidate.of(Solution::create);
        atm.addAccount("poor", "1111", 50);
        atm.loadCash(100, 10);
        atm.loadCash(10, 10);
        String s = atm.authenticate("poor", "1111").orElseThrow();

        assertThat(atm.withdraw(s, 60)).isEmpty();
        assertThat(atm.balance(s)).hasValue(50L);
    }

    @Test
    void shouldRefuseWhenMachineCannotPayExactlyAndLeaveEverythingUntouched() {
        AtmApi atm = Candidate.of(Solution::create);
        atm.addAccount("card-1", "1234", 2000);
        atm.loadCash(50, 10);
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.withdraw(s, 30)).isEmpty();

        assertThat(atm.balance(s)).hasValue(2000L);
        assertThat(atm.cashInMachine()).isEqualTo(500);
        assertThat(atm.withdraw(s, 100)).hasValue(Map.of(50, 2));
    }

    @Test
    void shouldFindValidCombinationWhenGreedyChoiceWouldFail() {
        AtmApi atm = Candidate.of(Solution::create);
        atm.addAccount("card-1", "1234", 2000);
        atm.loadCash(50, 1);
        atm.loadCash(20, 3);
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.withdraw(s, 60)).hasValue(Map.of(20, 3));
    }

    @Test
    void shouldEnforceSessionWithdrawalLimit() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.withdraw(s, 300)).isPresent();
        assertThat(atm.withdraw(s, 300)).isEmpty();
        assertThat(atm.withdraw(s, 200)).isPresent();
        assertThat(atm.withdraw(s, 10)).isEmpty();
    }

    @Test
    void shouldStartFreshLimitInNewSession() {
        AtmApi atm = atm();
        String first = atm.authenticate("card-1", "1234").orElseThrow();
        atm.withdraw(first, 500);
        atm.endSession(first);

        String second = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.withdraw(second, 500)).isPresent();
    }

    @Test
    void shouldInvalidateEndedSession() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        atm.endSession(s);

        assertThat(atm.balance(s)).isEmpty();
        assertThat(atm.withdraw(s, 10)).isEmpty();
        assertThat(atm.deposit(s, 10)).isFalse();
        assertThat(atm.balance("bogus")).isEmpty();
    }

    @Test
    void shouldCreditDeposits() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.deposit(s, 250)).isTrue();

        assertThat(atm.balance(s)).hasValue(2250L);
    }

    @Test
    void shouldRejectNonPositiveDeposits() {
        AtmApi atm = atm();
        String s = atm.authenticate("card-1", "1234").orElseThrow();

        assertThat(atm.deposit(s, 0)).isFalse();
        assertThat(atm.deposit(s, -5)).isFalse();
        assertThat(atm.balance(s)).hasValue(2000L);
    }
}

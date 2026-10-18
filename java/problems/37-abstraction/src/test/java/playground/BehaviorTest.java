package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Account;
import solution.CheckingAccount;
import solution.SavingsAccount;

class BehaviorTest {

    /** A tiny subclass written in the test: only the two abstract decisions are supplied. */
    private static final class Configurable extends Account {
        private final long fee;
        private final boolean allow;

        Configurable(long fee, boolean allow) {
            this.fee = fee;
            this.allow = allow;
        }

        @Override protected long monthlyFee() { return fee; }
        @Override protected boolean canWithdraw(long amount) { return allow; }
    }

    @Test
    void shouldBeAbstractWithExactlyTwoAbstractMethods() {
        assertThat(Modifier.isAbstract(Account.class.getModifiers())).isTrue();
        assertThat(Arrays.stream(Account.class.getDeclaredMethods())
                .filter(m -> Modifier.isAbstract(m.getModifiers()))
                .map(Method::getName))
                .containsExactlyInAnyOrder("monthlyFee", "canWithdraw");
        // the shared behaviour is concrete: a subclass with only the two decisions can already be used
        assertThat(new Configurable(0, true).balance()).isZero();
    }

    @Test
    void shouldImplementDepositAndWithdrawOnceInTheBaseClass() {
        Account a = new Configurable(0, true);

        a.deposit(500);
        a.withdraw(200);

        assertThat(a.balance()).isEqualTo(300);
    }

    @Test
    void shouldAskTheSubclassBeforeEveryWithdrawal() {
        Account a = new Configurable(0, false);
        a.deposit(500);

        assertThatThrownBy(() -> a.withdraw(1)).isInstanceOf(IllegalStateException.class);
        assertThat(a.balance()).isEqualTo(500);
    }

    @Test
    void shouldChargeTheSubclassFeeAtMonthEnd() {
        Account a = new Configurable(75, true);
        a.deposit(100);

        a.monthEnd();
        a.monthEnd();

        assertThat(a.balance()).isEqualTo(-50);
    }

    @Test
    void shouldRejectNonPositiveAmounts() {
        Account a = new Configurable(0, true);

        assertThatThrownBy(() -> a.deposit(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.deposit(-5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.withdraw(0)).isInstanceOf(IllegalArgumentException.class);
        assertThat(a.balance()).isZero();
    }

    @Test
    void shouldKeepASavingsAccountAboveItsMinimumBalance() {
        Account s = new SavingsAccount(1000);
        s.deposit(1500);

        s.withdraw(500);
        assertThatThrownBy(() -> s.withdraw(1)).isInstanceOf(IllegalStateException.class);
        s.monthEnd();

        assertThat(s.balance()).isEqualTo(1000);
    }

    @Test
    void shouldLetACheckingAccountOverdrawWithinTheLimitAndChargeAFee() {
        Account c = new CheckingAccount(500);
        c.deposit(100);

        c.withdraw(600);
        assertThat(c.balance()).isEqualTo(-500);
        assertThatThrownBy(() -> c.withdraw(1)).isInstanceOf(IllegalStateException.class);
        c.monthEnd();

        assertThat(c.balance()).isEqualTo(-700);
    }

    @Test
    void shouldWaiveTheCheckingFeeForRichCustomers() {
        Account c = new CheckingAccount(0);
        c.deposit(100_000);

        c.monthEnd();

        assertThat(c.balance()).isEqualTo(100_000);
    }

    @Test
    void shouldTreatDifferentAccountsUniformlyThroughTheBaseType() {
        List<Account> all = List.of(new SavingsAccount(0), new CheckingAccount(0), new Configurable(10, true));
        all.forEach(a -> a.deposit(1000));

        all.forEach(Account::monthEnd);

        assertThat(all).extracting(Account::balance).containsExactly(1000L, 800L, 990L);
    }
}

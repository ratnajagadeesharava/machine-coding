package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.WalletApi;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private WalletApi service(String... wallets) {
        WalletApi api = Candidate.of(Solution::create);
        for (String w : wallets) {
            api.createWallet(w);
        }
        return api;
    }

    @Test
    void transferToTheSameWalletShouldBeRefusedAsInvalid() {
        WalletApi api = service("alice");
        api.credit("k0", "alice", 100);

        assertThat(api.transfer("k1", "alice", "alice", 10)).isEqualTo("INVALID_AMOUNT");

        assertThat(api.balance("alice")).hasValue(100);
        assertThat(api.statement("alice")).containsExactly("+100");
    }

    @Test
    void duplicateShouldWinOverAnyOtherCheck() {
        WalletApi api = service("alice");
        api.credit("k1", "alice", 100);

        assertThat(api.credit("k1", "alice", 0)).isEqualTo("DUPLICATE");
        assertThat(api.debit("k1", "ghost", 5)).isEqualTo("DUPLICATE");
        assertThat(api.debit("k1", "alice", 1_000_000)).isEqualTo("DUPLICATE");
    }

    @Test
    void invalidAmountShouldWinOverUnknownWalletAndUnknownWalletOverFunds() {
        WalletApi api = service("alice");

        assertThat(api.debit("k1", "ghost", 0)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.transfer("k2", "ghost", "alice", -1)).isEqualTo("INVALID_AMOUNT");
        assertThat(api.debit("k3", "ghost", 10)).isEqualTo("UNKNOWN_WALLET");
        assertThat(api.transfer("k4", "alice", "ghost", 10)).isEqualTo("UNKNOWN_WALLET");
        assertThat(api.debit("k5", "alice", 10)).isEqualTo("INSUFFICIENT_FUNDS");
    }

    @Test
    void anUnknownWalletRefusalShouldNotBurnTheKey() {
        WalletApi api = service("alice");
        api.credit("k0", "alice", 100);

        assertThat(api.transfer("k1", "alice", "bob", 40)).isEqualTo("UNKNOWN_WALLET");
        api.createWallet("bob");

        assertThat(api.transfer("k1", "alice", "bob", 40)).isEqualTo("APPLIED");
        assertThat(api.balance("alice")).hasValue(60);
        assertThat(api.balance("bob")).hasValue(40);
    }

    @Test
    void shouldTransferTheWholeBalanceAndKeepAnIndependentStatementSnapshot() {
        WalletApi api = service("alice", "bob");
        api.credit("k0", "alice", 70);

        List<String> before = api.statement("alice");
        assertThat(api.transfer("k1", "alice", "bob", 70)).isEqualTo("APPLIED");

        assertThat(api.balance("alice")).hasValue(0);
        assertThat(before).containsExactly("+70");
        assertThat(api.statement("alice")).containsExactly("+70", "-70");
    }
}

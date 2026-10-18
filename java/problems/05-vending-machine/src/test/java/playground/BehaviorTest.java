package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.VendingMachineApi;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private VendingMachineApi machine() {
        return Candidate.of(Solution::create);
    }

    private VendingMachineApi stockedMachine() {
        VendingMachineApi m = machine();
        m.loadProduct("A1", "Chips", 65, 2);
        m.loadProduct("B1", "Soda", 100, 1);
        m.loadCoins(25, 4);
        m.loadCoins(10, 4);
        m.loadCoins(5, 4);
        return m;
    }

    @Test
    void shouldAcceptSupportedCoinsAndTrackBalance() {
        VendingMachineApi m = stockedMachine();

        assertThat(m.insertCoin(25)).isTrue();
        assertThat(m.insertCoin(100)).isTrue();

        assertThat(m.balance()).isEqualTo(125);
    }

    @Test
    void shouldRejectUnsupportedCoin() {
        VendingMachineApi m = stockedMachine();

        assertThat(m.insertCoin(3)).isFalse();
        assertThat(m.insertCoin(50)).isFalse();

        assertThat(m.balance()).isZero();
    }

    @Test
    void shouldDispenseProductAndReturnChange() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(100);

        assertThat(m.select("A1")).isEqualTo("DISPENSED");

        assertThat(m.takeReturnedCoins()).containsExactly(25, 10);
        assertThat(m.balance()).isZero();
        assertThat(m.stockOf("A1")).isEqualTo(1);
    }

    @Test
    void shouldEmptyTrayWhenCoinsAreTaken() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(100);
        m.select("A1");

        m.takeReturnedCoins();

        assertThat(m.takeReturnedCoins()).isEmpty();
    }

    @Test
    void shouldReturnNoChangeForExactPayment() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(100);

        assertThat(m.select("B1")).isEqualTo("DISPENSED");

        assertThat(m.takeReturnedCoins()).isEmpty();
    }

    @Test
    void shouldRefuseWhenFundsAreInsufficientAndKeepBalance() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(25);

        assertThat(m.select("A1")).isEqualTo("INSUFFICIENT_FUNDS");

        assertThat(m.balance()).isEqualTo(25);
        assertThat(m.stockOf("A1")).isEqualTo(2);
    }

    @Test
    void shouldCompletePurchaseAfterAddingMoreCoins() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(25);
        m.select("A1");

        m.insertCoin(25);
        m.insertCoin(25);

        assertThat(m.select("A1")).isEqualTo("DISPENSED");
        assertThat(m.takeReturnedCoins()).containsExactly(10);
    }

    @Test
    void shouldRefuseSoldOutProduct() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(100);
        m.select("B1");
        m.insertCoin(100);

        assertThat(m.select("B1")).isEqualTo("SOLD_OUT");
        assertThat(m.balance()).isEqualTo(100);
    }

    @Test
    void shouldRefuseInvalidSlot() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(100);

        assertThat(m.select("Z9")).isEqualTo("INVALID_SLOT");
        assertThat(m.balance()).isEqualTo(100);
    }

    @Test
    void shouldRefuseWhenExactChangeCannotBeMade() {
        VendingMachineApi m = machine();
        m.loadProduct("A1", "Chips", 65, 1);
        m.insertCoin(100);

        assertThat(m.select("A1")).isEqualTo("CANNOT_MAKE_CHANGE");

        assertThat(m.balance()).isEqualTo(100);
        assertThat(m.stockOf("A1")).isEqualTo(1);
        assertThat(m.takeReturnedCoins()).isEmpty();
    }

    @Test
    void shouldAcceptExactPaymentEvenWithEmptyCoinInventory() {
        VendingMachineApi m = machine();
        m.loadProduct("B1", "Soda", 100, 1);
        for (int i = 0; i < 4; i++) {
            m.insertCoin(25);
        }

        assertThat(m.select("B1")).isEqualTo("DISPENSED");
    }

    @Test
    void shouldRefundExactlyTheInsertedCoinsOnCancel() {
        VendingMachineApi m = stockedMachine();
        m.insertCoin(25);
        m.insertCoin(100);
        m.insertCoin(5);

        m.cancel();

        assertThat(m.balance()).isZero();
        assertThat(m.takeReturnedCoins()).containsExactly(100, 25, 5);
    }

    @Test
    void shouldDoNothingWhenCancellingWithoutCredit() {
        VendingMachineApi m = stockedMachine();

        m.cancel();

        assertThat(m.takeReturnedCoins()).isEmpty();
    }

    @Test
    void shouldUseFewestCoinsForChange() {
        VendingMachineApi m = machine();
        m.loadProduct("A1", "Gum", 10, 1);
        m.loadCoins(25, 2);
        m.loadCoins(10, 5);
        m.loadCoins(5, 5);
        m.insertCoin(25);

        assertThat(m.select("A1")).isEqualTo("DISPENSED");

        List<Integer> change = m.takeReturnedCoins();
        assertThat(change.stream().mapToInt(Integer::intValue).sum()).isEqualTo(15);
        assertThat(change).hasSize(2); // 10 + 5
    }

    @Test
    void shouldTrackProductQuantityPerSlot() {
        VendingMachineApi m = stockedMachine();

        assertThat(m.stockOf("A1")).isEqualTo(2);
        assertThat(m.stockOf("B1")).isEqualTo(1);
        assertThat(m.stockOf("C7")).isZero();
    }
}

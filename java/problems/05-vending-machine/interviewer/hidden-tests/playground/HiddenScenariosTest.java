package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.VendingMachineApi;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    private VendingMachineApi machine() {
        return Candidate.of(Solution::create);
    }

    @Test
    void invalidSlotShouldWinOverEveryOtherRule() {
        VendingMachineApi m = machine();
        assertThat(m.select("Q1")).isEqualTo("INVALID_SLOT");
    }

    @Test
    void soldOutShouldWinOverInsufficientFunds() {
        VendingMachineApi m = machine();
        m.loadProduct("A1", "Gum", 50, 0);
        assertThat(m.select("A1")).isEqualTo("SOLD_OUT");
    }

    @Test
    void shouldNotBeAbleToMakeChangeOnceDimesRunOut() {
        VendingMachineApi m = machine();
        m.loadProduct("A1", "Gum", 90, 3);
        m.loadCoins(10, 1);

        m.insertCoin(100);
        assertThat(m.select("A1")).isEqualTo("DISPENSED");
        m.takeReturnedCoins();
        m.insertCoin(100);

        assertThat(m.select("A1")).isEqualTo("CANNOT_MAKE_CHANGE");
        assertThat(m.stockOf("A1")).isEqualTo(2);
    }

    @Test
    void cancelShouldReturnInsertedCoinsEvenWhenInventoryCouldProvideOthers() {
        VendingMachineApi m = machine();
        m.loadCoins(5, 10);
        m.insertCoin(25);
        m.cancel();
        assertThat(m.takeReturnedCoins()).containsExactly(25);
    }
}

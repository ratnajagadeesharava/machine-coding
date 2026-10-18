package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.PaymentMethod;
import org.junit.jupiter.api.Test;
import solution.CashOnDelivery;

class CashOnDeliveryTest extends PaymentMethodContractTest {

    @Override
    protected PaymentMethod method() {
        return new CashOnDelivery(50_000);
    }

    @Override
    protected long amountThatIsDeclined() {
        return 50_001;
    }

    @Test
    void shouldNumberItsReferences() {
        CashOnDelivery cash = new CashOnDelivery(50_000);

        assertThat(cash.charge(10).reference()).isEqualTo("cod-1");
        assertThat(cash.charge(50_000).reference()).isEqualTo("cod-2");
        assertThat(cash.charge(50_001).approved()).isFalse();
    }
}

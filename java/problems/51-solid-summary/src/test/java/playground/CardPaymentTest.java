package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.PaymentMethod;
import org.junit.jupiter.api.Test;
import solution.CardPayment;

class CardPaymentTest extends PaymentMethodContractTest {

    @Override
    protected PaymentMethod method() {
        return new CardPayment(10_000);
    }

    @Override
    protected long amountThatIsDeclined() {
        return 10_001;
    }

    @Test
    void shouldNumberItsReferencesInOrderOfApproval() {
        CardPayment card = new CardPayment(1000);

        assertThat(card.charge(10).reference()).isEqualTo("card-1");
        assertThat(card.charge(5000).approved()).isFalse();
        assertThat(card.charge(10).reference()).isEqualTo("card-2");
        assertThat(card.charge(1000).approved()).isTrue();
    }

    @Test
    void shouldRefundWithinWhatWasChargedAndNoMore() {
        CardPayment card = new CardPayment(10_000);
        String ref = card.charge(1000).reference();

        assertThat(card.refund(ref, 600)).isTrue();
        assertThat(card.refund(ref, 500)).isFalse();
        assertThat(card.refund(ref, 400)).isTrue();
        assertThat(card.refund("card-99", 1)).isFalse();
        assertThat(card.refund(ref, 1)).isFalse();
    }
}

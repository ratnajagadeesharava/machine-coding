package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.PaymentMethod;
import contract.PaymentResult;
import org.junit.jupiter.api.Test;

/** Shared Liskov suite: every PaymentMethod must pass ALL of it. */
abstract class PaymentMethodContractTest {

    protected abstract PaymentMethod method();

    /** An amount this method declines. */
    protected abstract long amountThatIsDeclined();

    @Test
    void shouldApproveAnOrdinaryChargeWithANonBlankReference() {
        PaymentResult r = method().charge(500);

        assertThat(r.approved()).isTrue();
        assertThat(r.reference()).isNotBlank();
    }

    @Test
    void shouldDeclineWithAnEmptyReferenceAndWithoutThrowing() {
        PaymentResult r = method().charge(amountThatIsDeclined());

        assertThat(r.approved()).isFalse();
        assertThat(r.reference()).isEmpty();
    }

    @Test
    void shouldNeverThrowForAValidAmountNoMatterHowLarge() {
        PaymentMethod m = method();

        for (long amount : new long[] {1, 10_000, 1_000_000_000_000L, Long.MAX_VALUE}) {
            assertThat(m.charge(amount)).isNotNull();
        }
    }

    @Test
    void shouldRejectZeroAndNegativeAmountsWithIllegalArgumentException() {
        PaymentMethod m = method();

        assertThatThrownBy(() -> m.charge(0)).isExactlyInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.charge(-1)).isExactlyInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldIssueADifferentReferenceForEveryApprovedCharge() {
        PaymentMethod m = method();

        String first = m.charge(100).reference();
        String second = m.charge(100).reference();

        assertThat(first).isNotEqualTo(second);
    }
}

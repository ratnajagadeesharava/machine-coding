package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.PaymentGateway;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.BankTransferCheckout;
import solution.CardCheckout;
import solution.CheckoutFlow;

class BehaviorTest {

    /** A creator written in the test: it plugs in its own gateway through the factory method. */
    private static final class RecordingCheckout extends CheckoutFlow {
        final List<String> charges = new ArrayList<>();
        int created;
        boolean accept = true;

        @Override
        protected PaymentGateway createGateway() {
            created++;
            return new PaymentGateway() {
                public String name() { return "fake"; }
                public boolean charge(String customerId, long amountCents) {
                    charges.add(customerId + ":" + amountCents);
                    return accept;
                }
            };
        }
    }

    @Test
    void shouldCheckOutByCardWithTheCardGateway() {
        assertThat(new CardCheckout().checkout("cust-1", 5000)).isEqualTo("RECEIPT card cust-1 5000");
        assertThat(new CardCheckout().checkout("cust-1", 100_001)).isEqualTo("DECLINED card cust-1 100001");
        assertThat(new CardCheckout().checkout("cust-2", 100_000)).isEqualTo("RECEIPT card cust-2 100000");
    }

    @Test
    void shouldCheckOutByBankTransferWithTheBankGateway() {
        assertThat(new BankTransferCheckout().checkout("cust-1", 250_000)).isEqualTo("RECEIPT bank cust-1 250000");
        assertThat(new BankTransferCheckout().checkout("cust-1", 999)).isEqualTo("DECLINED bank cust-1 999");
        assertThat(new BankTransferCheckout().checkout("cust-1", 1000)).isEqualTo("RECEIPT bank cust-1 1000");
    }

    @Test
    void shouldUseWhateverGatewayTheSubclassCreates() {
        RecordingCheckout flow = new RecordingCheckout();

        String receipt = flow.checkout("cust-7", 1234);

        assertThat(receipt).isEqualTo("RECEIPT fake cust-7 1234");
        assertThat(flow.charges).containsExactly("cust-7:1234");
    }

    @Test
    void shouldReportADeclineFromTheCreatedGateway() {
        RecordingCheckout flow = new RecordingCheckout();
        flow.accept = false;

        assertThat(flow.checkout("cust-7", 50)).isEqualTo("DECLINED fake cust-7 50");
    }

    @Test
    void shouldAskTheFactoryMethodExactlyOncePerCheckout() {
        RecordingCheckout flow = new RecordingCheckout();

        flow.checkout("a", 1);
        flow.checkout("b", 2);
        flow.checkout("c", 3);

        assertThat(flow.created).isEqualTo(3);
    }

    @Test
    void shouldValidateBeforeCreatingAnyGateway() {
        RecordingCheckout flow = new RecordingCheckout();

        assertThatThrownBy(() -> flow.checkout(" ", 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> flow.checkout(null, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> flow.checkout("c", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> flow.checkout("c", -5)).isInstanceOf(IllegalArgumentException.class);

        assertThat(flow.created).isZero();
        assertThat(flow.checkout("c", 5)).isEqualTo("RECEIPT fake c 5");
    }

    @Test
    void shouldFixTheAlgorithmAndLeaveOnlyTheCreationStepOpen() throws Exception {
        assertThat(new CardCheckout().checkout("c", 1)).startsWith("RECEIPT card");

        Method checkout = CheckoutFlow.class.getMethod("checkout", String.class, long.class);
        Method factory = CheckoutFlow.class.getDeclaredMethod("createGateway");
        assertThat(Modifier.isAbstract(CheckoutFlow.class.getModifiers())).isTrue();
        assertThat(Modifier.isFinal(checkout.getModifiers())).as("checkout must be final").isTrue();
        assertThat(Modifier.isAbstract(factory.getModifiers())).as("createGateway must be abstract").isTrue();
        assertThat(Modifier.isProtected(factory.getModifiers())).as("createGateway must be protected").isTrue();
    }
}

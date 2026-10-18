package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DeliveryFeePolicy;
import contract.Item;
import contract.Notifier;
import contract.OrderRepository;
import contract.PaymentMethod;
import contract.PaymentResult;
import contract.Placement;
import contract.Refundable;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;
import solution.CardPayment;
import solution.CashOnDelivery;
import solution.DistanceBandFee;
import solution.OrderPricer;
import solution.OrderProcessor;

class BehaviorTest {

    private static final List<Item> MEAL = List.of(new Item("burger", 2, 800), new Item("fries", 1, 300));

    private final List<String> events = new ArrayList<>();
    private final Map<String, String> statuses = new HashMap<>();

    private final OrderRepository repository = new OrderRepository() {
        public void save(String orderId, String status, long totalCents) {
            events.add("save:" + orderId + ":" + status + ":" + totalCents);
            statuses.put(orderId, status);
        }

        public Optional<String> statusOf(String orderId) {
            return Optional.ofNullable(statuses.get(orderId));
        }
    };

    private final Notifier notifier = (orderId, message) -> events.add("notify:" + orderId + ":" + message);

    private class FakePayment implements PaymentMethod {
        boolean approve = true;
        int calls;
        long lastAmount;

        public PaymentResult charge(long amountCents) {
            calls++;
            lastAmount = amountCents;
            events.add("charge:" + amountCents);
            return approve ? new PaymentResult(true, "fake-1") : new PaymentResult(false, "");
        }
    }

    private OrderProcessor processor() {
        return new OrderProcessor(new OrderPricer(), repository, notifier);
    }

    @Test
    void shouldPriceItemsPlusDelivery() {
        OrderPricer pricer = new OrderPricer();

        assertThat(pricer.total(MEAL, 2500, new DistanceBandFee())).isEqualTo(2200);
        assertThat(pricer.total(MEAL, 500, new DistanceBandFee())).isEqualTo(1900);
        assertThatThrownBy(() -> pricer.total(List.of(), 500, new DistanceBandFee())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> pricer.total(List.of(new Item("x", 0, 5)), 500, new DistanceBandFee()))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> pricer.total(MEAL, -1, new DistanceBandFee())).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldPriceDeliveryInDistanceBands() {
        DeliveryFeePolicy fee = new DistanceBandFee();

        assertThat(fee.fee(0)).isZero();
        assertThat(fee.fee(1000)).isZero();
        assertThat(fee.fee(1001)).isEqualTo(300);
        assertThat(fee.fee(3000)).isEqualTo(300);
        assertThat(fee.fee(3001)).isEqualTo(400);
        assertThat(fee.fee(4000)).isEqualTo(400);
        assertThat(fee.fee(4001)).isEqualTo(500);
        assertThatThrownBy(() -> fee.fee(-1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldChargeThenSaveThenNotifyWhenPaymentIsApproved() {
        FakePayment payment = new FakePayment();

        Placement placement = processor().place("o1", MEAL, 2500, new DistanceBandFee(), payment);

        assertThat(placement).isEqualTo(new Placement("PAID", 2200, "fake-1"));
        assertThat(payment.lastAmount).isEqualTo(2200);
        assertThat(events).containsExactly("charge:2200", "save:o1:PAID:2200", "notify:o1:Order o1 PAID");
    }

    @Test
    void shouldRecordADeclinedPayment() {
        FakePayment payment = new FakePayment();
        payment.approve = false;
        OrderProcessor processor = processor();

        Placement placement = processor.place("o2", MEAL, 500, new DistanceBandFee(), payment);

        assertThat(placement).isEqualTo(new Placement("PAYMENT_DECLINED", 1900, ""));
        assertThat(events).containsExactly("charge:1900", "save:o2:PAYMENT_DECLINED:1900", "notify:o2:Order o2 PAYMENT_DECLINED");
        assertThat(processor.status("o2")).hasValue("PAYMENT_DECLINED");
        assertThat(processor.status("nope")).isEmpty();
    }

    @Test
    void shouldNotChargeAnythingForAFreeOrder() {
        FakePayment payment = new FakePayment();

        Placement placement = processor().place("free", List.of(new Item("sample", 1, 0)), 500, new DistanceBandFee(), payment);

        assertThat(placement).isEqualTo(new Placement("PAID", 0, ""));
        assertThat(payment.calls).isZero();
    }

    @Test
    void shouldAcceptNewFeePoliciesAndPaymentMethodsWithoutAnyChange() {
        DeliveryFeePolicy flat = distance -> 999;
        PaymentMethod voucher = amount -> new PaymentResult(true, "voucher-7");

        Placement placement = processor().place("o3", MEAL, 123_456, flat, voucher);

        assertThat(placement).isEqualTo(new Placement("PAID", 1900 + 999, "voucher-7"));
    }

    @Test
    void shouldRefundThroughAnythingThatIsRefundableEvenIfItCannotCharge() {
        Refundable refundOnly = (reference, amount) -> reference.equals("ok");
        OrderProcessor processor = processor();

        assertThat(processor.refund("o1", "ok", 500, refundOnly)).isTrue();
        assertThat(events).containsExactly("save:o1:REFUNDED:500", "notify:o1:Order o1 REFUNDED");
        events.clear();
        assertThat(processor.refund("o1", "unknown", 500, refundOnly)).isFalse();
        assertThat(events).isEmpty();
        assertThatThrownBy(() -> processor.refund("o1", "ok", 500, null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldWorkEndToEndWithTheRealCardAndRefundWithinTheAmountPaid() {
        CardPayment card = new CardPayment(100_000);
        OrderProcessor processor = processor();
        Placement placement = processor.place("o9", MEAL, 2500, new DistanceBandFee(), card);

        assertThat(placement.reference()).isEqualTo("card-1");
        assertThat(processor.refund("o9", placement.reference(), 2000, card)).isTrue();
        assertThat(processor.refund("o9", placement.reference(), 300, card)).isFalse();
        assertThat(processor.status("o9")).hasValue("REFUNDED");
    }

    @Test
    void shouldKeepSegregatedInterfacesAndInjectedDependencies() {
        assertThat(CardPayment.class.getInterfaces()).containsExactlyInAnyOrder(PaymentMethod.class, Refundable.class);
        assertThat(CashOnDelivery.class.getInterfaces()).containsExactly(PaymentMethod.class);
        assertThat(Refundable.class.isAssignableFrom(CashOnDelivery.class)).isFalse();
        assertThat(OrderProcessor.class.getConstructors()).hasSize(1);
        assertThat(OrderProcessor.class.getConstructors()[0].getParameterTypes())
                .containsExactly(OrderPricer.class, OrderRepository.class, Notifier.class);
        assertThat(processor().status("x")).isEmpty();
    }

    @Test
    void shouldRejectABlankOrderIdBeforeDoingAnything() {
        FakePayment payment = new FakePayment();

        assertThatThrownBy(() -> processor().place(" ", MEAL, 500, new DistanceBandFee(), payment))
                .isInstanceOf(IllegalArgumentException.class);

        assertThat(payment.calls).isZero();
        assertThat(events).isEmpty();
    }
}

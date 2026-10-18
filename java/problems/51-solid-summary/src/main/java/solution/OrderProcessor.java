package solution;

import contract.DeliveryFeePolicy;
import contract.Item;
import contract.Notifier;
import contract.OrderRepository;
import contract.PaymentMethod;
import contract.Placement;
import contract.Refundable;
import java.util.List;
import java.util.Optional;

/**
 * Orchestrates an order. It prices through the {@link OrderPricer}, charges the {@link PaymentMethod} it is given,
 * stores through the {@link OrderRepository} and tells the customer through the {@link Notifier}. It creates and
 * knows no concrete payment method, fee policy, database or channel.
 */
public class OrderProcessor {

    public OrderProcessor(OrderPricer pricer, OrderRepository repository, Notifier notifier) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Blank id -> IllegalArgumentException. total = pricer.total(...). If total > 0: charge the payment method;
     * approved -> status "PAID", declined -> "PAYMENT_DECLINED". If total == 0 nothing is charged (the method is
     * not called), the status is "PAID" and the reference is "". Then, in this order:
     * repository.save(orderId, status, total) and notifier.notify(orderId, "Order " + orderId + " " + status).
     * Returns the Placement(status, total, reference).
     */
    public Placement place(String orderId, List<Item> items, int distanceMeters, DeliveryFeePolicy fee, PaymentMethod payment) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Asks {@code method} to refund. If it says true: repository.save(orderId, "REFUNDED", amountCents), then
     * notifier.notify(orderId, "Order " + orderId + " REFUNDED"), and return true. If false: nothing is saved or
     * sent and false is returned. Null method -> IllegalArgumentException. Note the parameter type: only
     * {@link Refundable} is required, not a whole payment method.
     */
    public boolean refund(String orderId, String reference, long amountCents, Refundable method) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> status(String orderId) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

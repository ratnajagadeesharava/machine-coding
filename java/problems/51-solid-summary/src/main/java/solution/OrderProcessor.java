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

/** Places and refunds orders. */
public class OrderProcessor {

    public OrderProcessor(OrderPricer pricer, OrderRepository repository, Notifier notifier) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Placement place(String orderId, List<Item> items, int distanceMeters, DeliveryFeePolicy fee, PaymentMethod payment) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean refund(String orderId, String reference, long amountCents, Refundable method) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> status(String orderId) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

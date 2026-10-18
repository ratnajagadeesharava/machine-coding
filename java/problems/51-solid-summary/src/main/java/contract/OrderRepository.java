package contract;

import java.util.Optional;

/** Provided abstraction: where orders are kept. */
public interface OrderRepository {

    void save(String orderId, String status, long totalCents);

    Optional<String> statusOf(String orderId);
}

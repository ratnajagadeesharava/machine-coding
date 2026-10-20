package contract;

import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Everything lives on an integer grid; distance is Manhattan distance |x1-x2| + |y1-y2|.
 *
 * <p>Order states: PLACED -> ACCEPTED -> READY -> PICKED_UP -> DELIVERED, or CANCELLED (from PLACED, ACCEPTED or
 * READY). Any other transition is an IllegalStateException; an unknown id is an IllegalArgumentException.
 */
public interface FoodDeliveryApi {

    void addRestaurant(String id, String cuisine, int x, int y);

    /** A menu item with {@code stock} portions available. Re-adding an item replaces its price and adds to the stock. */
    void addItem(String restaurantId, String item, long price, int stock);

    void addPartner(String partnerId, int x, int y);

    /** Restaurants of the cuisine within {@code maxDistance} of (x, y), nearest first, ties by id. */
    List<String> searchRestaurants(String cuisine, int x, int y, int maxDistance);

    /**
     * Places an order for {@code item -> quantity} (all quantities >= 1) and reserves the stock, all-or-nothing.
     * IllegalStateException if any item lacks stock. Returns the order id.
     */
    String placeOrder(String customer, String restaurantId, Map<String, Integer> items);

    long totalOf(String orderId);

    String statusOf(String orderId);

    /** PLACED -> ACCEPTED. */
    void accept(String orderId);

    /**
     * ACCEPTED -> READY and the order looks for a delivery partner: the FREE partner nearest to the restaurant (ties
     * by partner id) gets it. If nobody is free the order waits (first come, first served) and is assigned the moment a
     * partner becomes free. A partner has at most one order at a time, from assignment until delivery or cancellation.
     */
    void markReady(String orderId);

    /** The partner assigned to the order, if any. */
    Optional<String> partnerOf(String orderId);

    /** READY (with a partner assigned) -> PICKED_UP. IllegalStateException if no partner is assigned yet. */
    void pickUp(String orderId);

    /** PICKED_UP -> DELIVERED; the partner becomes free (and takes the oldest waiting order, if any). */
    void deliver(String orderId);

    /** Cancels the order, restores its stock and frees its partner. Only the customer who placed it. */
    void cancel(String orderId, String customer);

    /** Portions of the item still available for new orders. */
    int stockOf(String restaurantId, String item);
}

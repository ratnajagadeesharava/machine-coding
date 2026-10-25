package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>All money is in whole cents (long). Every method throws {@link IllegalArgumentException} for an unknown id, a
 * blank/null string argument, or an argument outside the documented range; a call that throws changes nothing.
 */
public interface CartService {

    // ---- catalogue ----

    /** priceCents &gt; 0, stock &gt;= 0, non-blank sku/name/category. A second product with the same sku is an error. */
    void addProduct(String sku, String name, long priceCents, String category, int stock);

    /** Replaces the units on hand (n &gt;= 0). */
    void setStock(String sku, int n);

    /** Units currently on hand. */
    int stock(String sku);

    /** Tax rate of a category in basis points, 0..10000 (1 bp = 0.01%). Categories without a rate are taxed 0. */
    void setTaxRate(String category, int basisPoints);

    /** Promotion: in every complete group of x+y units, y units are free. x &gt;= 1, y &gt;= 1. Replaces any promotion of the sku. */
    void setBuyXGetY(String sku, int x, int y);

    /** Promotion: percent (integer 1..90) off the line subtotal. Replaces any promotion of the sku. */
    void setPercentOff(String sku, int percent);

    /** Removes the promotion of the sku (no-op if there is none). */
    void clearPromotion(String sku);

    // ---- coupons ----

    /**
     * kind is "PERCENT" (value 1..100) or "FIXED" (value &gt;= 1 cents). minSubtotalCents &gt;= 0, maxUses &gt;= 1.
     * A second coupon with the same code is an error.
     */
    void defineCoupon(String code, String kind, long value, long minSubtotalCents, int maxUses);

    /** Number of successful checkouts that have consumed the coupon so far. */
    int couponUses(String code);

    // ---- carts ----

    /** A second cart with the same id is an error. */
    void createCart(String cartId, String userId);

    /** Adds qty (&gt;= 1) units to the line of the sku (creating it). If the line would exceed the units on hand: error "insufficient stock", cart unchanged. */
    void addItem(String cartId, String sku, int qty);

    /** Sets the line to exactly qty (&gt;= 0) units; 0 removes the line. Same stock rule as addItem. */
    void setQuantity(String cartId, String sku, int qty);

    /** Removes the line of the sku; error if the cart has no such line. */
    void removeItem(String cartId, String sku);

    /** Units of the sku in the cart (0 if no line). */
    int quantity(String cartId, String sku);

    /** Replaces the cart's coupon with this one (the code must be defined). Eligibility is not checked here. */
    void applyCoupon(String cartId, String code);

    /** Detaches the cart's coupon (no-op if none). */
    void removeCoupon(String cartId);

    /** "NONE", "APPLIED" or "NOT_ELIGIBLE", evaluated now. */
    String couponStatus(String cartId);

    /** "subtotal|promoDiscount|couponDiscount|tax|total" in cents, evaluated now. */
    String total(String cartId);

    /**
     * Atomically: check stock of every line, decrement stock, consume the coupon use (if the coupon is eligible),
     * empty the cart (coupon included), record the order. Returns the order id ("ORD-1", "ORD-2", ... in checkout order).
     * Error "insufficient stock: SKU" naming the first lacking sku in ascending sku order (nothing changes);
     * error for an empty cart.
     */
    String checkout(String cartId);

    /** The user's orders in checkout order, each as "orderId|totalCents". Empty list for a user without orders. */
    List<String> orders(String userId);
}

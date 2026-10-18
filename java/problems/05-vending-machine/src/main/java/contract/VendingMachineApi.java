package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. */
public interface VendingMachineApi {

    void loadProduct(String slot, String name, int priceCents, int quantity);

    /** Adds coins of the given denomination (5, 10, 25 or 100) to the machine's inventory. */
    void loadCoins(int denomination, int count);

    /** Returns false for unsupported denominations. */
    boolean insertCoin(int denomination);

    /** DISPENSED, INVALID_SLOT, SOLD_OUT, INSUFFICIENT_FUNDS or CANNOT_MAKE_CHANGE. */
    String select(String slot);

    /** Refunds the inserted coins into the tray. */
    void cancel();

    /** Credit currently inserted. */
    int balance();

    /** Empties the tray; coins in descending order. */
    List<Integer> takeReturnedCoins();

    int stockOf(String slot);
}

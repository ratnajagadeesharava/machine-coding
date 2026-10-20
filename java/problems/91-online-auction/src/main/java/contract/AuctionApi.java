package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory (epoch milliseconds). Amounts are whole currency units.
 */
public interface AuctionApi {

    /**
     * Opens an auction that runs from now until {@code endTimeMs}. {@code startingPrice} is the lowest allowed first
     * bid; {@code reservePrice} (>= 0) is the price the winner must reach for the item to sell. IllegalArgumentException
     * for a duplicate id, non-positive starting price, or an end time that is not in the future.
     */
    void createAuction(String auctionId, String seller, long startingPrice, long reservePrice, long endTimeMs);

    /**
     * Places a bid. A bid is accepted (returns true) only if the auction is open, the bidder is not the seller,
     * and the amount is at least the starting price and strictly higher than the current highest bid. A lower or equal
     * bid is simply not accepted (returns false). A bid accepted in the last 5 minutes before the end moves the end to
     * "bid time + 5 minutes" (anti-sniping).
     * IllegalArgumentException for an unknown auction or amount <= 0; IllegalStateException if the auction has ended;
     * SecurityException if the bidder is the seller.
     */
    boolean placeBid(String auctionId, String bidder, long amount);

    /** "bidder:amount" of the current highest bid, if any. */
    Optional<String> highestBid(String auctionId);

    /** Accepted bids in the order they were accepted, "bidder:amount". */
    List<String> bidHistory(String auctionId);

    /** The (possibly extended) end time. */
    long endTimeOf(String auctionId);

    /**
     * Outcome of a finished auction: "SOLD:bidder:amount" if the highest bid reached the reserve, "RESERVE_NOT_MET",
     * or "NO_BIDS". IllegalStateException while the auction is still open.
     */
    String result(String auctionId);

    /** The seller may cancel an auction that has no bids yet. SecurityException for others; IllegalStateException with bids or after the end. */
    void cancel(String auctionId, String seller);

    /** Messages for a user, oldest first: "OUTBID:auctionId:newHighestAmount" each time another bidder takes the lead from them. */
    List<String> notifications(String user);
}

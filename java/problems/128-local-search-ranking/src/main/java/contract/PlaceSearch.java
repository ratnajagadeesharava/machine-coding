package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface PlaceSearch {

    /**
     * Adds a place. IllegalArgumentException if the id, name or category is null/blank, the id already exists,
     * ratingTimes10 is outside 0..50 or priceLevel is outside 1..4.
     */
    void addPlace(String placeId, String name, String category, int x, int y, int ratingTimes10, int priceLevel);

    /**
     * Replaces every attribute of an existing place in one step (same validation as addPlace).
     * IllegalArgumentException for an unknown id or invalid values; a failed update changes nothing.
     */
    void updatePlace(String placeId, String name, String category, int x, int y, int ratingTimes10, int priceLevel);

    /** Removes a place. IllegalArgumentException for an unknown id. The id may be added again later. */
    void removePlace(String placeId);

    /**
     * Ranked place ids, best first, at most k. Filters, score and tie-breaks are defined in the README.
     * query and category may be null (no filter). IllegalArgumentException if k &lt;= 0 or radius &lt; 0.
     */
    List<String> search(
            String query,
            int userX,
            int userY,
            int radius,
            String category,
            int minRatingTimes10,
            int maxPriceLevel,
            int k);

    /** The k places closest to the user (Manhattan distance), ties by placeId ascending. IllegalArgumentException if k &lt;= 0. */
    List<String> nearest(int userX, int userY, int k);

    /** One "category:count" entry per category that has at least one place, sorted by category ascending. */
    List<String> categories();
}

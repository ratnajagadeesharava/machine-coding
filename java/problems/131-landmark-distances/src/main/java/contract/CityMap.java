package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every method throws {@link IllegalArgumentException} for a null argument, an unknown location, or any other
 * invalid argument named below; a failed call changes nothing.
 */
public interface CityMap {

    /** Adds a location. Names are case-sensitive and used as given. Blank/null or already existing: error. */
    void addLocation(String name);

    /**
     * Adds an undirected road of {@code distance} (> 0) between two different, existing locations. Several roads may
     * join the same pair; the shortest one is the one that matters for routing. Self-loop or distance <= 0: error.
     */
    void addRoad(String a, String b, long distance);

    /**
     * Removes every road joining {@code a} and {@code b} and returns how many were removed (0 if there were none).
     * Both locations must exist.
     */
    int removeRoad(String a, String b);

    /**
     * Tags a location as a landmark of {@code type}. Types are trimmed and case-insensitive ("Cafe " == "cafe"); a
     * blank type is an error. A location may carry several types; tagging twice is harmless.
     */
    void tagLandmark(String location, String type);

    /** Removes the tag; returns true if it was present. Same type rules as {@link #tagLandmark}. */
    boolean untagLandmark(String location, String type);

    /** Length of the shortest route in total distance; 0 if {@code a} equals {@code b}; -1 if unreachable. */
    long shortestDistance(String a, String b);

    /**
     * Location names along a shortest route, {@code a} first and {@code b} last. {@code a == b} gives {@code [a]}.
     * Among routes of equal total distance the one whose list of names is lexicographically smallest wins (names
     * compared with {@code String.compareTo}, element by element). Empty list if unreachable.
     */
    List<String> shortestPath(String a, String b);

    /**
     * Up to {@code k} landmarks of {@code type} that can be reached from {@code source}, as {@code "location:distance"}
     * strings ordered by distance ascending, then location name ascending. The source counts (distance 0) if tagged.
     * {@code k <= 0}: error. A type nobody carries: empty list.
     */
    List<String> nearestLandmarks(String source, String type, int k);

    /** Names of all locations at distance <= {@code radius} from {@code source} (source included), sorted by name. radius < 0: error. */
    List<String> withinRadius(String source, long radius);
}

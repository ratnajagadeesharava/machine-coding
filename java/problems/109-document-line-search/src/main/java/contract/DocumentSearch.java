package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface DocumentSearch {

    /**
     * Ids of the live lines that contain every word of {@code query}, in increasing id order.
     * Words are maximal runs of letters/digits, compared case-insensitively and as whole words.
     * A query without any word (empty, blank, punctuation only) returns an empty list.
     * The returned list is a fresh copy the caller may modify. IllegalArgumentException if {@code query} is null.
     */
    List<Integer> search(String query);

    /**
     * Removes the line for good; every other line keeps its id and a deleted id is never reused.
     * Returns true if the line was live, false if the id is unknown or was already deleted.
     */
    boolean deleteLine(int id);
}

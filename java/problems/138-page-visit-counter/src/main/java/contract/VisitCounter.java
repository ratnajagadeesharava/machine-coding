package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every {@code url} argument is normalised first (see the README); every method that takes a url or visitor id throws
 * {@link IllegalArgumentException} for a null or blank value (and for a url whose host is empty after normalisation).
 * Visitor ids are compared exactly as given (case-sensitive, not trimmed).
 */
public interface VisitCounter {

    /** Records one visit of {@code visitorId} to the page. */
    void visit(String url, String visitorId);

    /** Number of visits to the page, 0 if it was never visited (or was reset). */
    long totalVisits(String url);

    /** Number of distinct visitor ids that visited the page. Exact. 0 for an unknown page. */
    long uniqueVisitors(String url);

    /** Number of visits to the page by this visitor, 0 if none. */
    long visitsBy(String url, String visitorId);

    /**
     * The {@code k} most visited pages as {@code "url:total"} (normalised url), ordered by total descending then url
     * ascending (plain string order). Fewer entries if there are fewer pages. {@code k <= 0} is an
     * {@link IllegalArgumentException}.
     */
    List<String> topPages(int k);

    /**
     * The {@code k} most frequent visitors of the page as {@code "visitorId:count"}, ordered by count descending then
     * visitor id ascending. Empty for an unknown page. {@code k <= 0} is an {@link IllegalArgumentException}.
     */
    List<String> topVisitors(String url, int k);

    /** Total number of visits over all pages. */
    long totalVisits();

    /** Number of pages that currently have at least one recorded visit. */
    int pageCount();

    /** Forgets the page and all its counts. Returns true if the page existed. */
    boolean reset(String url);

    /** Forgets every page. */
    void resetAll();

    /**
     * A point-in-time view of all pages as {@code "url:total:unique"} lines sorted by url ascending. The list is an
     * immutable copy: later visits do not change it, and it equals the state after some prefix of the serial history of
     * all visits (never a page counted "ahead of" another page that was updated earlier).
     */
    List<String> snapshot();

    /**
     * Like {@link #snapshot()}, but per visitor: {@code "url:visitorId:count"} lines sorted by url then visitor id
     * (plain string order). Taken at a single point in time as well.
     */
    List<String> snapshotDetail();
}

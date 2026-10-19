package contract;

import java.util.Set;

/** Thin test entry point. Your design lives behind it. May be called from several threads (separate crawls). */
public interface Crawler {

    /**
     * Crawls every page reachable from {@code start} and returns the set of all URLs reached (including
     * {@code start} and broken pages). Rules:
     * <ul>
     *   <li>every URL is fetched exactly once, however many pages link to it;</li>
     *   <li>at most {@code workers} fetches run at the same time;</li>
     *   <li>a page whose fetch throws counts as reached but contributes no links; the crawl goes on;</li>
     *   <li>returns only when no work is left, and leaves no thread running behind it.</li>
     * </ul>
     * IllegalArgumentException if workers < 1.
     */
    Set<String> crawl(String start, int workers, Fetcher fetcher);
}

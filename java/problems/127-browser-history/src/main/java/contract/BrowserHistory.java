package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every tab starts on {@code about:blank}. All urls are compared as exact, case-sensitive strings.
 * Unless a method says otherwise, an unknown (never issued or already closed) tab id, a null/blank url or a negative
 * count is an {@link IllegalArgumentException}, and a failed call changes nothing.
 */
public interface BrowserHistory {

    /**
     * Opens a tab and returns its id: 1, 2, 3, ... in order of opening, never reused, even after a close.
     * The new tab shows {@code about:blank} as its only entry; that opening is NOT recorded in the global history.
     */
    int openTab();

    /** Closes the tab. Its global history entries stay. The id is unknown from now on. */
    void closeTab(int tabId);

    /**
     * Navigates the tab to {@code url}: the tab's forward entries are discarded, the url becomes the new current
     * entry, and one visit is recorded in the global history stamped with the clock's current millis and the tab id.
     * If {@code url} equals the tab's current url the tab is left exactly as it was (no new entry, forward entries
     * kept), but the visit is still recorded globally.
     */
    void visit(int tabId, String url);

    /**
     * Moves the tab back by up to {@code steps} entries (fewer if there are not that many) and returns the url now
     * shown. Never fails because of a lack of entries; {@code steps == 0} just returns the current url.
     * IllegalArgumentException if {@code steps < 0}. The global history is not touched.
     */
    String back(int tabId, int steps);

    /** Mirror of {@link #back}: moves forward by up to {@code steps} entries and returns the url now shown. */
    String forward(int tabId, int steps);

    /** The url the tab shows now. */
    String currentUrl(int tabId);

    /** True if the tab has at least one earlier entry. */
    boolean canGoBack(int tabId);

    /** True if the tab has at least one later entry. */
    boolean canGoForward(int tabId);

    /**
     * The most recent {@code limit} global entries, most recent first, each formatted {@code "time|url|tabId"}
     * (for example {@code "1000|https://a.com|2"}). Fewer if fewer exist; {@code limit == 0} gives an empty list.
     * IllegalArgumentException if {@code limit < 0}.
     */
    List<String> history(int limit);

    /**
     * Global entries whose time lies in {@code [fromMs, toMs]} (both inclusive), most recent first, same format as
     * {@link #history}. IllegalArgumentException if {@code fromMs > toMs}.
     */
    List<String> historyBetween(long fromMs, long toMs);

    /**
     * Distinct urls of the global history that contain {@code substring}, ignoring case, ordered by their latest
     * visit, most recent first. IllegalArgumentException if {@code substring} is null or blank.
     */
    List<String> search(String substring);

    /**
     * At most {@code k} distinct urls of the global history ordered by number of global entries (most first), then by
     * the time of the latest entry (later first), then by url (ascending, natural String order).
     * IllegalArgumentException if {@code k < 0}.
     */
    List<String> mostVisited(int k);

    /**
     * Removes every global entry for {@code url} and returns how many were removed (0 if none). Open tabs keep their
     * back/forward entries.
     */
    int deleteFromHistory(String url);

    /** Removes every global entry with time {@code >= sinceMs} and returns how many were removed. */
    int clearHistory(long sinceMs);
}

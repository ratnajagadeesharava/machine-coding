package solution;

import contract.PageFetcher;
import java.util.Iterator;

/**
 * A feed that looks like a plain {@code Iterable<String>} but is backed by a paginated remote API.
 * Each call to {@link #iterator()} returns a NEW, independent iterator that starts again at page 0.
 */
public class PagedFeed implements Iterable<String> {

    /** Nothing is fetched here. */
    public PagedFeed(PageFetcher fetcher) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * The returned iterator is LAZY:
     * <ul>
     *   <li>creating it fetches nothing; the first hasNext()/next() fetches page 0;</li>
     *   <li>the next page is fetched only when the current page's items are used up AND somebody asks hasNext()/next() again;</li>
     *   <li>empty pages (even the first) are skipped by fetching the following page;</li>
     *   <li>hasNext() is idempotent (calling it twice does not skip anything or fetch twice);</li>
     *   <li>next() at the end throws NoSuchElementException;</li>
     *   <li>if the fetcher throws, the exception propagates from hasNext()/next() and the iterator stays usable:
     *       asking again retries the SAME page.</li>
     * </ul>
     */
    @Override
    public Iterator<String> iterator() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

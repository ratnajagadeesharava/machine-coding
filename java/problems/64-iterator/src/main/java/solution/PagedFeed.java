package solution;

import contract.PageFetcher;
import java.util.Iterator;

/** A feed of items backed by a paginated remote API. */
public class PagedFeed implements Iterable<String> {

    public PagedFeed(PageFetcher fetcher) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** See the README for the required behavior. */
    @Override
    public Iterator<String> iterator() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

package contract;

import java.util.List;

/**
 * Provided by the tests (the "internet"). Thread-safe, may be slow, may contain duplicate and self links.
 * Throws a RuntimeException for a broken page.
 */
public interface Fetcher {

    /** The outgoing links of a page. */
    List<String> links(String url);
}

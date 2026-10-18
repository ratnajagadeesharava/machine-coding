package contract;

/** Provided: the remote API. Every call is slow and costs money. It may throw a RuntimeException. */
public interface PageFetcher {

    /** Page numbers start at 0. A page may be empty even when {@code hasNext} is true. */
    Page fetch(int pageNumber);
}

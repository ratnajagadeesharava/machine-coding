package contract;

import java.util.Optional;

/** Provided subject interface: something that can fetch documents. The real store and every proxy implement it. */
public interface DocumentStore {

    /** The document text, or empty if there is no such document. */
    Optional<String> fetch(String id);
}

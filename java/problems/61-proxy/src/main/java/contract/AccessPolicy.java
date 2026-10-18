package contract;

/** Provided: decides who may read what. */
public interface AccessPolicy {

    boolean canRead(String user, String documentId);
}

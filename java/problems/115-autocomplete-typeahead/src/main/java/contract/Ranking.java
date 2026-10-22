package contract;

/** Rule used to order suggestions that start with the same prefix. */
public enum Ranking {
    /** Higher recorded count first. */
    FREQUENCY,
    /** Most recently recorded first (time comes from the clock handed to the factory). */
    RECENCY
}

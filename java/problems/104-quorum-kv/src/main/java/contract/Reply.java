package contract;

/** Result of a client request. {@code value} is set only for a successful get. */
public record Reply(Status status, String value) {

    public enum Status {
        /** not finished yet */
        PENDING,
        /** put: stored on a majority; get: majority answered and a value exists */
        OK,
        /** get: a majority answered and none of them has the key */
        NOT_FOUND,
        /** a majority did not answer within 1000 ms of the request */
        FAILED
    }
}

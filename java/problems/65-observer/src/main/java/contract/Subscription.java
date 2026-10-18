package contract;

/** Provided handle returned by subscribe: lets a subscriber stop listening. */
public interface Subscription {

    /** Stops notifications. Safe to call any number of times. */
    void cancel();

    boolean isActive();
}

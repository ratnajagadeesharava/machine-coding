package contract;

/** Provided contract: how one kind of vehicle is charged for parking. */
public interface FeePolicy {

    /** Whether this policy prices the given vehicle type (case-insensitive). */
    boolean supports(String vehicleType);

    /** Fee in cents for a stay of {@code minutes} (>= 0). */
    long fee(int minutes);
}

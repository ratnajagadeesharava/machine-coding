package contract;

/** Provided contract: how delivery is priced. */
public interface DeliveryFeePolicy {

    long fee(int distanceMeters);
}

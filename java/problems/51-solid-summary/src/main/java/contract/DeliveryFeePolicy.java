package contract;

/** Provided contract: how delivery is priced. New policies are added, existing code is not edited. */
public interface DeliveryFeePolicy {

    long fee(int distanceMeters);
}

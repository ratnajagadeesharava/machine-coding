package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface RideSharingApi {

    void addDriver(String driverId, int x, int y);

    /** Matches the nearest available driver within distance 10 and returns the ride id. */
    Optional<String> requestRide(String riderId, int pickupX, int pickupY, int dropX, int dropY);

    Optional<String> driverOf(String rideId);

    /** MATCHED, IN_PROGRESS, COMPLETED, CANCELLED or UNKNOWN. */
    String statusOf(String rideId);

    boolean startRide(String rideId);

    /** Returns the fare. */
    Optional<Long> completeRide(String rideId);

    boolean cancelRide(String rideId);

    /** "x,y" or empty. */
    Optional<String> driverLocation(String driverId);
}

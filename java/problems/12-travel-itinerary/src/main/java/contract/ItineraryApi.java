package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. */
public interface ItineraryApi {

    void createTrip(String tripId, int startDay, int endDay, long budget);

    /** Symmetric travel time between two locations; applies to future additions. */
    void setTravelMinutes(String locationA, String locationB, int minutes);

    boolean addActivity(String tripId, String activityId, String location,
                        int day, int startMinute, int endMinute, long cost);

    boolean removeActivity(String tripId, String activityId);

    /** Activity ids of the day ordered by start time. */
    List<String> agenda(String tripId, int day);

    long totalCost(String tripId);
}

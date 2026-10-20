package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Times are minutes (long) since some epoch; an event occupies the half-open range [start, end), so two events
 * that merely touch (one ends when the other starts) do not conflict. Users exist implicitly by being named.
 */
public interface CalendarApi {

    /**
     * Schedules an event for the organizer and the attendees and returns its id. IllegalArgumentException for
     * end <= start, a blank title, or duplicate people (the organizer may not be listed as an attendee).
     * IllegalStateException if ANY of the people already has an overlapping event; nothing is created then.
     */
    String schedule(String organizer, String title, long start, long end, List<String> attendees);

    /**
     * Schedules {@code count} occurrences (count >= 1) starting at start, each repeating {@code everyMinutes} later
     * (everyMinutes >= end - start so occurrences do not overlap). All-or-nothing: if any occurrence conflicts for
     * anybody, IllegalStateException and nothing is created. Returns the ids in time order.
     */
    List<String> scheduleRecurring(
            String organizer, String title, long start, long end, List<String> attendees, long everyMinutes, int count);

    /** Only the organizer may cancel; the event disappears for everybody. IllegalArgumentException for an unknown id, SecurityException for others. */
    void cancel(String user, String eventId);

    /** An attendee leaves the event; it stays for the others. The organizer cannot decline (SecurityException). */
    void decline(String user, String eventId);

    /** The user's events overlapping [from, to), sorted by start then title, formatted "start-end:title". */
    List<String> agenda(String user, long from, long to);

    /**
     * Windows inside [from, to) where ALL the users are free for at least {@code duration} minutes, merged and sorted,
     * formatted "start-end".
     */
    List<String> commonFreeSlots(List<String> users, long from, long to, long duration);
}

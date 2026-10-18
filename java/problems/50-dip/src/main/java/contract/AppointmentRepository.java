package contract;

import java.time.Instant;
import java.util.List;

/** Provided abstraction: where appointments live. Already-reminded appointments are not returned by findBetween. */
public interface AppointmentRepository {

    /** Appointments whose time is in [from, to] (both inclusive) that have not been reminded yet. */
    List<Appointment> findBetween(Instant from, Instant to);

    void markReminded(String appointmentId);
}

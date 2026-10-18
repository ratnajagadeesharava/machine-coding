package solution;

import contract.AppointmentRepository;
import contract.MessageSender;
import java.time.Clock;

/**
 * Sends reminders for appointments in the next 24 hours. It depends only on abstractions that are handed to it:
 * it never creates a gateway, a database connection or a clock itself.
 */
public class ReminderService {

    public ReminderService(MessageSender sender, AppointmentRepository repository, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * now = clock.instant(). Asks the repository for appointments in [now, now + 24h]. For each one sends
     * "Reminder: appointment at " + time (Instant#toString) to its phone. If the sender returns true (and does not
     * throw) the appointment is marked reminded and counted; otherwise it is NOT marked (it will be retried on a
     * later run) and the loop carries on with the others. Returns how many reminders were sent.
     */
    public int sendDueReminders() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

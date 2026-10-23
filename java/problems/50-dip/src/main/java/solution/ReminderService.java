package solution;

import contract.AppointmentRepository;
import contract.MessageSender;
import java.time.Clock;

/** Sends reminders for upcoming appointments. */
public class ReminderService {

    public ReminderService(MessageSender sender, AppointmentRepository repository, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int sendDueReminders() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

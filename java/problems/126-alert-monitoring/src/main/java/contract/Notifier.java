package contract;

/**
 * Delivery sink for alert notifications. PROVIDED: the tests implement it and record what they receive.
 * Not part of your design. It may throw any RuntimeException.
 */
public interface Notifier {

    /**
     * One delivery to one channel.
     *
     * @param channelName name the channel was registered with
     * @param ruleId      the rule that changed state
     * @param state       {@link AlertMonitor#FIRING} or {@link AlertMonitor#RESOLVED}
     * @param severity    the rule's severity ({@link AlertMonitor#INFO}, {@link AlertMonitor#WARN} or
     *                    {@link AlertMonitor#CRITICAL})
     * @param value       the rule's aggregate at the evaluation that caused the notification
     * @param timeMs      the clock time of that evaluation, in milliseconds
     */
    void notify(String channelName, String ruleId, String state, String severity, double value, long timeMs);
}

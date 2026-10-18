package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface PubSubApi {

    boolean createTopic(String topic);

    boolean subscribe(String topic, String subscriber);

    boolean unsubscribe(String topic, String subscriber);

    /** Number of subscribers the message was delivered to; -1 for an unknown topic. */
    int publish(String topic, String message);

    /** Up to max pending messages, oldest first, consumed by the call. */
    List<String> poll(String subscriber, String topic, int max);
}

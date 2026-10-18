package contract;

import java.util.function.Consumer;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface EventBusApi {

    /** Registers the handler for the topic and returns the subscription id. */
    String subscribe(String topic, Consumer<String> handler);

    /** True the first time for a known id; false for unknown or already removed ids. */
    boolean unsubscribe(String subscriptionId);

    /** Hands the event to every current subscriber of the topic; never waits for handlers. */
    void publish(String topic, String event);

    /** True once everything published so far (including by handlers) has been handled or discarded. */
    boolean awaitIdle(long timeoutMillis);

    /** Stops accepting new events; events already published are still delivered. Idempotent. */
    void close();
}

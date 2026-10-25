package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Errors: {@link IllegalArgumentException} for blank/null names, a null message, non-positive sizes, an unknown or
 * duplicate topic and a partition outside {@code 0..partitions-1}; {@link IllegalStateException} for a consumer that is
 * not a member of the group, a partition that is not assigned to the caller, and an invalid commit offset.
 * A failed call changes nothing.
 */
public interface StreamingService {

    /** Creates a topic with {@code partitions >= 1} partitions and unlimited retention. */
    void createTopic(String topic, int partitions);

    /**
     * Creates a topic whose every partition keeps at most {@code maxMessagesPerPartition >= 1} messages; when a
     * publish exceeds that, the oldest message of that partition is dropped.
     */
    void createTopic(String topic, int partitions, int maxMessagesPerPartition);

    /**
     * Appends a message and returns {@code "partition:offset"}. A non-null key (the empty string is a key) selects
     * the partition by {@code h = 31*h + c} over the key's UTF-16 code units (int arithmetic, overflow wraps),
     * partition {@code Math.floorMod(h, partitions)}. A null key uses the topic's round-robin counter (0, 1, 2, ...
     * modulo partitions, starting at 0, advanced only by key-less publishes). Offsets are per partition, start at 0
     * and have no gaps. The message may be empty but not null.
     */
    String publish(String topic, String key, String message);

    /** Adds the consumer to the group for the topic and rebalances. Joining twice is a no-op. */
    void joinGroup(String group, String consumer, String topic);

    /** Removes the consumer from the group for the topic and rebalances. IllegalStateException if not a member. */
    void leaveGroup(String group, String consumer, String topic);

    /** Partitions currently assigned to the consumer, ascending. IllegalStateException if not a member. */
    List<Integer> assignment(String group, String consumer, String topic);

    /**
     * Up to {@code max >= 1} messages from the consumer's assigned partitions, partition ascending then offset
     * ascending, starting per partition at {@code max(committed offset of the group, earliest offset)}. Each entry is
     * {@code "partition:offset:message"}. Does not change any committed offset.
     */
    List<String> poll(String group, String consumer, String topic, int max);

    /**
     * Sets the group's committed offset (the next offset to read) of a partition to {@code nextOffset}. The consumer
     * must be a member and the partition must be assigned to it. {@code nextOffset} must be at least the current
     * committed offset and at most the partition's latest offset, otherwise IllegalStateException.
     */
    void commit(String group, String consumer, String topic, int partition, long nextOffset);

    /** The group's committed offset of the partition; 0 if the group never committed. */
    long committedOffset(String group, String topic, int partition);

    /** Offset of the oldest retained message of the partition (equals latestOffset when it is empty). */
    long earliestOffset(String topic, int partition);

    /** Offset the next published message of the partition will get (0 for an empty, never-written partition). */
    long latestOffset(String topic, int partition);

    /** Number of retained messages in all partitions of the topic. */
    long messageCount(String topic);
}

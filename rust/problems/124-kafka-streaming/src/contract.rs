//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum StreamError {
    /// the topic does not exist
    UnknownTopic,
    /// createTopic for a name that is already taken
    TopicExists,
    /// blank name, zero partitions, zero retention, zero poll size, partition outside 0..partitions
    InvalidArgument,
    /// the consumer is not a member of the group for that topic
    NotMember,
    /// the partition is not assigned to the calling consumer
    NotAssigned,
    /// commit below the committed offset or past the partition's latest offset
    InvalidOffset,
}

/// Argument checks run in this order: blank names (InvalidArgument), unknown topic, other argument problems
/// (partition range, poll size, null-equivalents), group membership, assignment, offset. A failed call changes nothing.
pub trait StreamingService: Send + Sync {
    /// Creates a topic with `partitions >= 1` partitions. `max_messages_per_partition = Some(n)` (n >= 1) keeps at
    /// most n messages per partition: when a publish exceeds that, the oldest message of that partition is dropped.
    fn create_topic(
        &self,
        topic: &str,
        partitions: usize,
        max_messages_per_partition: Option<usize>,
    ) -> Result<(), StreamError>;

    /// Appends a message and returns `"partition:offset"`. `Some(key)` (the empty string is a key) selects the
    /// partition by `h = 31*h + c` over the key's UTF-16 code units (i32 arithmetic, overflow wraps), partition
    /// `h.rem_euclid(partitions)`. `None` uses the topic's round-robin counter (0, 1, 2, ... modulo partitions,
    /// starting at 0, advanced only by key-less publishes). Offsets are per partition, start at 0 and have no gaps.
    /// The message may be empty.
    fn publish(&self, topic: &str, key: Option<&str>, message: &str)
        -> Result<String, StreamError>;

    /// Adds the consumer to the group for the topic and rebalances. Joining twice is a no-op.
    fn join_group(&self, group: &str, consumer: &str, topic: &str) -> Result<(), StreamError>;

    /// Removes the consumer from the group for the topic and rebalances. `NotMember` if it is not a member.
    fn leave_group(&self, group: &str, consumer: &str, topic: &str) -> Result<(), StreamError>;

    /// Partitions currently assigned to the consumer, ascending. `NotMember` if it is not a member.
    fn assignment(
        &self,
        group: &str,
        consumer: &str,
        topic: &str,
    ) -> Result<Vec<usize>, StreamError>;

    /// Up to `max >= 1` messages from the consumer's assigned partitions, partition ascending then offset
    /// ascending, starting per partition at `max(committed offset of the group, earliest offset)`. Each entry is
    /// `"partition:offset:message"`. Does not change any committed offset.
    fn poll(
        &self,
        group: &str,
        consumer: &str,
        topic: &str,
        max: usize,
    ) -> Result<Vec<String>, StreamError>;

    /// Sets the group's committed offset (the next offset to read) of a partition. The consumer must be a member
    /// and the partition must be assigned to it. `next_offset` must be at least the current committed offset and at
    /// most the partition's latest offset, otherwise `InvalidOffset`.
    fn commit(
        &self,
        group: &str,
        consumer: &str,
        topic: &str,
        partition: usize,
        next_offset: u64,
    ) -> Result<(), StreamError>;

    /// The group's committed offset of the partition; 0 if the group never committed.
    fn committed_offset(
        &self,
        group: &str,
        topic: &str,
        partition: usize,
    ) -> Result<u64, StreamError>;

    /// Offset of the oldest retained message of the partition (equals `latest_offset` when it is empty).
    fn earliest_offset(&self, topic: &str, partition: usize) -> Result<u64, StreamError>;

    /// Offset the next published message of the partition will get (0 for a never-written partition).
    fn latest_offset(&self, topic: &str, partition: usize) -> Result<u64, StreamError>;

    /// Number of retained messages in all partitions of the topic.
    fn message_count(&self, topic: &str) -> Result<u64, StreamError>;
}

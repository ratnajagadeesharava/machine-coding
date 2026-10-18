package testkit;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneId;
import java.time.ZoneOffset;
import java.util.concurrent.atomic.AtomicLong;

/** Manually advanced clock. Thread-safe. Epoch-millis based, UTC. */
public final class FakeClock extends Clock {
    private final AtomicLong millis;

    public FakeClock() {
        this(0L);
    }

    public FakeClock(long startMillis) {
        this.millis = new AtomicLong(startMillis);
    }

    public void advance(long ms) {
        millis.addAndGet(ms);
    }

    public void advance(Duration d) {
        advance(d.toMillis());
    }

    public void set(long ms) {
        millis.set(ms);
    }

    @Override
    public long millis() {
        return millis.get();
    }

    @Override
    public Instant instant() {
        return Instant.ofEpochMilli(millis.get());
    }

    @Override
    public ZoneId getZone() {
        return ZoneOffset.UTC;
    }

    @Override
    public Clock withZone(ZoneId zone) {
        return this;
    }
}

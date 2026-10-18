package contract;

import java.time.Instant;

/** Provided value type. */
public record Appointment(String id, String phone, Instant time) {}

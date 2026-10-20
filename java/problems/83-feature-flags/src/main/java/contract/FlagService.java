package contract;

import java.util.Map;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface FlagService {

    /** Creates a boolean flag. IllegalArgumentException if the name is blank or the flag already exists. */
    void define(String flag, boolean defaultValue);

    /**
     * Appends a targeting rule: users whose attribute {@code attribute} equals {@code value} get {@code result}.
     * Rules are evaluated in the order they were added; the first matching rule wins.
     */
    void addRule(String flag, String attribute, String value, boolean result);

    /**
     * Percentage rollout 0..100 for users that match no rule. Once a rollout is set, users inside the percentage get
     * true and everybody else false (the default value is ignored). Each user must land in the same bucket every time
     * for the same flag, buckets of different flags are independent, and raising the percentage never turns anybody off.
     */
    void setRollout(String flag, int percent);

    /** A killed flag is false for everybody, whatever its rules, rollout or default say. Reversible. */
    void setKilled(String flag, boolean killed);

    /**
     * Evaluation order: killed -> first matching rule -> rollout (if set) -> default.
     * IllegalArgumentException for an unknown flag or a null/blank userId. Attributes may be empty but not null.
     */
    boolean isEnabled(String flag, String userId, Map<String, String> attributes);
}

package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface AlertMonitor {

    String GT = "GT";
    String GTE = "GTE";
    String LT = "LT";
    String LTE = "LTE";

    String AVG = "AVG";
    String MAX = "MAX";
    String MIN = "MIN";
    String COUNT = "COUNT";
    String SUM = "SUM";

    /** Severities, lowest to highest: INFO &lt; WARN &lt; CRITICAL. */
    String INFO = "INFO";
    String WARN = "WARN";
    String CRITICAL = "CRITICAL";

    /** Rule states. */
    String OK = "OK";
    String PENDING = "PENDING";
    String FIRING = "FIRING";
    String RESOLVED = "RESOLVED";

    /**
     * Stores a sample of the metric stamped with the clock's current time in milliseconds. Thread-safe; callable
     * while other threads record or evaluate. IllegalArgumentException for a null/blank metric or a NaN/infinite
     * value (nothing is stored).
     */
    void record(String metric, double value);

    /**
     * Registers a rule in state {@link #OK} with an empty history. Its condition is
     * {@code aggregation(samples of metric in (now - windowMs, now]) comparator threshold}.
     * IllegalArgumentException for: a null/blank rule id or metric, an id that already exists, an unknown comparator,
     * aggregation or severity name (case-sensitive), a NaN/infinite threshold, windowMs &lt;= 0, forMs &lt; 0 or
     * cooldownMs &lt; 0. Nothing changes on error.
     */
    void defineRule(String ruleId, String metric, String comparator, double threshold, long windowMs,
                    String aggregation, String severity, long forMs, long cooldownMs);

    /** Removes the rule with its state, history, silence and counters. IllegalArgumentException if unknown. */
    void deleteRule(String ruleId);

    /**
     * Registers a channel that receives every notification whose severity is &gt;= minSeverity.
     * IllegalArgumentException for a null/blank name, a name already registered or an unknown severity name.
     */
    void registerChannel(String name, String minSeverity);

    /** Evaluates every rule once at the clock's current time (see the README for the state machine). */
    void evaluate();

    /** Current state of the rule. IllegalArgumentException if unknown. */
    String state(String ruleId);

    /**
     * Ids of the rules currently in state {@link #FIRING} (silenced or not), highest severity first, then rule id
     * ascending (plain string order).
     */
    List<String> activeAlerts();

    /**
     * Every state transition of the rule, oldest first, as {@code "<timeMs>:<STATE>"} where STATE is the state
     * entered (for example {@code "61000:FIRING"}). IllegalArgumentException if unknown.
     */
    List<String> history(String ruleId);

    /**
     * Suppresses notifications for the rule while {@code clock time < untilMs}. State tracking continues. A later call
     * replaces the earlier one. IllegalArgumentException if the rule is unknown.
     */
    void silence(String ruleId, long untilMs);

    /** Number of samples currently retained for the metric (0 for a metric never recorded). */
    int sampleCount(String metric);

    /**
     * How many FIRING notifications of the rule were withheld because of its cooldown.
     * IllegalArgumentException if unknown.
     */
    int suppressedCount(String ruleId);
}

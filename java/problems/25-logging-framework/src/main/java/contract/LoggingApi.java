package contract;

/** Thin test entry point. Your domain model lives behind it. */
public interface LoggingApi {

    /** Levels: TRACE, DEBUG, INFO, WARN, ERROR. The root logger is named "root". False if invalid. */
    boolean setLevel(String loggerName, String level);

    /** Template placeholders are "{}". Ignored for an unknown level or blank logger name. */
    void log(String loggerName, String level, String template, String... args);

    /** False if the name is blank or taken, or the level is unknown. */
    boolean addAppender(String name, LogSink sink, String minLevel);

    boolean removeAppender(String name);
}

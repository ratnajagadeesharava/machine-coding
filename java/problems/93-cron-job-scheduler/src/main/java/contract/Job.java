package contract;

/** The work a job does. Provided by the tests. A job fails by throwing. */
@FunctionalInterface
public interface Job {
    void run() throws Exception;
}

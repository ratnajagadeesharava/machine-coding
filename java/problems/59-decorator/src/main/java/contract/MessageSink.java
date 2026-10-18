package contract;

/** Provided component interface. A sink may throw a RuntimeException when delivery fails. */
public interface MessageSink {

    void send(String message);
}

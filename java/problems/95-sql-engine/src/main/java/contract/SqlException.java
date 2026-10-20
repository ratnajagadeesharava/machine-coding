package contract;

/** Any syntax or semantic error. The statement that caused it changed nothing. Provided. */
public class SqlException extends RuntimeException {
    public SqlException(String message) {
        super(message);
    }
}

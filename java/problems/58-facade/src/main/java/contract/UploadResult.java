package contract;

/** Provided value type. {@code url} is "" unless the status is OK. */
public record UploadResult(String status, String url) {}

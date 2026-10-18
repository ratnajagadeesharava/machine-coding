package contract;

/** Provided value type: what an import run did. */
public record ImportReport(int read, int saved, int skipped, int failed) {}

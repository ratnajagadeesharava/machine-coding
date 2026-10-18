package contract;

import java.util.List;

/** Provided value type: one page of results and whether another page follows. */
public record Page(List<String> items, boolean hasNext) {}

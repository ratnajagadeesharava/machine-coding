package solution;

import contract.Element;
import contract.Visitor;
import java.util.List;

/**
 * A grid of cells. The rows must be non-empty, every row must have the same non-zero number of cells, and no cell
 * may be null, otherwise IllegalArgumentException. The table keeps its OWN copy of the rows: changing the lists
 * that were passed in afterwards must not change the table.
 */
public class Table implements Element {

    public Table(List<List<String>> rows) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Calls {@code visitor.visitTable(rows)} and returns its result. */
    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

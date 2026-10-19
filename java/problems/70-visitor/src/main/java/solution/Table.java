package solution;

import contract.Element;
import contract.Visitor;
import java.util.List;

/** A grid of cells. */
public class Table implements Element {

    public Table(List<List<String>> rows) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

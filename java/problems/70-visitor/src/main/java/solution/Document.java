package solution;

import contract.Element;
import contract.Visitor;
import java.util.List;

/** An ordered list of elements. */
public class Document {

    /** Null element -> IllegalArgumentException. */
    public void add(Element element) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int size() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Applies the visitor to every element; results in order. */
    public <R> List<R> visitAll(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The HTML of every element joined with "\n". Empty document -> "". */
    public String toHtml() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Total number of words. */
    public int totalWords() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

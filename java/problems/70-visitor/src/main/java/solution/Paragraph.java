package solution;

import contract.Element;
import contract.Visitor;

/** A block of text. */
public class Paragraph implements Element {

    public Paragraph(String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

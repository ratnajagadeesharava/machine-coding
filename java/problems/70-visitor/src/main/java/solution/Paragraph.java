package solution;

import contract.Element;
import contract.Visitor;

/** A block of text. Null text -> IllegalArgumentException. */
public class Paragraph implements Element {

    public Paragraph(String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Calls {@code visitor.visitParagraph(text)} and returns its result. */
    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

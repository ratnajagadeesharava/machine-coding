package solution;

import contract.Element;
import contract.Visitor;

/** An image. Blank url, or width/height <= 0 -> IllegalArgumentException. */
public class Image implements Element {

    public Image(String url, int width, int height) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Calls {@code visitor.visitImage(url, width, height)} and returns its result. */
    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

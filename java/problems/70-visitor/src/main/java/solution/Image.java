package solution;

import contract.Element;
import contract.Visitor;

/** An image. */
public class Image implements Element {

    public Image(String url, int width, int height) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public <R> R accept(Visitor<R> visitor) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

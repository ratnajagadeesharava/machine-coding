package contract;

import java.util.List;

/**
 * Provided visitor interface: one method per kind of document element, each returning an {@code R}. A new
 * operation on documents = a new Visitor class; the element classes never change.
 */
public interface Visitor<R> {

    R visitParagraph(String text);

    R visitImage(String url, int width, int height);

    /** {@code rows} is a rectangular, non-empty grid of cell texts. */
    R visitTable(List<List<String>> rows);
}

package contract;

import java.util.List;

/** Provided visitor interface: one method per kind of document element. */
public interface Visitor<R> {

    R visitParagraph(String text);

    R visitImage(String url, int width, int height);

    /** {@code rows} is a rectangular, non-empty grid of cell texts. */
    R visitTable(List<List<String>> rows);
}

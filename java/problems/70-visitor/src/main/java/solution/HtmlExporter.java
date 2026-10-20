package solution;

import contract.Visitor;
import java.util.List;

/** Turns elements into HTML. */
public class HtmlExporter implements Visitor<String> {

    @Override
    public String visitParagraph(String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String visitImage(String url, int width, int height) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String visitTable(List<List<String>> rows) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

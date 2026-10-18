package solution;

import contract.Visitor;
import java.util.List;

/** Counts words (runs of non-whitespace characters): a paragraph counts its words, an image 0, a table all words in all cells. */
public class WordCounter implements Visitor<Integer> {

    @Override
    public Integer visitParagraph(String text) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Integer visitImage(String url, int width, int height) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Integer visitTable(List<List<String>> rows) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

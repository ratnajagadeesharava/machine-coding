package solution;

import contract.Visitor;
import java.util.List;

/**
 * Turns elements into HTML. Text and attribute values are escaped: {@code &} -> {@code &amp;} (first), {@code <} ->
 * {@code &lt;}, {@code >} -> {@code &gt;}, {@code "} -> {@code &quot;}.
 * <ul>
 *   <li>paragraph: {@code <p>TEXT</p>}</li>
 *   <li>image: {@code <img src="URL" width="W" height="H"/>}</li>
 *   <li>table: {@code <table><tr><td>A</td><td>B</td></tr><tr>...</tr></table>}</li>
 * </ul>
 */
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

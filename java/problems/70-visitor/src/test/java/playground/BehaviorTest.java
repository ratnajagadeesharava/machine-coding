package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Element;
import contract.Visitor;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Document;
import solution.HtmlExporter;
import solution.Image;
import solution.Paragraph;
import solution.Table;
import solution.WordCounter;

class BehaviorTest {

    /** Tells which visit method was called with which data - the double dispatch made visible. */
    private static final class Trace implements Visitor<String> {
        public String visitParagraph(String text) { return "P(" + text + ")"; }
        public String visitImage(String url, int width, int height) { return "I(" + url + "," + width + "x" + height + ")"; }
        public String visitTable(List<List<String>> rows) { return "T(" + rows + ")"; }
    }

    /** A brand-new operation written in the test: it needs no change to any element class. */
    private static final class ImageCounter implements Visitor<Integer> {
        public Integer visitParagraph(String text) { return 0; }
        public Integer visitImage(String url, int width, int height) { return 1; }
        public Integer visitTable(List<List<String>> rows) { return 0; }
    }

    private static Document sample() {
        Document doc = new Document();
        doc.add(new Paragraph("Hello big world"));
        doc.add(new Image("cat.png", 640, 480));
        doc.add(new Table(List.of(List.of("a b", "c"), List.of("d", "e f g"))));
        doc.add(new Paragraph("The end"));
        return doc;
    }

    @Test
    void shouldDispatchToTheMethodThatMatchesTheElementType() {
        Visitor<String> trace = new Trace();

        assertThat(new Paragraph("hi").accept(trace)).isEqualTo("P(hi)");
        assertThat(new Image("u.png", 3, 4).accept(trace)).isEqualTo("I(u.png,3x4)");
        assertThat(new Table(List.of(List.of("x", "y"))).accept(trace)).isEqualTo("T([[x, y]])");
    }

    @Test
    void shouldExportEachElementAsHtml() {
        Element[] elements = {new Paragraph("Fish & Chips <3"), new Image("a\"b.png", 10, 20),
                new Table(List.of(List.of("1", "2"), List.of("3", "4")))};
        HtmlExporter html = new HtmlExporter();

        assertThat(elements[0].accept(html)).isEqualTo("<p>Fish &amp; Chips &lt;3</p>");
        assertThat(elements[1].accept(html)).isEqualTo("<img src=\"a&quot;b.png\" width=\"10\" height=\"20\"/>");
        assertThat(elements[2].accept(html)).isEqualTo("<table><tr><td>1</td><td>2</td></tr><tr><td>3</td><td>4</td></tr></table>");
    }

    @Test
    void shouldCountWordsInParagraphsAndTableCellsButNotImages() {
        WordCounter counter = new WordCounter();

        assertThat(new Paragraph("  two   words ").accept(counter)).isEqualTo(2);
        assertThat(new Paragraph("").accept(counter)).isZero();
        assertThat(new Image("x.png", 1, 1).accept(counter)).isZero();
        assertThat(new Table(List.of(List.of("a b", "c"), List.of("d", "e f g"))).accept(counter)).isEqualTo(7);
    }

    @Test
    void shouldApplyAVisitorToEveryElementInOrder() {
        List<String> trace = sample().visitAll(new Trace());

        assertThat(trace).containsExactly("P(Hello big world)", "I(cat.png,640x480)", "T([[a b, c], [d, e f g]])", "P(The end)");
    }

    @Test
    void shouldSupportANewOperationWithoutChangingAnyElementClass() {
        List<Integer> flags = sample().visitAll(new ImageCounter());

        assertThat(flags).containsExactly(0, 1, 0, 0);
        assertThat(flags.stream().mapToInt(Integer::intValue).sum()).isEqualTo(1);
    }

    @Test
    void shouldBuildWholeDocumentsFromTheVisitors() {
        Document doc = sample();

        assertThat(doc.size()).isEqualTo(4);
        assertThat(doc.totalWords()).isEqualTo(3 + 0 + 7 + 2);
        assertThat(doc.toHtml()).isEqualTo(String.join("\n",
                "<p>Hello big world</p>",
                "<img src=\"cat.png\" width=\"640\" height=\"480\"/>",
                "<table><tr><td>a b</td><td>c</td></tr><tr><td>d</td><td>e f g</td></tr></table>",
                "<p>The end</p>"));
        assertThat(new Document().toHtml()).isEmpty();
        assertThat(new Document().totalWords()).isZero();
    }

    @Test
    void shouldKeepTablesIndependentOfTheListsTheyWereBuiltFrom() {
        List<String> row = new ArrayList<>(List.of("a", "b"));
        List<List<String>> rows = new ArrayList<>(List.of(row));
        Table table = new Table(rows);

        row.set(0, "CHANGED");
        rows.add(List.of("x", "y"));

        assertThat(table.accept(new Trace())).isEqualTo("T([[a, b]])");
    }

    @Test
    void shouldRejectInvalidElements() {
        assertThatThrownBy(() -> new Paragraph(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Image(" ", 1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Image("u", 0, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Image("u", 1, -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Table(List.of())).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Table(List.of(List.of("a"), List.of("b", "c")))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Table(List.of(List.of()))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Document().add(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(new Paragraph("ok").accept(new Trace())).isEqualTo("P(ok)");
    }
}

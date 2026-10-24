package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.WordProcessor;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final String DEF = "Calibri|11|-|-|-|black";

    /** Deliberately naive reference: one format string and one char per position. */
    private static final class Model {
        final List<Character> chars = new ArrayList<>();
        final List<String[]> fmts = new ArrayList<>(); // font,size,B,I,U,color
        final List<String> aligns = new ArrayList<>(List.of("LEFT"));

        static String[] def() {
            return new String[] {"Calibri", "11", "-", "-", "-", "black"};
        }

        int newlines(int from, int to) {
            int k = 0;
            for (int i = from; i < to; i++) {
                if (chars.get(i) == '\n') k++;
            }
            return k;
        }

        void insert(int pos, String text, String[] f) {
            if (text.isEmpty()) return;
            int k = (int) text.chars().filter(c -> c == '\n').count();
            if (k > 0) {
                int line = newlines(0, pos);
                String a = aligns.get(line);
                for (int j = 0; j < k; j++) aligns.add(line + 1, a);
            }
            for (int i = 0; i < text.length(); i++) {
                chars.add(pos + i, text.charAt(i));
                fmts.add(pos + i, f.clone());
            }
        }

        void insert(int pos, String text) {
            insert(pos, text, pos == 0 ? def() : fmts.get(pos - 1));
        }

        void delete(int from, int to) {
            int k = newlines(from, to);
            if (k > 0) {
                int line = newlines(0, from);
                for (int j = 0; j < k; j++) aligns.remove(line + 1);
            }
            for (int i = to - 1; i >= from; i--) {
                chars.remove(i);
                fmts.remove(i);
            }
        }

        String str() {
            StringBuilder sb = new StringBuilder();
            chars.forEach(sb::append);
            return sb.toString();
        }

        static String f(String[] a) {
            return a[0] + "|" + a[1] + "|" + a[2] + "|" + a[3] + "|" + a[4] + "|" + a[5];
        }

        List<String> runs() {
            List<String> out = new ArrayList<>();
            int s = 0;
            for (int i = 1; i <= chars.size(); i++) {
                if (i == chars.size() || !f(fmts.get(i)).equals(f(fmts.get(s)))) {
                    out.add(s + "-" + i + ":" + f(fmts.get(s)));
                    s = i;
                }
            }
            return out;
        }

        List<Integer> find(String p, boolean cs) {
            String t = cs ? str() : str().toLowerCase();
            String q = cs ? p : p.toLowerCase();
            List<Integer> out = new ArrayList<>();
            for (int i = t.indexOf(q); i >= 0; i = t.indexOf(q, i + 1)) out.add(i);
            return out;
        }

        int replaceAll(String p, String r, boolean cs) {
            List<Integer> ms = find(p, cs);
            List<Integer> chosen = new ArrayList<>();
            int next = 0;
            for (int m : ms) {
                if (m >= next) {
                    chosen.add(m);
                    next = m + p.length();
                }
            }
            for (int j = chosen.size() - 1; j >= 0; j--) {
                int s = chosen.get(j);
                String[] f = fmts.get(s).clone();
                delete(s, s + p.length());
                insert(s, r, f);
            }
            return chosen.size();
        }
    }

    private static void assertSame(WordProcessor d, Model m) {
        assertThat(d.text()).isEqualTo(m.str());
        assertThat(d.length()).isEqualTo(m.chars.size());
        assertThat(d.runs()).isEqualTo(m.runs());
        assertThat(d.lineCount()).isEqualTo(m.aligns.size());
        for (int l = 0; l < m.aligns.size(); l++) {
            assertThat(d.alignment(l)).as("line " + l).isEqualTo(m.aligns.get(l));
        }
        Set<String> distinct = new HashSet<>();
        m.fmts.forEach(f -> distinct.add(Model.f(f)));
        assertThat(d.distinctFormatCount()).isEqualTo(distinct.size());
    }

    private static String randomText(Random rnd, int max) {
        String alphabet = "abAB  \n";
        int n = rnd.nextInt(max + 1);
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < n; i++) sb.append(alphabet.charAt(rnd.nextInt(alphabet.length())));
        return sb.toString();
    }

    @Test
    void shouldAgreeWithANaiveModelOverThousandsOfRandomOperations() {
        for (long seed = 1; seed <= 5; seed++) {
            WordProcessor d = Candidate.of(Solution::create);
            Model m = new Model();
            Random rnd = new Random(seed);
            String[] colors = {"red", "blue", "black"};
            String[] fonts = {"Arial", "Calibri"};
            for (int step = 0; step < 1_500; step++) {
                int len = m.chars.size();
                switch (rnd.nextInt(8)) {
                    case 0, 1 -> {
                        int pos = rnd.nextInt(len + 1);
                        String t = randomText(rnd, 6);
                        d.insert(pos, t);
                        m.insert(pos, t);
                    }
                    case 2 -> {
                        int a = rnd.nextInt(len + 1);
                        int b = Math.min(len, a + rnd.nextInt(8));
                        d.delete(a, b);
                        m.delete(a, b);
                    }
                    case 3, 4 -> {
                        int a = rnd.nextInt(len + 1);
                        int b = a + rnd.nextInt(len - a + 1);
                        String color = colors[rnd.nextInt(3)];
                        String font = fonts[rnd.nextInt(2)];
                        boolean bold = rnd.nextBoolean();
                        d.applyFormat(a, b, Map.of("color", color, "font", font, "bold", String.valueOf(bold)));
                        for (int i = a; i < b; i++) {
                            String[] f = m.fmts.get(i);
                            f[5] = color;
                            f[0] = font;
                            f[2] = bold ? "B" : "-";
                        }
                    }
                    case 5 -> {
                        String p = new String[] {"a", "ab", "aa", "b\n", "A"}[rnd.nextInt(5)];
                        boolean cs = rnd.nextBoolean();
                        assertThat(d.find(p, cs)).isEqualTo(m.find(p, cs));
                        String r = randomText(rnd, 3);
                        assertThat(d.replaceAll(p, r, cs)).isEqualTo(m.replaceAll(p, r, cs));
                    }
                    case 6 -> {
                        int line = rnd.nextInt(m.aligns.size());
                        String a = new String[] {"LEFT", "CENTER", "RIGHT", "JUSTIFY"}[rnd.nextInt(4)];
                        d.setAlignment(line, a);
                        m.aligns.set(line, a);
                    }
                    default -> {
                        if (len > 0) {
                            int p = rnd.nextInt(len);
                            assertThat(d.formatAt(p)).isEqualTo(Model.f(m.fmts.get(p)));
                        }
                    }
                }
                if (step % 50 == 0) assertSame(d, m);
            }
            assertSame(d, m);
        }
    }

    @Test
    void shouldCollapseAHeavilyFragmentedDocumentBackToOneRun() {
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "x".repeat(20_000));
        for (int i = 0; i < 20_000; i += 2) {
            d.applyFormat(i, i + 1, Map.of("bold", "true"));
        }
        assertThat(d.runs()).hasSize(20_000);
        assertThat(d.distinctFormatCount()).isEqualTo(2);

        for (int i = 0; i < 20_000; i += 2) {
            d.applyFormat(i, i + 1, Map.of("bold", "false"));
        }

        assertThat(d.runs()).containsExactly("0-20000:" + DEF);
        assertThat(d.distinctFormatCount()).isEqualTo(1);
    }

    @Test
    void shouldDeleteAcrossSeveralRunsAndLines() {
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "aa\nbb\ncc\ndd");
        d.applyFormat(0, 2, Map.of("bold", "true"));
        d.applyFormat(3, 5, Map.of("italic", "true"));
        d.applyFormat(6, 8, Map.of("underline", "true"));
        d.setAlignment(0, "CENTER");
        d.setAlignment(1, "RIGHT");
        d.setAlignment(2, "JUSTIFY");

        d.delete(1, 7);

        assertThat(d.text()).isEqualTo("ac\ndd");
        assertThat(d.runs()).containsExactly(
                "0-1:Calibri|11|B|-|-|black",
                "1-2:Calibri|11|-|-|U|black",
                "2-5:" + DEF);
        assertThat(d.lineCount()).isEqualTo(2);
        assertThat(d.alignment(0)).isEqualTo("CENTER");
        assertThat(d.alignment(1)).isEqualTo("LEFT");
    }

    @Test
    void shouldInheritAlignmentWhenReplacementsContainNewlines() {
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "a,b\nc,d");
        d.setAlignment(0, "CENTER");
        d.setAlignment(1, "RIGHT");

        assertThat(d.replaceAll(",", "\n", true)).isEqualTo(2);

        assertThat(d.text()).isEqualTo("a\nb\nc\nd");
        assertThat(d.lineCount()).isEqualTo(4);
        assertThat(d.alignment(0)).isEqualTo("CENTER");
        assertThat(d.alignment(1)).isEqualTo("CENTER");
        assertThat(d.alignment(2)).isEqualTo("RIGHT");
        assertThat(d.alignment(3)).isEqualTo("RIGHT");

        assertThat(d.replaceAll("\n", " ", true)).isEqualTo(3);
        assertThat(d.text()).isEqualTo("a b c d");
        assertThat(d.lineCount()).isEqualTo(1);
        assertThat(d.alignment(0)).isEqualTo("CENTER");
    }

    @Test
    void shouldLeaveTheDocumentUntouchedWhenAnyOperationFails() {
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "one\ntwo");
        d.applyFormat(0, 3, Map.of("bold", "true"));
        d.setAlignment(1, "RIGHT");
        List<String> runs = d.runs();

        try { d.delete(3, 99); } catch (IllegalArgumentException ignored) { }
        try { d.insert(99, "x\ny"); } catch (IllegalArgumentException ignored) { }
        try { d.applyFormat(0, 7, Map.of("color", "red", "size", "-1")); } catch (IllegalArgumentException ignored) { }
        try { d.replaceAll("o", null, true); } catch (IllegalArgumentException ignored) { }
        try { d.setAlignment(5, "LEFT"); } catch (IllegalArgumentException ignored) { }

        assertThat(d.text()).isEqualTo("one\ntwo");
        assertThat(d.runs()).isEqualTo(runs);
        assertThat(d.lineCount()).isEqualTo(2);
        assertThat(d.alignment(1)).isEqualTo("RIGHT");
    }

    @Test
    void shouldHandlePatternsLongerThanTheTextAndTheWholeText() {
        WordProcessor d = Candidate.of(Solution::create);

        assertThat(d.find("a", true)).isEmpty();
        assertThat(d.replaceAll("a", "b", true)).isZero();

        d.insert(0, "abc");
        assertThat(d.find("abcd", true)).isEmpty();
        assertThat(d.replaceAll("abcd", "x", true)).isZero();
        assertThat(d.replaceAll("abc", "xyz123", true)).isEqualTo(1);
        assertThat(d.text()).isEqualTo("xyz123");
        assertThat(d.runs()).containsExactly("0-6:" + DEF);
    }

    @Test
    void shouldReplaceInAMillionCharacterDocumentQuickly() {
        long start = System.nanoTime();
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "abcdefghij".repeat(100_000));
        for (int i = 0; i < 10; i++) {
            d.applyFormat(i * 100_000, i * 100_000 + 50_000, Map.of("color", "c" + i));
        }
        assertThat(d.runs()).hasSize(20);

        assertThat(d.find("jab", true)).hasSize(99_999);
        for (int i = 0; i < 50; i++) {
            d.insert(i * 16_000 + 7, "NEEDLE");
        }
        assertThat(d.runs()).hasSize(20);
        assertThat(d.find("NEEDLE", true)).hasSize(50);
        assertThat(d.replaceAll("needle", "N", false)).isEqualTo(50);

        assertThat(d.length()).isEqualTo(1_000_050);
        assertThat(d.runs()).hasSize(20);
        assertThat(d.distinctFormatCount()).isEqualTo(11);
        assertThat(d.formatAt(0)).isEqualTo("Calibri|11|-|-|-|c0");

        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(30_000);
    }

    @Test
    void shouldKeepTextRunsAndLinesConsistentUnderConcurrentEditing() {
        WordProcessor d = Candidate.of(Solution::create);
        d.insert(0, "line one\nline two\nline three\n");

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 500; i++) {
                switch (t % 4) {
                    case 0 -> d.insert(0, "ab\n");
                    case 1 -> d.insert(d.length(), "cd");
                    case 2 -> {
                        int n = d.length();
                        d.applyFormat(n / 4, n / 2, Map.of("bold", String.valueOf(i % 2 == 0)));
                    }
                    default -> {
                        List<String> runs = d.runs();
                        String text = d.text();
                        assertThat(d.lineCount()).isGreaterThanOrEqualTo(4);
                        assertThat(runs).isNotEmpty();
                        assertThat(text.length()).isGreaterThanOrEqualTo(29);
                    }
                }
            }
        });

        assertThat(d.length()).isEqualTo(29 + 2 * 500 * 3 + 2 * 500 * 2);
        assertThat(d.lineCount()).isEqualTo(3 + 1 + 2 * 500);
        List<String> runs = d.runs();
        assertThat(runs.get(0)).startsWith("0-");
        assertThat(runs.get(runs.size() - 1)).contains("-" + d.length() + ":");
    }
}

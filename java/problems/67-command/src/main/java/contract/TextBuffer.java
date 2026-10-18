package contract;

/** Provided receiver: the text being edited. Commands act on it; the buffer itself knows nothing about undo. */
public final class TextBuffer {
    private final StringBuilder text = new StringBuilder();

    public String text() {
        return text.toString();
    }

    public int length() {
        return text.length();
    }

    /** Inserts at {@code pos} (0..length). A bad position or null text -> IllegalArgumentException. */
    public void insert(int pos, String s) {
        if (s == null || pos < 0 || pos > text.length()) {
            throw new IllegalArgumentException("bad insert at " + pos);
        }
        text.insert(pos, s);
    }

    /** Removes [from, to) (0 <= from <= to <= length) and returns the removed text. Otherwise IllegalArgumentException. */
    public String delete(int from, int to) {
        if (from < 0 || to < from || to > text.length()) {
            throw new IllegalArgumentException("bad delete " + from + ".." + to);
        }
        String removed = text.substring(from, to);
        text.delete(from, to);
        return removed;
    }
}

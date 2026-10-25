package contract;

/**
 * A partial style update. A {@code null} field means "leave that part of the style as it is".
 * Valid values: {@code fontSize} 8..72; {@code textColor} and {@code fillColor} a non-empty name without whitespace and
 * without {@code '|'} (stored in lower case); {@code numberFormat} one of GENERAL, INTEGER, TWO_DECIMALS, PERCENT
 * (case-insensitive, stored in upper case).
 */
public record StyleChange(
        Boolean bold, Boolean italic, Integer fontSize, String textColor, String fillColor, String numberFormat) {

    /** A change that modifies nothing. */
    public static StyleChange none() {
        return new StyleChange(null, null, null, null, null, null);
    }
}

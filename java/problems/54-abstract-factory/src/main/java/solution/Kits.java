package solution;

import contract.RegionKit;

/** Chooses the factory. */
public final class Kits {
    private Kits() {}

    /** "US", "EU" or "IN" in any letter case. Null or unknown -> IllegalArgumentException. */
    public static RegionKit forRegion(String code) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

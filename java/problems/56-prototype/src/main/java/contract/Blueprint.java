package contract;

import java.util.List;

/** Provided contract: a server blueprint. */
public interface Blueprint {

    String name();

    void rename(String newName);

    /** "web", "db", ... */
    String kind();

    /** Blank tags and tags already present are ignored. */
    void addTag(String tag);

    /** Tags in insertion order. Callers cannot modify the blueprint through the returned list. */
    List<String> tags();

    /** Disk sizes in GB, in order. Callers cannot modify the blueprint through the returned list. */
    List<Integer> diskSizesGb();

    /** index out of range or newSizeGb <= 0 -> IllegalArgumentException. */
    void resizeDisk(int index, int newSizeGb);

    /**
     * A completely independent DEEP copy of the same concrete type, with the same name, tags and disk sizes.
     * Changing the copy never changes the original, and vice versa.
     */
    Blueprint copy();
}

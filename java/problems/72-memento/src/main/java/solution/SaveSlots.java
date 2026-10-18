package solution;

/** The caretaker for manual saves: it stores snapshots under a name and hands them back, never looking inside. */
public class SaveSlots {

    /** Stores a snapshot of the character in the slot, replacing any earlier one. Blank slot or null character -> IllegalArgumentException. */
    public void save(String slot, GameCharacter character) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Restores the slot's snapshot into the character. Returns false (and changes nothing) for an unknown slot. */
    public boolean load(String slot, GameCharacter character) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int slotCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

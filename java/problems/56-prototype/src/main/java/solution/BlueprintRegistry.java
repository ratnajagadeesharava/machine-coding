package solution;

import contract.Blueprint;

/** A catalogue of ready-made blueprints. New servers are stamped out by cloning a registered prototype. */
public class BlueprintRegistry {

    /**
     * Stores a COPY of the prototype under the key (replacing any earlier one), so later changes to the object the
     * caller passed in do not affect the catalogue. Blank key or null prototype -> IllegalArgumentException.
     */
    public void register(String key, Blueprint prototype) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Returns a new, independent copy of the registered prototype with its name set to newName. Unknown key, or a
     * blank newName -> IllegalArgumentException.
     */
    public Blueprint create(String key, String newName) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

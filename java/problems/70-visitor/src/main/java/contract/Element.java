package contract;

/** Provided element interface. {@code accept} is the entry point of the double dispatch. */
public interface Element {

    <R> R accept(Visitor<R> visitor);
}

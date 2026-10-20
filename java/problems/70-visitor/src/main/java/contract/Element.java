package contract;

/** Provided element interface. */
public interface Element {

    <R> R accept(Visitor<R> visitor);
}

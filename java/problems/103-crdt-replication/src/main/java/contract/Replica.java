package contract;

import java.util.Optional;
import java.util.Set;

/**
 * One replica of a small replicated data store made of state-based CRDTs. Every replica accepts writes on its own
 * (no coordination); replicas exchange whole-state snapshots and {@link #merge(String)} them, in any order, any number
 * of times, possibly duplicated. After the replicas have merged each other's state they must all show the same values.
 *
 * <p>Three data types, each living in its own namespace (a counter "a", a set "a" and a register "a" are unrelated):
 * <ul>
 *   <li><b>PN-counter</b>: {@code increment}/{@code decrement}, value = sum of all increments - sum of all decrements
 *       from all replicas. A counter nobody touched is 0. A negative amount is an {@link IllegalArgumentException}.</li>
 *   <li><b>OR-set</b> (observed-remove, add wins): {@code remove} only removes the additions this replica has
 *       OBSERVED. An add that is concurrent with a remove survives the merge. An element can be re-added after a
 *       remove. Removing an element that is not in the set does nothing.</li>
 *   <li><b>LWW-register</b>: {@code write(value, timestamp)}: the highest timestamp wins; equal timestamps are broken
 *       by the larger replica id (string comparison); a write with an older timestamp than the current value is
 *       ignored. A replica never writes the same timestamp twice to one register (the tests do not do it). Never written: empty.</li>
 * </ul>
 *
 * <p>The snapshot is an opaque string only your own {@code merge} has to understand. It must be a VALUE: later changes
 * of this replica must not change a snapshot that was already taken. {@code merge} must be idempotent, commutative and
 * associative and must never lose or resurrect anything: merging an old snapshot does not undo newer state.
 * The replica must be thread-safe.
 */
public interface Replica {

    String id();

    void increment(String counter, long amount);

    void decrement(String counter, long amount);

    long counter(String counter);

    void add(String set, String element);

    void remove(String set, String element);

    boolean contains(String set, String element);

    /** A copy: later changes do not show up in it. */
    Set<String> elements(String set);

    void write(String register, String value, long timestamp);

    Optional<String> read(String register);

    String snapshot();

    void merge(String snapshot);
}

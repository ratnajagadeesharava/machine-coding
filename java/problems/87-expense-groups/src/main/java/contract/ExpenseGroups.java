package contract;

import java.util.List;
import java.util.Map;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Amounts are whole currency units. Every method throws IllegalArgumentException for an unknown group, an unknown
 * member, a duplicate, or an invalid amount, and then changes nothing.
 */
public interface ExpenseGroups {

    /** Creates a group with at least one distinct member. */
    void createGroup(String group, List<String> members);

    void addMember(String group, String member);

    /** Removes a member only if their balance in the group is zero. Returns false (and keeps them) otherwise. */
    boolean removeMember(String group, String member);

    /**
     * {@code payer} paid {@code amount} for everyone in {@code participants} equally. A remainder is handed out one
     * unit at a time to the participants in ascending name order. The payer need not be a participant.
     */
    void addExpense(String group, String payer, long amount, List<String> participants);

    /** {@code payer} paid the sum of the shares; each member owes exactly their share. Shares must be > 0. */
    void addExactExpense(String group, String payer, Map<String, Long> shares);

    /** {@code from} paid {@code to} {@code amount} to reduce what they owe. (Overpaying is allowed.) */
    void settle(String group, String from, String to, long amount);

    /** Net position per member (positive = is owed money); zero balances omitted. */
    Map<String, Long> balances(String group);

    /** Sum of all expenses (not settlements) of the group. */
    long totalSpent(String group);

    /**
     * A list of payments "from>to:amount" that brings every balance to zero using the FEWEST payments possible.
     * Any optimal set is accepted; the list is sorted ascending. Empty if everybody is square.
     */
    List<String> settleUp(String group);
}

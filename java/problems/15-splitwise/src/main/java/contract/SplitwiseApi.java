package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. */
public interface SplitwiseApi {

    boolean addEqualExpense(String payer, long amount, List<String> participants);

    /** shares must add up to amount. */
    boolean addExactExpense(String payer, long amount, Map<String, Long> shares);

    /** percents (integers) must add up to 100. */
    boolean addPercentExpense(String payer, long amount, Map<String, Integer> percents);

    /** Net amount debtor owes creditor (>= 0). */
    long owed(String debtor, String creditor);

    /** Net position per person (positive = is owed); zero balances omitted. */
    Map<String, Long> balances();

    boolean settle(String from, String to, long amount);

    /** Payments "debtor>creditor:amount", sorted ascending, that would zero all balances. */
    List<String> simplify();
}

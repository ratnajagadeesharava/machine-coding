package solution;

import contract.*;

import java.time.Clock;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;
import java.util.function.*;

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {
    }

    public static Bank create() {
        return new MyBank();
    }
}

class MyBank implements Bank {
    private final Map<String, Account> accountMap;
    private long totalAmount;

    public MyBank() {
        this.accountMap = new HashMap<>();
        totalAmount = 0;
    }

    @Override
    public void open(String account, long balance) {
        if(accountMap.containsKey(account)){
            throw new IllegalArgumentException();
        }
        Account newAccount = new Account(account, balance);
        this.totalAmount += balance;
        accountMap.put(account, newAccount);
    }

    @Override
    public long balance(String account) {
        if (!accountMap.containsKey(account)) {
            throw new IllegalArgumentException();
        }
        return accountMap.get(account).getBalance();
    }

    @Override
    public boolean transfer(String from, String to, long amount) {
        if (from.equals(to)) {
            throw new IllegalArgumentException();
        }
        return Account.transfer(accountMap.get(from), accountMap.get(to), amount);
    }

    @Override
    public long total() {
        return totalAmount;
    }
}

class Account {
    private final String accountId;
    private long balance;
    private Lock lock = new ReentrantLock();

    public Account(String accountId, long initialBalance) {
        if (accountId == null || accountId.isBlank() || initialBalance < 0) {
            throw new IllegalArgumentException("Illegal");
        }
        this.accountId = accountId;
        this.balance = initialBalance;
    }

    static boolean transfer(Account from, Account to, long amount) {
        Account lock1;
        Account lock2;
        if (amount <= 0 || from == null || to == null) {
            throw new IllegalArgumentException("Illegal");
        }
        if (from.accountId.compareTo(to.accountId) > 0) {
            lock1 = from;
            lock2 = to;
        } else {
            lock1 = to;
            lock2 = from;
        }
        synchronized (lock1) {
            synchronized (lock2) {
                if (from.balance - amount < 0) {
                    return false;
                }
                from.balance -= amount;
                to.balance += amount;
                return true;
            }
        }
    }

    public synchronized long getBalance() {
        return balance;
    }


}

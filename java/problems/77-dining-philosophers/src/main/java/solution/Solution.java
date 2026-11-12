package solution;

import contract.*;

import java.util.*;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {
    }

    /**
     * {@code seats} is at least 2.
     */
    public static Table create(int seats) {
        return new MyTable(seats);
    }
}

class MyTable implements Table {
    private final int seats;
    private final Lock[] forks;

    MyTable(int seats) {
        this.seats = seats;
        this.forks = new ReentrantLock[seats];
        for (int i = 0; i < seats; i++) {
            forks[i] = new ReentrantLock();
        }
    }

    @Override
    public void dine(int philosopher, Runnable eat) throws InterruptedException {
        if (philosopher < 0 || philosopher >= seats) {
            throw new IllegalArgumentException("philosphers should be in the range of 0 to " + (seats - 1));
        }
        Lock lock1, lock2;
        int start = philosopher;
        int end = (philosopher + 1) % seats;
        lock1 = forks[Math.min(start, end)];
        lock2 = forks[Math.max(start, end)];    

        lock1.lockInterruptibly();
        try {
            lock2.lockInterruptibly();
            try {
                eat.run();
            } finally {
                lock2.unlock();
            }
        } finally {
            lock1.unlock();
        }
    }
}

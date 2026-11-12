package solution;

import contract.*;

import java.time.Clock;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;
import java.util.function.*;

class MainOrderedPrinter implements OrderedPrinter {
    private final Lock lock = new ReentrantLock();
    private int turn = 0;
    private final int workers;

    public MainOrderedPrinter(int workers) {
        this.workers = workers;
    }


    @Override
    public synchronized void takeTurn(int worker, Runnable action) throws InterruptedException {
        if (worker >= workers || worker < 0) {
            throw new IllegalArgumentException("woker is not correct");
        }
        while (turn % workers != worker) {
            wait();
        }
        try {
            action.run();
        } catch (Exception e) {
            throw e;
        } finally {
            turn = turn + 1;
            notifyAll();
        }


    }
}

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {
    }

    /**
     * Worker 0 goes first. {@code workers} is at least 1.
     */
    public static OrderedPrinter create(int workers) {
        return new MainOrderedPrinter(workers);
    }
}

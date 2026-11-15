package solution;

import contract.*;

import java.time.Clock;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.locks.Condition;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;
import java.util.function.*;
//requirments
//producers ,consumers run on diffrent thread
//Queue
//producer waits when the queue is full
//after all the producers finished there should be  a signal to stop the consumers

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {
    }

    public static <T> BlockingBuffer<T> create(int capacity) {
        return new BlockedBuffer<T>(capacity);
    }
}

class BlockedBuffer<T> implements BlockingBuffer<T> {
    private final int capacity;
    private boolean isClosed = false;
    private final Lock lock = new ReentrantLock();
    private final Condition notFull = lock.newCondition();
    private final Condition notEmpty = lock.newCondition();
    private Queue<T> queue;

    public BlockedBuffer(int capacity) {
        this.capacity = capacity;
        this.queue = new LinkedList<>();
    }

    @Override
    public void put(T item) throws InterruptedException {
        lock.lock();
        try {

            while (queue.size() == capacity && !isClosed) {
                notFull.await();
            }
            if (isClosed) {
                throw new IllegalStateException("");
//                return;
            }
            queue.add(item);
            notEmpty.signal();
        } finally {
            lock.unlock();
        }

    }

    @Override
    public Optional<T> take() throws InterruptedException {
        lock.lock();
        try {
            if (isClosed && queue.isEmpty()) {
                return Optional.empty();
            }
            while (queue.isEmpty() && !isClosed) {
                notEmpty.await();
            }

            T result = queue.poll();
            notFull.signal();
            return Optional.ofNullable(result);
        } finally {
            lock.unlock();
        }
    }

    @Override
    public boolean offer(T item, long timeoutMs) throws InterruptedException {
        long remaining = TimeUnit.MILLISECONDS.toNanos(timeoutMs);
        lock.lock();
        try {
            while (queue.size() == capacity && !isClosed) {
                if (remaining <= 0)
                    return false;
                remaining = notFull.awaitNanos(remaining);
            }
            if (isClosed) {
                throw new IllegalStateException("");
            }
            ;
            queue.add(item);
            notEmpty.signal();
            return true;
        } finally {
            lock.unlock();
        }

    }

    @Override
    public Optional<T> poll(long timeoutMs) throws InterruptedException {
        long remaining = TimeUnit.MILLISECONDS.toNanos(timeoutMs);
        lock.lock();
        try {
            while (queue.isEmpty() && !isClosed) {
                if (remaining <= 0) {
                    return Optional.empty();
                }

                remaining = notEmpty.awaitNanos(remaining);
            }
            if (isClosed && queue.isEmpty()) {
                return Optional.empty();
            }
            T result = queue.poll();
            notFull.signal();
            return Optional.ofNullable(result);
        } finally {
            lock.unlock();
        }
    }

    @Override
    public int size() {
        lock.lock();
        try {
            int result = queue.size();

            return result;
        } finally {
            lock.unlock();
        }
    }

    @Override
    public void close() {
        lock.lock();
        try {
            isClosed = true;
            notEmpty.signalAll();
            notFull.signalAll();
        } finally {
            lock.unlock();
        }
    }
}
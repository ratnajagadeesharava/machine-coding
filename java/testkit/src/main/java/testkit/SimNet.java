package testkit;

import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.PriorityQueue;
import java.util.Random;
import java.util.Set;
import java.util.function.BiConsumer;
import java.util.function.BooleanSupplier;
import java.util.function.LongConsumer;

/**
 * Deterministic single-threaded simulation of a network and a clock for distributed-systems problems.
 *
 * <p>Time advances in 1 ms steps. In every step the messages that are due are delivered first, then every running node
 * gets its tick. Nodes talk to each other only through {@link #sender(String)} (a function {@code (to, payload)}), so a
 * test can crash nodes, cut the network into partitions, drop, duplicate and delay messages and still replay exactly
 * the same run from the same seed.
 */
public final class SimNet {
    private record Msg(long at, long seq, String from, String to, String payload) {}

    private record Handler(BiConsumer<String, String> onMessage, LongConsumer onTick) {}

    private final Random rng;
    private final PriorityQueue<Msg> queue = new PriorityQueue<>((a, b) -> a.at != b.at ? Long.compare(a.at, b.at) : Long.compare(a.seq, b.seq));
    private final Map<String, Handler> nodes = new LinkedHashMap<>();
    private final Set<String> crashed = new HashSet<>();
    private final Map<String, Integer> groupOf = new HashMap<>();
    private final Map<String, Integer> sentBy = new HashMap<>();
    private final List<Runnable> observers = new ArrayList<>();
    private long now;
    private long seq;
    private double dropRate;
    private double duplicateRate;
    private int minLatency = 1;
    private int maxLatency = 5;
    private int delivered;

    public SimNet(long seed) {
        this.rng = new Random(seed);
    }

    /** Adds (or replaces, e.g. after a restart) a node. */
    public void register(String id, BiConsumer<String, String> onMessage, LongConsumer onTick) {
        nodes.put(id, new Handler(onMessage, onTick));
        crashed.remove(id);
    }

    /** The function a node uses to send: {@code (to, payload)}. */
    public BiConsumer<String, String> sender(String from) {
        return (to, payload) -> send(from, to, payload);
    }

    public void send(String from, String to, String payload) {
        if (crashed.contains(from)) {
            return;
        }
        sentBy.merge(from, 1, Integer::sum);
        if (rng.nextDouble() < dropRate) {
            return;
        }
        enqueue(from, to, payload);
        if (rng.nextDouble() < duplicateRate) {
            enqueue(from, to, payload);
        }
    }

    private void enqueue(String from, String to, String payload) {
        int latency = minLatency + (maxLatency > minLatency ? rng.nextInt(maxLatency - minLatency + 1) : 0);
        queue.add(new Msg(now + latency, seq++, from, to, payload));
    }

    /** Advances time by one millisecond. */
    public void step() {
        now++;
        while (!queue.isEmpty() && queue.peek().at <= now) {
            Msg m = queue.poll();
            Handler h = nodes.get(m.to);
            if (h == null || crashed.contains(m.to) || cut(m.from, m.to)) {
                continue;
            }
            delivered++;
            h.onMessage.accept(m.from, m.payload);
        }
        for (Map.Entry<String, Handler> e : new ArrayList<>(nodes.entrySet())) {
            if (!crashed.contains(e.getKey())) {
                e.getValue().onTick.accept(now);
            }
        }
        for (Runnable o : observers) {
            o.run();
        }
    }

    public void run(long ms) {
        for (long i = 0; i < ms; i++) {
            step();
        }
    }

    /** Steps until the condition holds (checked after every step) or {@code maxMs} have passed; returns whether it held. */
    public boolean runUntil(BooleanSupplier condition, long maxMs) {
        for (long i = 0; i < maxMs; i++) {
            if (condition.getAsBoolean()) {
                return true;
            }
            step();
        }
        return condition.getAsBoolean();
    }

    /** Called after every step, e.g. to check safety invariants continuously. */
    public void observe(Runnable observer) {
        observers.add(observer);
    }

    /** A crashed node gets no ticks and no messages, and its sends are ignored. Its in-memory state stays as it was. */
    public void crash(String id) {
        crashed.add(id);
    }

    public void recover(String id) {
        crashed.remove(id);
    }

    public boolean isCrashed(String id) {
        return crashed.contains(id);
    }

    /** Cuts the network into the given groups; nodes in different groups cannot talk. Unlisted nodes form their own group. */
    @SafeVarargs
    public final void partition(Collection<String>... groups) {
        groupOf.clear();
        int g = 0;
        for (Collection<String> group : groups) {
            for (String id : group) {
                groupOf.put(id, g);
            }
            g++;
        }
    }

    public void heal() {
        groupOf.clear();
    }

    private boolean cut(String a, String b) {
        if (groupOf.isEmpty()) {
            return false;
        }
        return !groupOf.getOrDefault(a, -1).equals(groupOf.getOrDefault(b, -1));
    }

    public void setDropRate(double p) {
        this.dropRate = p;
    }

    public void setDuplicateRate(double p) {
        this.duplicateRate = p;
    }

    public void setLatency(int minMs, int maxMs) {
        this.minLatency = minMs;
        this.maxLatency = maxMs;
    }

    public long now() {
        return now;
    }

    public int sentBy(String id) {
        return sentBy.getOrDefault(id, 0);
    }

    public int delivered() {
        return delivered;
    }

    public Random random() {
        return rng;
    }
}

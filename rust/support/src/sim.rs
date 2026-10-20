//! Deterministic single-threaded simulation of a network and a clock for the distributed-systems problems.
//!
//! Time advances in 1 ms steps. In every step the messages that are due are delivered first, then every running node
//! gets its tick. Nodes talk to each other only through the function returned by [`SimNet::sender`], so a test can
//! crash nodes, cut the network into partitions, drop, duplicate and delay messages and replay exactly the same run
//! from the same seed.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Called when a message arrives: `(from, payload)`.
pub type MsgFn = Arc<dyn Fn(&str, &str) + Send + Sync>;
/// Called once per millisecond with the current simulated time.
pub type TickFn = Arc<dyn Fn(u64) + Send + Sync>;
/// How a node sends: `(to, payload)`.
pub type SendFn = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Tiny deterministic random number generator (xorshift64*), so tests need no external crates.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in `0..n` (`n > 0`).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64 <= p && p > 0.0
    }
}

struct Msg {
    from: String,
    to: String,
    payload: String,
}

#[derive(Clone)]
struct Handler {
    on_message: MsgFn,
    on_tick: TickFn,
}

struct Inner {
    rng: Rng,
    now: u64,
    seq: u64,
    queue: BTreeMap<(u64, u64), Msg>,
    nodes: Vec<(String, Handler)>,
    crashed: HashSet<String>,
    group_of: HashMap<String, usize>,
    sent_by: HashMap<String, usize>,
    observers: Vec<Arc<dyn Fn() + Send + Sync>>,
    drop_rate: f64,
    duplicate_rate: f64,
    min_latency: u64,
    max_latency: u64,
    delivered: usize,
}

impl Inner {
    fn cut(&self, a: &str, b: &str) -> bool {
        if self.group_of.is_empty() {
            return false;
        }
        let ga = self.group_of.get(a).map(|g| *g as i64).unwrap_or(-1);
        let gb = self.group_of.get(b).map(|g| *g as i64).unwrap_or(-1);
        ga != gb
    }

    fn enqueue(&mut self, from: &str, to: &str, payload: &str) {
        let span = self.max_latency.saturating_sub(self.min_latency);
        let latency = self.min_latency + if span > 0 { self.rng.below(span + 1) } else { 0 };
        let key = (self.now + latency, self.seq);
        self.seq += 1;
        self.queue.insert(key, Msg { from: from.into(), to: to.into(), payload: payload.into() });
    }
}

/// Handle to the simulation. Cloning shares the same network.
#[derive(Clone)]
pub struct SimNet(Arc<Mutex<Inner>>);

impl SimNet {
    pub fn new(seed: u64) -> SimNet {
        SimNet(Arc::new(Mutex::new(Inner {
            rng: Rng::new(seed),
            now: 0,
            seq: 0,
            queue: BTreeMap::new(),
            nodes: Vec::new(),
            crashed: HashSet::new(),
            group_of: HashMap::new(),
            sent_by: HashMap::new(),
            observers: Vec::new(),
            drop_rate: 0.0,
            duplicate_rate: 0.0,
            min_latency: 1,
            max_latency: 5,
            delivered: 0,
        })))
    }

    /// Adds (or replaces, e.g. after a restart) a node.
    pub fn register(&self, id: &str, on_message: MsgFn, on_tick: TickFn) {
        let mut s = self.0.lock().unwrap();
        let handler = Handler { on_message, on_tick };
        match s.nodes.iter_mut().find(|(n, _)| n == id) {
            Some(slot) => slot.1 = handler,
            None => s.nodes.push((id.to_string(), handler)),
        }
        s.crashed.remove(id);
    }

    /// The function a node uses to send: `(to, payload)`.
    pub fn sender(&self, from: &str) -> SendFn {
        let net = self.clone();
        let from = from.to_string();
        Arc::new(move |to: &str, payload: &str| net.send(&from, to, payload))
    }

    pub fn send(&self, from: &str, to: &str, payload: &str) {
        let mut s = self.0.lock().unwrap();
        if s.crashed.contains(from) {
            return;
        }
        *s.sent_by.entry(from.to_string()).or_insert(0) += 1;
        let p = s.drop_rate;
        let drop = s.rng.chance(p);
        if drop {
            return;
        }
        s.enqueue(from, to, payload);
        let p = s.duplicate_rate;
        let dup = s.rng.chance(p);
        if dup {
            s.enqueue(from, to, payload);
        }
    }

    /// Advances time by one millisecond.
    pub fn step(&self) {
        let mut due: Vec<(MsgFn, String, String)> = Vec::new();
        {
            let mut s = self.0.lock().unwrap();
            s.now += 1;
            let now = s.now;
            while let Some((&key, _)) = s.queue.iter().next() {
                if key.0 > now {
                    break;
                }
                let m = s.queue.remove(&key).unwrap();
                let handler = s.nodes.iter().find(|(n, _)| *n == m.to).map(|(_, h)| h.on_message.clone());
                if let Some(h) = handler {
                    if !s.crashed.contains(&m.to) && !s.cut(&m.from, &m.to) {
                        s.delivered += 1;
                        due.push((h, m.from, m.payload));
                    }
                }
            }
        }
        for (h, from, payload) in due {
            h(&from, &payload);
        }
        let (now, ticks): (u64, Vec<TickFn>) = {
            let s = self.0.lock().unwrap();
            (s.now, s.nodes.iter().filter(|(n, _)| !s.crashed.contains(n)).map(|(_, h)| h.on_tick.clone()).collect())
        };
        for t in ticks {
            t(now);
        }
        let observers = self.0.lock().unwrap().observers.clone();
        for o in observers {
            o();
        }
    }

    pub fn run(&self, ms: u64) {
        for _ in 0..ms {
            self.step();
        }
    }

    /// Steps until the condition holds (checked after every step) or `max_ms` have passed; returns whether it held.
    pub fn run_until(&self, condition: &dyn Fn() -> bool, max_ms: u64) -> bool {
        for _ in 0..max_ms {
            if condition() {
                return true;
            }
            self.step();
        }
        condition()
    }

    /// Called after every step, e.g. to check safety invariants continuously.
    pub fn observe(&self, observer: Arc<dyn Fn() + Send + Sync>) {
        self.0.lock().unwrap().observers.push(observer);
    }

    /// A crashed node gets no ticks and no messages, and its sends are ignored. Its in-memory state stays as it was.
    pub fn crash(&self, id: &str) {
        self.0.lock().unwrap().crashed.insert(id.to_string());
    }

    pub fn recover(&self, id: &str) {
        self.0.lock().unwrap().crashed.remove(id);
    }

    pub fn is_crashed(&self, id: &str) -> bool {
        self.0.lock().unwrap().crashed.contains(id)
    }

    /// Cuts the network into the given groups; nodes in different groups cannot talk. Unlisted nodes form their own group.
    pub fn partition(&self, groups: &[&[&str]]) {
        let mut s = self.0.lock().unwrap();
        s.group_of.clear();
        for (g, group) in groups.iter().enumerate() {
            for id in group.iter() {
                s.group_of.insert(id.to_string(), g);
            }
        }
    }

    pub fn heal(&self) {
        self.0.lock().unwrap().group_of.clear();
    }

    pub fn set_drop_rate(&self, p: f64) {
        self.0.lock().unwrap().drop_rate = p;
    }

    pub fn set_duplicate_rate(&self, p: f64) {
        self.0.lock().unwrap().duplicate_rate = p;
    }

    pub fn set_latency(&self, min_ms: u64, max_ms: u64) {
        let mut s = self.0.lock().unwrap();
        s.min_latency = min_ms;
        s.max_latency = max_ms;
    }

    pub fn now(&self) -> u64 {
        self.0.lock().unwrap().now
    }

    pub fn sent_by(&self, id: &str) -> usize {
        self.0.lock().unwrap().sent_by.get(id).copied().unwrap_or(0)
    }

    pub fn delivered(&self) -> usize {
        self.0.lock().unwrap().delivered
    }

    /// A random number from the simulation's own generator (for fault schedules in tests).
    pub fn random_below(&self, n: u64) -> u64 {
        self.0.lock().unwrap().rng.below(n)
    }
}

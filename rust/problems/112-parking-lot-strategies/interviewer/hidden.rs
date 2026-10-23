#![allow(unused_imports, clippy::needless_range_loop)]
use p112_parking_lot_strategies::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TWO: &str = "TWO_WHEELER";
const FOUR: &str = "FOUR_WHEELER";
const LOWEST: &str = "LOWEST_INDEX_SPOT";
const MOST_FREE: &str = "MOST_FREE_FLOOR";

fn make(two: &[usize], four: &[usize], strategy: &str) -> Option<Arc<dyn ParkingLot>> {
    create(two, four, strategy).map(|r| r.expect("valid configuration must be accepted"))
}

fn park(l: &Arc<dyn ParkingLot>, t: &str, reg: &str) -> String {
    l.park(t, reg)
        .unwrap()
        .unwrap_or_else(|| panic!("expected a spot for {reg}"))
}

fn at(l: &Arc<dyn ParkingLot>, ticket: &str) -> String {
    l.find_by_ticket(ticket).unwrap().expect("active ticket")
}

#[test]
fn should_follow_the_most_free_floor_rule_through_a_full_fill_and_partial_drain() {
    // floors with 2, 4, 3 two-wheeler spots: global 0-1 | 2-5 | 6-8
    let l = sut!(make(&[2, 4, 3], &[0, 0, 0], MOST_FREE));
    let expected = [
        "1:2", "1:3", "2:6", "0:0", "1:4", "2:7", "0:1", "1:5", "2:8",
    ];
    let mut tickets = Vec::new();
    for (i, e) in expected.iter().enumerate() {
        let t = park(&l, TWO, &format!("v{i}"));
        assert_eq!(&at(&l, &t), e, "vehicle {i}");
        tickets.push(t);
    }
    assert_eq!(l.park(TWO, "extra"), Ok(None));

    // free 1:2, 2:6 and 0:1: each floor has exactly 1 free -> floor 0 first, then 1, then 2
    l.unpark(&tickets[0]).unwrap();
    l.unpark(&tickets[2]).unwrap();
    l.unpark(&tickets[6]).unwrap();
    assert_eq!(at(&l, &park(&l, TWO, "n1")), "0:1");
    assert_eq!(at(&l, &park(&l, TWO, "n2")), "1:2");
    assert_eq!(at(&l, &park(&l, TWO, "n3")), "2:6");
}

#[test]
fn should_fill_holes_in_global_order_after_switching_back_to_lowest_index() {
    let l = sut!(make(&[3, 3], &[0, 0], MOST_FREE));
    let t: Vec<String> = (0..6).map(|i| park(&l, TWO, &format!("v{i}"))).collect(); // 0:0 1:3 0:1 1:4 0:2 1:5
    assert_eq!(at(&l, &t[1]), "1:3");
    l.unpark(&t[5]).unwrap(); // 1:5
    l.unpark(&t[2]).unwrap(); // 0:1
    l.unpark(&t[3]).unwrap(); // 1:4

    l.set_strategy(LOWEST).unwrap();

    assert_eq!(at(&l, &park(&l, TWO, "a")), "0:1");
    assert_eq!(at(&l, &park(&l, TWO, "b")), "1:4");
    assert_eq!(at(&l, &park(&l, TWO, "c")), "1:5");
}

#[test]
fn should_survive_an_over_full_lot_then_fully_empty_it() {
    let l = sut!(make(&[2, 2], &[2, 2], LOWEST));
    let mut tickets = Vec::new();
    for i in 0..20 {
        let ty = if i % 2 == 0 { TWO } else { FOUR };
        if let Some(t) = l.park(ty, &format!("r{i}")).unwrap() {
            tickets.push(t);
        }
    }
    assert_eq!(tickets.len(), 8);

    for t in &tickets {
        assert_eq!(l.unpark(t), Ok(true));
    }

    assert_eq!(l.free_spots(TWO), Ok(4));
    assert_eq!(l.free_spots(FOUR), Ok(4));
    // "r2" was turned away earlier (lot full for two-wheelers) and must be accepted now
    assert_eq!(l.find_by_registration("r2"), Ok(None));
    assert!(l.park(TWO, "r2").unwrap().is_some());
}

/// Brute-force reference used only to cross-check the implementation.
struct Model {
    free: [Vec<BTreeSet<usize>>; 2],
    parked: HashMap<String, (usize, usize, usize)>,
    lowest: bool,
}

impl Model {
    fn new(two: &[usize], four: &[usize], lowest: bool) -> Self {
        let mut free = [Vec::new(), Vec::new()];
        let mut idx = 0;
        for f in 0..two.len() {
            let (mut a, mut b) = (BTreeSet::new(), BTreeSet::new());
            for _ in 0..two[f] {
                a.insert(idx);
                idx += 1;
            }
            for _ in 0..four[f] {
                b.insert(idx);
                idx += 1;
            }
            free[0].push(a);
            free[1].push(b);
        }
        Model {
            free,
            parked: HashMap::new(),
            lowest,
        }
    }

    fn park(&mut self, t: usize, reg: &str) -> Option<String> {
        let mut best: Option<usize> = None;
        for f in 0..self.free[t].len() {
            if self.free[t][f].is_empty() {
                continue;
            }
            best = match best {
                None => Some(f),
                Some(b) => {
                    let better = if self.lowest {
                        self.free[t][f].first() < self.free[t][b].first()
                    } else {
                        self.free[t][f].len() > self.free[t][b].len()
                    };
                    Some(if better { f } else { b })
                }
            };
        }
        let f = best?;
        let spot = self.free[t][f].pop_first().unwrap();
        self.parked.insert(reg.to_string(), (t, f, spot));
        Some(format!("{f}:{spot}"))
    }

    fn leave(&mut self, reg: &str) {
        let (t, f, s) = self.parked.remove(reg).unwrap();
        self.free[t][f].insert(s);
    }
}

#[test]
fn should_match_a_brute_force_model_over_a_random_workload_for_both_strategies() {
    let two = [3usize, 0, 4, 2];
    let four = [1usize, 3, 0, 2];
    for strategy in [LOWEST, MOST_FREE] {
        let l = sut!(make(&two, &four, strategy));
        let mut m = Model::new(&two, &four, strategy == LOWEST);
        let mut seed: u64 = 12345;
        let mut active: HashMap<String, String> = HashMap::new(); // reg -> ticket
        for step in 0..4_000 {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = ((seed >> 33) & 0x7fff_ffff) as usize;
            if r % 5 == 0 && !active.is_empty() {
                let mut regs: Vec<&String> = active.keys().collect();
                regs.sort();
                let reg = regs[(r / 5) % regs.len()].clone();
                let ticket = active.remove(&reg).unwrap();
                assert_eq!(l.unpark(&ticket), Ok(true));
                m.leave(&reg);
            } else if r % 97 == 0 {
                let lowest = (r / 97) % 2 == 0;
                l.set_strategy(if lowest { LOWEST } else { MOST_FREE })
                    .unwrap();
                m.lowest = lowest;
            } else {
                let t = (r / 3) % 2;
                let ty = if t == 0 { TWO } else { FOUR };
                let reg = format!("R{}", r % 40);
                if active.contains_key(&reg) {
                    assert_eq!(l.park(ty, &reg), Err(ParkingError::AlreadyParked));
                    continue;
                }
                let got = l.park(ty, &reg).unwrap();
                let expect = m.park(t, &reg);
                assert_eq!(got.is_some(), expect.is_some(), "step {step}");
                if let (Some(ticket), Some(e)) = (got, expect) {
                    assert_eq!(at(&l, &ticket), e, "step {step}");
                    assert_eq!(l.find_by_registration(&reg), Ok(Some(e)));
                    active.insert(reg, ticket);
                }
            }
            if step % 50 == 0 {
                for t in 0..2 {
                    let ty = if t == 0 { TWO } else { FOUR };
                    let total: usize = m.free[t].iter().map(|s| s.len()).sum();
                    assert_eq!(l.free_spots(ty), Ok(total));
                    for f in 0..two.len() {
                        assert_eq!(l.free_spots_on_floor(f, ty), Ok(m.free[t][f].len()));
                    }
                }
            }
        }
    }
}

#[test]
fn should_handle_a_large_lot_quickly_under_both_strategies() {
    let floors = 60;
    let two = vec![100usize; floors];
    let four = vec![100usize; floors];
    for strategy in [LOWEST, MOST_FREE] {
        let l = sut!(make(&two, &four, strategy));
        let start = Instant::now();
        let mut tickets = Vec::new();
        for i in 0..floors * 100 {
            tickets.push(park(&l, TWO, &format!("t{i}")));
        }
        assert_eq!(l.park(TWO, "overflow"), Ok(None));
        for i in (0..tickets.len()).step_by(2) {
            assert_eq!(l.unpark(&tickets[i]), Ok(true));
        }
        for i in 0..tickets.len() / 2 {
            park(&l, TWO, &format!("u{i}"));
        }
        assert_eq!(l.free_spots(TWO), Ok(0));
        assert_eq!(l.free_spots(FOUR), Ok(floors * 100));
        assert!(start.elapsed() < Duration::from_secs(20), "too slow");
    }
}

#[test]
fn should_never_share_one_spot_between_two_live_tickets_under_churn() {
    let l = sut!(make(&[3, 3, 3], &[2, 2, 2], MOST_FREE));
    let live: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new()); // spot -> reg
    let conflicts = AtomicUsize::new(0);

    run_concurrently(12, |t| {
        for i in 0..2_000 {
            let reg = format!("t{t}-{i}");
            let ty = if i % 3 == 0 { FOUR } else { TWO };
            if let Some(ticket) = l.park(ty, &reg).unwrap() {
                let spot = at(&l, &ticket);
                if live.lock().unwrap().insert(spot.clone(), reg).is_some() {
                    conflicts.fetch_add(1, Ordering::SeqCst);
                }
                live.lock().unwrap().remove(&spot);
                l.unpark(&ticket).unwrap();
            }
        }
    });

    assert_eq!(conflicts.load(Ordering::SeqCst), 0);
    assert_eq!(l.free_spots(TWO), Ok(9));
    assert_eq!(l.free_spots(FOUR), Ok(6));
}

#[test]
fn should_allow_strategy_switches_while_threads_park() {
    let l = sut!(make(&[20, 20], &[0, 0], LOWEST));

    let parked = collect_concurrently(9, |t| {
        if t == 8 {
            for i in 0..2_000 {
                l.set_strategy(if i % 2 == 0 { MOST_FREE } else { LOWEST })
                    .unwrap();
            }
            return 0;
        }
        (0..10)
            .filter(|i| l.park(TWO, &format!("p{t}-{i}")).unwrap().is_some())
            .count()
    });

    assert_eq!(parked.iter().sum::<usize>(), 40);
    assert_eq!(l.free_spots(TWO), Ok(0));
}

#[test]
fn should_keep_good_state_after_every_rejected_call() {
    let l = sut!(make(&[1], &[1], LOWEST));
    let t = park(&l, TWO, "A");

    assert_eq!(l.park(TWO, "A"), Err(ParkingError::AlreadyParked));
    assert_eq!(l.park("BUS", "B"), Err(ParkingError::InvalidArgument));
    assert_eq!(l.park(FOUR, " "), Err(ParkingError::InvalidArgument));

    assert_eq!(l.free_spots(FOUR), Ok(1));
    assert_eq!(at(&l, &t), "0:0");
    assert_eq!(l.find_by_registration("B"), Ok(None));
    assert_ne!(park(&l, FOUR, "B"), t);
}

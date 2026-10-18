#![allow(unused_imports, clippy::needless_range_loop)]
use p28_blocking_queue::{contract::*, create};
use playground_support::{eventually, stays, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[test]
fn should_deliver_items_in_fifo_order() {
    let b = sut!(create(5));
    for i in 1..=5 {
        b.put(i).unwrap();
    }

    for i in 1..=5 {
        assert_eq!(b.take(), Some(i));
    }
    assert_eq!(b.size(), 0);
}

#[test]
fn should_report_size() {
    let b = sut!(create(3));
    assert_eq!(b.size(), 0);

    b.put(1).unwrap();
    b.put(2).unwrap();
    assert_eq!(b.size(), 2);

    b.take();
    assert_eq!(b.size(), 1);
}

#[test]
fn should_block_put_while_full_until_space_is_available() {
    let b = sut!(create(2));
    b.put(1).unwrap();
    b.put(2).unwrap();
    let done = Arc::new(AtomicBool::new(false));
    {
        let (b, done) = (b.clone(), done.clone());
        thread::spawn(move || {
            b.put(3).unwrap();
            done.store(true, Ordering::SeqCst);
        });
    }

    assert!(
        stays(150, || !done.load(Ordering::SeqCst)),
        "put must wait while the buffer is full"
    );
    assert_eq!(b.size(), 2);

    assert_eq!(b.take(), Some(1));
    assert!(
        eventually(5_000, || done.load(Ordering::SeqCst)),
        "put completes once there is space"
    );
    assert_eq!(b.take(), Some(2));
    assert_eq!(b.take(), Some(3));
}

#[test]
fn should_block_take_while_empty_until_an_item_arrives() {
    let b = sut!(create(2));
    let got: Arc<Mutex<Option<Option<i32>>>> = Arc::new(Mutex::new(None));
    {
        let (b, got) = (b.clone(), got.clone());
        thread::spawn(move || {
            let item = b.take();
            *got.lock().unwrap() = Some(item);
        });
    }

    assert!(
        stays(150, || got.lock().unwrap().is_none()),
        "take must wait while the buffer is empty"
    );

    b.put(42).unwrap();

    assert!(eventually(5_000, || got.lock().unwrap().is_some()));
    assert_eq!(*got.lock().unwrap(), Some(Some(42)));
}

#[test]
fn should_time_out_offer_while_full() {
    let b = sut!(create(1));
    b.put(1).unwrap();

    let t0 = Instant::now();
    let accepted = b.offer(2, 100).unwrap();
    let elapsed = t0.elapsed().as_millis();

    assert!(!accepted);
    assert!((80..4_000).contains(&elapsed), "elapsed {elapsed} ms");
    assert_eq!(b.size(), 1);
    assert_eq!(b.take(), Some(1));
    assert_eq!(b.poll(0), None);
}

#[test]
fn should_accept_offer_when_space_appears_before_the_timeout() {
    let b = sut!(create(1));
    b.put(1).unwrap();
    let accepted: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
    {
        let (b, accepted) = (b.clone(), accepted.clone());
        thread::spawn(move || {
            let r = b.offer(2, 30_000).unwrap();
            *accepted.lock().unwrap() = Some(r);
        });
    }
    assert!(
        stays(100, || accepted.lock().unwrap().is_none()),
        "offer waits while full"
    );

    assert_eq!(b.take(), Some(1));

    assert!(eventually(5_000, || accepted.lock().unwrap().is_some()));
    assert_eq!(*accepted.lock().unwrap(), Some(true));
    assert_eq!(b.take(), Some(2));
}

#[test]
fn should_time_out_poll_while_empty() {
    let b = sut!(create(2));

    let t0 = Instant::now();
    let result = b.poll(100);
    let elapsed = t0.elapsed().as_millis();

    assert_eq!(result, None);
    assert!((80..4_000).contains(&elapsed), "elapsed {elapsed} ms");
}

#[test]
fn should_return_poll_result_when_an_item_arrives_before_the_timeout() {
    let b = sut!(create(2));
    let got: Arc<Mutex<Option<Option<i32>>>> = Arc::new(Mutex::new(None));
    {
        let (b, got) = (b.clone(), got.clone());
        thread::spawn(move || {
            let item = b.poll(30_000);
            *got.lock().unwrap() = Some(item);
        });
    }
    assert!(
        stays(100, || got.lock().unwrap().is_none()),
        "poll waits while empty"
    );

    b.put(7).unwrap();

    assert!(
        eventually(5_000, || got.lock().unwrap().is_some()),
        "poll returns the item well before its timeout"
    );
    assert_eq!(*got.lock().unwrap(), Some(Some(7)));
}

#[test]
fn should_not_wait_when_the_timeout_is_zero() {
    let b = sut!(create(1));

    assert_eq!(b.poll(0), None);
    assert_eq!(b.offer(1, 0), Ok(true));
    assert_eq!(b.offer(2, 0), Ok(false));
    assert_eq!(b.poll(0), Some(1));
    assert_eq!(b.size(), 0);
}

#[test]
fn should_hand_out_remaining_items_after_close_and_then_report_no_more_items() {
    let b = sut!(create(5));
    b.put(1).unwrap();
    b.put(2).unwrap();

    b.close();

    assert_eq!(b.size(), 2);
    assert_eq!(b.take(), Some(1));
    assert_eq!(b.poll(10), Some(2));
    assert_eq!(b.take(), None);
    assert_eq!(b.poll(10), None);
    assert_eq!(b.size(), 0);
}

#[test]
fn should_reject_put_and_offer_after_close() {
    let b = sut!(create(5));
    b.put(1).unwrap();
    b.close();
    b.close();

    assert_eq!(b.put(2), Err(Closed));
    assert_eq!(b.offer(3, 10), Err(Closed));
    assert_eq!(b.size(), 1);
    assert_eq!(b.take(), Some(1));
    assert_eq!(b.take(), None);
}

#[test]
fn should_wake_blocked_takers_when_closed() {
    let b = sut!(create(2));
    let done = Arc::new(Mutex::new(Vec::<Option<i32>>::new()));
    for _ in 0..3 {
        let (b, done) = (b.clone(), done.clone());
        thread::spawn(move || {
            let item = b.take();
            done.lock().unwrap().push(item);
        });
    }
    assert!(
        stays(150, || done.lock().unwrap().is_empty()),
        "takers wait"
    );

    b.close();

    assert!(
        eventually(5_000, || done.lock().unwrap().len() == 3),
        "close wakes every blocked taker"
    );
    assert!(done.lock().unwrap().iter().all(|r| r.is_none()));
}

#[test]
fn should_fail_blocked_producers_without_storing_their_items_when_closed() {
    let b = sut!(create(1));
    b.put(1).unwrap();
    let put_result: Arc<Mutex<Option<Result<(), Closed>>>> = Arc::new(Mutex::new(None));
    let offer_result: Arc<Mutex<Option<Result<bool, Closed>>>> = Arc::new(Mutex::new(None));
    {
        let (b, r) = (b.clone(), put_result.clone());
        thread::spawn(move || {
            let res = b.put(2);
            *r.lock().unwrap() = Some(res);
        });
    }
    {
        let (b, r) = (b.clone(), offer_result.clone());
        thread::spawn(move || {
            let res = b.offer(3, 60_000);
            *r.lock().unwrap() = Some(res);
        });
    }
    assert!(
        stays(150, || put_result.lock().unwrap().is_none()
            && offer_result.lock().unwrap().is_none()),
        "producers wait while full"
    );

    b.close();

    assert!(
        eventually(5_000, || put_result.lock().unwrap().is_some()
            && offer_result.lock().unwrap().is_some()),
        "close fails every blocked producer"
    );
    assert_eq!(*put_result.lock().unwrap(), Some(Err(Closed)));
    assert_eq!(*offer_result.lock().unwrap(), Some(Err(Closed)));
    assert_eq!(b.size(), 1);
    assert_eq!(b.take(), Some(1));
    assert_eq!(b.take(), None);
}

#[test]
fn should_deliver_every_item_exactly_once_with_many_producers_and_consumers() {
    const PRODUCERS: i32 = 4;
    const CONSUMERS: usize = 4;
    const PER_PRODUCER: i32 = 10_000;
    let b = sut!(create(16));
    let max_size = Arc::new(AtomicUsize::new(0));

    let consumers: Vec<_> = (0..CONSUMERS)
        .map(|_| {
            let b = b.clone();
            thread::spawn(move || {
                let mut mine = Vec::new();
                while let Some(v) = b.take() {
                    mine.push(v);
                }
                mine
            })
        })
        .collect();
    let producers: Vec<_> = (0..PRODUCERS)
        .map(|p| {
            let (b, max_size) = (b.clone(), max_size.clone());
            thread::spawn(move || {
                for i in 0..PER_PRODUCER {
                    b.put(p * PER_PRODUCER + i).unwrap();
                    max_size.fetch_max(b.size(), Ordering::SeqCst);
                }
            })
        })
        .collect();

    for p in producers {
        p.join().unwrap();
    }
    b.close();
    let received: Vec<Vec<i32>> = consumers.into_iter().map(|c| c.join().unwrap()).collect();

    let mut all = HashSet::new();
    let mut total = 0;
    for mine in &received {
        total += mine.len();
        all.extend(mine.iter().copied());
        let mut last = [-1i32; PRODUCERS as usize];
        for &v in mine {
            let p = (v / PER_PRODUCER) as usize;
            assert!(v > last[p], "per-producer order within one consumer");
            last[p] = v;
        }
    }
    assert_eq!(total, (PRODUCERS * PER_PRODUCER) as usize);
    assert_eq!(all.len(), (PRODUCERS * PER_PRODUCER) as usize);
    assert!(max_size.load(Ordering::SeqCst) <= 16);
}

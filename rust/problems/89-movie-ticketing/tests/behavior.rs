#![allow(unused_imports, clippy::needless_range_loop)]
use p89_movie_ticketing::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut, FakeClock};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn minutes(clock: &FakeClock, m: u64) {
    clock.advance(m * 60_000);
}

/// rows A,B premium; C,D regular; 20 seats; show1 at minute 600 for 150 minutes (regular 200, premium 350)
fn api() -> Option<(Arc<dyn Ticketing>, FakeClock)> {
    let clock = FakeClock::new(0);
    let t = create(clock.as_clock())?;
    t.add_screen("PVR", "blr", "s1", 4, 5).unwrap();
    t.add_show("show1", "PVR", "s1", "dune", 600, 150, 200, 350)
        .unwrap();
    Some((t, clock))
}

#[test]
fn should_list_all_seats_in_row_then_number_order() {
    let (t, _) = sut!(api());

    let seats = t.available_seats("show1").unwrap();

    assert_eq!(seats.len(), 20);
    assert_eq!(seats[..6], strs(&["A1", "A2", "A3", "A4", "A5", "B1"])[..]);
    assert_eq!(seats[19], "D5");
}

#[test]
fn should_search_upcoming_shows_by_city_and_movie_in_time_order() {
    let (t, clock) = sut!(api());
    t.add_show("late", "PVR", "s1", "dune", 900, 150, 200, 350)
        .unwrap();
    t.add_show("other", "PVR", "s1", "barbie", 750, 100, 200, 350)
        .unwrap();
    t.add_screen("INOX", "del", "s9", 2, 2).unwrap();
    t.add_show("delhi", "INOX", "s9", "dune", 610, 150, 100, 100)
        .unwrap();

    assert_eq!(t.search_shows("blr", "dune"), strs(&["show1", "late"]));
    assert_eq!(t.search_shows("del", "dune"), strs(&["delhi"]));
    assert!(t.search_shows("blr", "nothing").is_empty());

    minutes(&clock, 700); // show1 (600) has started
    assert_eq!(t.search_shows("blr", "dune"), strs(&["late"]));
}

#[test]
fn should_reject_overlapping_shows_on_the_same_screen_but_allow_touching() {
    let (t, _) = sut!(api()); // show1 600..750

    assert_eq!(
        t.add_show("clash", "PVR", "s1", "x", 700, 60, 1, 1),
        Err(TicketError::Conflict)
    );
    t.add_show("next", "PVR", "s1", "x", 750, 60, 1, 1).unwrap();
    assert_eq!(
        t.add_show("show1", "PVR", "s1", "x", 2_000, 60, 1, 1),
        Err(TicketError::InvalidInput)
    );
    assert_eq!(
        t.add_show("bad", "PVR", "nope", "x", 2_000, 60, 1, 1),
        Err(TicketError::InvalidInput)
    );
    assert_eq!(
        t.add_show("bad", "PVR", "s1", "x", 2_000, 0, 1, 1),
        Err(TicketError::InvalidInput)
    );
}

#[test]
fn should_hide_held_seats_from_others_and_refuse_double_holds() {
    let (t, _) = sut!(api());

    let hold = t.hold("show1", "ann", &["A1", "A2"]).unwrap();

    assert!(!hold.is_empty());
    let free = t.available_seats("show1").unwrap();
    assert_eq!(free.len(), 18);
    assert!(!free.contains(&"A1".to_string()) && !free.contains(&"A2".to_string()));
    assert_eq!(
        t.hold("show1", "bob", &["A2", "A3"]),
        Err(TicketError::Conflict)
    );
    assert!(
        t.available_seats("show1")
            .unwrap()
            .contains(&"A3".to_string()),
        "all-or-nothing"
    );
}

#[test]
fn should_reject_invalid_holds() {
    let (t, _) = sut!(api());

    assert_eq!(
        t.hold("nope", "ann", &["A1"]),
        Err(TicketError::InvalidInput)
    );
    assert_eq!(
        t.hold("show1", "ann", &["Z9"]),
        Err(TicketError::InvalidInput)
    );
    assert_eq!(
        t.hold("show1", "ann", &["A6"]),
        Err(TicketError::InvalidInput)
    );
    assert_eq!(t.hold("show1", "ann", &[]), Err(TicketError::InvalidInput));
    assert_eq!(
        t.hold("show1", "ann", &["A1", "A1"]),
        Err(TicketError::InvalidInput)
    );
}

#[test]
fn should_release_expired_holds_after_five_minutes() {
    let (t, clock) = sut!(api());
    let hold = t.hold("show1", "ann", &["A1"]).unwrap();

    minutes(&clock, 4);
    assert!(!t
        .available_seats("show1")
        .unwrap()
        .contains(&"A1".to_string()));
    minutes(&clock, 1);

    assert!(t
        .available_seats("show1")
        .unwrap()
        .contains(&"A1".to_string()));
    assert_eq!(t.confirm(&hold, "ann"), Err(TicketError::Gone));
    t.hold("show1", "bob", &["A1"]).unwrap();
}

#[test]
fn should_confirm_a_hold_into_a_booking_with_tiered_prices() {
    let (t, clock) = sut!(api());
    let hold = t.hold("show1", "ann", &["B2", "A1", "C3"]).unwrap();

    let booking = t.confirm(&hold, "ann").unwrap();

    assert_eq!(t.seats_of(&booking), Ok(strs(&["A1", "B2", "C3"])));
    assert_eq!(t.total_of(&booking), Ok(350 + 350 + 200));
    assert_eq!(t.available_seats("show1").unwrap().len(), 17);
    minutes(&clock, 10); // a confirmed seat never comes back
    let free = t.available_seats("show1").unwrap();
    assert_eq!(free.len(), 17);
    assert!(!free.contains(&"A1".to_string()));
}

#[test]
fn should_protect_holds_from_other_users_and_double_confirm() {
    let (t, _) = sut!(api());
    let hold = t.hold("show1", "ann", &["A1"]).unwrap();

    assert_eq!(t.confirm(&hold, "bob"), Err(TicketError::NotAllowed));
    assert_eq!(t.release(&hold, "bob"), Err(TicketError::NotAllowed));
    t.confirm(&hold, "ann").unwrap();
    assert_eq!(t.confirm(&hold, "ann"), Err(TicketError::Gone));
}

#[test]
fn should_free_seats_when_a_hold_is_released() {
    let (t, _) = sut!(api());
    let hold = t.hold("show1", "ann", &["A1", "A2"]).unwrap();

    t.release(&hold, "ann").unwrap();

    assert_eq!(t.available_seats("show1").unwrap().len(), 20);
    assert_eq!(t.release(&hold, "ann"), Err(TicketError::Gone));
}

#[test]
fn should_refuse_to_hold_after_the_show_started() {
    let (t, clock) = sut!(api());
    minutes(&clock, 600);

    assert_eq!(t.hold("show1", "ann", &["A1"]), Err(TicketError::Conflict));
}

#[test]
fn should_refund_according_to_how_close_the_show_is() {
    let (t, clock) = sut!(api()); // show at minute 600
    let b1 = t
        .confirm(&t.hold("show1", "ann", &["A1"]).unwrap(), "ann")
        .unwrap(); // 350
    let b2 = t
        .confirm(&t.hold("show1", "ann", &["C1", "C2", "C3"]).unwrap(), "ann")
        .unwrap(); // 600
    let b3 = t
        .confirm(&t.hold("show1", "ann", &["D1"]).unwrap(), "ann")
        .unwrap(); // 200

    assert_eq!(t.cancel(&b1, "ann"), Ok(350)); // 600 minutes away
    minutes(&clock, 500); // 100 minutes before the show
    assert_eq!(t.cancel(&b2, "ann"), Ok(300));
    minutes(&clock, 75); // 25 minutes before
    assert_eq!(t.cancel(&b3, "ann"), Ok(0));
    assert_eq!(t.available_seats("show1").unwrap().len(), 20);
    assert_eq!(t.cancel(&b3, "ann"), Err(TicketError::Gone));
}

#[test]
fn should_not_let_others_cancel_a_booking() {
    let (t, _) = sut!(api());
    let b = t
        .confirm(&t.hold("show1", "ann", &["A1"]).unwrap(), "ann")
        .unwrap();

    assert_eq!(t.cancel(&b, "bob"), Err(TicketError::NotAllowed));
}

#[test]
fn should_let_exactly_one_thread_hold_the_same_seat() {
    let (t, _) = sut!(api());

    let wins = collect_concurrently(16, |i| {
        t.hold("show1", &format!("user{i}"), &["A1"]).is_ok()
    });

    assert_eq!(wins.iter().filter(|w| **w).count(), 1);
}

#[test]
fn should_never_sell_the_same_seat_twice_under_load() {
    let clock = FakeClock::new(0);
    let t = sut!(create(clock.as_clock()));
    t.add_screen("PVR", "blr", "big", 10, 20).unwrap(); // 200 seats
    t.add_show("big", "PVR", "big", "m", 10_000, 120, 100, 200)
        .unwrap();
    let sold = Mutex::new(HashSet::new());
    let duplicates = AtomicUsize::new(0);

    run_concurrently(16, |u| {
        let mut s = (u as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut next = |n: u64| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s % n
        };
        for _ in 0..100 {
            let a = format!("{}{}", (b'A' + next(10) as u8) as char, 1 + next(20));
            let b = format!("{}{}", (b'A' + next(10) as u8) as char, 1 + next(20));
            let seats: Vec<&str> = if a == b { vec![&a] } else { vec![&a, &b] };
            let user = format!("user{u}");
            if let Ok(hold) = t.hold("big", &user, &seats) {
                let booking = t.confirm(&hold, &user).unwrap();
                for seat in t.seats_of(&booking).unwrap() {
                    if !sold.lock().unwrap().insert(seat) {
                        duplicates.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        }
    });

    assert_eq!(duplicates.load(Ordering::SeqCst), 0);
    assert_eq!(
        t.available_seats("big").unwrap().len(),
        200 - sold.lock().unwrap().len()
    );
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let (t, _) = sut!(api());
        let won = AtomicUsize::new(0);

        run_concurrently(8, |u| {
            if t.hold("show1", &format!("u{u}"), &["A1", "A2"]).is_ok() {
                won.fetch_add(1, Ordering::SeqCst);
            }
        });

        assert_eq!(won.load(Ordering::SeqCst), 1, "run {run}");
        assert_eq!(t.available_seats("show1").unwrap().len(), 18, "run {run}");
    }
}

#![allow(unused_imports, clippy::needless_range_loop)]
use p89_movie_ticketing::{contract::*, create};
use playground_support::{run_concurrently, sut, FakeClock};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn should_recycle_expiring_holds_while_others_keep_booking() {
    let clock = FakeClock::new(0);
    let t = sut!(create(clock.as_clock()));
    t.add_screen("PVR", "blr", "s", 5, 10).unwrap();
    t.add_show("sh", "PVR", "s", "m", 100_000, 60, 10, 20)
        .unwrap();
    let bookings = AtomicUsize::new(0);

    run_concurrently(9, |u| {
        for i in 0..200 {
            if u == 8 {
                clock.advance(60_000); // time passes: holds expire
                continue;
            }
            let seat = format!("{}{}", (b'A' + (u % 5) as u8) as char, 1 + i % 10);
            if let Ok(hold) = t.hold("sh", &format!("u{u}"), &[&seat]) {
                if i % 2 == 0 && t.confirm(&hold, &format!("u{u}")).is_ok() {
                    bookings.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    clock.advance(10 * 60_000); // every hold that is left has expired now
    assert_eq!(
        t.available_seats("sh").unwrap().len() + bookings.load(Ordering::SeqCst),
        50
    );
}

#[test]
fn should_search_across_many_shows() {
    let clock = FakeClock::new(0);
    let t = sut!(create(clock.as_clock()));
    t.add_screen("PVR", "blr", "s", 2, 2).unwrap();
    for i in 0..2_000u64 {
        t.add_show(
            &format!("sh{i}"),
            "PVR",
            "s",
            if i % 2 == 0 { "a" } else { "b" },
            1_000 + i * 200,
            100,
            10,
            20,
        )
        .unwrap();
    }

    assert_eq!(t.search_shows("blr", "a").len(), 1_000);
}

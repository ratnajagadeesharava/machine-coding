use p39_aggregation::contract::MusicError;
use p39_aggregation::{Playlist, Song};
use std::rc::Rc;

fn song(title: &str, secs: u32) -> Rc<Song> {
    Rc::new(Song::new(title, secs).unwrap())
}

#[test]
fn should_keep_songs_in_order_and_sum_their_length() {
    let mut p = Playlist::new("Road trip");
    let a = song("A", 180);
    let b = song("B", 200);

    assert!(p.add(a.clone()));
    assert!(p.add(b.clone()));

    assert_eq!(p.name(), "Road trip");
    assert_eq!(p.songs().len(), 2);
    assert!(Rc::ptr_eq(&p.songs()[0], &a));
    assert!(Rc::ptr_eq(&p.songs()[1], &b));
    assert_eq!(p.total_seconds(), 380);
}

#[test]
fn should_hold_the_same_instances_not_copies() {
    let mut p = Playlist::new("P");
    let s = song("Original", 100);
    p.add(s.clone());

    s.rename("Renamed").unwrap();

    assert_eq!(p.songs()[0].title(), "Renamed");
}

#[test]
fn should_count_references_as_playlists_come_and_go() {
    let s = song("S", 60);
    assert_eq!(Rc::strong_count(&s), 1);

    let mut one = Playlist::new("one");
    let mut two = Playlist::new("two");
    one.add(s.clone());
    two.add(s.clone());
    assert_eq!(Rc::strong_count(&s), 3); // the test + two playlists

    drop(one); // deleting a playlist releases its reference, nothing more
    assert_eq!(Rc::strong_count(&s), 2);
    two.clear();
    assert_eq!(Rc::strong_count(&s), 1);
    assert_eq!(s.title(), "S"); // the song is alive and usable
}

#[test]
fn should_let_songs_survive_when_the_playlist_is_dropped() {
    let s = song("Survivor", 60);
    let mut first = Playlist::new("first");
    first.add(s.clone());
    drop(first);

    let mut second = Playlist::new("second");

    assert!(second.add(s.clone()));
    assert_eq!(second.total_seconds(), 60);
}

#[test]
fn should_remove_only_from_that_playlist() {
    let s = song("S", 10);
    let mut one = Playlist::new("one");
    let mut two = Playlist::new("two");
    one.add(s.clone());
    two.add(s.clone());

    assert!(one.remove(&s));
    assert!(!one.remove(&s));

    assert!(one.songs().is_empty());
    assert_eq!(two.songs().len(), 1);
    assert_eq!(s.seconds(), 10);
}

#[test]
fn should_refuse_the_same_rc_twice_but_allow_equal_looking_songs() {
    let mut p = Playlist::new("P");
    let a = song("Same", 10);
    let twin = song("Same", 10);

    assert!(p.add(a.clone()));
    assert!(!p.add(a.clone()));
    assert!(p.add(twin));
    assert_eq!(p.songs().len(), 2);
}

#[test]
fn should_reject_invalid_songs() {
    assert!(matches!(Song::new(" ", 10), Err(MusicError::InvalidTitle)));
    assert!(matches!(Song::new("T", 0), Err(MusicError::InvalidDuration)));
    assert_eq!(Song::new("T", 5).unwrap().rename(""), Err(MusicError::InvalidTitle));
}

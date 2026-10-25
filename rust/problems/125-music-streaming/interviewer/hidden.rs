#![allow(unused_imports, clippy::needless_range_loop)]
use p125_music_streaming::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::sync::Mutex;
use std::time::Instant;

fn v(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_find_songs_in_a_large_catalogue_quickly() {
    let m = sut!(create());
    m.register_user("u").unwrap();
    for i in 0..100_000 {
        m.add_song(
            &format!("s{i}"),
            &format!("Song w{i} moon"),
            &format!("Artist a{}", i % 1000),
            100,
        )
        .unwrap();
    }
    m.play("u", "s4242").unwrap();

    let start = Instant::now();
    for i in 0..300 {
        let r = m.search("w4242 a242", 5).unwrap();
        assert_eq!(r, v(&["s4242"]));
        let n = i * 7 + 10;
        let r = m.search(&format!("w{n} a{}", n % 1000), 3).unwrap();
        assert!(r.contains(&format!("s{n}")));
    }
    assert!(start.elapsed().as_millis() < 30_000);
}

#[test]
fn should_re_rank_search_as_plays_accumulate() {
    let m = sut!(create());
    m.register_user("u").unwrap();
    m.add_song("a", "Alpha Wave", "Zen", 100).unwrap();
    m.add_song("b", "Beta Wave", "Zen", 100).unwrap();
    m.add_song("c", "Gamma Wave", "Zen", 100).unwrap();

    assert_eq!(m.search("wave", 3), Ok(v(&["a", "b", "c"])));
    m.play("u", "c").unwrap();
    assert_eq!(m.search("wave", 3), Ok(v(&["c", "a", "b"])));
    m.play("u", "b").unwrap();
    m.play("u", "b").unwrap();
    assert_eq!(m.search("wave", 3), Ok(v(&["b", "c", "a"])));
    m.play("u", "c").unwrap();
    assert_eq!(m.search("wave", 3), Ok(v(&["b", "c", "a"])));
    m.play("u", "c").unwrap();
    assert_eq!(m.search("wave", 2), Ok(v(&["c", "b"])));
}

#[test]
fn should_allow_a_query_token_to_match_the_same_word_as_another_token() {
    let m = sut!(create());
    m.add_song("x", "Rock Rocket", "Rocky Road", 100).unwrap();
    m.add_song("y", "Rock", "Band", 100).unwrap();

    assert_eq!(m.search("rock rock", 5), Ok(v(&["y", "x"])));
    assert_eq!(m.search("rocket rocky", 5), Ok(v(&["x"])));
    assert_eq!(m.search("r r r", 5), Ok(v(&["y", "x"])));
    assert_eq!(m.search("rock road band", 5), Ok(vec![]));
}

#[test]
fn should_keep_history_distinct_when_the_same_song_is_played_repeatedly() {
    let m = sut!(create());
    m.register_user("u").unwrap();
    for i in 0..11 {
        m.add_song(&format!("g{i}"), &format!("T{i}"), "B", 60).unwrap();
    }
    for i in 0..11 {
        m.play("u", &format!("g{i}")).unwrap();
        m.play("u", &format!("g{i}")).unwrap();
    }
    let h = m.recently_played("u").unwrap();
    assert_eq!(h.len(), 10);
    assert_eq!(h[0], "g10");
    assert!(!h.contains(&"g0".to_string()));
    assert_eq!(m.user_play_count("u", "g0"), Ok(2));

    m.play("u", "g0").unwrap();
    let h = m.recently_played("u").unwrap();
    assert_eq!(h.len(), 10);
    assert_eq!(&h[..2], &v(&["g0", "g10"])[..]);
    assert!(!h.contains(&"g1".to_string()));
}

#[test]
fn should_keep_two_users_completely_independent() {
    let m = sut!(create());
    m.register_user("a").unwrap();
    m.register_user("b").unwrap();
    m.add_song("s1", "One", "X", 10).unwrap();
    m.add_song("s2", "Two", "X", 10).unwrap();
    m.add_song("s3", "Three", "X", 10).unwrap();

    let pa = m.create_playlist("a", "same").unwrap();
    let pb = m.create_playlist("b", "same").unwrap();
    m.add_to_playlist("a", &pa, "s1").unwrap();
    m.add_to_playlist("b", &pb, "s2").unwrap();
    m.enqueue("a", "s3").unwrap();
    m.play("a", "s1").unwrap();

    assert_eq!(m.playlist("a", &pa), Ok(v(&["s1"])));
    assert_eq!(m.playlist("b", &pb), Ok(v(&["s2"])));
    assert_eq!(m.recently_played("b"), Ok(vec![]));
    assert_eq!(m.play_next("b"), Ok(None));
    assert_eq!(m.recommendations("b", 5), Ok(vec![]));
    assert_eq!(m.recommendations("a", 5), Ok(v(&["s2", "s3"])));
    m.delete_playlist("a", &pa).unwrap();
    assert_eq!(m.playlist("b", &pb), Ok(v(&["s2"])));
}

#[test]
fn should_exclude_queue_played_songs_from_recommendations_and_rank_by_artist_plays() {
    let m = sut!(create());
    m.register_user("u").unwrap();
    m.add_song("a1", "A one", "Alpha", 10).unwrap();
    m.add_song("a2", "A two", "alpha", 10).unwrap();
    m.add_song("a3", "A three", "Alpha", 10).unwrap();
    m.add_song("b1", "B one", "Beta", 10).unwrap();
    m.add_song("b2", "B two", "Beta", 10).unwrap();
    m.add_song("c1", "C one", "Gamma", 10).unwrap();

    for s in ["a1", "a2", "b1"] {
        m.enqueue("u", s).unwrap();
    }
    for _ in 0..3 {
        m.play_next("u").unwrap();
    }

    assert_eq!(m.recommendations("u", 10), Ok(v(&["a3", "b2"])));
    m.play("u", "a1").unwrap();
    for _ in 0..3 {
        m.play("u", "b1").unwrap();
    }
    assert_eq!(m.recommendations("u", 10), Ok(v(&["b2", "a3"])));
    assert_eq!(m.recommendations("u", 1), Ok(v(&["b2"])));
}

#[test]
fn should_stay_consistent_under_mixed_concurrent_traffic() {
    let m = sut!(create());
    for i in 0..50 {
        m.add_song(&format!("s{i}"), &format!("Song {i}"), &format!("Artist {}", i % 5), 100)
            .unwrap();
    }
    for i in 0..8 {
        m.register_user(&format!("u{i}")).unwrap();
    }
    let playlists = Mutex::new(vec![String::new(); 8]);

    run_concurrently(8, |t| {
        let u = format!("u{t}");
        let pl = m.create_playlist(&u, "list").unwrap();
        playlists.lock().unwrap()[t] = pl.clone();
        for i in 0..2_000 {
            let s = format!("s{}", i % 50);
            m.play(&u, &s).unwrap();
            if i % 7 == 0 {
                m.like(&u, &s).unwrap();
            }
            if i % 11 == 0 {
                m.enqueue(&u, &s).unwrap();
            }
            if i % 13 == 0 {
                m.play_next(&u).unwrap();
            }
            if i < 50 {
                m.add_to_playlist(&u, &pl, &s).unwrap();
            }
            m.search("song artist", 5).unwrap();
            m.top_songs(5).unwrap();
            m.recommendations(&u, 5).unwrap();
            m.recently_played(&u).unwrap();
        }
    });

    // Draining the queues turns every enqueue (182 per user: i % 11 == 0 over 0..1999) into exactly one play.
    for t in 0..8 {
        while m.play_next(&format!("u{t}")).unwrap().is_some() {}
    }
    let total: u64 = (0..50).map(|i| m.play_count(&format!("s{i}")).unwrap()).sum();
    assert_eq!(total, 8 * 2_000 + 8 * 182);
    assert_eq!(m.top_songs(100).unwrap().len(), 50);
    let expected: Vec<String> = (0..50).map(|i| format!("s{i}")).collect();
    let p3 = playlists.lock().unwrap()[3].clone();
    assert_eq!(m.playlist("u3", &p3), Ok(expected));
    assert_eq!(m.recently_played("u3").unwrap().len(), 10);
    assert_eq!(m.song_likes("s0"), Ok(8));
}

#[test]
fn should_serialize_concurrent_queue_appends_without_losing_any() {
    let m = sut!(create());
    m.add_song("s", "Song", "Artist", 100).unwrap();
    m.register_user("u").unwrap();
    run_concurrently(8, |_| {
        for _ in 0..1_000 {
            m.enqueue("u", "s").unwrap();
        }
    });
    let mut n = 0;
    while m.play_next("u").unwrap().is_some() {
        n += 1;
    }
    assert_eq!(n, 8_000);
    assert_eq!(m.play_count("s"), Ok(8_000));
}

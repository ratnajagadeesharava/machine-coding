#![allow(unused_imports, clippy::needless_range_loop)]
use p125_music_streaming::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

type Svc = Arc<dyn MusicService>;

fn v(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// Small catalogue plus users alice, bob.
fn catalogue(m: &Svc) {
    m.add_song("q1", "Bohemian Rhapsody", "Queen", 354).unwrap();
    m.add_song("q2", "Don't Stop Me Now", "Queen", 209).unwrap();
    m.add_song("q3", "Another One Bites the Dust", "Queen", 215).unwrap();
    m.add_song("j1", "Imagine", "John Lennon", 183).unwrap();
    m.add_song("b1", "Hey Jude", "The Beatles", 431).unwrap();
    m.add_song("b2", "Let It Be", "The Beatles", 243).unwrap();
    m.add_song("b3", "Yesterday", "The Beatles", 125).unwrap();
    m.register_user("alice").unwrap();
    m.register_user("bob").unwrap();
}

fn plays(m: &Svc, user: &str, song: &str, times: usize) {
    for _ in 0..times {
        m.play(user, song).unwrap();
    }
}

#[test]
fn should_reject_duplicate_and_invalid_songs() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.add_song("q1", "Other", "Other", 100), Err(MusicError::AlreadyExists));
    assert_eq!(m.add_song(" ", "T", "A", 100), Err(MusicError::InvalidArgument));
    assert_eq!(m.add_song("x", "", "A", 100), Err(MusicError::InvalidArgument));
    assert_eq!(m.add_song("x", "T", " ", 100), Err(MusicError::InvalidArgument));
    assert_eq!(m.add_song("x", "T", "A", 0), Err(MusicError::InvalidArgument));
    assert_eq!(m.search("other", 5), Ok(vec![]));
}

#[test]
fn should_reject_duplicate_and_blank_users() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.register_user("alice"), Err(MusicError::AlreadyExists));
    assert_eq!(m.register_user(" "), Err(MusicError::InvalidArgument));
}

#[test]
fn should_search_by_title_prefix_ignoring_case() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("BOHE", 10), Ok(v(&["q1"])));
    assert_eq!(m.search("rhapsody", 10), Ok(v(&["q1"])));
    assert_eq!(m.search("Imag", 10), Ok(v(&["j1"])));
}

#[test]
fn should_search_by_artist_word_prefix() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("queen", 10), Ok(v(&["q3", "q1", "q2"])));
    assert_eq!(m.search("ennon", 10), Ok(vec![]));
    assert_eq!(m.search("lenn", 10), Ok(v(&["j1"])));
}

#[test]
fn should_require_every_token_to_match_in_title_or_artist() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("queen rhap", 10), Ok(v(&["q1"])));
    assert_eq!(m.search("rhap queen", 10), Ok(v(&["q1"])));
    assert_eq!(m.search("john imag", 10), Ok(v(&["j1"])));
    assert_eq!(m.search("the be", 10), Ok(v(&["b1", "b2", "b3"])));
    assert_eq!(m.search("one dust", 10), Ok(v(&["q3"])));
    assert_eq!(m.search("queen imagine", 10), Ok(vec![]));
}

#[test]
fn should_match_words_not_substrings_and_not_whole_string_prefixes() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("hemian", 10), Ok(vec![]));
    assert_eq!(m.search("stop", 10), Ok(v(&["q2"])));
    assert_eq!(m.search("me now", 10), Ok(v(&["q2"])));
    assert_eq!(m.search("don't st", 10), Ok(v(&["q2"])));
    assert_eq!(m.search("stop don't", 10), Ok(v(&["q2"])));
}

#[test]
fn should_ignore_extra_whitespace_in_the_query() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("  queen \t  rhap  ", 10), Ok(v(&["q1"])));
}

#[test]
fn should_rank_search_results_by_plays_then_title_then_id() {
    let m = sut!(create());
    catalogue(&m);
    plays(&m, "alice", "b3", 2);
    plays(&m, "bob", "q1", 1);

    assert_eq!(m.search("the be", 10), Ok(v(&["b3", "b1", "b2"])));
    assert_eq!(m.search("queen", 10), Ok(v(&["q1", "q3", "q2"])));
}

#[test]
fn should_break_title_ties_case_insensitively_then_by_id() {
    let m = sut!(create());
    m.add_song("a1", "Banana", "Fruit", 100).unwrap();
    m.add_song("z1", "apple", "Fruit", 100).unwrap();
    m.add_song("e2", "Echo", "Zed", 100).unwrap();
    m.add_song("e1", "echo", "Zed", 100).unwrap();

    assert_eq!(m.search("fruit", 10), Ok(v(&["z1", "a1"])));
    assert_eq!(m.search("zed", 10), Ok(v(&["e1", "e2"])));
}

#[test]
fn should_limit_search_to_k_and_reject_bad_search_input() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.search("queen", 2), Ok(v(&["q3", "q1"])));
    assert_eq!(m.search("nothing matches", 5), Ok(vec![]));
    assert_eq!(m.search("queen", 0), Err(MusicError::InvalidArgument));
    assert_eq!(m.search("queen", -1), Err(MusicError::InvalidArgument));
    assert_eq!(m.search("   ", 5), Err(MusicError::InvalidArgument));
}

#[test]
fn should_count_plays_globally_and_per_user() {
    let m = sut!(create());
    catalogue(&m);
    plays(&m, "alice", "q1", 2);
    plays(&m, "bob", "q1", 1);

    assert_eq!(m.play_count("q1"), Ok(3));
    assert_eq!(m.user_play_count("alice", "q1"), Ok(2));
    assert_eq!(m.user_play_count("bob", "q1"), Ok(1));
    assert_eq!(m.user_play_count("alice", "q2"), Ok(0));
    assert_eq!(m.play_count("q2"), Ok(0));
}

#[test]
fn should_reject_unknown_users_and_songs_without_changing_state() {
    let m = sut!(create());
    catalogue(&m);

    assert_eq!(m.play("ghost", "q1"), Err(MusicError::UnknownUser));
    assert_eq!(m.play("alice", "nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.like("ghost", "q1"), Err(MusicError::UnknownUser));
    assert_eq!(m.like("alice", "nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.unlike("alice", "nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.play_count("nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.song_likes("nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.recently_played("ghost"), Err(MusicError::UnknownUser));
    assert_eq!(m.enqueue("alice", "nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.play_next("ghost"), Err(MusicError::UnknownUser));
    assert_eq!(m.play_count("q1"), Ok(0));
    assert_eq!(m.recently_played("alice"), Ok(vec![]));
}

#[test]
fn should_keep_recent_history_most_recent_first_without_duplicates() {
    let m = sut!(create());
    catalogue(&m);
    for s in ["q1", "q2", "q3", "q1"] {
        m.play("alice", s).unwrap();
    }

    assert_eq!(m.recently_played("alice"), Ok(v(&["q1", "q3", "q2"])));
    assert_eq!(m.recently_played("bob"), Ok(vec![]));
}

#[test]
fn should_keep_only_the_ten_most_recent_distinct_songs() {
    let m = sut!(create());
    m.register_user("u").unwrap();
    for i in 0..12 {
        m.add_song(&format!("g{i}"), &format!("Track {i}"), "Band", 60).unwrap();
    }
    for i in 0..12 {
        m.play("u", &format!("g{i}")).unwrap();
    }
    assert_eq!(
        m.recently_played("u"),
        Ok(v(&["g11", "g10", "g9", "g8", "g7", "g6", "g5", "g4", "g3", "g2"]))
    );

    m.play("u", "g5").unwrap();
    assert_eq!(
        m.recently_played("u"),
        Ok(v(&["g5", "g11", "g10", "g9", "g8", "g7", "g6", "g4", "g3", "g2"]))
    );

    m.play("u", "g0").unwrap();
    assert_eq!(
        m.recently_played("u"),
        Ok(v(&["g0", "g5", "g11", "g10", "g9", "g8", "g7", "g6", "g4", "g3"]))
    );
    assert_eq!(m.user_play_count("u", "g2"), Ok(1));
}

#[test]
fn should_count_likes_per_user_and_stay_idempotent() {
    let m = sut!(create());
    catalogue(&m);
    m.like("alice", "q1").unwrap();
    m.like("alice", "q1").unwrap();
    assert_eq!(m.song_likes("q1"), Ok(1));

    m.like("bob", "q1").unwrap();
    assert_eq!(m.song_likes("q1"), Ok(2));
    assert_eq!(m.song_likes("q2"), Ok(0));

    m.unlike("alice", "q1").unwrap();
    m.unlike("alice", "q1").unwrap();
    m.unlike("alice", "q2").unwrap();
    assert_eq!(m.song_likes("q1"), Ok(1));

    m.like("alice", "q1").unwrap();
    assert_eq!(m.song_likes("q1"), Ok(2));
}

#[test]
fn should_create_playlists_with_unique_names_per_user() {
    let m = sut!(create());
    catalogue(&m);
    let a = m.create_playlist("alice", "road trip").unwrap();
    let b = m.create_playlist("alice", "gym").unwrap();
    let c = m.create_playlist("bob", "road trip").unwrap();

    assert!(a != b && b != c && a != c);
    assert_eq!(m.playlist("alice", &a), Ok(vec![]));
    assert_eq!(m.create_playlist("alice", "road trip"), Err(MusicError::AlreadyExists));
    assert_eq!(m.create_playlist("alice", " "), Err(MusicError::InvalidArgument));
    assert_eq!(m.create_playlist("ghost", "x"), Err(MusicError::UnknownUser));
}

#[test]
fn should_keep_playlist_order_and_reject_duplicate_adds() {
    let m = sut!(create());
    catalogue(&m);
    let p = m.create_playlist("alice", "mix").unwrap();
    for s in ["q2", "b1", "q1"] {
        m.add_to_playlist("alice", &p, s).unwrap();
    }

    assert_eq!(m.playlist("alice", &p), Ok(v(&["q2", "b1", "q1"])));
    assert_eq!(m.add_to_playlist("alice", &p, "b1"), Err(MusicError::AlreadyExists));
    assert_eq!(m.playlist("alice", &p), Ok(v(&["q2", "b1", "q1"])));

    m.remove_from_playlist("alice", &p, "b1").unwrap();
    assert_eq!(m.playlist("alice", &p), Ok(v(&["q2", "q1"])));
    m.add_to_playlist("alice", &p, "b1").unwrap();
    assert_eq!(m.playlist("alice", &p), Ok(v(&["q2", "q1", "b1"])));
}

#[test]
fn should_reject_bad_playlist_edits() {
    let m = sut!(create());
    catalogue(&m);
    let p = m.create_playlist("alice", "mix").unwrap();

    assert_eq!(m.add_to_playlist("alice", &p, "nope"), Err(MusicError::UnknownSong));
    assert_eq!(m.remove_from_playlist("alice", &p, "q1"), Err(MusicError::NotInPlaylist));
    assert_eq!(
        m.add_to_playlist("alice", "pl-unknown", "q1"),
        Err(MusicError::UnknownPlaylist)
    );
    assert_eq!(m.playlist("alice", "pl-unknown"), Err(MusicError::UnknownPlaylist));
    assert_eq!(m.playlist("alice", &p), Ok(vec![]));
}

#[test]
fn should_hide_playlists_from_other_users() {
    let m = sut!(create());
    catalogue(&m);
    let p = m.create_playlist("alice", "mix").unwrap();
    m.add_to_playlist("alice", &p, "q1").unwrap();

    assert_eq!(m.playlist("bob", &p), Err(MusicError::UnknownPlaylist));
    assert_eq!(m.add_to_playlist("bob", &p, "q2"), Err(MusicError::UnknownPlaylist));
    assert_eq!(m.delete_playlist("bob", &p), Err(MusicError::UnknownPlaylist));
    assert_eq!(m.playlist("alice", &p), Ok(v(&["q1"])));
}

#[test]
fn should_delete_playlist_and_free_its_name() {
    let m = sut!(create());
    catalogue(&m);
    let p = m.create_playlist("alice", "mix").unwrap();
    m.add_to_playlist("alice", &p, "q1").unwrap();
    m.delete_playlist("alice", &p).unwrap();

    assert_eq!(m.playlist("alice", &p), Err(MusicError::UnknownPlaylist));
    assert_eq!(m.delete_playlist("alice", &p), Err(MusicError::UnknownPlaylist));
    let again = m.create_playlist("alice", "mix").unwrap();
    assert_ne!(again, p);
    assert_eq!(m.playlist("alice", &again), Ok(vec![]));
}

#[test]
fn should_play_queued_songs_in_order_and_count_them_as_plays() {
    let m = sut!(create());
    catalogue(&m);
    m.enqueue("alice", "q1").unwrap();
    m.enqueue("alice", "q2").unwrap();
    m.enqueue("alice", "q1").unwrap();

    assert_eq!(m.play_next("alice"), Ok(Some("q1".to_string())));
    assert_eq!(m.recently_played("alice"), Ok(v(&["q1"])));
    assert_eq!(m.play_next("alice"), Ok(Some("q2".to_string())));
    assert_eq!(m.play_next("alice"), Ok(Some("q1".to_string())));
    assert_eq!(m.play_next("alice"), Ok(None));

    assert_eq!(m.play_count("q1"), Ok(2));
    assert_eq!(m.user_play_count("alice", "q1"), Ok(2));
    assert_eq!(m.recently_played("alice"), Ok(v(&["q1", "q2"])));
    assert_eq!(m.play_next("bob"), Ok(None));
    assert_eq!(m.recently_played("bob"), Ok(vec![]));
}

#[test]
fn should_keep_queues_separate_per_user() {
    let m = sut!(create());
    catalogue(&m);
    m.enqueue("alice", "q1").unwrap();
    m.enqueue("bob", "q2").unwrap();

    assert_eq!(m.play_next("bob"), Ok(Some("q2".to_string())));
    assert_eq!(m.play_next("bob"), Ok(None));
    assert_eq!(m.play_next("alice"), Ok(Some("q1".to_string())));
}

#[test]
fn should_list_top_songs_by_play_count_then_title() {
    let m = sut!(create());
    catalogue(&m);
    assert_eq!(m.top_songs(5), Ok(vec![]));

    plays(&m, "alice", "q1", 2);
    plays(&m, "bob", "q1", 1);
    plays(&m, "bob", "b1", 3);
    m.play("alice", "q2").unwrap();

    assert_eq!(m.top_songs(10), Ok(v(&["q1", "b1", "q2"])));
    assert_eq!(m.top_songs(2), Ok(v(&["q1", "b1"])));
    assert_eq!(m.top_songs(0), Err(MusicError::InvalidArgument));
}

#[test]
fn should_recommend_unplayed_songs_of_played_artists() {
    let m = sut!(create());
    catalogue(&m);
    plays(&m, "alice", "q1", 2);
    m.play("alice", "q2").unwrap();
    m.play("alice", "b1").unwrap();
    plays(&m, "bob", "b3", 2);

    assert_eq!(m.recommendations("alice", 10), Ok(v(&["q3", "b3", "b2"])));
    assert_eq!(m.recommendations("alice", 2), Ok(v(&["q3", "b3"])));
    assert_eq!(m.recommendations("bob", 10), Ok(v(&["b1", "b2"])));
}

#[test]
fn should_rank_artists_before_global_popularity_in_recommendations() {
    let m = sut!(create());
    catalogue(&m);
    plays(&m, "alice", "b1", 3);
    m.play("alice", "q1").unwrap();
    plays(&m, "bob", "q3", 5);

    assert_eq!(m.recommendations("alice", 10), Ok(v(&["b2", "b3", "q3", "q2"])));
}

#[test]
fn should_recommend_nothing_for_new_users_or_fully_played_artists() {
    let m = sut!(create());
    catalogue(&m);
    assert_eq!(m.recommendations("alice", 5), Ok(vec![]));

    m.play("alice", "j1").unwrap();
    assert_eq!(m.recommendations("alice", 5), Ok(vec![]));
    assert_eq!(m.recommendations("alice", 0), Err(MusicError::InvalidArgument));
    assert_eq!(m.recommendations("ghost", 3), Err(MusicError::UnknownUser));
}

#[test]
fn should_count_queued_plays_towards_recommendations_and_treat_artist_names_case_insensitively() {
    let m = sut!(create());
    catalogue(&m);
    m.add_song("q4", "Radio Ga Ga", " queen ", 343).unwrap();
    m.enqueue("alice", "q4").unwrap();
    m.play_next("alice").unwrap();

    assert_eq!(m.recommendations("alice", 10), Ok(v(&["q1", "q2", "q3"])));
}

#[test]
fn should_count_every_concurrent_play() {
    let m = sut!(create());
    catalogue(&m);
    run_concurrently(16, |t| {
        for _ in 0..500 {
            m.play(if t % 2 == 0 { "alice" } else { "bob" }, "q1").unwrap();
        }
    });

    assert_eq!(m.play_count("q1"), Ok(8000));
    assert_eq!(m.user_play_count("alice", "q1"), Ok(4000));
    assert_eq!(m.user_play_count("bob", "q1"), Ok(4000));
    assert_eq!(m.recently_played("alice"), Ok(v(&["q1"])));
}

#[test]
fn should_pop_each_queued_song_exactly_once_under_contention() {
    let m = sut!(create());
    catalogue(&m);
    for i in 0..4000 {
        m.enqueue("alice", if i % 2 == 0 { "q1" } else { "q2" }).unwrap();
    }
    let popped = collect_concurrently(8, |_| {
        let mut n = 0;
        while m.play_next("alice").unwrap().is_some() {
            n += 1;
        }
        n
    });

    assert_eq!(popped.iter().sum::<usize>(), 4000);
    assert_eq!(m.play_count("q1"), Ok(2000));
    assert_eq!(m.play_count("q2"), Ok(2000));
}

#[test]
fn should_allow_exactly_one_winner_for_concurrent_duplicates() {
    let m = sut!(create());
    catalogue(&m);
    let (users, songs, lists) = (AtomicUsize::new(0), AtomicUsize::new(0), AtomicUsize::new(0));
    run_concurrently(8, |_| {
        if m.register_user("carol").is_ok() {
            users.fetch_add(1, Ordering::SeqCst);
        }
        if m.add_song("new", "New Song", "New Artist", 100).is_ok() {
            songs.fetch_add(1, Ordering::SeqCst);
        }
        if m.create_playlist("alice", "same").is_ok() {
            lists.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(users.load(Ordering::SeqCst), 1);
    assert_eq!(songs.load(Ordering::SeqCst), 1);
    assert_eq!(lists.load(Ordering::SeqCst), 1);
}

#[test]
fn should_count_concurrent_likes_from_different_users() {
    let m = sut!(create());
    m.add_song("s", "Song", "Artist", 100).unwrap();
    for i in 0..200 {
        m.register_user(&format!("u{i}")).unwrap();
    }
    run_concurrently(8, |_| {
        for i in 0..200 {
            m.like(&format!("u{i}"), "s").unwrap();
        }
    });

    assert_eq!(m.song_likes("s"), Ok(200));
}

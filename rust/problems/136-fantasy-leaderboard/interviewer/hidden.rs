#![allow(unused_imports, clippy::needless_range_loop)]
use p136_fantasy_leaderboard::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

type Board = Arc<dyn FantasyLeaderboard>;

fn squad(f: &Board, prefix: &str) -> Vec<String> {
    let roles = [
        "KEEPER",
        "BATTER",
        "BATTER",
        "BATTER",
        "BATTER",
        "BOWLER",
        "BOWLER",
        "BOWLER",
        "ALL_ROUNDER",
        "ALL_ROUNDER",
        "ALL_ROUNDER",
    ];
    (0..11)
        .map(|i| {
            let id = format!("{prefix}{}", i + 1);
            f.add_player(&id, "Player", roles[i]).unwrap();
            id
        })
        .collect()
}

fn refs(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

#[test]
fn should_return_to_a_tie_after_a_correction_and_restore_creation_order() {
    let f = sut!(create());
    f.create_team("A", "o", &refs(&squad(&f, "a")), "a1", "a2")
        .unwrap();
    f.create_team("B", "o", &refs(&squad(&f, "b")), "b1", "b2")
        .unwrap();
    f.record_event("b3", "WICKET", 1).unwrap();
    assert_eq!(f.leaderboard(2), vec!["1:B:50", "2:A:0"]);

    f.correct_event("b3", "WICKET", 1).unwrap();

    assert_eq!(f.leaderboard(2), vec!["1:A:0", "1:B:0"]);
    assert_eq!(f.rank_of("B"), Ok(1));
}

#[test]
fn should_skip_ranks_after_a_three_way_tie() {
    let f = sut!(create());
    for p in ["a", "b", "c", "d", "e"] {
        f.create_team(
            &format!("T{p}"),
            "o",
            &refs(&squad(&f, p)),
            &format!("{p}1"),
            &format!("{p}2"),
        )
        .unwrap();
    }
    f.record_event("a3", "RUN", 10).unwrap();
    f.record_event("b3", "RUN", 10).unwrap();
    f.record_event("c3", "RUN", 10).unwrap();
    f.record_event("d3", "RUN", 4).unwrap();

    assert_eq!(
        f.leaderboard(5),
        vec!["1:Ta:20", "1:Tb:20", "1:Tc:20", "4:Td:8", "5:Te:0"]
    );
    assert_eq!(f.leaderboard(2), vec!["1:Ta:20", "1:Tb:20"]);
    assert_eq!(f.rank_of("Te"), Ok(5));
}

#[test]
fn should_score_very_large_counts_without_overflow() {
    let f = sut!(create());
    f.create_team("T", "o", &refs(&squad(&f, "a")), "a1", "a2")
        .unwrap();

    f.record_event("a1", "WICKET", 2_000_000_000).unwrap();
    f.record_event("a1", "WICKET", 2_000_000_000).unwrap();

    assert_eq!(f.player_points("a1"), Ok(100_000_000_000));
    assert_eq!(f.team_score("T"), Ok(400_000_000_000));
    f.correct_event("a1", "WICKET", 2_000_000_000).unwrap();
    assert_eq!(f.team_score("T"), Ok(200_000_000_000));
}

#[test]
fn should_accept_exactly_one_of_sixteen_concurrent_creations_with_the_same_team_id() {
    let f = sut!(create());
    let squads: Vec<Vec<String>> = (0..16).map(|i| squad(&f, &format!("s{i}_"))).collect();
    let ok = AtomicUsize::new(0);

    run_concurrently(16, |t| {
        let s = &squads[t];
        if f.create_team("same", &format!("owner{t}"), &refs(s), &s[0], &s[1])
            .is_ok()
        {
            ok.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(ok.load(Ordering::SeqCst), 1);
    assert_eq!(f.leaderboard(5).len(), 1);
}

#[test]
fn should_keep_snapshots_valid_while_teams_are_created_and_events_recorded_concurrently() {
    let f = sut!(create());
    let shared = squad(&f, "s");
    let done = AtomicBool::new(false);
    let problems = Mutex::new(Vec::<String>::new());

    run_concurrently(16, |t| {
        if t < 4 {
            while !done.load(Ordering::SeqCst) {
                let lines = f.leaderboard(1_000);
                let mut seen = HashSet::new();
                let mut prev = i64::MAX;
                for line in &lines {
                    let p: Vec<&str> = line.split(':').collect();
                    let score: i64 = p[2].parse().unwrap();
                    if !seen.insert(p[1].to_string()) {
                        problems.lock().unwrap().push(format!("duplicate {}", p[1]));
                    }
                    if score > prev {
                        problems
                            .lock()
                            .unwrap()
                            .push(format!("order broken at {line}"));
                    }
                    prev = score;
                }
            }
            return;
        }
        if t < 12 {
            for i in 0..100 {
                f.create_team(
                    &format!("T{}", t * 100 + i),
                    "o",
                    &refs(&shared),
                    "s1",
                    "s2",
                )
                .unwrap();
            }
            return;
        }
        for i in 0..20_000 {
            f.record_event(&format!("s{}", 1 + i % 11), "RUN", 1)
                .unwrap();
        }
        if t == 12 {
            done.store(true, Ordering::SeqCst);
        }
    });
    done.store(true, Ordering::SeqCst);

    assert!(problems.lock().unwrap().is_empty());
    let lines = f.leaderboard(2_000);
    assert_eq!(lines.len(), 800);
    assert!(lines.iter().all(|l| l.starts_with("1:")));
    let expected = f.team_score("T400").unwrap();
    for t in 4..12 {
        for i in 0..100 {
            assert_eq!(f.team_score(&format!("T{}", t * 100 + i)), Ok(expected));
        }
    }
}

#[test]
fn should_match_rank_of_with_the_leaderboard_for_every_team() {
    let f = sut!(create());
    let n = 60;
    for i in 0..n {
        let s = squad(&f, &format!("p{i}_"));
        f.create_team(&format!("T{i}"), "o", &refs(&s), &s[0], &s[1])
            .unwrap();
    }
    for i in 0..n {
        f.record_event(&format!("p{i}_3"), "RUN", 1 + ((i * 7) % 10) as u32)
            .unwrap();
    }

    let lines = f.leaderboard(n);

    assert_eq!(lines.len(), n);
    for line in lines {
        let p: Vec<&str> = line.split(':').collect();
        assert_eq!(f.rank_of(p[1]), Ok(p[0].parse().unwrap()));
        assert_eq!(f.team_score(p[1]), Ok(p[2].parse().unwrap()));
    }
}

#[test]
fn should_not_lose_events_when_recording_and_correcting_the_same_player_from_many_threads() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.create_team("T", "o", &refs(&s), "a1", "a2").unwrap();
    f.create_team("U", "o", &refs(&s), "a3", "a1").unwrap();
    f.record_event("a5", "CATCH", 16 * 500).unwrap();

    run_concurrently(16, |_| {
        for _ in 0..500 {
            f.correct_event("a5", "CATCH", 1).unwrap();
            f.record_event("a5", "STUMPING", 1).unwrap();
        }
    });

    assert_eq!(f.player_points("a5"), Ok(16 * 500 * 12));
    assert_eq!(f.team_score("T"), Ok(2 * 16 * 500 * 12));
    assert_eq!(
        f.correct_event("a5", "CATCH", 1),
        Err(LeaderboardError::CorrectionExceedsRecorded)
    );
}

#[test]
fn should_record_two_hundred_thousand_events_over_many_teams_across_threads() {
    let f = sut!(create());
    let players = 2_000usize;
    let roles = ["KEEPER", "BATTER", "BATTER", "BOWLER", "ALL_ROUNDER"];
    for i in 0..players {
        f.add_player(&format!("P{i}"), "Player", roles[i % 5])
            .unwrap();
    }
    for i in 0..20_000usize {
        let ids: Vec<String> = (0..11)
            .map(|j| format!("P{}", (i * 3 + j * 437) % players))
            .collect();
        f.create_team(&format!("T{i}"), "o", &refs(&ids), &ids[0], &ids[1])
            .unwrap();
    }

    run_concurrently(16, |t| {
        for i in 0..12_500usize {
            f.record_event(&format!("P{}", (t * 131 + i * 17) % players), "RUN", 1)
                .unwrap();
            if i % 250 == 0 {
                assert_eq!(f.leaderboard(10).len(), 10);
            }
        }
    });

    let total: i64 = (0..players)
        .map(|i| f.player_points(&format!("P{i}")).unwrap())
        .sum();
    assert_eq!(total, 200_000);
}

#![allow(unused_imports, clippy::needless_range_loop)]
use p136_fantasy_leaderboard::{contract::*, create};
use playground_support::{run_concurrently, sut};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

type Board = Arc<dyn FantasyLeaderboard>;

/// Registers prefix1..prefix11: 1 KEEPER, 4 BATTER (2-5), 3 BOWLER (6-8), 3 ALL_ROUNDER (9-11).
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
            f.add_player(&id, &format!("Player {id}"), roles[i])
                .unwrap();
            id
        })
        .collect()
}

fn refs(v: &[String]) -> Vec<&str> {
    v.iter().map(|s| s.as_str()).collect()
}

/// Team with captain = first player of the squad and vice-captain = second.
fn team(f: &Board, team_id: &str, squad: &[String]) {
    f.create_team(
        team_id,
        &format!("owner-{team_id}"),
        &refs(squad),
        &squad[0],
        &squad[1],
    )
    .unwrap();
}

#[test]
fn should_start_every_team_at_zero_and_list_them_in_creation_order() {
    let f = sut!(create());
    team(&f, "T1", &squad(&f, "a"));
    team(&f, "T2", &squad(&f, "b"));
    team(&f, "T3", &squad(&f, "c"));

    assert_eq!(f.leaderboard(10), vec!["1:T1:0", "1:T2:0", "1:T3:0"]);
    assert_eq!(f.team_score("T2"), Ok(0));
    assert_eq!(f.rank_of("T3"), Ok(1));
}

#[test]
fn should_score_every_event_type_with_the_fixed_table() {
    let f = sut!(create());
    squad(&f, "a");
    f.record_event("a3", "RUN", 10).unwrap();
    f.record_event("a3", "FOUR_BONUS", 2).unwrap();
    f.record_event("a3", "SIX_BONUS", 3).unwrap();
    f.record_event("a3", "WICKET", 2).unwrap();
    f.record_event("a3", "CATCH", 1).unwrap();
    f.record_event("a3", "STUMPING", 1).unwrap();
    f.record_event("a3", "MAIDEN", 1).unwrap();
    f.record_event("a3", "DUCK", 1).unwrap();

    // 10 + 2 + 6 + 50 + 8 + 12 + 12 - 2
    assert_eq!(f.player_points("a3"), Ok(98));
    assert_eq!(f.player_points("a4"), Ok(0));
}

#[test]
fn should_double_the_captain_and_pay_the_vice_captain_one_and_a_half() {
    let f = sut!(create());
    team(&f, "T", &squad(&f, "a")); // captain a1, vice-captain a2

    f.record_event("a1", "RUN", 50).unwrap();
    assert_eq!(f.team_score("T"), Ok(200)); // 50 x2 = 100 points = 200 half points
    f.record_event("a2", "CATCH", 1).unwrap();
    assert_eq!(f.team_score("T"), Ok(224)); // + 8 x1.5 = 12 points = 24 half points
    f.record_event("a3", "WICKET", 1).unwrap();
    assert_eq!(f.team_score("T"), Ok(274)); // + 25 points = 50 half points
    assert_eq!(f.leaderboard(1), vec!["1:T:274"]);
}

#[test]
fn should_give_half_points_for_an_odd_vice_captain_total() {
    let f = sut!(create());
    team(&f, "T", &squad(&f, "a"));

    f.record_event("a2", "RUN", 1).unwrap(); // 1 x1.5 = 1.5 points

    assert_eq!(f.team_score("T"), Ok(3));
}

#[test]
fn should_update_every_team_containing_the_player_immediately() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.create_team("captainTeam", "x", &refs(&s), "a1", "a2")
        .unwrap();
    f.create_team("viceTeam", "y", &refs(&s), "a2", "a1")
        .unwrap();
    f.create_team("plainTeam", "z", &refs(&s), "a3", "a4")
        .unwrap();

    f.record_event("a1", "WICKET", 1).unwrap(); // 25 points

    assert_eq!(f.team_score("captainTeam"), Ok(100));
    assert_eq!(f.team_score("viceTeam"), Ok(75));
    assert_eq!(f.team_score("plainTeam"), Ok(50));
    assert_eq!(
        f.leaderboard(3),
        vec!["1:captainTeam:100", "2:viceTeam:75", "3:plainTeam:50"]
    );
}

#[test]
fn should_only_change_teams_that_contain_the_player() {
    let f = sut!(create());
    team(&f, "A", &squad(&f, "a"));
    team(&f, "B", &squad(&f, "b"));

    f.record_event("b5", "MAIDEN", 1).unwrap();

    assert_eq!(f.team_score("A"), Ok(0));
    assert_eq!(f.team_score("B"), Ok(24));
    assert_eq!(f.leaderboard(2), vec!["1:B:24", "2:A:0"]);
}

#[test]
fn should_count_events_recorded_before_a_team_was_created() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.record_event("a1", "RUN", 10).unwrap();
    f.record_event("a5", "RUN", 10).unwrap();

    team(&f, "late", &s);

    assert_eq!(f.team_score("late"), Ok(60)); // a1 captain: 40, a5 plain: 20
}

#[test]
fn should_break_ties_by_team_creation_order() {
    let f = sut!(create());
    team(&f, "first", &squad(&f, "a"));
    team(&f, "second", &squad(&f, "b"));

    f.record_event("b3", "RUN", 5).unwrap();
    assert_eq!(f.leaderboard(2), vec!["1:second:10", "2:first:0"]);

    f.record_event("a3", "RUN", 5).unwrap();
    assert_eq!(f.leaderboard(2), vec!["1:first:10", "1:second:10"]);
}

#[test]
fn should_use_competition_ranking_for_ties() {
    let f = sut!(create());
    let (top, mid1, mid2, low) = (
        squad(&f, "a"),
        squad(&f, "b"),
        squad(&f, "c"),
        squad(&f, "d"),
    );
    team(&f, "low", &low);
    team(&f, "mid1", &mid1);
    team(&f, "top", &top);
    team(&f, "mid2", &mid2);
    f.record_event("a3", "RUN", 100).unwrap();
    f.record_event("b3", "RUN", 50).unwrap();
    f.record_event("c3", "RUN", 50).unwrap();

    assert_eq!(
        f.leaderboard(4),
        vec!["1:top:200", "2:mid1:100", "2:mid2:100", "4:low:0"]
    );
    assert_eq!(f.rank_of("top"), Ok(1));
    assert_eq!(f.rank_of("mid1"), Ok(2));
    assert_eq!(f.rank_of("mid2"), Ok(2));
    assert_eq!(f.rank_of("low"), Ok(4));
}

#[test]
fn should_return_only_the_top_k_and_keep_the_rank_of_a_tied_cut() {
    let f = sut!(create());
    team(&f, "A", &squad(&f, "a"));
    team(&f, "B", &squad(&f, "b"));
    team(&f, "C", &squad(&f, "c"));
    f.record_event("c3", "RUN", 1).unwrap();

    assert!(f.leaderboard(0).is_empty());
    assert_eq!(f.leaderboard(1), vec!["1:C:2"]);
    assert_eq!(f.leaderboard(2), vec!["1:C:2", "2:A:0"]);
    assert_eq!(f.leaderboard(100), vec!["1:C:2", "2:A:0", "2:B:0"]);
    let empty = sut!(create());
    assert!(empty.leaderboard(5).is_empty());
}

#[test]
fn should_let_a_duck_make_team_scores_negative() {
    let f = sut!(create());
    team(&f, "T", &squad(&f, "a"));
    team(&f, "U", &squad(&f, "b"));

    f.record_event("a1", "DUCK", 1).unwrap();

    assert_eq!(f.player_points("a1"), Ok(-2));
    assert_eq!(f.team_score("T"), Ok(-8));
    assert_eq!(f.leaderboard(2), vec!["1:U:0", "2:T:-8"]);
}

#[test]
fn should_reverse_events_with_correct_event() {
    let f = sut!(create());
    team(&f, "T", &squad(&f, "a"));
    team(&f, "U", &squad(&f, "b"));
    f.record_event("a1", "WICKET", 2).unwrap();
    f.record_event("b3", "RUN", 10).unwrap();

    f.correct_event("a1", "WICKET", 1).unwrap();

    assert_eq!(f.player_points("a1"), Ok(25));
    assert_eq!(f.team_score("T"), Ok(100));
    assert_eq!(f.leaderboard(2), vec!["1:T:100", "2:U:20"]);

    f.correct_event("a1", "WICKET", 1).unwrap();
    assert_eq!(f.team_score("T"), Ok(0));
    assert_eq!(f.leaderboard(2), vec!["1:U:20", "2:T:0"]);
}

#[test]
fn should_reject_a_correction_beyond_the_recorded_count_and_change_nothing() {
    let f = sut!(create());
    team(&f, "T", &squad(&f, "a"));
    f.record_event("a1", "RUN", 5).unwrap();
    f.record_event("a1", "WICKET", 1).unwrap();

    let over = Err(LeaderboardError::CorrectionExceedsRecorded);
    assert_eq!(f.correct_event("a1", "RUN", 6), over);
    assert_eq!(f.correct_event("a1", "CATCH", 1), over);
    assert_eq!(f.correct_event("a3", "RUN", 1), over);

    assert_eq!(f.player_points("a1"), Ok(30));
    assert_eq!(f.team_score("T"), Ok(120));
    f.correct_event("a1", "RUN", 5).unwrap();
    assert_eq!(f.correct_event("a1", "RUN", 1), over);
}

#[test]
fn should_track_the_correction_limit_per_event_type_and_after_recording_again() {
    let f = sut!(create());
    squad(&f, "a");
    f.record_event("a1", "RUN", 3).unwrap();
    f.correct_event("a1", "RUN", 3).unwrap();
    f.record_event("a1", "RUN", 2).unwrap();

    f.correct_event("a1", "RUN", 2).unwrap();

    assert_eq!(f.player_points("a1"), Ok(0));
    assert_eq!(
        f.correct_event("a1", "RUN", 1),
        Err(LeaderboardError::CorrectionExceedsRecorded)
    );
}

#[test]
fn should_reject_invalid_events() {
    let f = sut!(create());
    squad(&f, "a");

    assert_eq!(
        f.record_event("nobody", "RUN", 1),
        Err(LeaderboardError::UnknownPlayer)
    );
    assert_eq!(
        f.record_event("a1", "HAT_TRICK", 1),
        Err(LeaderboardError::InvalidArgument)
    );
    assert_eq!(
        f.record_event("a1", "run", 1),
        Err(LeaderboardError::InvalidArgument)
    );
    assert_eq!(
        f.record_event("a1", "RUN", 0),
        Err(LeaderboardError::InvalidArgument)
    );
    assert_eq!(
        f.correct_event("a1", "RUN", 0),
        Err(LeaderboardError::InvalidArgument)
    );
    assert_eq!(
        f.correct_event("nobody", "RUN", 1),
        Err(LeaderboardError::UnknownPlayer)
    );
    assert_eq!(f.player_points("a1"), Ok(0));
}

#[test]
fn should_reject_bad_player_registrations() {
    let f = sut!(create());
    f.add_player("p1", "One", "KEEPER").unwrap();

    assert_eq!(
        f.add_player("p1", "Again", "BATTER"),
        Err(LeaderboardError::AlreadyExists)
    );
    let bad = Err(LeaderboardError::InvalidArgument);
    assert_eq!(f.add_player(" ", "Name", "BATTER"), bad);
    assert_eq!(f.add_player("p2", "", "BATTER"), bad);
    assert_eq!(f.add_player("p2", "Name", "COACH"), bad);
    assert_eq!(f.add_player("p2", "Name", "batter"), bad);
}

#[test]
fn should_reject_teams_that_do_not_have_exactly_eleven_distinct_registered_players() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.add_player("extra", "Extra", "BOWLER").unwrap();
    let roster = LeaderboardError::InvalidRoster;

    let ten = &refs(&s)[..10];
    let mut twelve = refs(&s);
    twelve.push("extra");
    let mut duplicate = refs(&s)[..10].to_vec();
    duplicate.push("a2");
    let mut unknown = refs(&s)[..10].to_vec();
    unknown.push("ghost");

    assert_eq!(f.create_team("t", "o", ten, "a1", "a2"), Err(roster));
    assert_eq!(
        f.create_team("t", "o", &twelve, "a1", "a2"),
        Err(LeaderboardError::InvalidRoster)
    );
    assert_eq!(
        f.create_team("t", "o", &duplicate, "a1", "a2"),
        Err(LeaderboardError::InvalidRoster)
    );
    assert_eq!(
        f.create_team("t", "o", &unknown, "a1", "a2"),
        Err(LeaderboardError::UnknownPlayer)
    );
    assert_eq!(
        f.create_team("t", "o", &[], "a1", "a2"),
        Err(LeaderboardError::InvalidRoster)
    );
    assert!(f.leaderboard(5).is_empty());
}

#[test]
fn should_reject_bad_captain_choices() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.add_player("outsider", "Outsider", "BOWLER").unwrap();
    let r = refs(&s);

    let roster = Err(LeaderboardError::InvalidRoster);
    assert_eq!(f.create_team("t", "o", &r, "outsider", "a2"), roster);
    assert_eq!(f.create_team("t", "o", &r, "a1", "outsider"), roster);
    assert_eq!(f.create_team("t", "o", &r, "a1", "a1"), roster);
    assert!(f.leaderboard(5).is_empty());
}

#[test]
fn should_enforce_the_role_rules() {
    let f = sut!(create());
    for i in 1..=7 {
        f.add_player(&format!("bat{i}"), "Bat", "BATTER").unwrap();
    }
    for i in 1..=5 {
        f.add_player(&format!("bowl{i}"), "Bowl", "BOWLER").unwrap();
    }
    f.add_player("keep", "Keeper", "KEEPER").unwrap();
    f.add_player("all", "All", "ALL_ROUNDER").unwrap();

    let no_keeper = [
        "bat1", "bat2", "bat3", "bat4", "bat5", "bowl1", "bowl2", "bowl3", "bowl4", "bowl5", "all",
    ];
    let seven = [
        "bat1", "bat2", "bat3", "bat4", "bat5", "bat6", "bat7", "bowl1", "bowl2", "keep", "all",
    ];
    let six = [
        "bat1", "bat2", "bat3", "bat4", "bat5", "bat6", "bowl1", "bowl2", "bowl3", "keep", "all",
    ];

    assert_eq!(
        f.create_team("noKeeper", "o", &no_keeper, "bat1", "bat2"),
        Err(LeaderboardError::InvalidRoster)
    );
    assert_eq!(
        f.create_team("seven", "o", &seven, "bat1", "bat2"),
        Err(LeaderboardError::InvalidRoster)
    );
    f.create_team("six", "o", &six, "bat1", "keep").unwrap();

    assert_eq!(f.leaderboard(5), vec!["1:six:0"]);
}

#[test]
fn should_reject_duplicate_team_ids_and_blank_inputs_without_disturbing_the_existing_team() {
    let f = sut!(create());
    let s = squad(&f, "a");
    let t = squad(&f, "b");
    team(&f, "T", &s);
    f.record_event("a1", "RUN", 1).unwrap();

    assert_eq!(
        f.create_team("T", "o", &refs(&t), "b1", "b2"),
        Err(LeaderboardError::AlreadyExists)
    );
    assert_eq!(
        f.create_team(" ", "o", &refs(&t), "b1", "b2"),
        Err(LeaderboardError::InvalidArgument)
    );
    assert_eq!(
        f.create_team("U", "", &refs(&t), "b1", "b2"),
        Err(LeaderboardError::InvalidArgument)
    );

    assert_eq!(f.leaderboard(5), vec!["1:T:4"]);
    team(&f, "U", &t); // a failed attempt must not have used up the id
    assert_eq!(f.leaderboard(5), vec!["1:T:4", "2:U:0"]);
}

#[test]
fn should_reject_unknown_teams_and_players_in_queries() {
    let f = sut!(create());

    assert_eq!(f.rank_of("nope"), Err(LeaderboardError::UnknownTeam));
    assert_eq!(f.team_score("nope"), Err(LeaderboardError::UnknownTeam));
    assert_eq!(
        f.player_points("nope"),
        Err(LeaderboardError::UnknownPlayer)
    );
}

fn check(lines: &[String], expected_size: usize, problems: &Mutex<Vec<String>>) {
    if lines.len() != expected_size {
        problems
            .lock()
            .unwrap()
            .push(format!("size {}", lines.len()));
        return;
    }
    let mut seen = HashSet::new();
    let mut prev = i64::MAX;
    let (mut twin_a, mut twin_b) = (i64::MIN, i64::MAX);
    for (i, line) in lines.iter().enumerate() {
        let p: Vec<&str> = line.split(':').collect();
        let score: i64 = p[2].parse().unwrap();
        if !seen.insert(p[1].to_string()) {
            problems.lock().unwrap().push(format!("duplicate {}", p[1]));
        }
        if score > prev {
            problems.lock().unwrap().push(format!("unsorted at {i}"));
        }
        prev = score;
        if p[1] == "twinA" {
            twin_a = score;
        }
        if p[1] == "twinB" {
            twin_b = score;
        }
    }
    if twin_a != twin_b {
        problems
            .lock()
            .unwrap()
            .push(format!("twins differ {twin_a} vs {twin_b}"));
    }
}

#[test]
fn should_keep_a_sorted_duplicate_free_leaderboard_while_sixteen_threads_record_events() {
    let f = sut!(create());
    let teams = 40;
    for i in 0..teams {
        team(&f, &format!("T{i}"), &squad(&f, &format!("p{i}_")));
    }
    // two teams that always hold the same squad must never be seen apart
    let shared = squad(&f, "s");
    f.create_team("twinA", "o", &refs(&shared), "s1", "s2")
        .unwrap();
    f.create_team("twinB", "o", &refs(&shared), "s1", "s2")
        .unwrap();
    let done = AtomicBool::new(false);
    let problems = Mutex::new(Vec::new());

    run_concurrently(16, |t| {
        if t < 4 {
            while !done.load(Ordering::SeqCst) {
                check(&f.leaderboard(teams + 2), teams + 2, &problems);
            }
            return;
        }
        for i in 0..5_000usize {
            let n = (t * 31 + i) % teams;
            f.record_event(&format!("p{n}_{}", 1 + i % 11), "RUN", (1 + i % 5) as u32)
                .unwrap();
            f.record_event(&format!("s{}", 1 + i % 11), "RUN", 1)
                .unwrap();
        }
        if t == 4 {
            done.store(true, Ordering::SeqCst);
        }
    });
    done.store(true, Ordering::SeqCst);

    assert!(problems.lock().unwrap().is_empty());
    assert_eq!(f.team_score("twinA"), f.team_score("twinB"));
}

#[test]
fn should_end_with_the_sequential_scores_after_concurrent_events_and_corrections() {
    let f = sut!(create());
    let s = squad(&f, "a");
    f.create_team("T", "o", &refs(&s), "a1", "a2").unwrap();
    f.create_team("U", "o", &refs(&s), "a2", "a3").unwrap();

    run_concurrently(16, |t| {
        for i in 0..1_000 {
            let player = &s[(t + i) % 11];
            f.record_event(player, "WICKET", 2).unwrap();
            f.record_event(player, "RUN", 3).unwrap();
            f.correct_event(player, "RUN", 1).unwrap();
        }
    });

    let mut total = 0;
    let (mut expected_t, mut expected_u) = (0, 0);
    for i in 0..11 {
        let pts = f.player_points(&s[i]).unwrap();
        total += pts;
        expected_t += pts * [4, 3, 2][i.min(2)];
        expected_u += pts
            * if i == 1 {
                4
            } else if i == 2 {
                3
            } else {
                2
            };
    }
    assert_eq!(total, 16 * 1_000 * (50 + 2));
    assert_eq!(f.team_score("T"), Ok(expected_t));
    assert_eq!(f.team_score("U"), Ok(expected_u));
    assert_eq!(f.leaderboard(2).len(), 2);
}

#[test]
fn should_handle_fifty_thousand_teams_two_hundred_thousand_events_and_thousand_leaderboard_calls() {
    let f = sut!(create());
    let players = 5_000usize;
    let roles = ["KEEPER", "BATTER", "BATTER", "BOWLER", "ALL_ROUNDER"];
    for i in 0..players {
        f.add_player(&format!("P{i}"), "Player", roles[i % 5])
            .unwrap();
    }
    let teams = 50_000usize;
    for i in 0..teams {
        let ids: Vec<String> = (0..11)
            .map(|j| format!("P{}", (i * 7 + j * 437) % players))
            .collect();
        f.create_team(
            &format!("T{i}"),
            &format!("owner{}", i % 100),
            &refs(&ids),
            &ids[0],
            &ids[1],
        )
        .unwrap();
    }
    let mut points = vec![0i64; players];
    let mut seed: u64 = 12345;
    let mut last = Vec::new();
    let start = Instant::now();
    for e in 0..200_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let p = ((seed >> 33) % players as u64) as usize;
        let count = 1 + ((seed >> 20) & 3) as u32;
        f.record_event(&format!("P{p}"), "RUN", count).unwrap();
        points[p] += count as i64;
        if e % 200 == 0 {
            last = f.leaderboard(10);
        }
    }
    let elapsed = start.elapsed().as_millis();

    assert_eq!(last.len(), 10);
    assert!(
        elapsed < 60_000,
        "200k events + 1k leaderboard calls took {elapsed} ms"
    );
    let expected: Vec<i64> = (0..teams)
        .map(|i| {
            (0..11)
                .map(|j| points[(i * 7 + j * 437) % players] * [4, 3, 2][j.min(2)])
                .sum()
        })
        .collect();
    for i in [0usize, 1, 777, 25_000, 49_999] {
        assert_eq!(f.team_score(&format!("T{i}")), Ok(expected[i]));
    }
    let best = expected.iter().max().unwrap();
    assert!(f.leaderboard(1)[0].ends_with(&format!(":{best}")));
}

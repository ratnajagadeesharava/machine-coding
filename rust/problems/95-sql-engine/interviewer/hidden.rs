#![allow(unused_imports, clippy::needless_range_loop)]
use p95_sql_engine::{contract::*, create};
use playground_support::sut;
use std::time::Instant;

#[test]
fn should_join_two_large_tables_without_a_quadratic_blow_up() {
    let db = sut!(create());
    db.execute("CREATE TABLE a (id INT PRIMARY KEY, v INT)").unwrap();
    db.execute("CREATE TABLE b (id INT PRIMARY KEY, aid INT)").unwrap();
    for i in 0..20_000 {
        db.execute(&format!("INSERT INTO a VALUES ({i}, {})", i % 7)).unwrap();
        db.execute(&format!("INSERT INTO b VALUES ({i}, {i})")).unwrap();
    }

    let start = Instant::now();
    let res = db.execute("SELECT COUNT(*) FROM a JOIN b ON a.id = b.aid").unwrap();

    assert_eq!(res.rows, vec![vec!["20000".to_string()]]);
    assert!(start.elapsed().as_secs() < 10, "400M row pairs would take far longer");
}

#[test]
fn should_look_up_by_primary_key_in_a_big_table_quickly() {
    let db = sut!(create());
    db.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    for i in 0..20_000 {
        db.execute(&format!("INSERT INTO t VALUES ({i}, 'v{i}')")).unwrap();
    }

    let start = Instant::now();
    for i in 0..1_000 {
        assert_eq!(db.execute(&format!("SELECT v FROM t WHERE id = {}", i * 19 % 20_000)).unwrap().rows.len(), 1);
    }

    assert!(start.elapsed().as_secs() < 60, "1,000 full scans of 20k rows are too slow for a keyed lookup");
}

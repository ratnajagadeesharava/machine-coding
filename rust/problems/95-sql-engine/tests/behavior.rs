#![allow(unused_imports, clippy::needless_range_loop)]
use p95_sql_engine::{contract::*, create};
use playground_support::{collect_concurrently, run_concurrently, sut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

fn db() -> Option<Arc<dyn SqlEngine>> {
    let db = create()?;
    db.execute("CREATE TABLE users (id INT PRIMARY KEY, name TEXT, age INT, city TEXT)").unwrap();
    db.execute("INSERT INTO users VALUES (1, 'ann', 31, 'blr')").unwrap();
    db.execute("INSERT INTO users VALUES (2, 'bob', 25, 'del')").unwrap();
    db.execute("INSERT INTO users VALUES (3, 'cy', 40, 'blr')").unwrap();
    db.execute("INSERT INTO users VALUES (4, 'dee', 25, 'mum');").unwrap();
    Some(db)
}

fn rows(db: &Arc<dyn SqlEngine>, sql: &str) -> Vec<Vec<String>> {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}: {e:?}")).rows
}

fn r(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn fails(db: &Arc<dyn SqlEngine>, sql: &str) {
    assert!(db.execute(sql).is_err(), "should fail: {sql}");
}

fn firsts(rows: Vec<Vec<String>>) -> Vec<String> {
    rows.into_iter().map(|mut r| r.remove(0)).collect()
}

#[test]
fn should_select_everything_in_insertion_order() {
    let db = sut!(db());

    let res = db.execute("SELECT * FROM users").unwrap();

    assert_eq!(res.columns, r(&["id", "name", "age", "city"]));
    assert_eq!(
        res.rows,
        vec![r(&["1", "ann", "31", "blr"]), r(&["2", "bob", "25", "del"]), r(&["3", "cy", "40", "blr"]), r(&["4", "dee", "25", "mum"])]
    );
}

#[test]
fn should_project_columns_in_the_requested_order_and_ignore_keyword_case() {
    let db = sut!(db());

    let res = db.execute("select city, name from users where id = 3").unwrap();

    assert_eq!(res.columns, r(&["city", "name"]));
    assert_eq!(res.rows, vec![r(&["blr", "cy"])]);
}

#[test]
fn should_filter_with_comparisons_and_and_or_with_precedence() {
    let db = sut!(db());

    assert_eq!(rows(&db, "SELECT name FROM users WHERE age > 25"), vec![r(&["ann"]), r(&["cy"])]);
    assert_eq!(rows(&db, "SELECT name FROM users WHERE age >= 31 AND city = 'blr'"), vec![r(&["ann"]), r(&["cy"])]);
    assert_eq!(rows(&db, "SELECT id FROM users WHERE city != 'blr' AND age = 25"), vec![r(&["2"]), r(&["4"])]);
    assert_eq!(rows(&db, "SELECT id FROM users WHERE city <> 'blr'").len(), 2);
    // AND binds tighter than OR: id=1 OR (id=2 AND age=99)
    assert_eq!(rows(&db, "SELECT id FROM users WHERE id = 1 OR id = 2 AND age = 99"), vec![r(&["1"])]);
    assert_eq!(rows(&db, "SELECT id FROM users WHERE (id = 1 OR id = 2) AND age = 25"), vec![r(&["2"])]);
    assert_eq!(rows(&db, "SELECT id FROM users WHERE age < 26 OR age > 39"), vec![r(&["2"]), r(&["3"]), r(&["4"])]);
}

#[test]
fn should_order_by_integers_numerically_and_text_alphabetically_with_limit() {
    let db = sut!(db());
    db.execute("INSERT INTO users VALUES (10, 'zed', 9, 'blr')").unwrap();

    assert_eq!(rows(&db, "SELECT id FROM users ORDER BY id DESC LIMIT 3"), vec![r(&["10"]), r(&["4"]), r(&["3"])]);
    assert_eq!(firsts(rows(&db, "SELECT name FROM users ORDER BY name")), r(&["ann", "bob", "cy", "dee", "zed"]));
    assert_eq!(rows(&db, "SELECT id FROM users ORDER BY age ASC LIMIT 1"), vec![r(&["10"])]);
}

#[test]
fn should_keep_the_insertion_order_for_ties() {
    let db = sut!(db());

    assert_eq!(firsts(rows(&db, "SELECT name FROM users ORDER BY age")), r(&["bob", "dee", "ann", "cy"]));
}

#[test]
fn should_aggregate_without_group_by() {
    let db = sut!(db());

    let res = db.execute("SELECT COUNT(*), SUM(age), MIN(age), MAX(name) FROM users WHERE city = 'blr'").unwrap();

    assert_eq!(res.columns, r(&["COUNT(*)", "SUM(age)", "MIN(age)", "MAX(name)"]));
    assert_eq!(res.rows, vec![r(&["2", "71", "31", "cy"])]);
    assert_eq!(rows(&db, "SELECT COUNT(*) FROM users WHERE id = 99"), vec![r(&["0"])]);
}

#[test]
fn should_group_by_a_column() {
    let db = sut!(db());

    let res = db.execute("SELECT city, COUNT(*), SUM(age) FROM users GROUP BY city ORDER BY city").unwrap();

    assert_eq!(res.columns, r(&["city", "COUNT(*)", "SUM(age)"]));
    assert_eq!(res.rows, vec![r(&["blr", "2", "71"]), r(&["del", "1", "25"]), r(&["mum", "1", "25"])]);
    assert_eq!(rows(&db, "SELECT city, COUNT(*) FROM users GROUP BY city ORDER BY COUNT(*) DESC LIMIT 1"), vec![r(&["blr", "2"])]);
}

#[test]
fn should_join_two_tables_on_an_equality() {
    let db = sut!(db());
    db.execute("CREATE TABLE orders (oid INT PRIMARY KEY, uid INT, amount INT)").unwrap();
    db.execute("INSERT INTO orders VALUES (100, 1, 50)").unwrap();
    db.execute("INSERT INTO orders VALUES (101, 1, 70)").unwrap();
    db.execute("INSERT INTO orders VALUES (102, 3, 20)").unwrap();
    db.execute("INSERT INTO orders VALUES (103, 99, 5)").unwrap();

    let res = db
        .execute("SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.uid ORDER BY orders.amount DESC")
        .unwrap();

    assert_eq!(res.columns, r(&["users.name", "orders.amount"]));
    assert_eq!(res.rows, vec![r(&["ann", "70"]), r(&["ann", "50"]), r(&["cy", "20"])]);
    assert_eq!(
        rows(&db, "SELECT name, SUM(amount) FROM users JOIN orders ON id = uid GROUP BY name ORDER BY name"),
        vec![r(&["ann", "120"]), r(&["cy", "20"])]
    );
    assert_eq!(
        db.execute("SELECT * FROM users JOIN orders ON users.id = orders.uid").unwrap().columns,
        r(&["users.id", "users.name", "users.age", "users.city", "orders.oid", "orders.uid", "orders.amount"])
    );
}

#[test]
fn should_update_with_literals_and_arithmetic_and_report_the_count() {
    let db = sut!(db());

    assert_eq!(db.execute("UPDATE users SET age = age + 1 WHERE city = 'blr'").unwrap().affected, 2);
    assert_eq!(db.execute("UPDATE users SET city = 'pune', age = age - 5 WHERE id = 2").unwrap().affected, 1);
    assert_eq!(db.execute("UPDATE users SET age = 0 WHERE id = 999").unwrap().affected, 0);

    assert_eq!(rows(&db, "SELECT age, city FROM users WHERE id = 1"), vec![r(&["32", "blr"])]);
    assert_eq!(rows(&db, "SELECT age, city FROM users WHERE id = 2"), vec![r(&["20", "pune"])]);
}

#[test]
fn should_delete_rows_and_report_the_count() {
    let db = sut!(db());

    assert_eq!(db.execute("DELETE FROM users WHERE age = 25").unwrap().affected, 2);
    assert_eq!(rows(&db, "SELECT id FROM users"), vec![r(&["1"]), r(&["3"])]);
    assert_eq!(db.execute("DELETE FROM users").unwrap().affected, 2);
    assert!(rows(&db, "SELECT * FROM users").is_empty());
}

#[test]
fn should_enforce_the_primary_key_on_insert_and_update_and_change_nothing_on_failure() {
    let db = sut!(db());

    fails(&db, "INSERT INTO users VALUES (1, 'dup', 1, 'x')");
    fails(&db, "UPDATE users SET id = 1 WHERE id = 2");
    fails(&db, "UPDATE users SET id = 7"); // would give four rows the same key

    assert_eq!(rows(&db, "SELECT id FROM users"), vec![r(&["1"]), r(&["2"]), r(&["3"]), r(&["4"])]);
}

#[test]
fn should_handle_quotes_in_text_literals() {
    let db = sut!(db());
    db.execute("INSERT INTO users VALUES (5, 'it''s', 1, 'a,b')").unwrap();

    assert_eq!(rows(&db, "SELECT name, city FROM users WHERE id = 5"), vec![r(&["it's", "a,b"])]);
    assert_eq!(rows(&db, "SELECT id FROM users WHERE name = 'it''s'"), vec![r(&["5"])]);
}

#[test]
fn should_reject_invalid_statements_with_an_error() {
    let db = sut!(db());

    fails(&db, "SELEC * FROM users");
    fails(&db, "SELECT * FROM nope");
    fails(&db, "SELECT nope FROM users");
    fails(&db, "SELECT * FROM users WHERE");
    fails(&db, "SELECT * FROM users WHERE age > 'x'");
    fails(&db, "SELECT name, COUNT(*) FROM users");
    fails(&db, "SELECT SUM(name) FROM users");
    fails(&db, "INSERT INTO users VALUES (6, 'x')");
    fails(&db, "INSERT INTO users VALUES ('six', 'x', 1, 'y')");
    fails(&db, "CREATE TABLE users (id INT)");
    fails(&db, "CREATE TABLE t (a FLOAT)");
    fails(&db, "UPDATE users SET nope = 1");
    fails(&db, "UPDATE users SET name = name + 1");
    fails(&db, "DELETE FROM nope");
    fails(&db, "SELECT * FROM users LIMIT -1");
    fails(&db, "");
}

#[test]
fn should_never_lose_an_insert_under_concurrency() {
    let db = sut!(create());
    db.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();

    run_concurrently(8, |w| {
        for i in 0..250 {
            db.execute(&format!("INSERT INTO t VALUES ({}, {i})", w * 1_000 + i)).unwrap();
        }
    });

    assert_eq!(rows(&db, "SELECT COUNT(*) FROM t"), vec![r(&["2000"])]);
}

#[test]
fn should_let_exactly_one_concurrent_insert_win_a_primary_key() {
    let db = sut!(create());
    db.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();

    let wins = collect_concurrently(16, |w| db.execute(&format!("INSERT INTO t VALUES (1, {w})")).is_ok());

    assert_eq!(wins.iter().filter(|x| **x).count(), 1);
    assert_eq!(rows(&db, "SELECT COUNT(*) FROM t"), vec![r(&["1"])]);
}

#[test]
fn should_not_lose_concurrent_increments() {
    let db = sut!(create());
    db.execute("CREATE TABLE c (id INT PRIMARY KEY, n INT)").unwrap();
    db.execute("INSERT INTO c VALUES (1, 0)").unwrap();

    run_concurrently(16, |_| {
        for _ in 0..250 {
            db.execute("UPDATE c SET n = n + 1 WHERE id = 1").unwrap();
        }
    });

    assert_eq!(rows(&db, "SELECT n FROM c WHERE id = 1"), vec![r(&["4000"])]);
}

#[test]
fn should_give_readers_consistent_snapshots_while_writers_move_values_around() {
    let db = sut!(create());
    db.execute("CREATE TABLE acc (id INT PRIMARY KEY, bal INT)").unwrap();
    db.execute("INSERT INTO acc VALUES (1, 1000)").unwrap();
    db.execute("INSERT INTO acc VALUES (2, 1000)").unwrap();
    let torn = AtomicUsize::new(0);

    run_concurrently(4, |w| {
        for _ in 0..300 {
            if w < 2 {
                // each single statement is atomic; a reader may see the money in flight between the two
                db.execute("UPDATE acc SET bal = bal - 1 WHERE id = 1").unwrap();
                db.execute("UPDATE acc SET bal = bal + 1 WHERE id = 2").unwrap();
            } else {
                let sum: i64 = rows(&db, "SELECT SUM(bal) FROM acc")[0][0].parse().unwrap();
                if !(1_998..=2_002).contains(&sum) {
                    torn.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });

    assert_eq!(torn.load(Ordering::SeqCst), 0);
    assert_eq!(rows(&db, "SELECT SUM(bal) FROM acc"), vec![r(&["2000"])]);
}

#[test]
fn should_stay_correct_over_100_runs() {
    for run in 0..100 {
        let db = sut!(create());
        db.execute("CREATE TABLE t (id INT PRIMARY KEY, n INT)").unwrap();

        run_concurrently(4, |w| {
            for i in 0..25 {
                db.execute(&format!("INSERT INTO t VALUES ({}, 1)", w * 100 + i)).unwrap();
                db.execute(&format!("UPDATE t SET n = n + 1 WHERE id = {}", w * 100 + i)).unwrap();
            }
        });

        assert_eq!(rows(&db, "SELECT COUNT(*), SUM(n) FROM t"), vec![r(&["100", "200"])], "run {run}");
    }
}

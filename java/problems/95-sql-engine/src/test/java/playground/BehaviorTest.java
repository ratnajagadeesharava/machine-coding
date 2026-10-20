package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.QueryResult;
import contract.SqlEngine;
import contract.SqlException;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private SqlEngine db() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE users (id INT PRIMARY KEY, name TEXT, age INT, city TEXT)");
        db.execute("INSERT INTO users VALUES (1, 'ann', 31, 'blr')");
        db.execute("INSERT INTO users VALUES (2, 'bob', 25, 'del')");
        db.execute("INSERT INTO users VALUES (3, 'cy', 40, 'blr')");
        db.execute("INSERT INTO users VALUES (4, 'dee', 25, 'mum');");
        return db;
    }

    private static List<List<String>> rows(SqlEngine db, String sql) {
        return db.execute(sql).rows();
    }

    private static void fails(SqlEngine db, String sql) {
        assertThatThrownBy(() -> db.execute(sql)).as(sql).isInstanceOf(SqlException.class);
    }

    @Test
    void shouldSelectEverythingInInsertionOrder() {
        SqlEngine db = db();

        QueryResult r = db.execute("SELECT * FROM users");

        assertThat(r.columns()).containsExactly("id", "name", "age", "city");
        assertThat(r.rows()).containsExactly(
                List.of("1", "ann", "31", "blr"), List.of("2", "bob", "25", "del"),
                List.of("3", "cy", "40", "blr"), List.of("4", "dee", "25", "mum"));
    }

    @Test
    void shouldProjectColumnsInTheRequestedOrderAndIgnoreKeywordCase() {
        SqlEngine db = db();

        QueryResult r = db.execute("select city, name from users where id = 3");

        assertThat(r.columns()).containsExactly("city", "name");
        assertThat(r.rows()).containsExactly(List.of("blr", "cy"));
    }

    @Test
    void shouldFilterWithComparisonsAndAndOrWithPrecedence() {
        SqlEngine db = db();

        assertThat(rows(db, "SELECT name FROM users WHERE age > 25")).containsExactly(List.of("ann"), List.of("cy"));
        assertThat(rows(db, "SELECT name FROM users WHERE age >= 31 AND city = 'blr'")).containsExactly(List.of("ann"), List.of("cy"));
        assertThat(rows(db, "SELECT id FROM users WHERE city != 'blr' AND age = 25")).containsExactly(List.of("2"), List.of("4"));
        assertThat(rows(db, "SELECT id FROM users WHERE city <> 'blr'")).hasSize(2);
        // AND binds tighter than OR: id=1 OR (id=2 AND age=99)
        assertThat(rows(db, "SELECT id FROM users WHERE id = 1 OR id = 2 AND age = 99")).containsExactly(List.of("1"));
        assertThat(rows(db, "SELECT id FROM users WHERE (id = 1 OR id = 2) AND age = 25")).containsExactly(List.of("2"));
        assertThat(rows(db, "SELECT id FROM users WHERE age < 26 OR age > 39")).containsExactly(List.of("2"), List.of("3"), List.of("4"));
    }

    @Test
    void shouldOrderByIntegersNumericallyAndTextAlphabeticallyWithLimit() {
        SqlEngine db = db();
        db.execute("INSERT INTO users VALUES (10, 'zed', 9, 'blr')");

        assertThat(rows(db, "SELECT id FROM users ORDER BY id DESC LIMIT 3")).containsExactly(List.of("10"), List.of("4"), List.of("3"));
        assertThat(rows(db, "SELECT name FROM users ORDER BY name")).extracting(r -> r.get(0)).containsExactly("ann", "bob", "cy", "dee", "zed");
        assertThat(rows(db, "SELECT id FROM users ORDER BY age ASC LIMIT 1")).containsExactly(List.of("10"));
    }

    @Test
    void shouldKeepTheInsertionOrderForTies() {
        SqlEngine db = db();

        assertThat(rows(db, "SELECT name FROM users ORDER BY age")).extracting(r -> r.get(0)).containsExactly("bob", "dee", "ann", "cy");
    }

    @Test
    void shouldAggregateWithoutGroupBy() {
        SqlEngine db = db();

        QueryResult r = db.execute("SELECT COUNT(*), SUM(age), MIN(age), MAX(name) FROM users WHERE city = 'blr'");

        assertThat(r.columns()).containsExactly("COUNT(*)", "SUM(age)", "MIN(age)", "MAX(name)");
        assertThat(r.rows()).containsExactly(List.of("2", "71", "31", "cy"));
        assertThat(rows(db, "SELECT COUNT(*) FROM users WHERE id = 99")).containsExactly(List.of("0"));
    }

    @Test
    void shouldGroupByAColumn() {
        SqlEngine db = db();

        QueryResult r = db.execute("SELECT city, COUNT(*), SUM(age) FROM users GROUP BY city ORDER BY city");

        assertThat(r.columns()).containsExactly("city", "COUNT(*)", "SUM(age)");
        assertThat(r.rows()).containsExactly(List.of("blr", "2", "71"), List.of("del", "1", "25"), List.of("mum", "1", "25"));
        assertThat(rows(db, "SELECT city, COUNT(*) FROM users GROUP BY city ORDER BY COUNT(*) DESC LIMIT 1")).containsExactly(List.of("blr", "2"));
    }

    @Test
    void shouldJoinTwoTablesOnAnEquality() {
        SqlEngine db = db();
        db.execute("CREATE TABLE orders (oid INT PRIMARY KEY, uid INT, amount INT)");
        db.execute("INSERT INTO orders VALUES (100, 1, 50)");
        db.execute("INSERT INTO orders VALUES (101, 1, 70)");
        db.execute("INSERT INTO orders VALUES (102, 3, 20)");
        db.execute("INSERT INTO orders VALUES (103, 99, 5)");

        QueryResult r = db.execute("SELECT users.name, orders.amount FROM users JOIN orders ON users.id = orders.uid ORDER BY orders.amount DESC");

        assertThat(r.columns()).containsExactly("users.name", "orders.amount");
        assertThat(r.rows()).containsExactly(List.of("ann", "70"), List.of("ann", "50"), List.of("cy", "20"));
        assertThat(rows(db, "SELECT name, SUM(amount) FROM users JOIN orders ON id = uid GROUP BY name ORDER BY name"))
                .containsExactly(List.of("ann", "120"), List.of("cy", "20"));
        assertThat(db.execute("SELECT * FROM users JOIN orders ON users.id = orders.uid").columns())
                .containsExactly("users.id", "users.name", "users.age", "users.city", "orders.oid", "orders.uid", "orders.amount");
    }

    @Test
    void shouldUpdateWithLiteralsAndArithmeticAndReportTheCount() {
        SqlEngine db = db();

        assertThat(db.execute("UPDATE users SET age = age + 1 WHERE city = 'blr'").affected()).isEqualTo(2);
        assertThat(db.execute("UPDATE users SET city = 'pune', age = age - 5 WHERE id = 2").affected()).isEqualTo(1);
        assertThat(db.execute("UPDATE users SET age = 0 WHERE id = 999").affected()).isZero();

        assertThat(rows(db, "SELECT age, city FROM users WHERE id = 1")).containsExactly(List.of("32", "blr"));
        assertThat(rows(db, "SELECT age, city FROM users WHERE id = 2")).containsExactly(List.of("20", "pune"));
    }

    @Test
    void shouldDeleteRowsAndReportTheCount() {
        SqlEngine db = db();

        assertThat(db.execute("DELETE FROM users WHERE age = 25").affected()).isEqualTo(2);
        assertThat(rows(db, "SELECT id FROM users")).containsExactly(List.of("1"), List.of("3"));
        assertThat(db.execute("DELETE FROM users").affected()).isEqualTo(2);
        assertThat(rows(db, "SELECT * FROM users")).isEmpty();
    }

    @Test
    void shouldEnforceThePrimaryKeyOnInsertAndUpdateAndChangeNothingOnFailure() {
        SqlEngine db = db();

        fails(db, "INSERT INTO users VALUES (1, 'dup', 1, 'x')");
        fails(db, "UPDATE users SET id = 1 WHERE id = 2");
        fails(db, "UPDATE users SET id = 7"); // would give four rows the same key

        assertThat(rows(db, "SELECT id FROM users")).containsExactly(List.of("1"), List.of("2"), List.of("3"), List.of("4"));
    }

    @Test
    void shouldHandleQuotesInTextLiterals() {
        SqlEngine db = db();
        db.execute("INSERT INTO users VALUES (5, 'it''s', 1, 'a,b')");

        assertThat(rows(db, "SELECT name, city FROM users WHERE id = 5")).containsExactly(List.of("it's", "a,b"));
        assertThat(rows(db, "SELECT id FROM users WHERE name = 'it''s'")).containsExactly(List.of("5"));
    }

    @Test
    void shouldRejectInvalidStatementsWithASqlException() {
        SqlEngine db = db();

        fails(db, "SELEC * FROM users");
        fails(db, "SELECT * FROM nope");
        fails(db, "SELECT nope FROM users");
        fails(db, "SELECT * FROM users WHERE");
        fails(db, "SELECT * FROM users WHERE age > 'x'");
        fails(db, "SELECT name, COUNT(*) FROM users");
        fails(db, "SELECT SUM(name) FROM users");
        fails(db, "INSERT INTO users VALUES (6, 'x')");
        fails(db, "INSERT INTO users VALUES ('six', 'x', 1, 'y')");
        fails(db, "CREATE TABLE users (id INT)");
        fails(db, "CREATE TABLE t (a FLOAT)");
        fails(db, "UPDATE users SET nope = 1");
        fails(db, "UPDATE users SET name = name + 1");
        fails(db, "DELETE FROM nope");
        fails(db, "SELECT * FROM users LIMIT -1");
        fails(db, "");
    }

    @Test
    void shouldNeverLoseAnInsertUnderConcurrency() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)");

        Concurrently.run(8, w -> {
            for (int i = 0; i < 250; i++) {
                db.execute("INSERT INTO t VALUES (" + (w * 1_000 + i) + ", " + i + ")");
            }
        });

        assertThat(rows(db, "SELECT COUNT(*) FROM t")).containsExactly(List.of("2000"));
    }

    @Test
    void shouldLetExactlyOneConcurrentInsertWinAPrimaryKey() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)");

        List<Boolean> wins = Concurrently.collect(16, w -> {
            try {
                db.execute("INSERT INTO t VALUES (1, " + w + ")");
                return true;
            } catch (SqlException e) {
                return false;
            }
        });

        assertThat(wins.stream().filter(x -> x).count()).isEqualTo(1);
        assertThat(rows(db, "SELECT COUNT(*) FROM t")).containsExactly(List.of("1"));
    }

    @Test
    void shouldNotLoseConcurrentIncrements() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE c (id INT PRIMARY KEY, n INT)");
        db.execute("INSERT INTO c VALUES (1, 0)");

        Concurrently.run(16, w -> {
            for (int i = 0; i < 250; i++) {
                db.execute("UPDATE c SET n = n + 1 WHERE id = 1");
            }
        });

        assertThat(rows(db, "SELECT n FROM c WHERE id = 1")).containsExactly(List.of("4000"));
    }

    @Test
    void shouldGiveReadersConsistentSnapshotsWhileWritersMoveValuesAround() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE acc (id INT PRIMARY KEY, bal INT)");
        db.execute("INSERT INTO acc VALUES (1, 1000)");
        db.execute("INSERT INTO acc VALUES (2, 1000)");
        AtomicInteger torn = new AtomicInteger();

        Concurrently.run(4, w -> {
            for (int i = 0; i < 300; i++) {
                if (w < 2) {
                    // the two updates of a transfer are separate statements, so a reader may see the money in flight;
                    // but each single statement is atomic: the total after both is always 2000
                    db.execute("UPDATE acc SET bal = bal - 1 WHERE id = 1");
                    db.execute("UPDATE acc SET bal = bal + 1 WHERE id = 2");
                } else {
                    int sum = Integer.parseInt(rows(db, "SELECT SUM(bal) FROM acc").get(0).get(0));
                    if (sum < 1_998 || sum > 2_002) { // at most two writers are between their two statements
                        torn.incrementAndGet();
                    }
                }
            }
        });

        assertThat(torn).hasValue(0);
        assertThat(rows(db, "SELECT SUM(bal) FROM acc")).containsExactly(List.of("2000"));
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        for (int run = 0; run < 100; run++) {
            SqlEngine db = Candidate.of(Solution::create);
            db.execute("CREATE TABLE t (id INT PRIMARY KEY, n INT)");

            Concurrently.run(4, w -> {
                for (int i = 0; i < 25; i++) {
                    db.execute("INSERT INTO t VALUES (" + (w * 100 + i) + ", 1)");
                    db.execute("UPDATE t SET n = n + 1 WHERE id = " + (w * 100 + i));
                }
            });

            assertThat(rows(db, "SELECT COUNT(*), SUM(n) FROM t")).as("run %d", run).containsExactly(List.of("100", "200"));
        }
    }
}

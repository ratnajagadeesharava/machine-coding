package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.SqlEngine;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldJoinTwoLargeTablesWithoutAQuadraticBlowUp() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE a (id INT PRIMARY KEY, v INT)");
        db.execute("CREATE TABLE b (id INT PRIMARY KEY, aid INT)");
        for (int i = 0; i < 20_000; i++) {
            db.execute("INSERT INTO a VALUES (" + i + ", " + (i % 7) + ")");
            db.execute("INSERT INTO b VALUES (" + i + ", " + i + ")");
        }

        long start = System.nanoTime();
        List<List<String>> r = db.execute("SELECT COUNT(*) FROM a JOIN b ON a.id = b.aid").rows();

        assertThat(r).containsExactly(List.of("20000"));
        assertThat((System.nanoTime() - start) / 1_000_000).as("ms: 400M row pairs would take far longer").isLessThan(10_000);
    }

    @Test
    void shouldLookUpByPrimaryKeyInAHundredThousandRowTableQuickly() {
        SqlEngine db = Candidate.of(Solution::create);
        db.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)");
        for (int i = 0; i < 100_000; i++) {
            db.execute("INSERT INTO t VALUES (" + i + ", 'v" + i + "')");
        }

        long start = System.nanoTime();
        for (int i = 0; i < 5_000; i++) {
            assertThat(db.execute("SELECT v FROM t WHERE id = " + (i * 19 % 100_000)).rows()).hasSize(1);
        }

        assertThat((System.nanoTime() - start) / 1_000_000).as("ms: 5,000 full scans of 100k rows would take ~minutes").isLessThan(20_000);
    }
}

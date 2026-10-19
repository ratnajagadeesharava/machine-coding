package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ParallelSorter;
import java.util.Arrays;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldSortTenMillionElements() {
        ParallelSorter s = Candidate.of(Solution::create);
        Random r = new Random(1);
        int[] a = new int[10_000_000];
        for (int i = 0; i < a.length; i++) {
            a[i] = r.nextInt();
        }
        int[] expected = a.clone();
        Arrays.sort(expected);

        s.sort(a, 8);

        assertThat(a).isEqualTo(expected);
    }

    @Test
    void shouldNotBeMuchSlowerThanTheSequentialSort() {
        ParallelSorter s = Candidate.of(Solution::create);
        Random r = new Random(2);
        int[] base = new int[4_000_000];
        for (int i = 0; i < base.length; i++) {
            base[i] = r.nextInt();
        }
        int[] seq = base.clone();
        int[] par = base.clone();
        s.sort(seq.clone(), 1); // warm up the JIT

        long t0 = System.nanoTime();
        s.sort(seq, 1);
        long sequentialNs = System.nanoTime() - t0;
        long t1 = System.nanoTime();
        s.sort(par, 4);
        long parallelNs = System.nanoTime() - t1;

        assertThat(par).isEqualTo(seq);
        assertThat(parallelNs).as("parallel version should not be 3x slower than sequential").isLessThan(sequentialNs * 3);
    }
}

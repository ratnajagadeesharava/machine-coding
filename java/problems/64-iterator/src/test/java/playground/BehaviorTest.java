package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Page;
import contract.PageFetcher;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.NoSuchElementException;
import org.junit.jupiter.api.Test;
import solution.Iterators;
import solution.PagedFeed;

class BehaviorTest {

    /** A fake remote API over fixed pages; counts and can fail fetches. */
    private static final class FakePages implements PageFetcher {
        final List<List<String>> pages;
        final List<Integer> fetched = new ArrayList<>();
        int failOnPage = -1;

        FakePages(List<String>... pages) { this.pages = List.of(pages); }

        public Page fetch(int pageNumber) {
            fetched.add(pageNumber);
            if (pageNumber == failOnPage) throw new IllegalStateException("remote error on page " + pageNumber);
            return new Page(pages.get(pageNumber), pageNumber + 1 < pages.size());
        }
    }

    @SafeVarargs
    private static FakePages pages(List<String>... pages) {
        return new FakePages(pages);
    }

    @Test
    void shouldFetchNothingUntilSomebodyAsksForAnItem() {
        FakePages api = pages(List.of("a", "b"), List.of("c"));

        PagedFeed feed = new PagedFeed(api);
        Iterator<String> it = feed.iterator();

        assertThat(api.fetched).isEmpty();
        assertThat(it.hasNext()).isTrue();
        assertThat(api.fetched).containsExactly(0);
    }

    @Test
    void shouldYieldEveryItemInOrderAcrossPages() {
        PagedFeed feed = new PagedFeed(pages(List.of("a", "b"), List.of("c", "d"), List.of("e")));

        List<String> all = new ArrayList<>();
        feed.forEach(all::add);

        assertThat(all).containsExactly("a", "b", "c", "d", "e");
    }

    @Test
    void shouldFetchTheNextPageOnlyWhenTheCurrentOneIsUsedUpAndMoreIsRequested() {
        FakePages api = pages(List.of("a", "b"), List.of("c", "d"), List.of("e"));
        Iterator<String> it = new PagedFeed(api).iterator();

        assertThat(it.next()).isEqualTo("a");
        assertThat(it.next()).isEqualTo("b");
        assertThat(api.fetched).containsExactly(0);
        assertThat(it.next()).isEqualTo("c");
        assertThat(api.fetched).containsExactly(0, 1);
    }

    @Test
    void shouldStopFetchingWhenTheConsumerStopsEarly() {
        FakePages api = pages(List.of("a", "b"), List.of("c", "d"), List.of("e", "f"));
        int taken = 0;

        for (String item : new PagedFeed(api)) {
            if (++taken == 3) {
                break;
            }
        }

        assertThat(api.fetched).containsExactly(0, 1);
    }

    @Test
    void shouldSkipEmptyPages() {
        FakePages api = pages(List.of(), List.of("a"), List.of(), List.of(), List.of("b"));
        List<String> all = new ArrayList<>();

        new PagedFeed(api).forEach(all::add);

        assertThat(all).containsExactly("a", "b");
        assertThat(api.fetched).containsExactly(0, 1, 2, 3, 4);
    }

    @Test
    void shouldHaveAnIdempotentHasNextAndFailCleanlyAtTheEnd() {
        FakePages api = pages(List.of("a"));
        Iterator<String> it = new PagedFeed(api).iterator();

        assertThat(it.hasNext()).isTrue();
        assertThat(it.hasNext()).isTrue();
        assertThat(it.next()).isEqualTo("a");
        assertThat(it.hasNext()).isFalse();
        assertThat(it.hasNext()).isFalse();
        assertThatThrownBy(it::next).isInstanceOf(NoSuchElementException.class);
        assertThat(api.fetched).containsExactly(0);
    }

    @Test
    void shouldGiveEveryCallerAnIndependentIterator() {
        FakePages api = pages(List.of("a", "b"), List.of("c"));
        PagedFeed feed = new PagedFeed(api);

        Iterator<String> one = feed.iterator();
        Iterator<String> two = feed.iterator();

        assertThat(one.next()).isEqualTo("a");
        assertThat(two.next()).isEqualTo("a");
        assertThat(one.next()).isEqualTo("b");
        assertThat(one.next()).isEqualTo("c");
        assertThat(two.next()).isEqualTo("b");
        assertThat(two.next()).isEqualTo("c");
        assertThat(one.hasNext()).isFalse();
        assertThat(two.hasNext()).isFalse();
    }

    @Test
    void shouldPropagateFetchFailuresAndRetryTheSamePageNextTime() {
        FakePages api = pages(List.of("a", "b"), List.of("c"));
        api.failOnPage = 1;
        Iterator<String> it = new PagedFeed(api).iterator();
        assertThat(it.next()).isEqualTo("a");
        assertThat(it.next()).isEqualTo("b");

        assertThatThrownBy(it::hasNext).isInstanceOf(IllegalStateException.class).hasMessageContaining("page 1");
        api.failOnPage = -1;

        assertThat(it.hasNext()).isTrue();
        assertThat(it.next()).isEqualTo("c");
        assertThat(api.fetched).containsExactly(0, 1, 1);
    }

    @Test
    void shouldFilterLazilyPullingNoMoreThanNeeded() {
        int[] pulled = {0};
        Iterator<Integer> source = new Iterator<>() {
            int next = 1;
            public boolean hasNext() { return next <= 10; }
            public Integer next() {
                pulled[0]++;
                return next++;
            }
        };
        Iterator<Integer> evens = Iterators.filter(source, n -> n % 2 == 0);

        assertThat(pulled[0]).isZero();
        assertThat(evens.hasNext()).isTrue();
        assertThat(evens.hasNext()).isTrue();
        assertThat(pulled[0]).isEqualTo(2);
        assertThat(evens.next()).isEqualTo(2);
        assertThat(pulled[0]).isEqualTo(2);
        assertThat(evens.next()).isEqualTo(4);
        assertThat(pulled[0]).isEqualTo(4);
    }

    @Test
    void shouldFilterAPagedFeedAndEndCleanly() {
        FakePages api = pages(List.of("aa", "b", "cc"), List.of("d", "ee"));
        Iterator<String> twoLetters = Iterators.filter(new PagedFeed(api).iterator(), s -> s.length() == 2);
        List<String> all = new ArrayList<>();

        twoLetters.forEachRemaining(all::add);

        assertThat(all).containsExactly("aa", "cc", "ee");
        assertThat(twoLetters.hasNext()).isFalse();
        assertThatThrownBy(twoLetters::next).isInstanceOf(NoSuchElementException.class);
        assertThatThrownBy(() -> Iterators.filter(null, x -> true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Iterators.filter(List.of(1).iterator(), null)).isInstanceOf(IllegalArgumentException.class);
    }
}

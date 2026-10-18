package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ImportReport;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Deque;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.CsvImporter;
import solution.DataImporter;

class BehaviorTest {

    /** A subclass written in the test: records every step in order so the template can be checked. */
    private static final class Recording extends DataImporter {
        final List<String> events = new ArrayList<>();
        final Deque<String> input = new ArrayDeque<>();
        final Set<String> invalid = new HashSet<>();
        String failTransformOn;
        String failSaveOn;
        String failReadOn;
        boolean failOpen;

        Recording(String... records) { input.addAll(List.of(records)); }

        @Override protected void open() {
            events.add("open");
            if (failOpen) throw new IllegalStateException("cannot open");
        }

        @Override protected Optional<String> readRecord() {
            events.add("read");
            if (input.isEmpty()) return Optional.empty();
            String next = input.poll();
            if (next.equals(failReadOn)) throw new IllegalStateException("read failed");
            return Optional.of(next);
        }

        @Override protected boolean validate(String record) {
            events.add("validate:" + record);
            return !invalid.contains(record);
        }

        @Override protected String transform(String record) {
            events.add("transform:" + record);
            if (record.equals(failTransformOn)) throw new IllegalStateException("transform failed");
            return record.toUpperCase();
        }

        @Override protected void save(String record) {
            events.add("save:" + record);
            if (record.equals(failSaveOn)) throw new IllegalStateException("save failed");
        }

        @Override protected void close() { events.add("close"); }
    }

    @Test
    void shouldRunTheStepsInTheFixedOrder() {
        Recording importer = new Recording("a", "b");

        ImportReport report = importer.run();

        assertThat(importer.events).containsExactly(
                "open", "read", "validate:a", "transform:a", "save:A",
                "read", "validate:b", "transform:b", "save:B", "read", "close");
        assertThat(report).isEqualTo(new ImportReport(2, 2, 0, 0));
    }

    @Test
    void shouldSkipInvalidRecordsWithoutTransformingOrSavingThem() {
        Recording importer = new Recording("a", "bad", "c");
        importer.invalid.add("bad");

        ImportReport report = importer.run();

        assertThat(importer.events).doesNotContain("transform:bad", "save:BAD");
        assertThat(report).isEqualTo(new ImportReport(3, 2, 1, 0));
    }

    @Test
    void shouldCountATransformFailureAndCarryOn() {
        Recording importer = new Recording("a", "boom", "c");
        importer.failTransformOn = "boom";

        ImportReport report = importer.run();

        assertThat(importer.events).doesNotContain("save:BOOM");
        assertThat(importer.events).contains("save:A", "save:C");
        assertThat(report).isEqualTo(new ImportReport(3, 2, 0, 1));
    }

    @Test
    void shouldCountASaveFailureAndCarryOn() {
        Recording importer = new Recording("a", "b", "c");
        importer.failSaveOn = "B";

        ImportReport report = importer.run();

        assertThat(importer.events).contains("save:B", "save:C");
        assertThat(report).isEqualTo(new ImportReport(3, 2, 0, 1));
    }

    @Test
    void shouldAlwaysCloseWhenReadingFailsAndPropagateTheFailure() {
        Recording importer = new Recording("a", "oops", "c");
        importer.failReadOn = "oops";

        assertThatThrownBy(importer::run).isInstanceOf(IllegalStateException.class).hasMessage("read failed");

        assertThat(importer.events).endsWith("close");
        assertThat(importer.events).containsOnlyOnce("close");
        assertThat(importer.events).contains("save:A");
    }

    @Test
    void shouldNotCloseWhatNeverOpened() {
        Recording importer = new Recording("a");
        importer.failOpen = true;

        assertThatThrownBy(importer::run).isInstanceOf(IllegalStateException.class).hasMessage("cannot open");

        assertThat(importer.events).containsExactly("open");
    }

    @Test
    void shouldProvideSensibleDefaultsForTheOptionalSteps() {
        List<String> saved = new ArrayList<>();
        Deque<String> input = new ArrayDeque<>(List.of("x", "   ", "y", ""));
        DataImporter minimal = new DataImporter() {
            @Override protected void open() { }
            @Override protected Optional<String> readRecord() { return Optional.ofNullable(input.poll()); }
            @Override protected void save(String record) { saved.add(record); }
        };

        ImportReport report = minimal.run();

        assertThat(saved).containsExactly("x", "y");
        assertThat(report).isEqualTo(new ImportReport(4, 2, 2, 0));
    }

    @Test
    void shouldImportCsvEndToEnd() {
        List<String> sink = new ArrayList<>();
        CsvImporter importer = new CsvImporter(List.of(
                "name,email,city",
                "ann , a@x.io, Rome",
                "bob,b@x.io",
                "",
                "cy,c@x.io,",
                "dee,d@x.io,Oslo"), sink);

        ImportReport report = importer.run();

        assertThat(sink).containsExactly("ann|a@x.io|Rome", "dee|d@x.io|Oslo");
        assertThat(report).isEqualTo(new ImportReport(5, 2, 3, 0));
        assertThat(new CsvImporter(List.of(), new ArrayList<>()).run()).isEqualTo(new ImportReport(0, 0, 0, 0));
        assertThat(new CsvImporter(List.of("only,a,header"), new ArrayList<>()).run()).isEqualTo(new ImportReport(0, 0, 0, 0));
        assertThatThrownBy(() -> new CsvImporter(null, sink)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new CsvImporter(List.of(), null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldFixTheAlgorithmAndLeaveOnlyTheStepsOpen() throws Exception {
        assertThat(new Recording("a").run().read()).isEqualTo(1);

        Method run = DataImporter.class.getMethod("run");
        assertThat(Modifier.isFinal(run.getModifiers())).as("run must be final").isTrue();
        assertThat(Modifier.isAbstract(DataImporter.class.getModifiers())).isTrue();
        for (String name : List.of("open", "readRecord", "save")) {
            Method step = find(name);
            assertThat(Modifier.isAbstract(step.getModifiers())).as(name + " is a required step").isTrue();
            assertThat(Modifier.isProtected(step.getModifiers())).as(name + " is protected").isTrue();
        }
        for (String name : List.of("validate", "transform", "close")) {
            Method hook = find(name);
            assertThat(Modifier.isAbstract(hook.getModifiers())).as(name + " has a default").isFalse();
            assertThat(Modifier.isProtected(hook.getModifiers())).as(name + " is protected").isTrue();
        }
    }

    private static Method find(String name) {
        for (Method m : DataImporter.class.getDeclaredMethods()) {
            if (m.getName().equals(name)) return m;
        }
        throw new AssertionError("no method " + name);
    }
}

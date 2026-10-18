package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Course;
import solution.Student;

class BehaviorTest {

    @Test
    void shouldUpdateBothSidesWhenAStudentEnrolls() {
        Student ann = new Student("Ann");
        Course math = new Course("MATH", 10);

        assertThat(ann.enroll(math)).isTrue();

        assertThat(ann.courses()).containsExactly(math);
        assertThat(math.students()).containsExactly(ann);
    }

    @Test
    void shouldUpdateBothSidesWhenTheCourseEnrolls() {
        Student ann = new Student("Ann");
        Course math = new Course("MATH", 10);

        assertThat(math.enroll(ann)).isTrue();

        assertThat(ann.courses()).containsExactly(math);
        assertThat(math.students()).containsExactly(ann);
    }

    @Test
    void shouldSupportManyToMany() {
        Student ann = new Student("Ann");
        Student bob = new Student("Bob");
        Course math = new Course("MATH", 10);
        Course art = new Course("ART", 10);

        ann.enroll(math);
        ann.enroll(art);
        bob.enroll(math);

        assertThat(ann.courses()).containsExactly(math, art);
        assertThat(math.students()).containsExactly(ann, bob);
        assertThat(art.students()).containsExactly(ann);
        assertThat(bob.courses()).containsExactly(math);
    }

    @Test
    void shouldNotEnrollTwiceFromEitherSide() {
        Student ann = new Student("Ann");
        Course math = new Course("MATH", 10);
        ann.enroll(math);

        assertThat(ann.enroll(math)).isFalse();
        assertThat(math.enroll(ann)).isFalse();

        assertThat(ann.courses()).hasSize(1);
        assertThat(math.students()).hasSize(1);
    }

    @Test
    void shouldRefuseAFullCourseAndLeaveBothSidesUnchanged() {
        Course tiny = new Course("TINY", 1);
        Student ann = new Student("Ann");
        Student bob = new Student("Bob");
        ann.enroll(tiny);

        assertThat(tiny.isFull()).isTrue();
        assertThat(bob.enroll(tiny)).isFalse();

        assertThat(bob.courses()).isEmpty();
        assertThat(tiny.students()).containsExactly(ann);
    }

    @Test
    void shouldLimitAStudentToThreeCourses() {
        Student ann = new Student("Ann");
        Course c1 = new Course("C1", 5);
        Course c2 = new Course("C2", 5);
        Course c3 = new Course("C3", 5);
        Course c4 = new Course("C4", 5);
        ann.enroll(c1);
        ann.enroll(c2);
        ann.enroll(c3);

        assertThat(ann.enroll(c4)).isFalse();

        assertThat(ann.courses()).hasSize(Student.MAX_COURSES);
        assertThat(c4.students()).isEmpty();
    }

    @Test
    void shouldFreeTheSeatOnBothSidesWhenDroppedFromEitherSide() {
        Course tiny = new Course("TINY", 1);
        Student ann = new Student("Ann");
        Student bob = new Student("Bob");
        ann.enroll(tiny);

        assertThat(ann.drop(tiny)).isTrue();
        assertThat(ann.drop(tiny)).isFalse();
        assertThat(tiny.students()).isEmpty();
        assertThat(ann.courses()).isEmpty();

        bob.enroll(tiny);
        assertThat(tiny.drop(bob)).isTrue();
        assertThat(bob.courses()).isEmpty();
        assertThat(tiny.isFull()).isFalse();
    }

    @Test
    void shouldKeepBothSidesInSyncAfterAnySequenceOfOperations() {
        List<Student> students = List.of(new Student("s0"), new Student("s1"), new Student("s2"), new Student("s3"));
        List<Course> courses = List.of(new Course("c0", 2), new Course("c1", 3), new Course("c2", 1), new Course("c3", 2));
        java.util.Random rnd = new java.util.Random(7);

        for (int i = 0; i < 300; i++) {
            Student s = students.get(rnd.nextInt(students.size()));
            Course c = courses.get(rnd.nextInt(courses.size()));
            switch (rnd.nextInt(4)) {
                case 0 -> s.enroll(c);
                case 1 -> c.enroll(s);
                case 2 -> s.drop(c);
                default -> c.drop(s);
            }
        }

        for (Student s : students) {
            assertThat(s.courses().size()).isLessThanOrEqualTo(Student.MAX_COURSES);
            for (Course c : courses) {
                assertThat(s.courses().contains(c)).as(s.name() + " / " + c.code()).isEqualTo(c.students().contains(s));
            }
        }
    }

    @Test
    void shouldLetEachSideOutliveTheOtherAndRejectBadInput() {
        Student ann = new Student("Ann");
        Course math = new Course("MATH", 2);
        ann.enroll(math);
        ann.drop(math);

        assertThat(math.students()).isEmpty();
        assertThat(math.code()).isEqualTo("MATH");
        assertThat(ann.name()).isEqualTo("Ann");
        assertThatThrownBy(() -> ann.enroll(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Course("", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Course("X", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> new Student(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> math.students().add(ann)).isInstanceOf(UnsupportedOperationException.class);
        assertThat(math.students()).isEmpty();
    }
}

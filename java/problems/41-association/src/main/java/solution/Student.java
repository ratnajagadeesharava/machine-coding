package solution;

import java.util.List;

/** A student can be enrolled in several courses (at most {@link #MAX_COURSES}). Neither side owns the other. */
public class Student {

    public static final int MAX_COURSES = 3;

    /** Blank name -> IllegalArgumentException. */
    public Student(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Enrols in the course; both sides must know about it afterwards. Returns false (and changes nothing) if
     * already enrolled, the course is full or the student already has MAX_COURSES. Null -> IllegalArgumentException.
     */
    public boolean enroll(Course course) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Leaves the course on both sides. Returns whether the student was enrolled. */
    public boolean drop(Course course) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Courses in enrolment order. Callers cannot modify the list. */
    public List<Course> courses() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

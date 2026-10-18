package solution;

import java.util.List;

/** A course has a seat limit. It does not own its students. */
public class Course {

    /** Blank code or capacity <= 0 -> IllegalArgumentException. */
    public Course(String code, int capacity) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String code() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Same effect as {@code student.enroll(this)}: whichever side you start from, both stay consistent. */
    public boolean enroll(Student student) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Same effect as {@code student.drop(this)}. */
    public boolean drop(Student student) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Students in enrolment order. Callers cannot modify the list. */
    public List<Student> students() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public boolean isFull() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

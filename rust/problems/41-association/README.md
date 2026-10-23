# 41 · Association (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — a `School` stub is provided; ids, the limit constant and `EnrollError` are provided. |

## Problem

A college lets students enrol in courses. A student may take several courses, a course has many students, and you
can ask either side about the other.

## Requirements

1. `add_student(name)` / `add_course(code, capacity)` return ids; blank name/code or zero capacity → `Err(EnrollError::InvalidInput)`.
2. `enroll(student, course)`: afterwards `courses_of(student)` contains the course code and `students_of(course)` contains the student name (both in enrolment order).
3. Failures change **nothing** and return: `UnknownStudent`, `UnknownCourse`, `AlreadyEnrolled`, `CourseFull`, or `StudentAtLimit` (already `MAX_COURSES_PER_STUDENT` = 3 courses).
4. `drop_course` removes the link (`NotEnrolled` if there is none) and frees the seat.
5. `remove_student` removes the student and frees all their seats; the courses live on. Afterwards that student id is unknown.
6. `is_full(course)` is true when enrolled count equals capacity.

## Running

```bash
cargo test -p p41_association     # from rust/ — tests FAIL until you implement School
cargo play run 41
```

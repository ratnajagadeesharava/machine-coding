# 41 · Association (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — a `School` stub is provided; ids, the limit constant and `EnrollError` are provided. |

## Problem

You are building the enrolment part of a college system, held in one `School`. A student may take several courses, a
course has many students, and you can ask either side about the other ("which courses is Ann in?", "who is in
MATH?"). Both views must always agree. Courses have a limited number of seats and students have a limit on how many
courses they can take.

## Glossary

- **id**: `StudentId` / `CourseId` (`u32`), returned by `add_student` / `add_course` and used in every later call.
- **enrol**: link one student with one course; both views then show the link.
- **capacity**: the number of seats of a course; `is_full(course)` is true when the enrolled count equals it.
- **`MAX_COURSES_PER_STUDENT`**: the maximum number of courses one student may be enrolled in (3).
- **enrolment order**: the order in which the links were made; `courses_of` (course codes) and `students_of` (student names) list in that order.

## Requirements

1. `add_student(name)` / `add_course(code, capacity)` return ids; blank name/code or zero capacity gives `Err(EnrollError::InvalidInput)`.
2. `enroll(student, course)`: afterwards `courses_of(student)` contains the course code and `students_of(course)` contains the student name (both in enrolment order).
3. Failures change **nothing** and return: `UnknownStudent`, `UnknownCourse`, `AlreadyEnrolled`, `CourseFull`, or `StudentAtLimit` (already `MAX_COURSES_PER_STUDENT` = 3 courses).
4. `drop_course(student, course)` removes the link (`Err(NotEnrolled)` if there is none) and frees the seat.
5. `remove_student` removes the student and frees all their seats; the courses live on. Afterwards that student id is unknown (e.g. `courses_of` gives `Err(UnknownStudent)`).
6. `is_full(course)` is true when enrolled count equals capacity. `courses_of`, `students_of` and `is_full` return `Err(UnknownStudent)` / `Err(UnknownCourse)` for ids that do not exist.

## Examples

```rust
let mut school = School::new();
let ann = school.add_student("Ann").unwrap();
let bob = school.add_student("Bob").unwrap();
let math = school.add_course("MATH", 10).unwrap();
let art = school.add_course("ART", 10).unwrap();
school.enroll(ann, math).unwrap();
school.enroll(ann, art).unwrap();
school.enroll(bob, math).unwrap();
school.courses_of(ann);          // Ok(["MATH", "ART"])
school.students_of(math);        // Ok(["Ann", "Bob"])
school.enroll(ann, math);        // Err(AlreadyEnrolled)

let tiny = school.add_course("TINY", 1).unwrap();
school.enroll(ann, tiny).unwrap();   // is_full(tiny) == Ok(true)
school.enroll(bob, tiny);            // Err(CourseFull); nothing changes
school.drop_course(ann, tiny).unwrap();
school.drop_course(ann, tiny);       // Err(NotEnrolled)
school.enroll(bob, tiny).unwrap();   // the seat was freed

school.remove_student(ann).unwrap();
school.students_of(math);        // Ok(["Bob"]) - the course lives on
school.courses_of(ann);          // Err(UnknownStudent)
```

With a student already in 3 courses, enrolling in a fourth gives `Err(StudentAtLimit)` and the fourth course stays empty.

## Edge cases

- `school.enroll(ann, 999)` gives `Err(UnknownCourse)`; `school.enroll(999, 0)` gives `Err(UnknownStudent)`.
- `add_student(" ")`, `add_course("", 5)` and `add_course("X", 0)` give `Err(InvalidInput)`.

## Running

```bash
cargo test -p p41_association     # from rust/ — tests FAIL until you implement School
cargo play run 41
```

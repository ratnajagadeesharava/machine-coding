# 41 · Association (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Student` and `Course` stubs are provided; you write the bodies. |

## Problem

You are building the enrolment part of a college system. A student may take several courses, a course has many
students, and each side can be asked about the other ("which courses is Ann in?", "who is in MATH?"). Both views must
always agree, whichever side the enrolment was made from. Courses have a limited number of seats and students have a
limit on how many courses they can take.

## Glossary

- **enrol**: link one student with one course; both sides then list each other.
- **capacity**: the number of seats of a course; `course.isFull()` is true when the number of enrolled students equals it.
- **`Student.MAX_COURSES`**: the maximum number of courses one student may be enrolled in (3).
- **enrolment order**: the order in which the links were made; `courses()` and `students()` list in that order.

## Requirements

1. `Student(name)` with `name()`, and `Course(code, capacity)` with `code()`, `isFull()`; blank name/code or `capacity <= 0` throws `IllegalArgumentException`.
2. `student.enroll(course)` and `course.enroll(student)` do the same thing: afterwards **both** `student.courses()` and `course.students()` contain each other, in enrolment order. They return `true` on success.
3. They return `false` and change **nothing on either side** when: already enrolled (from either side), the course is full, or the student already has `Student.MAX_COURSES` (3) courses. `null` throws `IllegalArgumentException`.
4. `student.drop(course)` / `course.drop(student)` remove the link on both sides and return `true` if it existed, `false` otherwise. A freed seat can be taken by someone else.
5. `courses()` and `students()` are read-only views (modifying them throws `UnsupportedOperationException`).
6. Neither side owns the other: a course keeps existing with zero students, a student with zero courses.

## Examples

```java
Student ann = new Student("Ann"), bob = new Student("Bob");
Course math = new Course("MATH", 10), art = new Course("ART", 10);
ann.enroll(math);        // true
ann.enroll(art);         // true
math.enroll(bob);        // true (enrolling from the course side)
ann.courses();           // [math, art]
math.students();         // [ann, bob]
bob.courses();           // [math]
ann.enroll(math);        // false (already enrolled); math.enroll(ann) is false too

Course tiny = new Course("TINY", 1);
ann.enroll(tiny);        // true -> tiny.isFull() == true
bob.enroll(tiny);        // false; bob.courses() unchanged, tiny.students() == [ann]
tiny.drop(ann);          // true, seat freed (ann.courses() no longer has tiny)
tiny.drop(ann);          // false
bob.enroll(tiny);        // true
```

With `Ann` in `C1`, `C2`, `C3`, `ann.enroll(c4)` returns `false` and `c4.students()` stays empty.

## Edge cases

- `ann.enroll(null)` throws `IllegalArgumentException`; so do `new Course("", 5)`, `new Course("X", 0)` and `new Student(" ")`.
- After `ann.drop(math)`, `math` still exists with `students()` empty and `ann` still exists with `courses()` empty.
- Whatever mix of enrol/drop calls is made from either side, `s.courses().contains(c)` always equals `c.students().contains(s)`, and no student ever has more than 3 courses.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

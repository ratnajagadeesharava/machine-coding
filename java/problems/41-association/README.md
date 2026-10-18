# 41 · Association (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Student` and `Course` stubs. Bodies throw `UnsupportedOperationException`. |

> **Concept:** association is a *uses/knows-about* relationship between independent objects. Here it is
> **many-to-many and bidirectional**: students know their courses and courses know their students, and neither owns
> the other. The hard part is keeping both directions consistent.

## Problem

A college lets students enrol in courses. A student may take several courses, a course has many students, and each
side can be asked about the other.

## Requirements

1. `Student(name)` and `Course(code, capacity)`; blank name/code or `capacity <= 0` → `IllegalArgumentException`.
2. `student.enroll(course)` and `course.enroll(student)` do the same thing: afterwards **both** `student.courses()` and `course.students()` contain each other, in enrolment order. They return `true` on success.
3. They return `false` and change **nothing on either side** when: already enrolled, the course is full, or the student already has `Student.MAX_COURSES` (3) courses. `null` → `IllegalArgumentException`.
4. `student.drop(course)` / `course.drop(student)` remove the link on both sides and return whether it existed. A freed seat can be taken by someone else.
5. `courses()` and `students()` are read-only views.
6. Neither side owns the other: a course keeps existing with zero students, a student with zero courses.

## Constraints

- After any sequence of operations from either side, `s.courses().contains(c) == c.students().contains(s)` for every pair (the tests run 300 random operations and check this).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

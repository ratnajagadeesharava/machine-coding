//! Association — students and courses know about each other (many-to-many) but neither owns the other.
//!
//! Rust adaptation: two-way references (`Rc<RefCell<..>>` pointing at each other) leak and fight the borrow
//! checker. The idiomatic model keeps the association in ONE place - `School` - and refers to things by id.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

pub struct School;

impl School {
    pub fn new() -> School {
        todo!()
    }

    /// Blank name -> `InvalidInput`.
    pub fn add_student(&mut self, name: &str) -> Result<StudentId, EnrollError> {
        todo!()
    }

    /// Blank code or zero capacity -> `InvalidInput`.
    pub fn add_course(&mut self, code: &str, capacity: usize) -> Result<CourseId, EnrollError> {
        todo!()
    }

    /// Errors (nothing changes): `UnknownStudent`, `UnknownCourse`, `AlreadyEnrolled`, `CourseFull`,
    /// `StudentAtLimit` (already `MAX_COURSES_PER_STUDENT` courses).
    pub fn enroll(&mut self, student: StudentId, course: CourseId) -> Result<(), EnrollError> {
        todo!()
    }

    /// `NotEnrolled` if the link does not exist; unknown ids as above.
    pub fn drop_course(&mut self, student: StudentId, course: CourseId) -> Result<(), EnrollError> {
        todo!()
    }

    /// Removes the student and frees every seat they held. The courses live on.
    pub fn remove_student(&mut self, student: StudentId) -> Result<(), EnrollError> {
        todo!()
    }

    /// Course codes in enrolment order.
    pub fn courses_of(&self, student: StudentId) -> Result<Vec<String>, EnrollError> {
        todo!()
    }

    /// Student names in enrolment order.
    pub fn students_of(&self, course: CourseId) -> Result<Vec<String>, EnrollError> {
        todo!()
    }

    pub fn is_full(&self, course: CourseId) -> Result<bool, EnrollError> {
        todo!()
    }
}

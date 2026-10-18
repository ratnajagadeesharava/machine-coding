//! Provided: ids, limits and the error type.

pub type StudentId = u32;
pub type CourseId = u32;

/// A student may be enrolled in at most this many courses.
pub const MAX_COURSES_PER_STUDENT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnrollError {
    UnknownStudent,
    UnknownCourse,
    AlreadyEnrolled,
    NotEnrolled,
    CourseFull,
    StudentAtLimit,
    /// Blank name / code or zero capacity.
    InvalidInput,
}

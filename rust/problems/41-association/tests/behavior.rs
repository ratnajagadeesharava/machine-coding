use p41_association::contract::*;
use p41_association::School;

fn names(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn should_show_the_link_from_both_sides_after_enrolling() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();
    let math = school.add_course("MATH", 10).unwrap();

    school.enroll(ann, math).unwrap();

    assert_eq!(school.courses_of(ann).unwrap(), names(&["MATH"]));
    assert_eq!(school.students_of(math).unwrap(), names(&["Ann"]));
}

#[test]
fn should_support_many_to_many() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();
    let bob = school.add_student("Bob").unwrap();
    let math = school.add_course("MATH", 10).unwrap();
    let art = school.add_course("ART", 10).unwrap();

    school.enroll(ann, math).unwrap();
    school.enroll(ann, art).unwrap();
    school.enroll(bob, math).unwrap();

    assert_eq!(school.courses_of(ann).unwrap(), names(&["MATH", "ART"]));
    assert_eq!(school.students_of(math).unwrap(), names(&["Ann", "Bob"]));
    assert_eq!(school.students_of(art).unwrap(), names(&["Ann"]));
    assert_eq!(school.courses_of(bob).unwrap(), names(&["MATH"]));
}

#[test]
fn should_not_enroll_twice() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();
    let math = school.add_course("MATH", 10).unwrap();
    school.enroll(ann, math).unwrap();

    assert_eq!(school.enroll(ann, math), Err(EnrollError::AlreadyEnrolled));

    assert_eq!(school.courses_of(ann).unwrap().len(), 1);
    assert_eq!(school.students_of(math).unwrap().len(), 1);
}

#[test]
fn should_refuse_a_full_course_and_leave_both_sides_unchanged() {
    let mut school = School::new();
    let tiny = school.add_course("TINY", 1).unwrap();
    let ann = school.add_student("Ann").unwrap();
    let bob = school.add_student("Bob").unwrap();
    school.enroll(ann, tiny).unwrap();

    assert!(school.is_full(tiny).unwrap());
    assert_eq!(school.enroll(bob, tiny), Err(EnrollError::CourseFull));

    assert!(school.courses_of(bob).unwrap().is_empty());
    assert_eq!(school.students_of(tiny).unwrap(), names(&["Ann"]));
}

#[test]
fn should_limit_a_student_to_the_maximum_number_of_courses() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();
    let ids: Vec<CourseId> = (0..=MAX_COURSES_PER_STUDENT)
        .map(|i| school.add_course(&format!("C{i}"), 5).unwrap())
        .collect();
    for id in &ids[..MAX_COURSES_PER_STUDENT] {
        school.enroll(ann, *id).unwrap();
    }

    assert_eq!(school.enroll(ann, ids[MAX_COURSES_PER_STUDENT]), Err(EnrollError::StudentAtLimit));

    assert_eq!(school.courses_of(ann).unwrap().len(), MAX_COURSES_PER_STUDENT);
    assert!(school.students_of(ids[MAX_COURSES_PER_STUDENT]).unwrap().is_empty());
}

#[test]
fn should_free_the_seat_on_both_sides_when_dropped() {
    let mut school = School::new();
    let tiny = school.add_course("TINY", 1).unwrap();
    let ann = school.add_student("Ann").unwrap();
    let bob = school.add_student("Bob").unwrap();
    school.enroll(ann, tiny).unwrap();

    school.drop_course(ann, tiny).unwrap();
    assert_eq!(school.drop_course(ann, tiny), Err(EnrollError::NotEnrolled));
    assert!(school.courses_of(ann).unwrap().is_empty());
    assert!(!school.is_full(tiny).unwrap());

    school.enroll(bob, tiny).unwrap();
    assert_eq!(school.students_of(tiny).unwrap(), names(&["Bob"]));
}

#[test]
fn should_keep_the_course_alive_when_a_student_is_removed() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();
    let bob = school.add_student("Bob").unwrap();
    let math = school.add_course("MATH", 2).unwrap();
    school.enroll(ann, math).unwrap();
    school.enroll(bob, math).unwrap();

    school.remove_student(ann).unwrap();

    assert_eq!(school.students_of(math).unwrap(), names(&["Bob"]));
    assert_eq!(school.courses_of(ann), Err(EnrollError::UnknownStudent));
    assert!(!school.is_full(math).unwrap());
}

#[test]
fn should_reject_unknown_ids_and_bad_input() {
    let mut school = School::new();
    let ann = school.add_student("Ann").unwrap();

    assert_eq!(school.enroll(ann, 999), Err(EnrollError::UnknownCourse));
    assert_eq!(school.enroll(999, 0), Err(EnrollError::UnknownStudent));
    assert_eq!(school.add_student(" "), Err(EnrollError::InvalidInput));
    assert_eq!(school.add_course("", 5), Err(EnrollError::InvalidInput));
    assert_eq!(school.add_course("X", 0), Err(EnrollError::InvalidInput));
}

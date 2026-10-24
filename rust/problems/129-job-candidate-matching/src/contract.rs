//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MatchError {
    /// blank id / title / location / skill, `k == 0`
    InvalidArgument,
    /// the job id was never posted
    UnknownJob,
    /// the candidate id was never registered
    UnknownCandidate,
    /// `post_job` / `register_candidate` with an id that already exists
    AlreadyExists,
    /// `apply` on a closed job
    JobClosed,
    /// `apply` when the candidate already has an application for the job
    AlreadyApplied,
    /// `apply` when the candidate is not eligible for the job
    NotEligible,
    /// `withdraw` without an application
    NotApplied,
    /// `close_job` on a job that is already closed
    AlreadyClosed,
}

pub trait JobMatching: Send + Sync {
    /// Posts an open job. Skills are trimmed and compared case-insensitively; a skill in both sets counts as
    /// required only. Checks, in this order: blank `job_id` / `title` / `location` or a blank skill is
    /// `InvalidArgument`; an existing id is `AlreadyExists`. A rejected call changes nothing.
    #[allow(clippy::too_many_arguments)]
    fn post_job(
        &self,
        job_id: &str,
        title: &str,
        required_skills: &[&str],
        nice_to_have_skills: &[&str],
        min_years: u32,
        location: &str,
        max_salary: u64,
    ) -> Result<(), MatchError>;

    /// Registers a candidate. Same normalisation and error rules as `post_job` (`InvalidArgument` before
    /// `AlreadyExists`).
    fn register_candidate(
        &self,
        candidate_id: &str,
        skills: &[&str],
        years: u32,
        location: &str,
        expected_salary: u64,
        remote_ok: bool,
    ) -> Result<(), MatchError>;

    /// Replaces ALL data of an existing candidate; effective for every later query at once. `InvalidArgument` for
    /// invalid data (checked first), then `UnknownCandidate`. Applications already made are not touched.
    fn update_candidate(
        &self,
        candidate_id: &str,
        skills: &[&str],
        years: u32,
        location: &str,
        expected_salary: u64,
        remote_ok: bool,
    ) -> Result<(), MatchError>;

    /// The score defined in the README: 0 if the candidate is not eligible or the job is closed, otherwise 50..=140.
    /// `UnknownJob` / `UnknownCandidate` (the job is reported when both are unknown).
    fn match_score(&self, job_id: &str, candidate_id: &str) -> Result<u32, MatchError>;

    /// Ids of the eligible candidates of an OPEN job: score descending, then years descending, then candidate id
    /// ascending (byte-wise string order). At most `k`; empty for a closed job. `k == 0` is `InvalidArgument`
    /// (checked first), then `UnknownJob`.
    fn top_candidates(&self, job_id: &str, k: usize) -> Result<Vec<String>, MatchError>;

    /// Ids of the OPEN jobs the candidate is eligible for: score descending, then job id ascending. At most `k`.
    /// `k == 0` is `InvalidArgument` (checked first), then `UnknownCandidate`.
    fn top_jobs(&self, candidate_id: &str, k: usize) -> Result<Vec<String>, MatchError>;

    /// The candidate applies to the job. Checks in this order: unknown job, unknown candidate, `JobClosed`,
    /// `AlreadyApplied`, `NotEligible`. A failed call changes nothing.
    fn apply(&self, candidate_id: &str, job_id: &str) -> Result<(), MatchError>;

    /// Removes the candidate's application (allowed on a closed job). `UnknownJob`, `UnknownCandidate`, then
    /// `NotApplied`.
    fn withdraw(&self, candidate_id: &str, job_id: &str) -> Result<(), MatchError>;

    /// Candidate ids of the job's current applications in the order they were made. Readable after the job closes.
    fn applications(&self, job_id: &str) -> Result<Vec<String>, MatchError>;

    /// Closes the job; `AlreadyClosed` if it already is.
    fn close_job(&self, job_id: &str) -> Result<(), MatchError>;
}

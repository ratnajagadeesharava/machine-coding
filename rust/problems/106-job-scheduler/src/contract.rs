//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// Criterion name: among eligible machines pick the one with the fewest unfinished jobs.
pub const FEWEST_UNFINISHED: &str = "FEWEST_UNFINISHED";
/// Criterion name: among eligible machines pick the one with the most finished jobs.
pub const MOST_FINISHED: &str = "MOST_FINISHED";

#[derive(Debug, PartialEq, Eq)]
pub enum SchedulerError {
    /// the machine id is already registered
    DuplicateMachine,
    /// the machine id is not registered
    UnknownMachine,
    /// the job id is already assigned (finished or not)
    DuplicateJob,
    /// the job id was never assigned
    UnknownJob,
    /// `finish_job` on a job that is already finished
    AlreadyFinished,
    /// not one of the criterion names above (names are case-sensitive)
    UnknownCriterion,
    /// blank machine id / job id / capability
    InvalidArgument,
}

pub trait JobScheduler: Send + Sync {
    /// Registers a machine with the capabilities it offers (may be empty). Its counters start at 0.
    /// `InvalidArgument` for a blank id or blank capability, `DuplicateMachine` if the id exists.
    /// Nothing changes on error (a duplicate does not replace the original capabilities).
    fn add_machine(&self, machine_id: &str, capabilities: &[&str]) -> Result<(), SchedulerError>;

    /// Sets the criterion used by `submit_job`. Until it is called the default is `FEWEST_UNFINISHED`.
    /// `UnknownCriterion` for any other name (the previous default stays).
    fn set_default_criterion(&self, criterion: &str) -> Result<(), SchedulerError>;

    /// Same as `submit_job_with` using the scheduler's default criterion.
    fn submit_job(
        &self,
        job_id: &str,
        required_capabilities: &[&str],
    ) -> Result<Option<String>, SchedulerError>;

    /// Assigns the job to an eligible machine and returns its id. A machine is eligible when it offers ALL
    /// required capabilities (an empty requirement fits every machine). Among eligible machines the criterion
    /// decides; ties are broken by the lexicographically smallest machine id. The chosen machine's unfinished
    /// count goes up by one. `Ok(None)` when no machine is eligible; then nothing is recorded and the job id
    /// stays unused. `InvalidArgument` for a blank job id or blank capability, `DuplicateJob` for an id that
    /// is already assigned, `UnknownCriterion` for an unknown name. Nothing changes on error.
    fn submit_job_with(
        &self,
        job_id: &str,
        required_capabilities: &[&str],
        criterion: &str,
    ) -> Result<Option<String>, SchedulerError>;

    /// Marks an assigned job as finished: unfinished -1 and finished +1 on its machine.
    /// `UnknownJob` if it was never assigned, `AlreadyFinished` if it was finished before.
    fn finish_job(&self, job_id: &str) -> Result<(), SchedulerError>;

    /// Jobs assigned to the machine and not yet finished. `UnknownMachine` for an unregistered id.
    fn unfinished_count(&self, machine_id: &str) -> Result<usize, SchedulerError>;

    /// Jobs the machine has finished. `UnknownMachine` for an unregistered id.
    fn finished_count(&self, machine_id: &str) -> Result<usize, SchedulerError>;

    /// The machine a job was assigned to. `UnknownJob` if the job was never assigned.
    fn machine_of(&self, job_id: &str) -> Result<String, SchedulerError>;
}

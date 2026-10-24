package contract;

import java.util.List;
import java.util.Set;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface JobMatching {

    /**
     * Posts an open job. Skills are trimmed and compared case-insensitively. IllegalArgumentException if
     * {@code jobId} (compared exactly) is null/blank or already exists, {@code title} or {@code location} is
     * null/blank, a skill set is null or contains a null/blank skill, or {@code minYears} / {@code maxSalary} is
     * negative. A rejected call changes nothing.
     */
    void postJob(String jobId, String title, Set<String> requiredSkills, Set<String> niceToHaveSkills,
            int minYears, String location, long maxSalary);

    /**
     * Registers a candidate. Same normalisation and error rules as {@code postJob}: IllegalArgumentException for a
     * null/blank or already registered {@code candidateId}, a null set or null/blank skill, a null/blank location,
     * negative years or salary.
     */
    void registerCandidate(String candidateId, Set<String> skills, int years, String location,
            long expectedSalary, boolean remoteOk);

    /**
     * Replaces ALL data of an existing candidate (skills, years, location, salary, remote flag). Takes effect for
     * every later query at once. IllegalArgumentException for an unknown candidate or invalid data (the candidate is
     * then left exactly as it was). Applications the candidate already made are not touched.
     */
    void updateCandidate(String candidateId, Set<String> skills, int years, String location,
            long expectedSalary, boolean remoteOk);

    /**
     * The score defined in the README: 0 if the candidate is not eligible for the job or the job is closed,
     * otherwise a number from 50 to 140. IllegalArgumentException for an unknown job or candidate.
     */
    int matchScore(String jobId, String candidateId);

    /**
     * Ids of the eligible candidates of an OPEN job, best first: score descending, then years descending, then
     * candidateId ascending ({@code String.compareTo}). At most {@code k} entries (fewer if fewer are eligible); an
     * empty list for a closed job. IllegalArgumentException for an unknown job or {@code k < 1}.
     */
    List<String> topCandidates(String jobId, int k);

    /**
     * Ids of the OPEN jobs the candidate is eligible for, best first: score descending, then jobId ascending. At most
     * {@code k} entries. IllegalArgumentException for an unknown candidate or {@code k < 1}.
     */
    List<String> topJobs(String candidateId, int k);

    /**
     * The candidate applies to the job; eligibility is checked now. IllegalArgumentException for an unknown candidate
     * or job. IllegalStateException if the job is closed, the candidate already has an application for the job, or
     * the candidate is not eligible (checked in that order).
     */
    void apply(String candidateId, String jobId);

    /**
     * Removes the candidate's application (allowed on a closed job too). IllegalArgumentException for an unknown
     * candidate or job; IllegalStateException if there is no such application.
     */
    void withdraw(String candidateId, String jobId);

    /** Candidate ids of the job's current applications in the order they were made (a new list). Readable after the job closes. IllegalArgumentException for an unknown job. */
    List<String> applications(String jobId);

    /** Closes the job. IllegalArgumentException for an unknown job; IllegalStateException if it is already closed. */
    void closeJob(String jobId);
}

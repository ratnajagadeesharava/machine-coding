# Follow-ups — 129 Job Posting with Candidate Matching

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Skill synonyms and levels
`"js"` and `"javascript"` are the same skill, and a candidate's skill has a level (1 to 5) while the job asks for a minimum level per required skill. Where does normalisation live, how do levels change eligibility and the score, and what happens to the data of candidates registered before a synonym was added?

### V3 — Weighted skills
Each nice-to-have skill carries its own weight instead of a flat 10, and the 50-point cap becomes a cap on the sum. Which part of your scoring changes, and how do you keep the existing examples as a special case (all weights 10)? Can the scoring rule be swapped per job without touching the ranking code?

### V4 — Notify matching candidates when a job is posted
Candidates subscribe with a minimum score; when a job is posted, every subscribed candidate whose score reaches it is notified, in a deterministic order, and a notification is never sent twice for the same job. Who calls whom, what happens if a listener is slow or throws, and how do you avoid scanning every candidate on each post?

### V5 — Interview pipeline
An application moves through states (applied, screening, interview, offer, hired, rejected, withdrawn) with allowed transitions, and a hire fills one of a job's N openings and closes it when full. How do you model the transitions so illegal ones are impossible, what happens to the other applications when the job fills, and how does `withdraw` change?

### V6 — Blocked employers
A candidate can block employers; jobs carry an employer id, and blocked employers' jobs vanish from the candidate's `top_jobs` and from `top_candidates` for that employer's jobs, immediately and without breaking the existing tie-break rules. Where does the check go, and what does blocking do to applications that already exist?

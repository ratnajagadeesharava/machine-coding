# Quiz progress database

`progress.db` (created on first run) is a SQLite file with your quiz progress: finished attempts, per-quiz best scores, AI-generated quizzes and the quiz
you are in the middle of. It lives next to the code on purpose: **commit it on your own branch** (for example `my-solutions`) and your progress travels
with that branch. Each branch has its own copy.

- Do **not** commit it on `main`: `main` is the shared, clean playground and ships no progress file.
- **Reset** (Quizzes page, "Reset all progress") empties every table, so all your progress is back to zero. Commit the file afterwards to keep the reset.
- **Sync browser progress**: if you used the quizzes before this database existed, the page offers to copy the progress stored in your browser into
  the database once. Nothing is lost: results from both places are merged.
- The file is binary. If two branches both changed it, git cannot merge it: keep one side (`git checkout --theirs hld/data/progress.db`).
- Different location: set `HLD_DB=/path/to/file.db` before `npm start`.

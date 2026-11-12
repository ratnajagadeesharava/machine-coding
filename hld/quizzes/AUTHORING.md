# Writing quizzes

Quizzes live in `hld/quizzes/*.json` and are validated on server start (`npm test` checks them too). The AI quiz generator is
told to follow the same rules.

## Format

```json
{
  "id": "scaling-reads",
  "title": "Scaling Reads",
  "topic": "System design pattern",
  "description": "One or two sentences shown on the card.",
  "source": "Where the material comes from (optional)",
  "estimatedMinutes": 20,
  "questions": [
    {
      "id": "slow-email-login",
      "concept": "Indexing",
      "type": "single",
      "difficulty": 2,
      "prompt": "A complete question or scenario that ends with ? or .",
      "options": [
        { "id": "a", "text": "Add an index on the email column.", "why": "Correct: ..." },
        { "id": "b", "text": "Add three read replicas.", "why": "Why this fails." },
        { "id": "c", "text": "...", "why": "..." }
      ],
      "correct": ["a"],
      "explanation": "Teaches the idea and the interview takeaway."
    }
  ]
}
```

- Optional grouping: give several small quizzes the same `series` id plus `seriesTitle` and `seriesOrder` (1, 2, ...) and the quiz page shows them together as sub-topics with a progress bar. Keep each quiz to about 5-10 questions.
- `type`: `single` (one correct), `multi` (2+ correct, at least one wrong; say "Select all that apply" in the prompt),
  `truefalse` (options exactly `True` then `False`).
- `difficulty`: 1 recall, 2 application, 3 trade-off or edge case. Use all three.
- `concept`: a short label; reuse it across related questions. The results page groups scores by concept and lists the weakest first.
- Options are shuffled for every attempt (true/false keeps its order), so write `correct` however is convenient.

## Rules of thumb (enforced where a machine can check them)

1. **Test understanding, not trivia.** Mix recall, "which would you choose", and trade-off questions. Aim for about a third scenarios.
2. **One concept per question**, and a stem that can be answered without looking at the options.
3. **Plausible distractors.** Each wrong option is a real misconception or a technique that fits a different situation. No jokes.
4. **No "all/none of the above"** (checked).
5. **Do not let the answer's shape give it away.** Keep options similar in length and detail. Checked: the correct option may be the
   longest in at most 40% of single-answer questions, and never wildly longer or shorter than the distractors.
6. **Every option gets a `why`** (checked): why it is right, or the specific way it fails. Learners see these after answering.
7. **Mix types and keep true/false balanced** (checked): not every true/false answer should be "False".
8. **Be accurate and cite your source** in `source`. Do not invent statistics.

## JEE-Advanced style question types

Besides `single`, `multi` and `truefalse`, a question can be one of five richer types. They have no `options`/`correct` (except
`assertion`, which has `correct`). Everything else (`id`, `concept`, `difficulty`, `prompt`, `explanation`) is the same, and the prompt must
still end with `?` or `.`.

```json
{ "type": "integer",   "answer": 3, "tolerance": 0, "unit": "replicas" }
{ "type": "assertion", "assertion": "...", "reason": "...", "correct": ["a"], "whys": { "a": "...", "b": "...", "c": "...", "d": "..." } }
{ "type": "matrix",    "left": [{"id":"a","text":"..."}], "right": [{"id":"p","text":"..."}], "pairs": { "a": "p" } }
{ "type": "fill",      "text": "A {1} index speeds up {2} lookups.", "blanks": { "1": "B-tree", "2": "range" }, "bank": ["B-tree", "range", "hash", "full-text"] }
{ "type": "code",      "language": "sql", "starter": "SELECT ", "checks": [{ "label": "groups by customer", "pattern": "group\\s+by\\s+customer_id" }], "model": "SELECT ..." }
```

- **integer**: the learner types a number (counts, sizes, big-O exponents, complexity-as-number, "how many rows", "how many bytes"). Exact unless you set `tolerance`.
  Make the answer derivable by reasoning, not a number you must have memorised.
- **assertion** (assertion-reason): `a` = both true and R explains A, `b` = both true but R does not explain A, `c` = A true, R false, `d` = A false, R true.
  The four options are fixed; give a `why` for each in `whys`. Use all four letters across a quiz. Write R so that it is plausible even when it does not explain A.
- **matrix** (match the columns): 3-6 left items, as many or more right items (extra right items are distractors), a one-to-one `pairs` map.
- **fill** (drag the words): `{1}`, `{2}` ... mark blanks (max 6). `bank` holds every answer plus 1-3 distractors; words are shuffled per attempt.
- **code**: the learner writes a short snippet (Java, SQL, Python, JSON, YAML, a shell command, a regex...). It is graded by `checks`: case-insensitive regular
  expressions run on the learner's text (use `\\s+`, `\\s*` for whitespace), plus a `model` answer shown afterwards. The model answer MUST pass all of its own checks
  (the validator enforces it). Checks look for the essential elements, not exact formatting; keep them tolerant of aliases and spacing, and say honestly in
  the explanation that other correct solutions exist. Keep snippets to about 1-8 lines.

Roadmap quizzes (series `interview-roadmap`) must mix at least four distinct types, including at least one `code` and one `integer` question.

## Code in questions

The quiz page renders text with two bits of Markdown, in the prompt, the options (`text` and `why`), the explanation, assertion/reason and matrix items:

- A fenced block ```` ```java ... ``` ```` becomes a **snippet card** with a language label, syntax colours and a copy button. Tags: `java`, `python`, `sql`, `yaml`, `json`, `bash`, `js`, `http`, and `text` for pseudocode.
- `` `inline code` `` becomes a monospace chip.

Rules (a test enforces the first two):

1. Code with more than one statement, or any method/class definition, goes in a fenced block, one statement per line, 4-space indent. Never write run-on code like
   `List<Integer> k = new ArrayList<>(); m.put(k, "v"); k.add(2);` inside a sentence.
2. The prompt still ends with `?` or `.`: put the question sentence after the closing fence.
3. Short code in a sentence (identifiers, one expression, SQL keywords) goes in backticks. Plain-English options stay plain.
4. Separate paragraphs with a blank line (`\n\n`). In JSON the newlines are `\n`.

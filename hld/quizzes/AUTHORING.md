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

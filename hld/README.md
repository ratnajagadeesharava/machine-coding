# HLD Room

Practise **high-level system design interviews** on a whiteboard, with a real AI interviewer.

You pick a question, press **Start**, and sketch your design while **Aria** (powered by Claude) watches the board and the
chat. Ask her to clarify requirements, ask for a hint, or have her review your board. She stays quiet unless you talk to her
or she spots something worth raising. When time is up she writes a scored feedback report.

![HLD Room](docs/screenshot.png)

## Quick start

Requirements: **Node.js 20+** and an **Anthropic API key** (https://console.anthropic.com/).

```bash
cd hld
npm install
cp .env.example .env        # then put your key in .env:  ANTHROPIC_API_KEY=sk-ant-...
npm run check               # optional: two tiny real API calls to confirm the key and model work
npm start                   # open the URL it prints (http://127.0.0.1:3000)
```

No key yet? `npm run demo` starts the same app in **demo mode** with canned replies and no API calls, which is handy for
trying the whiteboard, timer and layout.

> **Your key stays on the server.** It is read from `hld/.env` (git-ignored) or the `ANTHROPIC_API_KEY` environment
> variable. The browser never sees it, and the server only listens on `127.0.0.1` unless you change `HLD_HOST`.

## Quizzes

The **Quizzes** tab (`/quiz.html`) is for checking what you know between interviews.

- **Scaling Reads** ships with the app: 29 questions (single choice, select-all, true/false) covering indexing, denormalisation,
  read replicas, sharding, caching, invalidation, CDNs, hot keys and stampedes, and interview judgement.
- **Modes:** full quiz, **Quick 10** (a random ten), and **Retry missed** (only what you got wrong last time). A toggle switches between
  feedback after every question and exam style with everything revealed at the end.
- **Every option explains itself** once you have answered, and a summary shows your score by concept, weakest first, with a review
  of each answer. Options are shuffled each attempt.
- **Explain more with Claude** opens a tutor on any question: it explains why the right answer is right and why your pick was
  tempting, and you can ask follow-ups.
- **Create a quiz with AI:** give a topic, or paste your own study notes, choose the length and level, and Claude writes a quiz
  (validated against the same quality rules). Generated quizzes are saved in your browser and can be deleted.

![Quiz question with explanations](docs/quiz.png)

Your scores and generated quizzes live in the browser (localStorage). To add a quiz to the bundled set see
[quizzes/AUTHORING.md](quizzes/AUTHORING.md).

## Using it

| Do this | What happens |
|---|---|
| Pick a question on the left | The prompt and assumptions appear. Each question has its own timer, board and chat, saved in your browser. |
| **Start** | The timer runs and Aria starts watching. Pause, resume or **End** at any time. |
| Draw | Boxes (services), ovals (clients/actors), cylinders (databases), arrows that stick to shapes, and free text. Double-click anything to rename it. |
| **Clarify requirements** | Aria answers from the interviewer's ground truth, so her answers stay consistent. |
| **Give me a hint** | One graduated hint, only as strong as needed. |
| **Review my board** | Strengths, the biggest gap, and a probing question about your actual components. |
| Ask anything in the chat | Free conversation, with your board and the clock attached. |
| **End** (or time runs out) | A feedback report: verdict, 0-5 scores for requirements, estimation, API, data model, architecture, scaling and trade-offs, plus strengths, gaps and next steps. |

Whiteboard shortcuts: `V` select, `R` box, `O` oval, `D` database, `A` arrow, `T` text, `E` eraser,
`Delete` removes the selection, `Ctrl/Cmd+Z` undoes, scroll to pan, `Ctrl/Cmd`+scroll to zoom, drag the background to pan.
Arrows snap to shapes and follow them when you move them; drag an arrow's end handle to reconnect it.

## How Aria sees your work

Aria does not receive a screenshot. Each request carries a **text description of the board** (components, what connects to
what, arrows with a loose end, components with no arrows, your notes) plus the chat and the time used, so she can talk about
your actual boxes by name. Quiet check-ins happen only while the timer runs: after you stop editing for a few seconds the
server asks the model whether anything is worth saying, and the model usually answers "no". Unprompted remarks are rate
limited (at most one every 75 seconds, ten per interview).

## Configuration

Set in `hld/.env` or the environment.

| Variable | Default | Meaning |
|---|---|---|
| `ANTHROPIC_API_KEY` | - | Your key. Required for the real AI. |
| `HLD_MODEL` | `claude-opus-5-5` | Model Aria uses. For lower cost try `claude-sonnet-5-5`. |
| `HLD_EFFORT` | `default` | `default` = low effort for chat and check-ins, high for the final report. Or `low`/`medium`/`high`/`xhigh`/`max` for everything, or `none` to send no effort setting (needed for older models that do not support it). |
| `PORT` | `3000` | Port to listen on. |
| `HLD_HOST` | `127.0.0.1` | Bind address. Only change it if you understand that anyone who can reach the port can spend your key. |
| `HLD_MOCK` | - | `1` = demo mode (same as `npm run demo`). |

A session makes a handful of small streaming requests plus one larger request for the report. Cost depends on the model you
choose; the question's interviewer notes are cached between requests.

## Project layout

```
hld/
  server/        index.js (HTTP + SSE API), ai.js (Claude calls), prompts.js (Aria's instructions), quiz.js (quiz loading, tutor and generator prompts), mock.js, env.js, check.js
  shared/        board.js (board model, text description for the AI), quiz.js (quiz validation and grading), used by browser and server
  public/        index.html + app.js (interview room), whiteboard.js (SVG editor), quiz.html + quiz.js (quizzes), styles.css, quiz.css
  questions/     questions.json - the interview question bank (26 questions)
  quizzes/       the bundled quizzes (JSON) and AUTHORING.md
  test/          unit and API tests, plus an end-to-end browser test
```

API (all JSON except the two streaming endpoints, which use Server-Sent Events): `GET /api/health`, `GET /api/questions`,
`POST /api/chat` (stream), `POST /api/watch`, `POST /api/evaluate`, `GET /api/quizzes`, `GET /api/quizzes/:id`,
`POST /api/quiz/explain` (stream), `POST /api/quiz/generate`.

## Adding or editing questions

Edit `questions/questions.json`. Each entry has:

- `id`, `title`, `category`, `difficulty` (`Easy`/`Medium`/`Hard`), `minutes`, `prompt`, `assumptions` - shown to the candidate.
- `clarifications` - `{topic, question, answer}` facts the interviewer uses to answer requirement questions consistently.
- `rubric` - `requirements`, `estimation`, `api`, `dataModel`, `architecture`, `scaling`, `tradeoffs`, `pitfalls`: what a strong answer covers.
- `followups` - deeper questions for later in the session.

`clarifications`, `rubric` and `followups` never leave the server. `npm test` validates every question's shape.

## Tests

```bash
npm test        # unit + API tests (board model, prompts, quiz model and bank quality rules, Claude request shapes with a fake client, HTTP/SSE server)
npm run e2e     # drives the real UI (interview room and quizzes) in headless Chromium against the demo-mode server (set HLD_CHROME if Chromium is not found)
npm run check   # live check of your API key and model (three tiny real requests; add "-- --quiz" to also generate a quiz)
```

## Troubleshooting

- **"Aria is not connected"** banner: `hld/.env` is missing or has no key. Restart the server after editing it.
- **"The Anthropic API key was rejected"**: the key is wrong, revoked, or from another workspace.
- **"The configured model was not found"** or a 400 mentioning effort: check `HLD_MODEL`; for older models set `HLD_EFFORT=none`.
- **Aria never speaks unprompted**: that is by design; ask her something or press a quick action. Use "Review my board" for feedback on demand.
- **Lost your board?** It is saved per question in the browser (localStorage). Clearing site data removes it.

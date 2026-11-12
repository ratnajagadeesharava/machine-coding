import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { validateQuiz } from '../shared/quiz.js';
import { loadQuizzes, quizSummary } from '../server/quiz.js';

const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'quizzes');
const quizzes = loadQuizzes(dir);

test('the bank is non-empty, valid, and has unique ids', () => {
  assert.ok(quizzes.length >= 1);
  assert.equal(new Set(quizzes.map((q) => q.id)).size, quizzes.length);
  for (const q of quizzes) assert.deepEqual(validateQuiz(q), [], q.id);
});

for (const quiz of quizzes) {
  test(`quiz "${quiz.id}" follows the quality rules`, () => {
    const singles = quiz.questions.filter((q) => q.type === 'single');
    // every option explains itself
    for (const q of quiz.questions) for (const o of q.options || []) assert.ok(o.why && o.why.length >= 15, `${q.id}/${o.id} needs a "why"`);
    // the correct answer must not be recognisable by length alone
    let longest = 0;
    for (const q of singles) {
      const lens = q.options.map((o) => o.text.length);
      const c = q.options.findIndex((o) => q.correct.includes(o.id));
      if (lens[c] === Math.max(...lens)) longest += 1;
      const others = lens.filter((_, i) => i !== c);
      const avg = others.reduce((a, b) => a + b, 0) / others.length;
      assert.ok(lens[c] <= avg * 1.8 && lens[c] >= avg * 0.45, `${q.id}: correct option length ${lens[c]} vs distractors ${Math.round(avg)}`);
    }
    assert.ok(!singles.length || longest / singles.length <= 0.4, `correct option is the longest in ${longest}/${singles.length} single-answer questions`);
    // variety
    assert.ok(new Set(quiz.questions.map((q) => q.type)).size >= 2, 'mix question types');
    assert.ok(new Set(quiz.questions.map((q) => q.difficulty)).size === 3, 'cover all three difficulties');
    assert.ok(new Set(quiz.questions.map((q) => q.concept)).size >= 5, 'cover several concepts');
    // true/false answers are balanced when there are several
    const tf = quiz.questions.filter((q) => q.type === 'truefalse');
    if (tf.length >= 2) {
      const trues = tf.filter((q) => q.options.find((o) => q.correct.includes(o.id)).text === 'True').length;
      assert.ok(trues >= 1 && trues < tf.length, 'true/false answers should not all be the same');
    }
    // prompts are real questions or instructions
    for (const q of quiz.questions) assert.match(q.prompt, /[?.]$/, `${q.id} prompt should end with ? or .`);
  });
}

for (const quiz of quizzes.filter((q) => q.id.startsWith('rm-'))) {
  test(`roadmap quiz "${quiz.id}" mixes the JEE-style question types`, () => {
    const types = new Set(quiz.questions.map((q) => q.type));
    assert.ok(types.size >= 4, `only ${types.size} question types`);
    assert.ok(types.has('code') && types.has('integer'), 'needs at least one code and one integer question');
    const rich = quiz.questions.filter((q) => !['single', 'multi', 'truefalse'].includes(q.type)).length;
    assert.ok(rich / quiz.questions.length >= 0.5, 'at least half of the questions should be integer/assertion/matrix/fill/code');
    assert.ok(quiz.questions.length >= 10, 'roadmap quizzes need at least 10 questions');
  });
}

test('the Scaling Reads quiz covers the whole pattern', () => {
  const quiz = quizzes.find((q) => q.id === 'scaling-reads');
  assert.ok(quiz && quiz.questions.length >= 25);
  const concepts = new Set(quiz.questions.map((q) => q.concept));
  for (const c of ['Indexing', 'Denormalisation', 'Read replicas', 'Sharding', 'Caching', 'Cache invalidation', 'CDN and edge caching', 'Hot keys and stampedes', 'Interview judgement']) {
    assert.ok(concepts.has(c), `missing concept ${c}`);
  }
});

test('quizSummary reports counts without leaking answers', () => {
  const s = quizSummary(quizzes[0]);
  assert.equal(s.questionCount, quizzes[0].questions.length);
  assert.ok(!('questions' in s));
});

test('loadQuizzes rejects an invalid quiz file with a readable message', async () => {
  const fs = await import('node:fs');
  const os = await import('node:os');
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'hld-quiz-'));
  fs.writeFileSync(path.join(tmp, 'bad.json'), JSON.stringify({ id: 'bad', title: 'Bad', questions: [{ id: 'x' }] }));
  assert.throws(() => loadQuizzes(tmp), /Invalid quiz bad\.json/);
  assert.deepEqual(loadQuizzes(path.join(tmp, 'missing')), []);
});

test('the PostgreSQL series is split into small sub-topic quizzes', () => {
  const pg = quizzes.filter((q) => q.series === 'postgres-system-design').sort((a, b) => a.seriesOrder - b.seriesOrder);
  assert.equal(pg.length, 8);
  assert.deepEqual(pg.map((q) => q.seriesOrder), [1, 2, 3, 4, 5, 6, 7, 8]);
  for (const q of pg) {
    assert.ok(q.questions.length >= 5 && q.questions.length <= 10, `${q.id} should stay small`);
    assert.equal(q.seriesTitle, pg[0].seriesTitle);
  }
  const summary = quizSummary(pg[0]);
  assert.equal(summary.series, 'postgres-system-design');
  assert.equal(summary.seriesOrder, 1);
});

test('the Redis series is split into small sub-topic quizzes', () => {
  const redis = quizzes.filter((q) => q.series === 'redis-system-design').sort((a, b) => a.seriesOrder - b.seriesOrder);
  assert.equal(redis.length, 8);
  assert.deepEqual(redis.map((q) => q.seriesOrder), [1, 2, 3, 4, 5, 6, 7, 8]);
  for (const q of redis) {
    assert.ok(q.questions.length >= 5 && q.questions.length <= 10, `${q.id} should stay small`);
    assert.ok(q.seriesDescription && q.seriesTitle);
  }
});

test('the Scaling Writes series is split into small sub-topic quizzes', () => {
  const sw = quizzes.filter((q) => q.series === 'scaling-writes-pattern').sort((a, b) => a.seriesOrder - b.seriesOrder);
  assert.equal(sw.length, 8);
  assert.deepEqual(sw.map((q) => q.seriesOrder), [1, 2, 3, 4, 5, 6, 7, 8]);
  for (const q of sw) assert.ok(q.questions.length >= 5 && q.questions.length <= 10, `${q.id} should stay small`);
});

test('the Kafka series is split into small sub-topic quizzes', () => {
  const k = quizzes.filter((q) => q.series === 'kafka-system-design').sort((a, b) => a.seriesOrder - b.seriesOrder);
  assert.equal(k.length, 9);
  assert.deepEqual(k.map((q) => q.seriesOrder), [1, 2, 3, 4, 5, 6, 7, 8, 9]);
  for (const q of k) assert.ok(q.questions.length >= 5 && q.questions.length <= 10, `${q.id} should stay small`);
});

// ---------- Java, Python and language-independent quizzes are never mixed ----------
// A quiz declares `language`: java, python or neutral. Java quizzes never mention Python things, Python quizzes never mention Java things, and neutral
// quizzes mention neither (SQL, YAML, JSON, shell, regex, pseudocode and plain explanations are fine everywhere).
const JAVA_TERMS = /\b(Java|JVM|JDK|JRE|JIT|Spring|Hibernate|JPA|JDBC|JUnit|Mockito|Maven|Gradle|Lombok|Hikari(CP)?|Resilience4j|Micrometer|Logback|SLF4J|javac|Flyway|Liquibase|Tomcat|Jackson|Kotlin|LangChain4j|Quarkus|Micronaut)\b|\bpublic\s+(static\s+)?(class|void|interface)\b|@Override\b/;
const PY_TERMS = /\b(Python|CPython|FastAPI|Pydantic|SQLAlchemy|Alembic|asyncio|pytest|Django|Flask|Starlette|Uvicorn|Celery|tenacity|pybreaker|aiobreaker|asyncpg|psycopg|httpx|pandas|numpy|py-spy|cProfile|PyJWT|PyMongo|Motor|slowapi|APScheduler|LangChain|LangGraph|Langfuse|Ragas|DeepEval|sqlglot|dataclass(es)?|PEP \d+|GIL|pip|venv|Poetry|mypy|TypeVar|OrderedDict|defaultdict|heapq|bisect|itertools|functools|contextvars?)\b|\bdef\s+\w+\(|\bself\b\s*[,)]|\bTrue\b.*\bNone\b/;
const SKIP_KEYS = new Set(['source', 'seriesDescription', 'seriesTitle', 'topic']);
function textOf(node, out = []) {
  if (typeof node === 'string') out.push(node);
  else if (Array.isArray(node)) node.forEach((n) => textOf(n, out));
  else if (node && typeof node === 'object') for (const [k, v] of Object.entries(node)) if (!SKIP_KEYS.has(k)) textOf(v, out);
  return out;
}
const LANGUAGE_REQUIRED = true;
for (const quiz of quizzes.filter((q) => q.id.startsWith('rm-'))) {
  test(`roadmap quiz "${quiz.id}" keeps its language separate`, () => {
    if (!quiz.language) { assert.ok(!LANGUAGE_REQUIRED, 'declare "language": java, python or neutral'); return; }
    assert.ok(quiz.id.endsWith(`-${quiz.language === 'neutral' ? 'core' : quiz.language}`), `id must end with -${quiz.language === 'neutral' ? 'core' : quiz.language}`);
    assert.ok(quiz.series?.startsWith(`${quiz.language === 'neutral' ? 'core' : quiz.language}-`), `series "${quiz.series}" must start with ${quiz.language === 'neutral' ? 'core' : quiz.language}- so a series never mixes languages`);
    const text = textOf(quiz.questions).join('\n');
    if (quiz.language !== 'java') { const m = JAVA_TERMS.exec(text); assert.ok(!m, `${quiz.language} quiz mentions Java: "${m?.[0]}"`); }
    if (quiz.language !== 'python') { const m = PY_TERMS.exec(text); assert.ok(!m, `${quiz.language} quiz mentions Python: "${m?.[0]}"`); }
  });
}

// ---------- every quiz belongs to a subject card on the Quizzes page ----------
import { SUBJECTS, subjectIdForTopic } from '../shared/subjects.js';
test('every quiz topic maps to a subject, and a subject never mixes Java and Python', () => {
  const bySubject = new Map();
  for (const q of quizzes) {
    const topicId = q.series || `quiz-${q.id}`;
    const sid = subjectIdForTopic(topicId);
    assert.notEqual(sid, 'other', `${topicId} (${q.id}) has no subject: add it to shared/subjects.js`);
    bySubject.set(sid, [...(bySubject.get(sid) || []), q.language || 'none']);
  }
  assert.ok(SUBJECTS.every((s) => s.title && s.description && s.icon));
  assert.ok(!bySubject.get('java')?.some((l) => l !== 'java'), 'the Java subject holds only Java quizzes');
  assert.ok(!bySubject.get('python')?.some((l) => l !== 'python'), 'the Python subject holds only Python quizzes');
});

// ---------- code in a prompt is written as a fenced snippet (```lang ... ```) or `inline code`, never as run-on text ----------
const stripCode = (t) => t.replace(/```[\s\S]*?```/g, ' ').replace(/`[^`\n]*`/g, ' ');
const CODE_LEAK = /;\s*\S|[{}]|\bdef\s+\w+\(|=>|\bnew\s+[A-Z]\w*[<(]|\bSELECT\b[^.?]*\bFROM\b|\b[a-z]\w*\.[a-z]\w*\([^)]*\)\s*(;|\.|\+)|\w+\s*=\s*\w+\(|\bimport\s+[a-z]\w*(\.\w+)*\b|\bfor\s+\w+\s+in\s+\w|\bclass\s+[A-Z]\w*\s*[({:]/;
for (const quiz of quizzes) {
  test(`quiz "${quiz.id}" formats code in prompts`, () => {
    for (const q of quiz.questions) {
      const prose = stripCode([q.prompt, q.assertion, q.reason].filter(Boolean).join('\n'));
      const m = CODE_LEAK.exec(prose);
      assert.ok(!m, `${q.id}: put the code in a \`\`\`lang fenced block or \`inline code\`: ...${prose.slice(Math.max(0, m?.index - 20), (m?.index ?? 0) + 40).replace(/\n/g, ' ')}...`);
      assert.equal((q.prompt.match(/```/g) || []).length % 2, 0, `${q.id}: unbalanced code fence`);
    }
  });
}

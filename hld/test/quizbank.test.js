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
    for (const q of quiz.questions) for (const o of q.options) assert.ok(o.why && o.why.length >= 15, `${q.id}/${o.id} needs a "why"`);
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
    assert.ok(longest / singles.length <= 0.4, `correct option is the longest in ${longest}/${singles.length} single-answer questions`);
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

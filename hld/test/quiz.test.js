import assert from 'node:assert/strict';
import test from 'node:test';
import {
  isCorrect, numberFor, mulberry32, pickQuestions, presentQuestion, scoreAttempt, shuffle, validateQuestion, validateQuiz,
} from '../shared/quiz.js';

const opt = (id, text) => ({ id, text, why: `why ${id}` });
const good = (over = {}) => ({
  id: 'q-one', concept: 'Indexing', type: 'single', difficulty: 2, prompt: 'Which index type suits equality lookups best?',
  options: [opt('a', 'Hash index'), opt('b', 'Bitmap index'), opt('c', 'Spatial index')], correct: ['a'], explanation: 'Hash indexes are built for exact matches.', ...over,
});
const quiz = (qs) => ({ id: 'z', title: 'Z', description: 'd', questions: qs });

test('a well-formed question passes validation', () => {
  assert.deepEqual(validateQuestion(good()), []);
  assert.deepEqual(validateQuestion(good({ type: 'truefalse', options: [opt('a', 'True'), opt('b', 'False')], correct: ['b'] })), []);
  assert.deepEqual(validateQuestion(good({ type: 'multi', options: [opt('a', 'One'), opt('b', 'Two'), opt('c', 'Three')], correct: ['a', 'b'] })), []);
});

test('validation catches structural mistakes', () => {
  const cases = [
    [{ id: 'Bad Id' }, /id must be/],
    [{ type: 'essay' }, /type must be/],
    [{ difficulty: 5 }, /difficulty/],
    [{ concept: '' }, /concept/],
    [{ prompt: 'short' }, /prompt/],
    [{ explanation: '' }, /explanation/],
    [{ options: [opt('a', 'Only'), opt('b', 'Two')] }, /3-6 options/],
    [{ options: [opt('a', 'Same'), opt('b', 'same'), opt('c', 'Other')] }, /duplicate option/],
    [{ options: [opt('a', 'X'), opt('a', 'Y'), opt('c', 'Z')] }, /ids must be unique/],
    [{ options: [opt('a', 'X'), opt('b', 'Y'), opt('c', 'All of the above')] }, /all\/none of the above/],
    [{ correct: [] }, /at least one/],
    [{ correct: ['z'] }, /not an option/],
    [{ correct: ['a', 'b'] }, /exactly one correct/],
    [{ type: 'truefalse' }, /exactly 2 options/],
    [{ type: 'multi', correct: ['a'] }, /2\+ correct/],
    [{ type: 'multi', correct: ['a', 'b', 'c'] }, /2\+ correct/],
  ];
  for (const [over, re] of cases) {
    const errs = validateQuestion(good(over));
    assert.ok(errs.some((e) => re.test(e)), `${JSON.stringify(over)} -> ${errs.join(' | ')}`);
  }
  assert.ok(validateQuestion(null)[0].includes('not an object'));
});

test('validateQuiz checks counts, duplicates and nested questions', () => {
  assert.deepEqual(validateQuiz(quiz([good()])), []);
  assert.ok(validateQuiz(quiz([])).some((e) => /needs 1-/.test(e)));
  assert.ok(validateQuiz(quiz([good(), good()])).some((e) => /duplicate id/.test(e)));
  assert.ok(validateQuiz({ ...quiz([good()]), id: 'NO' }).some((e) => /id must be/.test(e)));
  assert.ok(validateQuiz({ ...quiz([good()]), title: '' }).some((e) => /title/.test(e)));
  assert.deepEqual(validateQuiz({ ...quiz([good()]), description: undefined }), [], 'description is optional');
  assert.ok(validateQuiz({ ...quiz([good()]), description: 'x'.repeat(401) }).some((e) => /description/.test(e)));
  assert.ok(validateQuiz(quiz([good({ prompt: 'x' })])).some((e) => /question 1/.test(e)));
});

test('isCorrect needs the exact set, in any order, ignoring duplicates', () => {
  const m = good({ type: 'multi', options: [opt('a', 'A1'), opt('b', 'B1'), opt('c', 'C1'), opt('d', 'D1')], correct: ['a', 'c'] });
  assert.equal(isCorrect(m, ['c', 'a']), true);
  assert.equal(isCorrect(m, ['a', 'c', 'c']), true);
  assert.equal(isCorrect(m, ['a']), false);
  assert.equal(isCorrect(m, ['a', 'c', 'd']), false);
  assert.equal(isCorrect(m, []), false);
  assert.equal(isCorrect(m, undefined), false);
});

test('scoreAttempt totals, groups by concept (weakest first) and lists the missed ids', () => {
  const qs = [
    good({ id: 'q1', concept: 'Indexing' }), good({ id: 'q2', concept: 'Indexing' }),
    good({ id: 'q3', concept: 'Caching' }), good({ id: 'q4', concept: 'Caching' }), good({ id: 'q5', concept: 'Sharding' }),
  ];
  const r = scoreAttempt(quiz(qs), { q1: ['a'], q2: ['b'], q3: ['b'], q4: ['c'], q5: ['a'] });
  assert.equal(r.score, 2);
  assert.equal(r.total, 5);
  assert.equal(r.percent, 40);
  assert.deepEqual(r.missed, ['q2', 'q3', 'q4']);
  assert.deepEqual(r.byConcept.map((c) => [c.concept, c.correct, c.total]), [['Caching', 0, 2], ['Indexing', 1, 2], ['Sharding', 1, 1]]);
  const none = scoreAttempt(quiz(qs), {});
  assert.equal(none.score, 0);
  assert.equal(scoreAttempt(quiz([]), {}).percent, 0);
});

test('shuffle is deterministic per seed, keeps every item, and does not mutate its input', () => {
  const list = [1, 2, 3, 4, 5, 6, 7, 8];
  const a = shuffle(list, mulberry32(42));
  const b = shuffle(list, mulberry32(42));
  assert.deepEqual(a, b);
  assert.deepEqual([...a].sort(), list);
  assert.deepEqual(list, [1, 2, 3, 4, 5, 6, 7, 8]);
  assert.notDeepEqual(shuffle(list, mulberry32(1)), shuffle(list, mulberry32(2)));
});

test('presentQuestion shuffles options but never true/false, and keeps ids so grading still works', () => {
  const q = good({ options: [opt('a', 'A1'), opt('b', 'B1'), opt('c', 'C1'), opt('d', 'D1'), opt('e', 'E1')] });
  let moved = false;
  for (let seed = 1; seed < 20; seed++) {
    const p = presentQuestion(q, mulberry32(seed));
    assert.deepEqual(p.options.map((o) => o.id).sort(), ['a', 'b', 'c', 'd', 'e']);
    if (p.options[0].id !== 'a') moved = true;
    assert.equal(isCorrect(p, ['a']), true);
  }
  assert.ok(moved, 'options should move for some seeds');
  const tf = good({ type: 'truefalse', options: [opt('a', 'True'), opt('b', 'False')], correct: ['a'] });
  for (let seed = 1; seed < 10; seed++) assert.deepEqual(presentQuestion(tf, mulberry32(seed)).options.map((o) => o.id), ['a', 'b']);
});

test('pickQuestions returns n unique questions in original order, or everything when n is large', () => {
  const qs = Array.from({ length: 12 }, (_, i) => good({ id: `q${i + 1}` }));
  const picked = pickQuestions(quiz(qs), 5, mulberry32(7));
  assert.equal(picked.length, 5);
  assert.equal(new Set(picked.map((q) => q.id)).size, 5);
  const order = picked.map((q) => Number(q.id.slice(1)));
  assert.deepEqual(order, [...order].sort((x, y) => x - y));
  assert.equal(pickQuestions(quiz(qs), 50, mulberry32(7)).length, 12);
});

test('numberFor labels positions with numbers', () => {
  assert.equal(numberFor(0), '1');
  assert.equal(numberFor(3), '4');
});

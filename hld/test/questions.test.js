import assert from 'node:assert/strict';
import test from 'node:test';
import { loadQuestions, publicQuestion } from '../server/index.js';

const questions = loadQuestions();
const RUBRIC_KEYS = ['requirements', 'estimation', 'api', 'dataModel', 'architecture', 'scaling', 'tradeoffs', 'pitfalls'];
const nonEmpty = (s) => typeof s === 'string' && s.trim().length > 0;

test('the bank has unique ids and the expected size', () => {
  assert.equal(questions.length, 26);
  assert.equal(new Set(questions.map((q) => q.id)).size, questions.length);
});

for (const q of questions) {
  test(`question "${q.id}" is complete`, () => {
    assert.ok(/^[a-z0-9-]+$/.test(q.id));
    assert.ok(nonEmpty(q.title) && nonEmpty(q.prompt));
    assert.ok(['Easy', 'Medium', 'Hard'].includes(q.difficulty));
    assert.ok(Number.isInteger(q.minutes) && q.minutes >= 20 && q.minutes <= 60);
    assert.ok(nonEmpty(q.category));
    assert.ok(q.assumptions.length >= 3 && q.assumptions.every(nonEmpty));
    assert.ok(q.clarifications.length >= 6);
    for (const c of q.clarifications) assert.ok(nonEmpty(c.topic) && nonEmpty(c.question) && nonEmpty(c.answer));
    for (const k of RUBRIC_KEYS) assert.ok(Array.isArray(q.rubric[k]) && q.rubric[k].length >= 3 && q.rubric[k].every(nonEmpty), `rubric.${k}`);
    assert.ok(q.followups.length >= 3);
  });
}

test('publicQuestion never leaks the interviewer-only fields', () => {
  const pub = publicQuestion(questions[0]);
  assert.deepEqual(Object.keys(pub).sort(), ['assumptions', 'category', 'difficulty', 'id', 'minutes', 'prompt', 'title']);
});

import assert from 'node:assert/strict';
import test from 'node:test';
import { loadQuestions, publicQuestion } from '../server/index.js';
import { cleanInterviewQuestion } from '../server/question.js';

const questions = loadQuestions();
const nonEmpty = (s) => typeof s === 'string' && s.trim().length > 0;
const KEYS = ['assumptions', 'category', 'difficulty', 'id', 'minutes', 'prompt', 'title'];

test('the bank has unique ids and the expected size', () => {
  assert.equal(questions.length, 26);
  assert.equal(new Set(questions.map((q) => q.id)).size, questions.length);
});

for (const q of questions) {
  test(`question "${q.id}" is complete`, () => {
    assert.deepEqual(Object.keys(q).sort(), KEYS, 'only the candidate-facing fields are stored');
    assert.ok(/^[a-z0-9-]+$/.test(q.id));
    assert.ok(nonEmpty(q.title) && nonEmpty(q.prompt));
    assert.ok(['Easy', 'Medium', 'Hard'].includes(q.difficulty));
    assert.ok(Number.isInteger(q.minutes) && q.minutes >= 20 && q.minutes <= 60);
    assert.ok(nonEmpty(q.category));
    assert.ok(q.assumptions.length >= 3 && q.assumptions.every(nonEmpty));
  });
  test(`question "${q.id}" is accepted by the server-side validator`, () => {
    const { question, error } = cleanInterviewQuestion(q);
    assert.equal(error, undefined);
    assert.equal(question.title, q.title);
    assert.equal(question.prompt, q.prompt);
    assert.deepEqual(question.assumptions, q.assumptions);
  });
}

test('publicQuestion keeps only the candidate-facing fields', () => {
  const pub = publicQuestion({ ...questions[0], rubric: { a: 1 }, clarifications: [], followups: [] });
  assert.deepEqual(Object.keys(pub).sort(), KEYS);
});

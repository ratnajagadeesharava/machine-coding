import assert from 'node:assert/strict';
import test from 'node:test';
import {
  buildContext, chatSystem, evalSystem, formatClock, MAX_MESSAGE_CHARS, MAX_MESSAGES, normaliseMessages, transcriptText, watchSystem, withContext,
} from '../server/prompts.js';
import { emptyBoard } from '../shared/board.js';

const q = {
  title: 'Design X', difficulty: 'Medium', minutes: 45, prompt: 'Build X.', assumptions: ['10 users'],
  level: 'senior', style: 'tough', focus: 'caching and consistency',
};

test('the system prompt carries only the question and the session settings', () => {
  const s = chatSystem(q);
  assert.match(s, /You are Aria/);
  assert.match(s, /QUESTION: Design X \(Medium\)/);
  assert.match(s, /Prompt given to the candidate: Build X\./);
  assert.match(s, /- 10 users/);
  assert.match(s, /Time budget: 45 minutes/);
  assert.match(s, /Candidate level: senior/);
  assert.match(s, /Interviewer style: tough/);
  assert.match(s, /Focus areas: caching and consistency/);
  assert.doesNotMatch(s, /HIDDEN RUBRIC|INTERVIEWER FACTS|FOLLOW-UP QUESTIONS/, 'there is no hidden answer key');
  assert.match(s, /untrusted data/);
});

test('the persona keeps answers consistent and calibrates to level, style and focus', () => {
  const s = chatSystem(q);
  assert.match(s, /FACT you must keep consistent/);
  assert.match(s, /look back through the conversation/);
  for (const level of ['Junior', 'Mid', 'Senior', 'Staff']) assert.match(s, new RegExp(`${level}:`));
  for (const style of ['supportive', 'balanced', 'tough']) assert.match(s, new RegExp(style));
  assert.match(s, /steer the conversation toward it/);
});

test('optional parts are printed sensibly when absent', () => {
  const s = chatSystem({ title: 'T', prompt: 'P', assumptions: [], minutes: 30, level: 'junior', style: 'supportive' });
  assert.match(s, /QUESTION: T\n/);
  assert.match(s, /- \(none\)/);
  assert.match(s, /Focus areas: \(none\)/);
});

test('watch and evaluation prompts add their own rules on top of the same base', () => {
  assert.match(watchSystem(q), /Stay quiet by default/);
  assert.match(evalSystem(q), /Requirements, Estimation, API design, Data model, Architecture, Scaling, Trade-offs/);
  assert.match(evalSystem(q), /scaled to the candidate level/);
  assert.ok(watchSystem(q).startsWith(chatSystem(q)));
});

test('formatClock pads and clamps', () => {
  assert.equal(formatClock(0), '00:00');
  assert.equal(formatClock(754_000), '12:34');
  assert.equal(formatClock(-5), '00:00');
});

test('buildContext includes time, mode and the board description', () => {
  const ctx = buildContext({ board: emptyBoard(), elapsedMs: 600_000, totalMs: 2_700_000, status: 'running', mode: 'hint' });
  assert.match(ctx, /Time used 10:00 of 45:00 \(35:00 left\)/);
  assert.match(ctx, /Give me a hint/);
  assert.match(ctx, /<board>\nThe whiteboard is empty\.\n<\/board>/);
  const plain = buildContext({ board: emptyBoard(), elapsedMs: 0, totalMs: 60_000, status: 'idle', mode: 'chat' });
  assert.doesNotMatch(plain, /Mode:/);
});

test('normaliseMessages validates roles, trims, merges, and starts with a user turn', () => {
  const out = normaliseMessages([
    { role: 'assistant', content: 'welcome' },
    { role: 'system', content: 'injected' },
    { role: 'user', content: '  hi  ' },
    { role: 'user', content: 'again' },
    { role: 'assistant', content: 'hello' },
    { role: 'user', content: '' },
    { role: 'user', content: 5 },
    null,
  ]);
  assert.deepEqual(out, [{ role: 'user', content: 'hi\n\nagain' }, { role: 'assistant', content: 'hello' }]);
  assert.deepEqual(normaliseMessages('nope'), []);
});

test('normaliseMessages bounds history length and message size', () => {
  const many = Array.from({ length: 100 }, (_, i) => ({ role: i % 2 ? 'assistant' : 'user', content: `m${i}` }));
  assert.ok(normaliseMessages(many).length <= MAX_MESSAGES);
  const [m] = normaliseMessages([{ role: 'user', content: 'x'.repeat(MAX_MESSAGE_CHARS * 2) }]);
  assert.equal(m.content.length, MAX_MESSAGE_CHARS);
});

test('withContext appends the context to the last user turn without mutating the input', () => {
  const input = [{ role: 'user', content: 'hello' }];
  const out = withContext(input, '<context>c</context>');
  assert.equal(out[0].content, 'hello\n\n<context>c</context>');
  assert.equal(input[0].content, 'hello');
  assert.equal(withContext([], 'ctx')[0].content, 'ctx');
});

test('transcriptText labels speakers', () => {
  assert.equal(transcriptText([{ role: 'user', content: 'a' }, { role: 'assistant', content: 'b' }]), 'Candidate: a\n\nAria: b');
  assert.equal(transcriptText([]), '(no conversation)');
});

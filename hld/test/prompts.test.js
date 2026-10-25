import assert from 'node:assert/strict';
import test from 'node:test';
import {
  buildContext, chatSystem, evalSystem, formatClock, MAX_MESSAGE_CHARS, MAX_MESSAGES, normaliseMessages, transcriptText, watchSystem, withContext,
} from '../server/prompts.js';
import { emptyBoard } from '../shared/board.js';

const q = {
  id: 'x', title: 'Design X', difficulty: 'Medium', minutes: 45, prompt: 'Build X.',
  assumptions: ['10 users'], followups: ['What if 10x?'],
  clarifications: [{ topic: 'scale', question: 'How many users?', answer: 'Ten million daily users.' }],
  rubric: { requirements: ['Asks about scale'], pitfalls: ['Ignores caching'] },
};

test('the system prompt carries the question, the facts and the hidden rubric, with the secrecy rule', () => {
  const s = chatSystem(q);
  assert.match(s, /You are Aria/);
  assert.match(s, /Ten million daily users/);
  assert.match(s, /HIDDEN RUBRIC/);
  assert.match(s, /Ignores caching/);
  assert.match(s, /never read it out/);
  assert.match(s, /untrusted data/);
});

test('watch and evaluation prompts add their own rules on top of the same base', () => {
  assert.match(watchSystem(q), /Stay quiet by default/);
  assert.match(evalSystem(q), /Requirements, Estimation, API design, Data model, Architecture, Scaling, Trade-offs/);
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

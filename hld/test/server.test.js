import assert from 'node:assert/strict';
import http from 'node:http';
import test from 'node:test';
import Anthropic from '@anthropic-ai/sdk';
import { createAi } from '../server/ai.js';
import { createServer, loadQuestions } from '../server/index.js';
import { createMockClient } from '../server/mock.js';

const questions = loadQuestions();
const board = { nextId: 3, items: [{ id: 1, type: 'rect', x: 0, y: 0, w: 100, h: 60, text: 'API' }, { id: 2, type: 'database', x: 200, y: 0, w: 100, h: 80, text: 'DB' }] };
const mockAi = () => createAi({ client: createMockClient({ delayMs: 0 }), model: 'mock', effort: 'none' });

async function start(opts = {}) {
  const server = createServer({ ai: mockAi(), questions, mode: 'mock', model: 'mock', ...opts });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}`;
  return { base, port: server.address().port, close: () => new Promise((r) => { server.closeAllConnections?.(); server.close(r); }) };
}

const post = (base, path, body, raw) => fetch(base + path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: raw ?? JSON.stringify(body) });

async function readSse(res) {
  const events = [];
  const text = await res.text();
  for (const block of text.split('\n\n').filter(Boolean)) {
    events.push({ event: /^event: (.+)$/m.exec(block)[1], data: JSON.parse(/^data: (.+)$/m.exec(block)[1]) });
  }
  return events;
}

const chatBody = (extra = {}) => ({
  questionId: questions[0].id, board, elapsedMs: 5000, status: 'running', mode: 'chat',
  messages: [{ role: 'assistant', content: 'hi' }, { role: 'user', content: 'Can I ask about scale?' }], ...extra,
});

test('health, question list and static files', async () => {
  const s = await start();
  try {
    const health = await (await fetch(s.base + '/api/health')).json();
    assert.deepEqual(health, { ok: true, ai: 'mock', model: 'mock' });
    const list = await (await fetch(s.base + '/api/questions')).json();
    assert.equal(list.length, 26);
    assert.ok(list.every((q) => !('rubric' in q) && !('clarifications' in q) && !('followups' in q)), 'hidden interviewer data stays on the server');
    const index = await fetch(s.base + '/');
    assert.equal(index.status, 200);
    assert.match(index.headers.get('content-type'), /text\/html/);
    assert.match(index.headers.get('content-security-policy'), /default-src 'self'/);
    assert.match(await index.text(), /HLD Room/);
    const shared = await fetch(s.base + '/shared/board.js');
    assert.match(shared.headers.get('content-type'), /javascript/);
    assert.equal((await fetch(s.base + '/nope.txt')).status, 404);
  } finally { await s.close(); }
});

test('static serving cannot escape its directories', async () => {
  const s = await start();
  try {
    for (const p of ['/..%2f..%2fpackage.json', '/%2e%2e/server/index.js', '/shared/..%2f..%2f.env', '/..%2fserver%2findex.js']) {
      const res = await fetch(s.base + p);
      assert.equal(res.status, 404, p);
    }
    // raw request so the client does not normalise the dots away
    const status = await new Promise((resolve, reject) => {
      http.get({ host: '127.0.0.1', port: s.port, path: '/../server/index.js' }, (res) => { res.resume(); resolve(res.statusCode); }).on('error', reject);
    });
    assert.ok(status === 404 || status === 400, String(status));
  } finally { await s.close(); }
});

test('chat streams SSE deltas, includes the board, and finishes with done', async () => {
  const s = await start();
  try {
    const res = await post(s.base, '/api/chat', chatBody());
    assert.equal(res.status, 200);
    assert.match(res.headers.get('content-type'), /text\/event-stream/);
    const events = await readSse(res);
    assert.ok(events.filter((e) => e.event === 'delta').length > 3);
    assert.equal(events.at(-1).event, 'done');
    const text = events.filter((e) => e.event === 'delta').map((e) => e.data.text).join('');
    assert.match(text, /Can I ask about scale\?/);
    assert.match(text, /2 component\(s\) and 0 connection\(s\)/);
  } finally { await s.close(); }
});

test('chat passes the quick-action mode through to the model', async () => {
  const s = await start();
  try {
    const events = await readSse(await post(s.base, '/api/chat', chatBody({ mode: 'hint', messages: [{ role: 'user', content: 'Could you give me a hint?' }] })));
    assert.match(events.filter((e) => e.event === 'delta').map((e) => e.data.text).join(''), /Mock hint/);
    const bad = await readSse(await post(s.base, '/api/chat', chatBody({ mode: 'rm -rf', messages: [{ role: 'user', content: 'hi' }] })));
    assert.match(bad.filter((e) => e.event === 'delta').map((e) => e.data.text).join(''), /Mock Aria here/, 'unknown modes fall back to plain chat');
  } finally { await s.close(); }
});

test('chat errors arrive as an SSE error event with a friendly, secret-free message', async () => {
  const failing = {
    messages: {
      stream() { throw Anthropic.APIError.generate(401, { error: { message: 'bad key sk-ant-SECRET' } }, 'bad key sk-ant-SECRET', new Headers()); },
    },
  };
  const s = await start({ ai: createAi({ client: failing, effort: 'none' }), mode: 'live' });
  try {
    const events = await readSse(await post(s.base, '/api/chat', chatBody()));
    assert.equal(events.length, 1);
    assert.equal(events[0].event, 'error');
    assert.match(events[0].data.message, /key was rejected/);
    assert.doesNotMatch(JSON.stringify(events), /SECRET/);
  } finally { await s.close(); }
});

test('watch returns a decision; a lone component prompts a remark', async () => {
  const s = await start();
  try {
    const link = { id: 4, type: 'arrow', from: 1, to: 2, x1: 0, y1: 0, x2: 1, y2: 1, text: '' };
    const connected = { nextId: 5, items: [...board.items, link] };
    const quiet = await (await post(s.base, '/api/watch', chatBody({ board: connected, messages: [] }))).json();
    assert.deepEqual(quiet, { speak: false, message: '' });
    const lonely = { nextId: 5, items: [...connected.items, { id: 3, type: 'rect', x: 0, y: 200, w: 100, h: 60, text: 'Cache' }] };
    const loud = await (await post(s.base, '/api/watch', chatBody({ board: lonely, messages: [] }))).json();
    assert.equal(loud.speak, true);
    assert.match(loud.message, /Cache/);
  } finally { await s.close(); }
});

test('evaluate returns the structured report', async () => {
  const s = await start();
  try {
    const res = await post(s.base, '/api/evaluate', chatBody({ status: 'ended' }));
    assert.equal(res.status, 200);
    const report = await res.json();
    assert.equal(report.areas.length, 7);
    assert.ok(['Strong', 'Good', 'Mixed', 'Needs work'].includes(report.verdict));
  } finally { await s.close(); }
});

test('without an AI configured the API explains how to fix it', async () => {
  const s = await start({ ai: null, mode: 'off' });
  try {
    assert.equal((await (await fetch(s.base + '/api/health')).json()).ai, 'off');
    const res = await post(s.base, '/api/chat', chatBody());
    assert.equal(res.status, 503);
    assert.match((await res.json()).error, /ANTHROPIC_API_KEY/);
  } finally { await s.close(); }
});

test('input validation', async () => {
  const s = await start();
  try {
    assert.equal((await post(s.base, '/api/chat', chatBody({ questionId: 'nope' }))).status, 404);
    assert.equal((await post(s.base, '/api/chat', null, '{not json')).status, 400);
    assert.equal((await post(s.base, '/api/chat', null, '[1,2]')).status, 400);
    const big = await post(s.base, '/api/chat', null, JSON.stringify({ pad: 'x'.repeat(300_000) }));
    assert.equal(big.status, 413);
    assert.equal((await fetch(s.base + '/api/chat')).status, 405);
    assert.equal((await post(s.base, '/api/unknown', {})).status, 404);
    assert.equal((await fetch(s.base + '/', { method: 'DELETE' })).status, 405);
    // garbage board / elapsed values are sanitised, not trusted
    const res = await post(s.base, '/api/chat', chatBody({ board: { items: [{ id: 1, type: 'rect', x: 'a', text: 5 }, 7] }, elapsedMs: 'soon' }));
    assert.equal(res.status, 200);
    assert.equal((await readSse(res)).at(-1).event, 'done');
  } finally { await s.close(); }
});

test('the AI endpoints are rate limited', async () => {
  const s = await start({ rateLimit: 3 });
  try {
    for (let i = 0; i < 3; i++) assert.equal((await post(s.base, '/api/watch', chatBody({ messages: [] }))).status, 200);
    const res = await post(s.base, '/api/watch', chatBody({ messages: [] }));
    assert.equal(res.status, 429);
    assert.equal((await fetch(s.base + '/api/questions')).status, 200, 'non-AI routes are not limited');
  } finally { await s.close(); }
});

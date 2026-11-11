import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { currentBranch } from '../server/branch.js';
import { createServer } from '../server/index.js';
import { openProgress } from '../server/progress.js';

const attempt = (over = {}) => ({ quizId: 'q1', title: 'Quiz one', at: 1000, seconds: 30, mode: 'full', percent: 80, score: 8, total: 10, weak: ['Caching'], ...over });
const stats = { q1: { best: 80, count: 2, last: { at: 1000, percent: 80 } } };
const tmp = () => path.join(fs.mkdtempSync(path.join(os.tmpdir(), 'hld-progress-')), 'data', 'progress.db');

test('a fresh database is empty and has the schema', () => {
  const db = openProgress(':memory:');
  const r = db.read();
  assert.equal(r.empty, true);
  assert.deepEqual(r.history, []);
  assert.deepEqual(r.attempts, {});
  assert.deepEqual(r.custom, []);
  db.close();
});

test('saved progress survives closing and reopening the file (it travels with the code)', () => {
  const file = tmp();
  let db = openProgress(file);
  db.save('history', [attempt(), attempt({ at: 2000, percent: 50 })]);
  db.save('attempts', stats);
  db.save('custom', [{ id: 'mine', title: 'My quiz' }]);
  db.save('run', { quizId: 'q1', i: 3 });
  db.save('instant', false);
  db.close();
  assert.ok(fs.existsSync(file));
  assert.equal(fs.readdirSync(path.dirname(file)).length, 1, 'a single file, no -wal/-shm/-journal left behind');

  db = openProgress(file);
  const r = db.read();
  assert.deepEqual(r.history.map((h) => h.at), [2000, 1000]);
  assert.equal(r.history[1].weak[0], 'Caching');
  assert.deepEqual(r.attempts, stats);
  assert.equal(r.custom[0].id, 'mine');
  assert.deepEqual(r.run, { quizId: 'q1', i: 3 });
  assert.equal(r.instant, false);
  assert.equal(r.empty, false);
  db.close();
});

test('invalid rows are dropped and unknown keys refused', () => {
  const db = openProgress(':memory:');
  db.save('history', [attempt(), { title: 'no percent', at: 5 }, null, 'x']);
  assert.equal(db.read().history.length, 1);
  assert.throws(() => db.save('history', 'nope'), TypeError);
  assert.throws(() => db.save('password', 1), RangeError);
  db.save('history', [attempt({ percent: 250 })]);
  assert.equal(db.read().history[0].percent, 100);
  db.close();
});

test('history keeps the newest 100 attempts', () => {
  const db = openProgress(':memory:');
  db.save('history', Array.from({ length: 130 }, (_, i) => attempt({ at: i + 1, title: `t${i}` })));
  const h = db.read().history;
  assert.equal(h.length, 100);
  assert.equal(h[0].at, 100);
  db.close();
});

test('sync merges browser progress into what the database already holds', () => {
  const db = openProgress(':memory:');
  db.save('history', [attempt({ at: 3000, title: 'Existing' })]);
  db.save('attempts', { q1: { best: 60, count: 5, last: { at: 3000 } } });
  const r = db.sync({
    history: [attempt({ at: 1000 }), attempt({ at: 3000, title: 'Existing' }), { bad: true }],
    attempts: { q1: { best: 90, count: 2, last: { at: 1000 } }, q2: { best: 40, count: 1, last: { at: 900 } } },
    custom: [{ id: 'c1', title: 'Generated' }],
  });
  assert.deepEqual(r.history.map((h) => h.at), [3000, 1000], 'union, no duplicates');
  assert.equal(r.attempts.q1.best, 90);
  assert.equal(r.attempts.q1.count, 5);
  assert.equal(r.attempts.q1.last.at, 3000, 'the most recent attempt wins');
  assert.equal(r.attempts.q2.count, 1);
  assert.equal(r.custom.length, 1);
  assert.equal(db.sync({ history: [attempt({ at: 1000 })] }).history.length, 2, 'syncing twice does not duplicate');
  db.close();
});

test('reset brings everything back to zero and the schema still works', () => {
  const file = tmp();
  const db = openProgress(file);
  db.save('history', [attempt()]);
  db.save('attempts', stats);
  db.save('custom', [{ id: 'c1' }]);
  db.save('run', { x: 1 });
  db.save('limit', true);
  const r = db.reset();
  assert.equal(r.empty, true);
  assert.deepEqual([r.history, r.attempts, r.custom, r.run, r.limit], [[], {}, [], null, null]);
  assert.ok(r.resetAt);
  db.save('history', [attempt()]);
  assert.equal(db.read().history.length, 1, 'usable after reset');
  db.close();
  assert.equal(openProgress(file).reset().empty, true, 'reset is durable across reopen');
});

test('currentBranch reads .git/HEAD', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hld-git-'));
  fs.mkdirSync(path.join(dir, '.git'));
  fs.mkdirSync(path.join(dir, 'a', 'b'), { recursive: true });
  fs.writeFileSync(path.join(dir, '.git', 'HEAD'), 'ref: refs/heads/my-solutions\n');
  assert.equal(currentBranch(path.join(dir, 'a', 'b')), 'my-solutions');
  fs.writeFileSync(path.join(dir, '.git', 'HEAD'), '0123456789abcdef\n');
  assert.match(currentBranch(dir), /^detached at 0123456/);
  assert.equal(currentBranch(fs.mkdtempSync(path.join(os.tmpdir(), 'hld-nogit-'))), null);
});

// ---------- HTTP ----------
async function start(opts = {}) {
  const progress = openProgress(':memory:');
  const server = createServer({ ai: null, questions: [], quizzes: [], mode: 'off', progress, branch: 'my-solutions', ...opts });
  await new Promise((r) => server.listen(0, '127.0.0.1', r));
  const base = `http://127.0.0.1:${server.address().port}`;
  return { base, progress, close: () => new Promise((r) => { server.closeAllConnections?.(); server.close(r); }) };
}
const call = (base, method, p, body, headers = { 'x-hld-progress': '1' }) =>
  fetch(base + p, { method, headers: { 'content-type': 'application/json', ...headers }, body: body === undefined ? undefined : JSON.stringify(body) });

test('progress API: read, save, sync, reset', async () => {
  const s = await start();
  try {
    let r = await (await call(s.base, 'GET', '/api/progress', undefined, {})).json();
    assert.equal(r.empty, true);
    assert.equal(r.branch, 'my-solutions');
    assert.equal(r.file, 'hld/data/progress.db');

    assert.equal((await call(s.base, 'PUT', '/api/progress/history', { value: [attempt()] })).status, 200);
    assert.equal((await call(s.base, 'PUT', '/api/progress/attempts', { value: stats })).status, 200);
    r = await (await call(s.base, 'GET', '/api/progress')).json();
    assert.equal(r.history.length, 1);
    assert.equal(r.empty, false);

    const synced = await (await call(s.base, 'POST', '/api/progress/sync', { history: [attempt({ at: 5 })] })).json();
    assert.equal(synced.history.length, 2);

    const reset = await (await call(s.base, 'DELETE', '/api/progress')).json();
    assert.equal(reset.empty, true);
    assert.deepEqual(reset.history, []);
    assert.equal(s.progress.read().empty, true);
  } finally { await s.close(); }
});

test('progress API refuses changes without the header, from another origin, and with bad keys', async () => {
  const s = await start();
  try {
    assert.equal((await call(s.base, 'DELETE', '/api/progress', undefined, {})).status, 403);
    assert.equal((await call(s.base, 'PUT', '/api/progress/history', { value: [] }, {})).status, 403);
    assert.equal((await call(s.base, 'DELETE', '/api/progress', undefined, { 'x-hld-progress': '1', origin: 'https://evil.example' })).status, 403);
    assert.equal((await call(s.base, 'PUT', '/api/progress/secrets', { value: 1 })).status, 400);
    assert.equal((await call(s.base, 'PUT', '/api/progress/history', { value: 'x' })).status, 400);
    assert.equal((await call(s.base, 'PATCH', '/api/progress', {})).status, 405);
  } finally { await s.close(); }
});

test('without a database the API says so and the page falls back to the browser', async () => {
  const s = await start({ progress: null });
  try {
    assert.equal((await call(s.base, 'GET', '/api/progress')).status, 503);
  } finally { await s.close(); }
});

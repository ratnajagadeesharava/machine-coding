// Quiz progress in a SQLite file that travels with the code (hld/data/progress.db by default).
//
// The file lives in the repository working tree, so whichever branch you commit it on carries your progress with it, and each branch has its own
// copy. It is a single plain file (rollback journal, no -wal/-shm side files) so committing it is enough. Reset wipes every table back to zero.
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';

const require = createRequire(import.meta.url);

export const SCHEMA_VERSION = 1;
export const KV_KEYS = ['run', 'instant', 'limit'];
const MAX_HISTORY = 100;
const MAX_CUSTOM = 20;

const SCHEMA = `
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
-- one row per finished attempt (newest first in the UI)
CREATE TABLE IF NOT EXISTS attempts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  quiz_id TEXT, title TEXT NOT NULL, at INTEGER NOT NULL, seconds INTEGER, mode TEXT,
  percent INTEGER NOT NULL, score INTEGER, total INTEGER, weak TEXT NOT NULL DEFAULT '[]',
  UNIQUE (at, title)
);
-- per quiz: best full-quiz score, number of attempts, and the last attempt (json)
CREATE TABLE IF NOT EXISTS quiz_stats (quiz_id TEXT PRIMARY KEY, best INTEGER, count INTEGER NOT NULL DEFAULT 0, last TEXT);
-- quizzes generated with AI (json)
CREATE TABLE IF NOT EXISTS custom_quizzes (id TEXT PRIMARY KEY, body TEXT NOT NULL, position INTEGER NOT NULL);
-- small things: the quiz in progress (resume) and the two toggles
CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);
`;

const num = (v, d = null) => (Number.isFinite(v) ? v : d);
const str = (v, max = 300) => (typeof v === 'string' ? v.slice(0, max) : null);

export function cleanAttempt(r) {
  if (!r || typeof r !== 'object' || typeof r.title !== 'string' || !Number.isFinite(r.percent) || !Number.isFinite(r.at)) return null;
  return {
    quizId: str(r.quizId, 80), title: r.title.slice(0, 200), at: Math.round(r.at), seconds: num(r.seconds), mode: str(r.mode, 20),
    percent: Math.max(0, Math.min(100, Math.round(r.percent))), score: num(r.score), total: num(r.total),
    weak: Array.isArray(r.weak) ? r.weak.filter((w) => typeof w === 'string').slice(0, 5).map((w) => w.slice(0, 80)) : [],
  };
}

function cleanStats(map) {
  const out = {};
  if (!map || typeof map !== 'object') return out;
  for (const [id, s] of Object.entries(map).slice(0, 500)) {
    if (!s || typeof s !== 'object' || !Number.isFinite(s.count)) continue;
    out[id.slice(0, 80)] = { best: num(s.best), count: Math.max(0, Math.round(s.count)), last: s.last && typeof s.last === 'object' ? s.last : null };
  }
  return out;
}

export function openProgress(file = path.join(path.dirname(new URL(import.meta.url).pathname), '..', 'data', 'progress.db')) {
  if (file !== ':memory:') fs.mkdirSync(path.dirname(file), { recursive: true });
  const Database = require('better-sqlite3'); // loaded here so the rest of the server still starts if the native module is missing
  const db = new Database(file);
  db.pragma('journal_mode = DELETE'); // keep it a single file that can be committed
  db.pragma('synchronous = FULL');
  db.exec(SCHEMA);
  db.prepare("INSERT OR IGNORE INTO meta (key, value) VALUES ('schema_version', ?)").run(String(SCHEMA_VERSION));
  db.prepare("INSERT OR IGNORE INTO meta (key, value) VALUES ('created_at', ?)").run(new Date().toISOString());

  const q = {
    history: db.prepare('SELECT quiz_id, title, at, seconds, mode, percent, score, total, weak FROM attempts ORDER BY at DESC LIMIT ?'),
    insertAttempt: db.prepare('INSERT OR IGNORE INTO attempts (quiz_id, title, at, seconds, mode, percent, score, total, weak) VALUES (@quizId, @title, @at, @seconds, @mode, @percent, @score, @total, @weak)'),
    clearAttempts: db.prepare('DELETE FROM attempts'),
    trimAttempts: db.prepare('DELETE FROM attempts WHERE id NOT IN (SELECT id FROM attempts ORDER BY at DESC LIMIT ?)'),
    stats: db.prepare('SELECT quiz_id, best, count, last FROM quiz_stats'),
    upsertStats: db.prepare('INSERT INTO quiz_stats (quiz_id, best, count, last) VALUES (?, ?, ?, ?) ON CONFLICT(quiz_id) DO UPDATE SET best = excluded.best, count = excluded.count, last = excluded.last'),
    clearStats: db.prepare('DELETE FROM quiz_stats'),
    custom: db.prepare('SELECT body FROM custom_quizzes ORDER BY position'),
    insertCustom: db.prepare('INSERT OR REPLACE INTO custom_quizzes (id, body, position) VALUES (?, ?, ?)'),
    clearCustom: db.prepare('DELETE FROM custom_quizzes'),
    kvAll: db.prepare('SELECT key, value FROM kv'),
    kvSet: db.prepare('INSERT INTO kv (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value'),
    kvDel: db.prepare('DELETE FROM kv WHERE key = ?'),
    clearKv: db.prepare('DELETE FROM kv'),
    metaSet: db.prepare('INSERT INTO meta (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value'),
    metaGet: db.prepare('SELECT value FROM meta WHERE key = ?'),
    counts: db.prepare('SELECT (SELECT COUNT(*) FROM attempts) a, (SELECT COUNT(*) FROM quiz_stats) s, (SELECT COUNT(*) FROM custom_quizzes) c'),
  };

  const rowToAttempt = (r) => ({ quizId: r.quiz_id, title: r.title, at: r.at, seconds: r.seconds, mode: r.mode, percent: r.percent, score: r.score, total: r.total, weak: JSON.parse(r.weak) });

  function read() {
    const attempts = {};
    for (const r of q.stats.all()) attempts[r.quiz_id] = { best: r.best, count: r.count, last: r.last ? JSON.parse(r.last) : null };
    const kv = {};
    for (const r of q.kvAll.all()) kv[r.key] = JSON.parse(r.value);
    const { a, s, c } = q.counts.get();
    return {
      attempts, history: q.history.all(MAX_HISTORY).map(rowToAttempt), custom: q.custom.all().map((r) => JSON.parse(r.body)),
      run: kv.run ?? null, instant: kv.instant ?? null, limit: kv.limit ?? null,
      empty: a + s + c === 0 && Object.keys(kv).filter((k) => k !== 'run').length === 0,
      resetAt: q.metaGet.get('reset_at')?.value ?? null, file,
    };
  }

  const setHistory = db.transaction((list) => {
    q.clearAttempts.run();
    for (const r of list.map(cleanAttempt).filter(Boolean).slice(0, MAX_HISTORY)) q.insertAttempt.run({ ...r, weak: JSON.stringify(r.weak) });
  });
  const setStats = db.transaction((map) => {
    q.clearStats.run();
    for (const [id, s] of Object.entries(cleanStats(map))) q.upsertStats.run(id, s.best, s.count, s.last ? JSON.stringify(s.last) : null);
  });
  const setCustom = db.transaction((list) => {
    q.clearCustom.run();
    list.slice(0, MAX_CUSTOM).forEach((quiz, i) => { if (quiz && typeof quiz.id === 'string') q.insertCustom.run(quiz.id.slice(0, 80), JSON.stringify(quiz), i); });
  });

  return {
    file,
    read,
    // replace one piece of state (what the page does after every change)
    save(key, value) {
      if (key === 'history') { if (!Array.isArray(value)) throw new TypeError('history must be an array'); setHistory(value); }
      else if (key === 'attempts') setStats(value);
      else if (key === 'custom') { if (!Array.isArray(value)) throw new TypeError('custom must be an array'); setCustom(value); }
      else if (KV_KEYS.includes(key)) { if (value == null) q.kvDel.run(key); else q.kvSet.run(key, JSON.stringify(value)); }
      else throw new RangeError(`unknown progress key "${key}"`);
    },
    // merge progress that was kept in a browser's localStorage into the database (keeps whatever is already there)
    sync({ attempts, history, custom } = {}) {
      const have = read();
      const merged = { ...cleanStats(attempts) };
      for (const [id, s] of Object.entries(have.attempts)) {
        const o = merged[id];
        if (!o) { merged[id] = s; continue; }
        const lastAt = (x) => x?.last?.at ?? 0;
        merged[id] = { best: Math.max(o.best ?? 0, s.best ?? 0) || (o.best ?? s.best), count: Math.max(o.count, s.count), last: lastAt(o) >= lastAt(s) ? o.last : s.last };
      }
      const rows = [...have.history, ...(Array.isArray(history) ? history : [])].map(cleanAttempt).filter(Boolean)
        .sort((x, y) => y.at - x.at).filter((r, i, all) => all.findIndex((o) => o.at === r.at && o.title === r.title) === i).slice(0, MAX_HISTORY);
      const known = new Set(have.custom.map((x) => x.id));
      const extra = (Array.isArray(custom) ? custom : []).filter((x) => x && typeof x.id === 'string' && !known.has(x.id));
      setStats(merged); setHistory(rows); setCustom([...have.custom, ...extra]);
      return read();
    },
    // back to zero: every table emptied (the schema stays)
    reset: db.transaction(() => {
      q.clearAttempts.run(); q.clearStats.run(); q.clearCustom.run(); q.clearKv.run();
      q.metaSet.run('reset_at', new Date().toISOString());
      return read();
    }),
    close: () => db.close(),
  };
}

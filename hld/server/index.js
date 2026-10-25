// HLD Room server: static files + a small JSON/SSE API in front of Claude. No framework, no secrets in the browser.
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import Anthropic from '@anthropic-ai/sdk';
import { sanitizeBoard } from '../shared/board.js';
import { AriaError, createAi, DEFAULT_MODEL, friendlyError } from './ai.js';
import { loadEnvFile } from './env.js';
import { createMockClient } from './mock.js';
import { cleanInterviewQuestion } from './question.js';
import { cleanQuestion, GEN_LIMITS, loadQuizzes, quizSummary } from './quiz.js';

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(here, '..');
const PUBLIC_DIR = path.join(ROOT, 'public');
const SHARED_DIR = path.join(ROOT, 'shared');

const MAX_BODY = 768 * 1024;
const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
};
const SECURITY_HEADERS = {
  'content-security-policy': "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; frame-ancestors 'none'",
  'x-content-type-options': 'nosniff',
  'referrer-policy': 'no-referrer',
};

export function publicQuestion(q) {
  const { id, title, category, difficulty, minutes, prompt, assumptions } = q;
  return { id, title, category, difficulty, minutes, prompt, assumptions };
}

function json(res, status, body) {
  const data = JSON.stringify(body);
  res.writeHead(status, { 'content-type': TYPES['.json'], 'cache-control': 'no-store', ...SECURITY_HEADERS });
  res.end(data);
}

class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

async function readJson(req) {
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > MAX_BODY) throw new HttpError(413, 'Request too large.');
    chunks.push(chunk);
  }
  try {
    const body = JSON.parse(Buffer.concat(chunks).toString('utf8') || '{}');
    if (!body || typeof body !== 'object' || Array.isArray(body)) throw new Error('not an object');
    return body;
  } catch {
    throw new HttpError(400, 'Request body must be a JSON object.');
  }
}

function serveStatic(req, res, pathname) {
  let dir = PUBLIC_DIR;
  let rel = pathname;
  if (pathname.startsWith('/shared/')) {
    dir = SHARED_DIR;
    rel = pathname.slice('/shared'.length);
  }
  if (rel === '/' || rel === '') rel = '/index.html';
  let decoded;
  try {
    decoded = decodeURIComponent(rel);
  } catch {
    throw new HttpError(400, 'Bad path.');
  }
  const file = path.normalize(path.join(dir, decoded));
  if (file !== dir && !file.startsWith(dir + path.sep)) throw new HttpError(404, 'Not found.');
  let stat;
  try {
    stat = fs.statSync(file);
  } catch {
    throw new HttpError(404, 'Not found.');
  }
  if (!stat.isFile()) throw new HttpError(404, 'Not found.');
  const type = TYPES[path.extname(file).toLowerCase()] || 'application/octet-stream';
  res.writeHead(200, { 'content-type': type, 'cache-control': 'no-cache', ...SECURITY_HEADERS });
  if (req.method === 'HEAD') return res.end();
  fs.createReadStream(file).pipe(res);
}

function makeRateLimiter(limit, windowMs, now = () => Date.now()) {
  const hits = new Map();
  return (key) => {
    const t = now();
    const list = (hits.get(key) || []).filter((x) => t - x < windowMs);
    if (list.length >= limit) {
      hits.set(key, list);
      return false;
    }
    list.push(t);
    hits.set(key, list);
    return true;
  };
}

// config: { ai | null, questions, mode: 'live' | 'mock' | 'off', model, rateLimit }
// `makeAi(key)` (optional) builds a client for a key the browser sends in the x-anthropic-key header.
export function createServer({ ai, questions, quizzes = [], mode = 'off', model = DEFAULT_MODEL, rateLimit = 60, makeAi = null }) {
  const KEY_RE = /^sk-ant-[A-Za-z0-9_-]{10,400}$/;
  const keyed = new Map();
  // The AI to use for this request: the server's own (env key / demo mode), else one built from the browser's key.
  function aiFor(req) {
    if (ai) return ai;
    const key = req.headers['x-anthropic-key'];
    if (!makeAi || typeof key !== 'string' || !key) return null;
    if (!KEY_RE.test(key)) throw new HttpError(400, 'That does not look like an Anthropic API key (it should start with sk-ant-).');
    if (!keyed.has(key)) {
      if (keyed.size >= 5) keyed.delete(keyed.keys().next().value);
      keyed.set(key, makeAi(key));
    }
    return keyed.get(key);
  }
  const quizById = new Map(quizzes.map((q) => [q.id, q]));
  const allow = makeRateLimiter(rateLimit, 60_000);

  function requireAi(req) {
    const found = aiFor(req);
    if (!found) throw new HttpError(503, 'Aria is not connected. Add your Claude API key in Settings (key icon), or put ANTHROPIC_API_KEY in hld/.env and restart, or run "npm run demo" for offline demo mode.');
    if (!allow(req.socket.remoteAddress || 'local')) throw new HttpError(429, 'Slow down a little - too many requests.');
    return found;
  }

  // The AI endpoints take the whole question from the client ({ question: {...} }); the server never looks one up.
  function common(body) {
    const cleaned = cleanInterviewQuestion(body.question);
    if (cleaned.error) throw new HttpError(400, `Invalid question: ${cleaned.error}`);
    const { question } = cleaned;
    const totalMs = question.minutes * 60000;
    const elapsedMs = Number.isFinite(body.elapsedMs) ? Math.max(0, Math.min(totalMs * 3, body.elapsedMs)) : 0;
    const status = ['idle', 'running', 'paused', 'ended'].includes(body.status) ? body.status : 'running';
    const mode = ['clarify', 'hint', 'review'].includes(body.mode) ? body.mode : 'chat';
    return { question, board: sanitizeBoard(body.board), elapsedMs, status, mode, messages: body.messages };
  }

  async function chat(req, res) {
    const ai = requireAi(req);
    const c = common(await readJson(req));
    const ctl = new AbortController();
    res.on('close', () => { if (!res.writableEnded) ctl.abort(); });
    res.writeHead(200, {
      'content-type': 'text/event-stream; charset=utf-8',
      'cache-control': 'no-cache, no-transform',
      connection: 'keep-alive',
      'x-accel-buffering': 'no',
      ...SECURITY_HEADERS,
    });
    const send = (event, data) => res.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
    try {
      for await (const text of ai.chat({ ...c, signal: ctl.signal })) send('delta', { text });
      send('done', {});
    } catch (err) {
      if (!ctl.signal.aborted) send('error', { message: friendlyError(err).message });
    }
    res.end();
  }

  async function watch(req, res) {
    const ai = requireAi(req);
    const body = await readJson(req);
    const c = common(body);
    const sinceAriaMs = Number.isFinite(body.sinceAriaMs) ? Math.max(0, body.sinceAriaMs) : null;
    try {
      json(res, 200, await ai.watch({ ...c, sinceAriaMs }));
    } catch (err) {
      const fe = friendlyError(err);
      json(res, fe.status, { error: fe.message });
    }
  }

  async function evaluate(req, res) {
    const ai = requireAi(req);
    const c = common(await readJson(req));
    try {
      json(res, 200, await ai.evaluate(c));
    } catch (err) {
      const fe = friendlyError(err);
      json(res, fe.status, { error: fe.message });
    }
  }

  async function quizExplain(req, res) {
    const ai = requireAi(req);
    const body = await readJson(req);
    const cleaned = cleanQuestion(body.question);
    if (cleaned.error) throw new HttpError(400, `Invalid question: ${cleaned.error}`);
    const ids = new Set(cleaned.question.options.map((o) => o.id));
    const selected = Array.isArray(body.selected) ? body.selected.filter((x) => typeof x === 'string' && ids.has(x)) : [];
    const ctl = new AbortController();
    res.on('close', () => { if (!res.writableEnded) ctl.abort(); });
    res.writeHead(200, {
      'content-type': 'text/event-stream; charset=utf-8',
      'cache-control': 'no-cache, no-transform',
      connection: 'keep-alive',
      'x-accel-buffering': 'no',
      ...SECURITY_HEADERS,
    });
    const send = (event, data) => res.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
    try {
      for await (const text of ai.explain({ question: cleaned.question, selected, history: body.history, signal: ctl.signal })) send('delta', { text });
      send('done', {});
    } catch (err) {
      if (!ctl.signal.aborted) send('error', { message: friendlyError(err).message });
    }
    res.end();
  }

  async function quizGenerate(req, res) {
    const ai = requireAi(req);
    const body = await readJson(req);
    const topic = typeof body.topic === 'string' ? body.topic.trim() : '';
    if (topic.length < 3 || topic.length > GEN_LIMITS.topic) throw new HttpError(400, `Give a topic of 3-${GEN_LIMITS.topic} characters.`);
    const notes = typeof body.notes === 'string' ? body.notes.trim().slice(0, GEN_LIMITS.notes) : '';
    const count = Math.max(GEN_LIMITS.minCount, Math.min(GEN_LIMITS.maxCount, Number.isInteger(body.count) ? body.count : 10));
    const level = ['beginner', 'intermediate', 'advanced'].includes(body.level) ? body.level : 'intermediate';
    try {
      const { quiz, dropped } = await ai.generateQuiz({ topic, notes, count, level });
      json(res, 200, { quiz, dropped });
    } catch (err) {
      const fe = friendlyError(err);
      json(res, fe.status, { error: fe.message });
    }
  }

  return http.createServer(async (req, res) => {
    try {
      const url = new URL(req.url, 'http://localhost');
      const p = url.pathname;
      if (p.startsWith('/api/')) {
        if (p === '/api/health' && req.method === 'GET') {
          const a = ai || (() => { try { return aiFor(req); } catch { return null; } })();
          return json(res, 200, { ok: true, ai: ai ? mode : a ? 'live' : 'off', model: a ? model : null, ...(!ai && makeAi ? { browserKey: !!a, canUseBrowserKey: true } : {}) });
        }
        if (p === '/api/questions' && req.method === 'GET') return json(res, 200, questions.map(publicQuestion));
        if (p === '/api/quizzes' && req.method === 'GET') return json(res, 200, quizzes.map(quizSummary));
        const m = /^\/api\/quizzes\/([^/]+)$/.exec(p);
        if (m && req.method === 'GET') {
          const quiz = quizById.get(m[1]);
          if (!quiz) throw new HttpError(404, 'Unknown quiz.');
          return json(res, 200, quiz);
        }
        if (req.method !== 'POST') throw new HttpError(405, 'Method not allowed.');
        if (p === '/api/quiz/explain') return await quizExplain(req, res);
        if (p === '/api/quiz/generate') return await quizGenerate(req, res);
        if (p === '/api/chat') return await chat(req, res);
        if (p === '/api/watch') return await watch(req, res);
        if (p === '/api/evaluate') return await evaluate(req, res);
        throw new HttpError(404, 'Not found.');
      }
      if (req.method !== 'GET' && req.method !== 'HEAD') throw new HttpError(405, 'Method not allowed.');
      return serveStatic(req, res, p);
    } catch (err) {
      const status = err instanceof HttpError ? err.status : err instanceof AriaError ? err.status : 500;
      if (res.headersSent) return res.end();
      const message = err instanceof HttpError || err instanceof AriaError ? err.message : 'Internal error.';
      if (status === 500) console.error(err);
      return json(res, status, { error: message });
    }
  });
}

export function loadQuestions(file = path.join(ROOT, 'questions', 'questions.json')) {
  const list = JSON.parse(fs.readFileSync(file, 'utf8'));
  if (!Array.isArray(list) || !list.length) throw new Error('questions.json must be a non-empty array');
  return list.map(publicQuestion);
}

function main() {
  loadEnvFile(path.join(ROOT, '.env'));
  const env = process.env;
  const port = Number(env.PORT) || 3000;
  const host = env.HLD_HOST || '127.0.0.1';
  const model = env.HLD_MODEL || DEFAULT_MODEL;
  const questions = loadQuestions();
  const quizzes = loadQuizzes(path.join(ROOT, 'quizzes'));

  let ai = null;
  let mode = 'off';
  if (env.HLD_MOCK === '1') {
    ai = createAi({ client: createMockClient(), model: 'mock', effort: 'none' });
    mode = 'mock';
  } else if (env.ANTHROPIC_API_KEY) {
    ai = createAi({ client: new Anthropic({ apiKey: env.ANTHROPIC_API_KEY }), model, effort: env.HLD_EFFORT || 'default' });
    mode = 'live';
  }
  const makeAi = (apiKey) => createAi({ client: new Anthropic({ apiKey }), model, effort: env.HLD_EFFORT || 'default' });
  const server = createServer({ ai, questions, quizzes, mode, model: mode === 'mock' ? 'mock' : model, makeAi });
  server.listen(port, host, () => {
    console.log(`HLD Room: http://${host === '0.0.0.0' ? 'localhost' : host}:${port}`);
    if (mode === 'live') console.log(`Aria is live (model ${model}).`);
    else if (mode === 'mock') console.log('Demo mode: Aria gives canned replies, no API calls are made.');
    else console.log('Aria is NOT connected: set ANTHROPIC_API_KEY in hld/.env (copy .env.example) or run "npm run demo".');
    if (host !== '127.0.0.1' && host !== 'localhost') console.warn(`Warning: listening on ${host}. Anyone who can reach this port can spend your API key.`);
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();

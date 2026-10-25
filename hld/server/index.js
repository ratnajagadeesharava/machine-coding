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

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(here, '..');
const PUBLIC_DIR = path.join(ROOT, 'public');
const SHARED_DIR = path.join(ROOT, 'shared');

const MAX_BODY = 256 * 1024;
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
  'content-security-policy': "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'",
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
export function createServer({ ai, questions, mode = 'off', model = DEFAULT_MODEL, rateLimit = 60 }) {
  const byId = new Map(questions.map((q) => [q.id, q]));
  const allow = makeRateLimiter(rateLimit, 60_000);

  function requireAi(req) {
    if (!ai) throw new HttpError(503, 'Aria is not connected. Add ANTHROPIC_API_KEY to hld/.env (see .env.example) and restart, or run "npm run demo" for offline demo mode.');
    if (!allow(req.socket.remoteAddress || 'local')) throw new HttpError(429, 'Slow down a little - too many requests.');
  }

  function common(body) {
    const question = byId.get(body.questionId);
    if (!question) throw new HttpError(404, 'Unknown question.');
    const totalMs = question.minutes * 60000;
    const elapsedMs = Number.isFinite(body.elapsedMs) ? Math.max(0, Math.min(totalMs * 3, body.elapsedMs)) : 0;
    const status = ['idle', 'running', 'paused', 'ended'].includes(body.status) ? body.status : 'running';
    const mode = ['clarify', 'hint', 'review'].includes(body.mode) ? body.mode : 'chat';
    return { question, board: sanitizeBoard(body.board), elapsedMs, status, mode, messages: body.messages };
  }

  async function chat(req, res) {
    requireAi(req);
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
    requireAi(req);
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
    requireAi(req);
    const c = common(await readJson(req));
    try {
      json(res, 200, await ai.evaluate(c));
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
        if (p === '/api/health' && req.method === 'GET') return json(res, 200, { ok: true, ai: ai ? mode : 'off', model: ai ? model : null });
        if (p === '/api/questions' && req.method === 'GET') return json(res, 200, questions.map(publicQuestion));
        if (req.method !== 'POST') throw new HttpError(405, 'Method not allowed.');
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
  return list;
}

function main() {
  loadEnvFile(path.join(ROOT, '.env'));
  const env = process.env;
  const port = Number(env.PORT) || 3000;
  const host = env.HLD_HOST || '127.0.0.1';
  const model = env.HLD_MODEL || DEFAULT_MODEL;
  const questions = loadQuestions();

  let ai = null;
  let mode = 'off';
  if (env.HLD_MOCK === '1') {
    ai = createAi({ client: createMockClient(), model: 'mock', effort: 'none' });
    mode = 'mock';
  } else if (env.ANTHROPIC_API_KEY) {
    ai = createAi({ client: new Anthropic({ apiKey: env.ANTHROPIC_API_KEY }), model, effort: env.HLD_EFFORT || 'default' });
    mode = 'live';
  }
  const server = createServer({ ai, questions, mode, model: mode === 'mock' ? 'mock' : model });
  server.listen(port, host, () => {
    console.log(`HLD Room: http://${host === '0.0.0.0' ? 'localhost' : host}:${port}`);
    if (mode === 'live') console.log(`Aria is live (model ${model}).`);
    else if (mode === 'mock') console.log('Demo mode: Aria gives canned replies, no API calls are made.');
    else console.log('Aria is NOT connected: set ANTHROPIC_API_KEY in hld/.env (copy .env.example) or run "npm run demo".');
    if (host !== '127.0.0.1' && host !== 'localhost') console.warn(`Warning: listening on ${host}. Anyone who can reach this port can spend your API key.`);
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();

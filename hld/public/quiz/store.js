// Quiz progress. The SQLite file next to the code (hld/data/progress.db, see server/progress.js) is the source of truth when the server has it;
// localStorage stays as a local cache and as the fallback when there is no database. Pages listen to the `progress` event to redraw.
import { validateQuiz } from '/shared/quiz.js';

const PREFIX = 'hld-quiz:v1:';
export const S = { attempts: {}, history: [], custom: [], summaries: [], health: { ai: 'off' }, full: new Map(), run: null, last: null };
// hold = "do not write to the database yet" while the browser's older progress waits to be synced
export const DB = { on: false, hold: false, offer: false, branch: null, file: null, info: null };
export const bus = new EventTarget();
export const emit = (name = 'progress') => bus.dispatchEvent(new Event(name));

const REMOTE_KEYS = new Set(['attempts', 'history', 'custom', 'run', 'instant', 'limit']);
const queued = new Map();
let flushTimer = null;
let saveFailed = false;
export let onSaveError = (msg) => console.warn(msg);
export const setSaveErrorHandler = (fn) => { onSaveError = fn; };

export async function dbFetch(method, path, body, keepalive = false) {
  const res = await fetch(path, { method, keepalive, headers: { 'content-type': 'application/json', 'x-hld-progress': '1' }, body: body === undefined ? undefined : JSON.stringify(body) });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.error || `Request failed (${res.status}).`);
  return data;
}
function flushSaves(keepalive = false) {
  clearTimeout(flushTimer); flushTimer = null;
  for (const [key, value] of queued) {
    dbFetch('PUT', `/api/progress/${key}`, { value }, keepalive).then(() => { saveFailed = false; }).catch((err) => {
      if (!saveFailed) { saveFailed = true; onSaveError(`Could not save progress to the database: ${err.message} (kept in this browser).`); }
    });
  }
  queued.clear();
}
function queueSave(key, value) {
  queued.set(key, value);
  if (!flushTimer) flushTimer = setTimeout(() => flushSaves(), 250);
}
window.addEventListener('pagehide', () => { if (queued.size) flushSaves(true); });

export const store = {
  get(key, fallback) {
    try { const v = JSON.parse(localStorage.getItem(PREFIX + key)); return v ?? fallback; } catch { return fallback; }
  },
  setLocal(key, value) {
    try { localStorage.setItem(PREFIX + key, JSON.stringify(value)); } catch { /* storage unavailable: the page still works for this visit */ }
  },
  set(key, value) {
    this.setLocal(key, value);
    if (DB.on && !DB.hold && REMOTE_KEYS.has(key)) queueSave(key, value);
  },
};

export const validHistory = (list) => (Array.isArray(list) ? list : []).filter((r) => r && typeof r.title === 'string' && Number.isFinite(r.percent));
const hasLocalProgress = () => Object.keys(store.get('attempts', {}) || {}).length > 0 || validHistory(store.get('history', [])).length > 0 || (store.get('custom', []) || []).length > 0;

// Make the page state (and the local cache) equal to what the database holds.
function applyInfo(info) {
  DB.info = info;
  S.attempts = info.attempts || {};
  S.history = validHistory(info.history);
  S.custom = (info.custom || []).filter((q) => validateQuiz(q).length === 0);
  store.setLocal('attempts', S.attempts); store.setLocal('history', S.history); store.setLocal('custom', S.custom);
  store.setLocal('instant', info.instant ?? true); store.setLocal('limit', info.limit === true);
  store.setLocal('run', info.run ?? null);
}
function applyLocal() {
  S.attempts = store.get('attempts', {});
  S.history = validHistory(store.get('history', []));
  S.custom = (store.get('custom', []) || []).filter((q) => validateQuiz(q).length === 0);
}

export async function loadProgress() {
  try {
    const info = await dbFetch('GET', '/api/progress');
    DB.on = true; DB.branch = info.branch; DB.file = info.file;
    DB.offer = hasLocalProgress() && !store.get('synced', false);
    DB.hold = DB.offer && info.empty; // the database is still empty: keep the browser's progress on screen until the user decides
    if (!DB.hold) applyInfo(info); else DB.info = info;
  } catch { DB.on = false; }
  if (!DB.on || DB.hold) applyLocal();
  emit();
}

export async function syncNow() {
  const info = await dbFetch('POST', '/api/progress/sync', { attempts: store.get('attempts', {}), history: store.get('history', []), custom: store.get('custom', []) });
  store.setLocal('synced', true);
  DB.offer = false; DB.hold = false;
  applyInfo(info);
  emit();
  return info;
}
export function declineSync() {
  store.setLocal('synced', true);
  DB.offer = false;
  if (DB.hold) { DB.hold = false; applyInfo(DB.info); }
  emit();
}
export async function resetAll() {
  if (DB.on) { const info = await dbFetch('DELETE', '/api/progress'); DB.hold = false; applyInfo(info); }
  else { S.attempts = {}; S.history = []; S.custom = []; store.setLocal('attempts', {}); store.setLocal('history', []); store.setLocal('custom', []); store.setLocal('run', null); }
  store.setLocal('synced', true);
  DB.offer = false;
  S.run = null; S.last = null;
  emit();
}

export function exportProgress() {
  const data = { app: 'hld-room-quizzes', version: 1, at: new Date().toISOString(), attempts: S.attempts, history: S.history, custom: S.custom };
  const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
  const a = document.createElement('a'); a.href = url; a.download = 'hld-quiz-progress.json';
  document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
export function importProgressFile(file) {
  return file.text().then((raw) => {
    const data = JSON.parse(raw);
    if (data?.app !== 'hld-room-quizzes') throw new Error('not a progress file');
    S.attempts = { ...S.attempts, ...(data.attempts && typeof data.attempts === 'object' ? data.attempts : {}) };
    S.history = [...(Array.isArray(data.history) ? data.history : []), ...S.history].filter((r) => r && typeof r.title === 'string' && Number.isFinite(r.percent))
      .sort((x, y) => y.at - x.at).filter((r, i, all) => all.findIndex((o) => o.at === r.at && o.title === r.title) === i).slice(0, 100);
    const known = new Set(S.custom.map((q) => q.id));
    S.custom = [...S.custom, ...(Array.isArray(data.custom) ? data.custom.filter((q) => validateQuiz(q).length === 0 && !known.has(q.id)) : [])];
    store.set('attempts', S.attempts); store.set('history', S.history); store.set('custom', S.custom);
    emit();
  });
}

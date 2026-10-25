// Browser-side persistence shared by the Home page and the interview room (localStorage only; every access is guarded).
export const PREFIX = 'hld-room:v2:';
export const LEVELS = [
  { id: 'junior', label: 'Junior', hint: 'Core components and a working flow' },
  { id: 'mid', label: 'Mid-level', hint: 'Solid requirements, estimates and data model' },
  { id: 'senior', label: 'Senior', hint: 'Drives the design, justifies trade-offs' },
  { id: 'staff', label: 'Staff', hint: 'Depth, multi-region, cost, ambiguity' },
];
export const STYLES = [
  { id: 'supportive', label: 'Supportive', hint: 'More hints and encouragement' },
  { id: 'balanced', label: 'Balanced', hint: 'Fair and neutral' },
  { id: 'tough', label: 'Tough', hint: 'Probing, fewer hints' },
];
export const TIME_PRESETS = [15, 30, 45, 60, 90];
// Same limits the server enforces (server/question.js).
export const LIMITS = { title: [3, 120], prompt: [10, 2500], assumptions: 8, assumption: 240, minutes: [10, 120], focus: 300, customs: 30 };

function read(key) {
  try { const raw = localStorage.getItem(key); return raw == null ? null : JSON.parse(raw); } catch { return null; }
}
function write(key, value) {
  try { localStorage.setItem(key, JSON.stringify(value)); return true; } catch { return false; }
}
export const remove = (key) => { try { localStorage.removeItem(key); } catch { /* ignore */ } };

const CONTROL = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F]/g;
const oneOf = (v, list, d) => (list.some((x) => x.id === v) ? v : d);
const clampMinutes = (v) => (Number.isFinite(v) ? Math.max(LIMITS.minutes[0], Math.min(LIMITS.minutes[1], Math.round(v))) : 45);

// ---- launch options for a bank question ----
export const launchKey = (id) => `${PREFIX}launch:${id}`;
export function getLaunch(id, defaults = {}) {
  const raw = read(launchKey(id));
  return {
    level: oneOf(raw && raw.level, LEVELS, defaults.level || 'senior'),
    style: oneOf(raw && raw.style, STYLES, defaults.style || 'balanced'),
    minutes: clampMinutes(raw && Number.isFinite(raw.minutes) ? raw.minutes : defaults.minutes),
  };
}
export function setLaunch(id, { level, style, minutes }) {
  return write(launchKey(id), { level: oneOf(level, LEVELS, 'senior'), style: oneOf(style, STYLES, 'balanced'), minutes: clampMinutes(minutes) });
}

// ---- custom questions (array, newest first, at most 30) ----
const CUSTOMS = `${PREFIX}customs`;
function cleanCustom(c) {
  if (!c || typeof c !== 'object' || typeof c.id !== 'string' || !/^custom-[a-z0-9]+$/.test(c.id)) return null;
  if (typeof c.title !== 'string' || typeof c.prompt !== 'string') return null;
  return {
    id: c.id, title: c.title, prompt: c.prompt,
    assumptions: Array.isArray(c.assumptions) ? c.assumptions.filter((a) => typeof a === 'string') : [],
    minutes: clampMinutes(c.minutes), level: oneOf(c.level, LEVELS, 'senior'), style: oneOf(c.style, STYLES, 'balanced'),
    focus: typeof c.focus === 'string' ? c.focus : '', createdAt: Number.isFinite(c.createdAt) ? c.createdAt : 0,
  };
}
export function getCustoms() {
  const raw = read(CUSTOMS);
  return Array.isArray(raw) ? raw.map(cleanCustom).filter(Boolean).slice(0, LIMITS.customs) : [];
}
export const getCustom = (id) => getCustoms().find((c) => c.id === id) || null;
export function newCustomId() {
  const rnd = (typeof crypto !== 'undefined' && crypto.getRandomValues) ? [...crypto.getRandomValues(new Uint8Array(6))].map((b) => b.toString(36).padStart(2, '0')).join('') : Math.random().toString(36).slice(2, 12);
  return `custom-${rnd.replace(/[^a-z0-9]/g, '').slice(0, 12)}`;
}
// Saves (or replaces, when q.id exists) and moves the question to the front. Returns the saved question, or null if storage failed.
export function saveCustom(q) {
  const entry = cleanCustom({ ...q, id: q.id || newCustomId(), createdAt: Date.now() });
  if (!entry) return null;
  const list = [entry, ...getCustoms().filter((c) => c.id !== entry.id)].slice(0, LIMITS.customs);
  return write(CUSTOMS, list) ? entry : null;
}
export function deleteCustom(id) {
  write(CUSTOMS, getCustoms().filter((c) => c.id !== id));
  remove(sessionKey('custom', id));
  remove(`${PREFIX}board:custom:${id}`);
}

// ---- form validation, matching the server's limits. input: raw form values -> { errors, question } ----
export function validateCustom(input) {
  const errors = {};
  const title = String(input.title || '').replace(CONTROL, '').replace(/\s+/g, ' ').trim();
  const prompt = String(input.prompt || '').replace(CONTROL, '').replace(/\r\n?/g, '\n').trim();
  const assumptions = String(input.assumptions || '').replace(CONTROL, '').split('\n').map((l) => l.replace(/^\s*[-*•]\s*/, '').replace(/\s+/g, ' ').trim()).filter(Boolean);
  const minutes = Number(input.minutes);
  const focus = String(input.focus || '').replace(CONTROL, '').replace(/\s+/g, ' ').trim();
  if (title.length < LIMITS.title[0] || title.length > LIMITS.title[1]) errors.title = `Give it a title of ${LIMITS.title[0]}-${LIMITS.title[1]} characters.`;
  if (prompt.length < LIMITS.prompt[0] || prompt.length > LIMITS.prompt[1]) errors.prompt = `Describe the problem in ${LIMITS.prompt[0]}-${LIMITS.prompt[1]} characters (now ${prompt.length}).`;
  if (assumptions.length > LIMITS.assumptions) errors.assumptions = `At most ${LIMITS.assumptions} assumptions, one per line.`;
  else if (assumptions.some((a) => a.length > LIMITS.assumption)) errors.assumptions = `Each assumption can be at most ${LIMITS.assumption} characters.`;
  if (!Number.isFinite(minutes) || minutes < LIMITS.minutes[0] || minutes > LIMITS.minutes[1]) errors.minutes = `Time limit must be ${LIMITS.minutes[0]}-${LIMITS.minutes[1]} minutes.`;
  if (focus.length > LIMITS.focus) errors.focus = `Keep focus areas under ${LIMITS.focus} characters (now ${focus.length}).`;
  return {
    errors,
    question: {
      title, prompt, assumptions, minutes: clampMinutes(minutes), level: oneOf(input.level, LEVELS, 'senior'), style: oneOf(input.style, STYLES, 'balanced'), focus,
    },
  };
}

// ---- interview sessions (state of one question's interview; the board has its own key, managed by the whiteboard) ----
export const sessionKey = (kind, id) => `${kind === 'custom' ? 'custom' : 'q'}:${id}`;
export const sessionStore = (key) => `${PREFIX}session:${key}`;
export const boardStore = (key) => `${PREFIX}board:${key}`;
export const loadSession = (key) => { const raw = read(sessionStore(key)); return raw && typeof raw === 'object' ? raw : null; };
export const saveSession = (key, record) => write(sessionStore(key), record);

// Sessions worth continuing, newest first: { key, kind, id, title, status, remainingMs, href }
export function listSessions() {
  const out = [];
  try {
    for (let i = 0; i < localStorage.length; i++) {
      const k = localStorage.key(i);
      if (!k || !k.startsWith(`${PREFIX}session:`)) continue;
      const key = k.slice(`${PREFIX}session:`.length);
      const m = /^(q|custom):(.+)$/.exec(key);
      const rec = read(k);
      if (!m || !rec || typeof rec.title !== 'string' || !['paused', 'ended'].includes(rec.status)) continue;
      const kind = m[1] === 'custom' ? 'custom' : 'q';
      out.push({
        key, kind, id: m[2], title: rec.title, status: rec.status, remainingMs: Number.isFinite(rec.remainingMs) ? rec.remainingMs : 0,
        updatedAt: Number.isFinite(rec.updatedAt) ? rec.updatedAt : 0,
        href: kind === 'custom' ? `/room.html?custom=${encodeURIComponent(m[2])}` : `/room.html?q=${encodeURIComponent(m[2])}`,
      });
    }
  } catch { /* storage unavailable */ }
  return out.sort((a, b) => b.updatedAt - a.updatedAt);
}

// ---- panel visibility in the room ----
const UI = `${PREFIX}ui`;
export function getUi() {
  const raw = read(UI) || {};
  return { questionOpen: raw.questionOpen !== false, botOpen: raw.botOpen !== false };
}
export function setUi(patch) { write(UI, { ...getUi(), ...patch }); }

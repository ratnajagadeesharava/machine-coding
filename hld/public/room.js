// Interview room: a question panel, the standalone whiteboard, and Aria. The room receives ONLY the question
// (title, prompt, assumptions, time, level, style, focus) and passes exactly that to the AI.
import './apikey.js';
import { boardSignature, boardStats, emptyBoard, toAiBoard } from '/shared/board.js';
import { mountWhiteboard } from '/wb/mount.js';
import { makeResizable } from '/resize.js';
import {
  LEVELS, STYLES, boardStore, getCustom, getLaunch, getUi, loadSession, saveSession, sessionKey, setUi,
} from '/store.js';

const $ = (sel) => document.querySelector(sel);
const WATCH_SETTLE_MS = 12_000;      // board must be still this long before Aria considers speaking
const WATCH_MIN_GAP_MS = 75_000;     // minimum time between Aria's unprompted remarks
const MAX_NUDGES = 10;
const MILESTONES = [0.25, 0.5, 0.75, 0.9];

const state = {
  q: null,            // the launch question: { title, prompt, assumptions, minutes, level, style, focus?, difficulty? }
  key: null,          // session key: q:<id> | custom:<id>
  health: { ai: 'off' },
  s: null,            // the session
  botOpen: true,
  streaming: false,
  watching: false,
  watchTimer: null,
  lastTick: 0,
  lastSave: 0,
  app: null,
};

const els = {
  title: $('#q-title'), chips: $('#q-chips'), timer: $('#timer'), start: $('#btn-start'), pause: $('#btn-pause'),
  pauseLabel: $('#btn-pause-label'), end: $('#btn-end'), feedback: $('#btn-feedback'), banner: $('#banner'),
  qPanel: $('#q-panel'), qTab: $('#q-tab'), qClose: $('#q-close'), botPanel: $('#bot-panel'), botTab: $('#bot-tab'), botClose: $('#bot-close'),
  ariaStatus: $('#aria-status'), ariaStatusText: $('#aria-status-text'), ariaBoard: $('#aria-board'),
  chat: $('#chat'), form: $('#chat-form'), input: $('#chat-input'), send: $('#chat-send'),
  chips2: [...document.querySelectorAll('[data-quick]')],
  report: $('#report'), reportBody: $('#report-body'), reportClose: $('#report-close'),
  confirm: $('#confirm'), confirmText: $('#confirm-text'), confirmYes: $('#confirm-yes'), confirmNo: $('#confirm-no'),
};
makeResizable(els.qPanel, 'question', 'right');
makeResizable(els.botPanel, 'aria', 'left');

// ---------- helpers ----------
const clock = (ms) => {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};
const stamp = (ms) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};
const node = (tag, cls, text) => {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
};
const totalMs = () => state.q.minutes * 60_000;
const elapsedMs = () => totalMs() - state.s.remainingMs;
// Aria is on only while her panel is open and the server has an AI configured.
const ariaOn = () => state.botOpen && state.health.ai !== 'off';

function goHome(notice) {
  location.replace(notice ? `/?notice=${encodeURIComponent(notice)}` : '/');
}

function greeting(q) {
  return [{
    role: 'assistant', at: 0, kind: 'greeting',
    content: `Hi, I am Aria, your interviewer today. We will work on "${q.title}". Ask me about the requirements first. When you press Start, I will watch your board as you build.`,
  }];
}

function newSession() {
  return {
    status: 'idle', remainingMs: totalMs(), messages: greeting(state.q), evaluation: null,
    lastChangeAt: null, lastAriaAt: null, lastWatchedSig: boardSignature(emptyBoard()), nudges: 0, milestones: {},
  };
}

function save() {
  const s = state.s;
  if (!s) return;
  saveSession(state.key, {
    title: state.q.title, minutes: state.q.minutes, updatedAt: Date.now(),
    status: s.status === 'running' ? 'paused' : s.status, remainingMs: Math.round(s.remainingMs),
    messages: s.messages.slice(-80), evaluation: s.evaluation, nudges: s.nudges, milestones: s.milestones,
  });
}

function load() {
  const s = newSession();
  const raw = loadSession(state.key);
  if (!raw) return s;
  s.status = ['idle', 'paused', 'ended'].includes(raw.status) ? raw.status : 'idle';
  const old = Number.isFinite(raw.minutes) ? raw.minutes : state.q.minutes;
  let rem = Number.isFinite(raw.remainingMs) ? raw.remainingMs : s.remainingMs;
  rem += (state.q.minutes - old) * 60_000; // a changed time limit adds or removes the difference
  s.remainingMs = s.status === 'idle' ? totalMs() : Math.max(0, Math.min(totalMs(), rem));
  if (Array.isArray(raw.messages) && raw.messages.length) {
    s.messages = raw.messages.filter((m) => m && typeof m.content === 'string' && ['user', 'assistant', 'system'].includes(m.role));
  }
  s.evaluation = raw.evaluation && typeof raw.evaluation === 'object' ? raw.evaluation : null;
  s.nudges = Number.isInteger(raw.nudges) ? raw.nudges : 0;
  s.milestones = raw.milestones && typeof raw.milestones === 'object' ? raw.milestones : {};
  return s;
}

// Dialogs: whiteboard shortcuts are switched off while one is open.
function openModal(dialog) {
  state.app?.setKeyboard(false);
  dialog.addEventListener('close', () => state.app?.setKeyboard(true), { once: true });
  dialog.showModal();
}
function askConfirm(text, yesLabel = 'Confirm') {
  return new Promise((resolve) => {
    els.confirmText.textContent = text;
    els.confirmYes.textContent = yesLabel;
    const done = (v) => { els.confirm.close(); resolve(v); };
    els.confirmYes.onclick = () => done(true);
    els.confirmNo.onclick = () => done(false);
    els.confirm.oncancel = () => resolve(false);
    openModal(els.confirm);
  });
}

// ---------- question panel ----------
const levelLabel = (id) => LEVELS.find((l) => l.id === id)?.label || id;
const styleLabel = (id) => STYLES.find((s) => s.id === id)?.label || id;
function chipList(q) {
  const out = [];
  const add = (text, cls = '') => out.push(node('span', `chip-pill ${cls}`.trim(), text));
  if (q.difficulty) add(q.difficulty, q.difficulty);
  add(`${q.minutes} min`, 'plain');
  add(levelLabel(q.level), 'plain');
  add(`${styleLabel(q.style)} interviewer`, 'plain');
  return out;
}
function renderQuestion() {
  const q = state.q;
  els.title.textContent = q.title;
  els.chips.replaceChildren(...chipList(q));
  $('#qp-title').textContent = q.title;
  $('#qp-chips').replaceChildren(...chipList(q));
  $('#q-prompt').textContent = q.prompt;
  $('#q-assume').replaceChildren(...q.assumptions.map((a) => node('li', '', a)));
  $('#qp-assume-wrap').hidden = !q.assumptions.length;
  $('#qp-focus-wrap').hidden = !q.focus;
  $('#qp-focus').textContent = q.focus || '';
  document.title = `${q.title} - HLD Room`;
}

function setQuestionOpen(open, persist = true) {
  els.qPanel.hidden = !open;
  els.qTab.hidden = open;
  if (persist) setUi({ questionOpen: open });
  if (persist) (open ? els.qClose : els.qTab).focus({ preventScroll: true });
}

function setBotOpen(open, persist = true) {
  state.botOpen = open;
  els.botPanel.hidden = !open;
  els.botTab.hidden = open;
  if (persist) setUi({ botOpen: open });
  if (!open) {
    // Aria is completely off: no check-ins, no requests in flight.
    clearTimeout(state.watchTimer);
    if (state.abort) state.abort.abort();
    els.input.blur();
  } else {
    scheduleWatch();
  }
  state.app?.setKeyboard(true);
  if (persist) (open ? els.botClose : els.botTab).focus({ preventScroll: true });
  if (state.s) { renderControls(); renderStatus(); }
}

// ---------- timer / lifecycle ----------
function renderTimer() {
  const s = state.s;
  els.timer.textContent = clock(s.remainingMs);
  els.timer.classList.toggle('low', s.status !== 'idle' && s.remainingMs < 5 * 60_000);
}

function renderControls() {
  const s = state.s;
  const running = s.status === 'running';
  const active = running || s.status === 'paused';
  els.start.hidden = active;
  els.start.textContent = s.status === 'ended' ? 'Try again' : 'Start';
  els.pause.hidden = !active;
  els.pauseLabel.textContent = running ? 'Pause' : 'Resume';
  els.end.hidden = !active;
  els.feedback.hidden = !(s.status === 'ended' && s.evaluation);
  els.chips2.forEach((c) => { c.disabled = state.streaming || !ariaOn(); });
  els.send.disabled = state.streaming;
}

function renderStatus() {
  const s = state.s;
  const st = els.ariaStatus;
  st.classList.toggle('live', s.status === 'running' && !state.streaming && ariaOn());
  st.classList.toggle('thinking', state.streaming);
  let text;
  if (state.health.ai === 'off') text = 'Not connected';
  else if (state.streaming) text = 'Thinking...';
  else if (s.status === 'running') text = s.lastChangeAt ? `Watching - last change ${Math.max(0, Math.round((Date.now() - s.lastChangeAt) / 1000))}s ago` : 'Watching - no changes yet';
  else if (s.status === 'paused') text = 'Paused';
  else if (s.status === 'ended') text = 'Interview ended';
  else text = 'Ready when you are';
  els.ariaStatusText.textContent = text;
  if (state.app) {
    const b = boardStats(state.app.getBoard());
    els.ariaBoard.textContent = `${b.components} component${b.components === 1 ? '' : 's'} · ${b.connections} connection${b.connections === 1 ? '' : 's'} in view`;
  }
}

function start() {
  const s = state.s;
  if (s.status === 'ended') {
    state.s = newSession();
    state.app.setBoard(emptyBoard(), { fit: false });
    renderChat();
  }
  const cur = state.s;
  cur.status = 'running';
  cur.lastChangeAt = null;
  state.lastTick = performance.now();
  if (!cur.messages.some((m) => m.kind === 'started')) {
    addMessage({ role: 'assistant', kind: 'started', content: 'The timer is running. Spend the first few minutes on requirements. I will stay quiet unless you ask me something or I spot something worth raising.' });
  }
  save();
  renderAll();
  scheduleWatch();
}

function pause() {
  const s = state.s;
  if (s.status === 'running') s.status = 'paused';
  else if (s.status === 'paused') { s.status = 'running'; state.lastTick = performance.now(); }
  save();
  renderAll();
  scheduleWatch();
}

async function end({ timeUp = false } = {}) {
  const s = state.s;
  if (s.status !== 'running' && s.status !== 'paused') return;
  const ask = ariaOn() ? 'End the interview now and get feedback from Aria?' : 'End the interview now?';
  if (!timeUp && !(await askConfirm(ask, ariaOn() ? 'End and get feedback' : 'End interview'))) return;
  s.status = 'ended';
  clearTimeout(state.watchTimer);
  save();
  renderAll();
  await showFeedback(true);
}

setInterval(() => {
  if (!state.s) return;
  const s = state.s;
  if (s.status === 'running') {
    const now = performance.now();
    s.remainingMs = Math.max(0, s.remainingMs - (now - state.lastTick));
    state.lastTick = now;
    if (now - state.lastSave > 5000) { state.lastSave = now; save(); }
    checkMilestones();
    if (s.remainingMs <= 0) {
      addMessage({ role: 'assistant', kind: 'system-note', content: ariaOn() ? 'Time is up. Let me put together your feedback.' : 'Time is up.' });
      end({ timeUp: true });
    }
  }
  renderTimer();
  renderStatus();
}, 250);

// ---------- chat ----------
function renderText(container, text) {
  container.replaceChildren();
  let list = null;
  for (const line of String(text).split('\n')) {
    const bullet = /^\s*[-*•]\s+(.*)$/.exec(line);
    if (bullet) {
      if (!list) { list = document.createElement('ul'); container.appendChild(list); }
      list.appendChild(node('li', '', bullet[1]));
    } else if (line.trim()) {
      list = null;
      container.appendChild(node('p', '', line));
    } else list = null;
  }
}

function messageNode(m) {
  const wrap = node('div', `msg ${m.role}${m.kind === 'nudge' ? ' nudge' : ''}`);
  const bubble = node('div', 'bubble');
  renderText(bubble, m.content);
  wrap.appendChild(bubble);
  if (m.role !== 'system') wrap.appendChild(node('div', 'stamp', stamp(m.at || 0)));
  return { wrap, bubble };
}

function renderChat() {
  els.chat.replaceChildren(...state.s.messages.map((m) => messageNode(m).wrap));
  els.chat.scrollTop = els.chat.scrollHeight;
}

function addMessage(m) {
  const s = state.s;
  const msg = { at: elapsedMs(), ...m };
  s.messages.push(msg);
  if (m.role === 'assistant' && m.kind !== 'greeting') s.lastAriaAt = Date.now();
  els.chat.appendChild(messageNode(msg).wrap);
  els.chat.scrollTop = els.chat.scrollHeight;
  save();
  return msg;
}

const apiMessages = () => state.s.messages.filter((m) => m.role !== 'system').map((m) => ({ role: m.role, content: m.content }));

// The question sent to the server: exactly what the candidate sees plus the session settings.
function launchQuestion() {
  const { title, prompt, assumptions, minutes, level, style, focus, difficulty } = state.q;
  const q = { title, prompt, assumptions, minutes, level, style };
  if (focus) q.focus = focus;
  if (difficulty) q.difficulty = difficulty;
  return q;
}

function payload(extra = {}) {
  return {
    question: launchQuestion(), board: toAiBoard(state.app.getBoard()), elapsedMs: elapsedMs(),
    status: state.s.status, messages: apiMessages(), ...extra,
  };
}

async function readError(res) {
  try { return (await res.json()).error || `Request failed (${res.status}).`; } catch { return `Request failed (${res.status}).`; }
}

async function askAria(text, mode = 'chat', shown = text) {
  if (state.streaming || !text.trim()) return;
  const s = state.s;
  addMessage({ role: 'user', content: shown });
  if (!ariaOn()) {
    addMessage({ role: 'system', content: state.health.ai === 'off' ? 'Aria is not connected, so she cannot answer. The whiteboard and timer still work.' : 'Aria is switched off.' });
    return;
  }
  // the model sees the quick-action text as the user's turn; the display text is just the button label
  const body = payload({ mode });
  body.messages[body.messages.length - 1].content = text;
  state.streaming = true;
  renderControls();
  renderStatus();
  const typing = node('div', 'msg assistant');
  const tb = node('div', 'bubble');
  const dots = node('span', 'typing');
  dots.append(document.createElement('i'), document.createElement('i'), document.createElement('i'));
  tb.appendChild(dots);
  typing.appendChild(tb);
  els.chat.appendChild(typing);
  els.chat.scrollTop = els.chat.scrollHeight;
  let reply = '';
  let msgNode = null;
  state.abort = new AbortController();
  try {
    const res = await fetch('/api/chat', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body), signal: state.abort.signal });
    if (!res.ok) throw new Error(await readError(res));
    const reader = res.body.getReader();
    const decoder = new TextDecoder();
    let buf = '';
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      buf += decoder.decode(value, { stream: true });
      let idx;
      while ((idx = buf.indexOf('\n\n')) >= 0) {
        const block = buf.slice(0, idx);
        buf = buf.slice(idx + 2);
        const ev = /^event: (.+)$/m.exec(block)?.[1];
        const data = JSON.parse(/^data: (.+)$/m.exec(block)?.[1] || '{}');
        if (ev === 'delta') {
          reply += data.text;
          if (!msgNode) {
            msgNode = messageNode({ role: 'assistant', content: reply, at: elapsedMs() });
            typing.replaceWith(msgNode.wrap);
          } else renderText(msgNode.bubble, reply);
          els.chat.scrollTop = els.chat.scrollHeight;
        } else if (ev === 'error') throw new Error(data.message || 'Aria could not answer.');
      }
    }
    if (!reply.trim()) throw new Error('Aria sent an empty reply. Try again.');
    typing.remove();
    msgNode?.wrap.remove();
    addMessage({ role: 'assistant', content: reply.trim() });
  } catch (err) {
    typing.remove();
    msgNode?.wrap.remove();
    if (reply.trim()) addMessage({ role: 'assistant', content: reply.trim() });
    if (err.name !== 'AbortError') addMessage({ role: 'system', content: err.message });
  } finally {
    state.streaming = false;
    state.abort = null;
    s.lastAriaAt = Date.now();
    renderControls();
    renderStatus();
    scheduleWatch();
  }
}

els.form.addEventListener('submit', (e) => {
  e.preventDefault();
  const text = els.input.value.trim();
  if (!text) return;
  els.input.value = '';
  askAria(text);
});
els.input.addEventListener('focus', () => state.app?.setKeyboard(false));
els.input.addEventListener('blur', () => state.app?.setKeyboard(true));
const QUICK = {
  clarify: ['Clarify requirements', 'I would like to clarify the requirements.'],
  hint: ['Give me a hint', 'Could you give me a hint?'],
  review: ['Review my board', 'Please review my board.'],
};
els.chips2.forEach((c) => c.addEventListener('click', () => {
  const [label, text] = QUICK[c.dataset.quick];
  askAria(text, c.dataset.quick, label);
}));

// ---------- Aria watching the board ----------
function scheduleWatch() {
  clearTimeout(state.watchTimer);
  const s = state.s;
  if (!s || s.status !== 'running' || !ariaOn()) return;
  state.watchTimer = setTimeout(() => watchNow(false), WATCH_SETTLE_MS);
}

function checkMilestones() {
  const s = state.s;
  if (!ariaOn()) return;
  const frac = 1 - s.remainingMs / totalMs();
  for (const m of MILESTONES) {
    if (frac >= m && !s.milestones[m]) {
      s.milestones[m] = true;
      watchNow(true);
      return;
    }
  }
}

async function watchNow(force) {
  const s = state.s;
  if (!s || s.status !== 'running' || state.streaming || state.watching || !ariaOn() || s.nudges >= MAX_NUDGES) return;
  const board = state.app.getBoard();
  const sig = boardSignature(board);
  if (!force && sig === s.lastWatchedSig) return;
  const sinceAria = s.lastAriaAt == null ? null : Date.now() - s.lastAriaAt;
  if (sinceAria != null && sinceAria < WATCH_MIN_GAP_MS) { state.watchTimer = setTimeout(() => watchNow(force), WATCH_MIN_GAP_MS - sinceAria + 500); return; }
  s.lastWatchedSig = sig;
  state.watching = true;
  try {
    const res = await fetch('/api/watch', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(payload({ sinceAriaMs: sinceAria })) });
    if (!res.ok) return;
    const out = await res.json();
    if (ariaOn() && state.s === s && s.status === 'running' && out.speak && out.message) {
      s.nudges += 1;
      addMessage({ role: 'assistant', kind: 'nudge', content: out.message });
    }
  } catch { /* a missed silent check-in is harmless */ } finally {
    state.watching = false;
  }
}

// ---------- feedback report ----------
function section(title, items) {
  const box = document.createElement('div');
  const ul = document.createElement('ul');
  for (const t of items.length ? items : ['Nothing to list.']) ul.appendChild(node('li', '', t));
  box.append(node('h3', '', title), ul);
  return box;
}

function renderReport(ev) {
  const body = els.reportBody;
  body.replaceChildren();
  const v = node('div', 'verdict');
  const total = ev.areas.reduce((a, x) => a + x.score, 0);
  v.append(node('b', '', `${total} / ${ev.areas.length * 5}`), node('span', `tag${ev.verdict === 'Needs work' ? ' bad' : ''}`, ev.verdict));
  const areas = node('div', 'areas');
  for (const a of ev.areas) {
    const row = node('div', 'area');
    const top = node('div', 'area-top');
    top.append(node('span', '', a.name), node('span', '', `${a.score} / 5`));
    const bar = node('div', 'bar');
    const fill = document.createElement('i');
    fill.style.width = `${(a.score / 5) * 100}%`;
    bar.appendChild(fill);
    row.append(top, bar, node('p', '', a.comment));
    areas.appendChild(row);
  }
  const cols = node('div', 'cols');
  cols.append(section('What went well', ev.strengths), section('Gaps', ev.gaps), section('Next steps', ev.next_steps));
  body.append(v, node('p', '', ev.summary), areas, cols);
}

function reportMessage(text, buttonLabel, onClick, isError) {
  els.reportBody.replaceChildren();
  const p = node('p', isError ? 'error-text' : '', text);
  els.reportBody.appendChild(p);
  if (buttonLabel) {
    const b = node('button', 'btn primary', buttonLabel);
    b.type = 'button';
    b.addEventListener('click', onClick);
    els.reportBody.appendChild(b);
  }
}

async function showFeedback(generate) {
  const s = state.s;
  if (!els.report.open) openModal(els.report);
  if (s.evaluation) return renderReport(s.evaluation);
  if (!generate) return undefined;
  if (state.health.ai === 'off') {
    reportMessage('Aria is not connected, so there is no written feedback. The whiteboard and timer worked without her.', null, null, false);
    return undefined;
  }
  if (!state.botOpen) {
    reportMessage('Aria is switched off. Open her panel to get written feedback.', 'Open Aria and get feedback', () => { setBotOpen(true); showFeedback(true); }, false);
    return undefined;
  }
  els.reportBody.replaceChildren();
  const loading = node('div', 'loading');
  const dots = node('span', 'typing');
  dots.append(document.createElement('i'), document.createElement('i'), document.createElement('i'));
  loading.append(dots, node('p', '', 'Aria is writing your feedback...'));
  els.reportBody.appendChild(loading);
  try {
    const res = await fetch('/api/evaluate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(payload({ status: 'ended' })) });
    if (!res.ok) throw new Error(await readError(res));
    s.evaluation = await res.json();
    save();
    renderControls();
    renderReport(s.evaluation);
  } catch (err) {
    reportMessage(err.message, 'Try again', () => showFeedback(true), true);
  }
  return undefined;
}

// ---------- wiring ----------
els.start.addEventListener('click', start);
els.pause.addEventListener('click', pause);
els.end.addEventListener('click', () => end());
els.feedback.addEventListener('click', () => showFeedback(false));
els.reportClose.addEventListener('click', () => els.report.close());
els.qClose.addEventListener('click', () => setQuestionOpen(false));
els.qTab.addEventListener('click', () => setQuestionOpen(true));
els.botClose.addEventListener('click', () => setBotOpen(false));
els.botTab.addEventListener('click', () => setBotOpen(true));
window.addEventListener('beforeunload', () => { if (state.s) save(); });

function renderAll() {
  renderTimer();
  renderControls();
  renderStatus();
}

function renderBanner() {
  const ai = state.health.ai;
  const b = els.banner;
  b.classList.remove('info');
  b.replaceChildren();
  if (ai === 'off') {
    b.hidden = false;
    b.append('Aria is not connected, so chat and feedback are unavailable. Click "Add key" (top right) to paste your Claude key, or set ', node('code', '', 'ANTHROPIC_API_KEY'), ' in ', node('code', '', 'hld/.env'), '. The whiteboard and timer work meanwhile.');
  } else if (ai === 'mock') {
    b.hidden = false;
    b.classList.add('info');
    b.textContent = 'Demo mode: Aria gives canned replies and no API calls are made. Add your Claude key (top right) for real interviews.';
  } else b.hidden = true;
}

// Resolve ?q=<bank id> or ?custom=<id> into the launch question, or null.
async function resolveQuestion() {
  const params = new URLSearchParams(location.search);
  const bankId = params.get('q');
  const customId = params.get('custom');
  if (customId) {
    const c = getCustom(customId);
    if (!c) return null;
    return { key: sessionKey('custom', c.id), q: { title: c.title, prompt: c.prompt, assumptions: c.assumptions, minutes: c.minutes, level: c.level, style: c.style, focus: c.focus || undefined } };
  }
  if (bankId) {
    let list;
    try { list = await (await fetch('/api/questions')).json(); } catch { return null; }
    const b = Array.isArray(list) ? list.find((x) => x.id === bankId) : null;
    if (!b) return null;
    const launch = getLaunch(b.id, { minutes: b.minutes });
    return {
      key: sessionKey('q', b.id),
      q: { title: b.title, prompt: b.prompt, assumptions: b.assumptions, minutes: launch.minutes, level: launch.level, style: launch.style, difficulty: b.difficulty },
    };
  }
  return null;
}

async function init() {
  const resolved = await resolveQuestion();
  if (!resolved) { goHome('question-not-found'); return; }
  state.q = resolved.q;
  state.key = resolved.key;
  try { state.health = await (await fetch('/api/health')).json(); } catch { state.health = { ai: 'off' }; }

  state.app = mountWhiteboard($('#board'), {
    storageKey: boardStore(state.key),
    menuExtra: [{ label: 'Back to home', action: () => { location.href = '/'; } }],
  });
  state.app.on('change', () => {
    if (!state.s) return;
    state.s.lastChangeAt = Date.now();
    renderStatus();
    scheduleWatch();
  });

  state.s = load();
  state.s.lastWatchedSig = boardSignature(state.app.getBoard());
  const ui = getUi();
  renderQuestion();
  renderBanner();
  setQuestionOpen(ui.questionOpen, false);
  setBotOpen(ui.botOpen, false);
  renderChat();
  renderAll();
}

init();

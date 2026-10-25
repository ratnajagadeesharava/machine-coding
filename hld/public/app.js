// HLD Room front end: question picker, timer, whiteboard, and the conversation with Aria.
import { boardSignature, boardStats, emptyBoard, sanitizeBoard } from '/shared/board.js';
import { Whiteboard } from '/whiteboard.js';

const $ = (sel) => document.querySelector(sel);
const STORE = 'hld-room:v1:';
const WATCH_SETTLE_MS = 12_000;      // board must be still this long before Aria considers speaking
const WATCH_MIN_GAP_MS = 75_000;     // minimum time between Aria's unprompted remarks
const MAX_NUDGES = 10;
const MILESTONES = [0.25, 0.5, 0.75, 0.9];

const state = {
  questions: [],
  qid: null,
  health: { ai: 'off' },
  sessions: new Map(),
  filter: { text: '', diff: 'All' },
  streaming: false,
  watching: false,
  watchTimer: null,
  lastTick: 0,
  lastSave: 0,
};

const els = {
  title: $('#q-title'), sub: $('#q-sub'), timer: $('#timer'), start: $('#btn-start'), pause: $('#btn-pause'),
  pauseLabel: $('#btn-pause-label'), end: $('#btn-end'), feedback: $('#btn-feedback'), banner: $('#banner'),
  search: $('#q-search'), diff: $('#q-diff'), list: $('#q-list'), prompt: $('#q-prompt'), assume: $('#q-assume'),
  pill: $('#pill'), pillText: $('#pill-text'), ariaStatus: $('#aria-status'), ariaStatusText: $('#aria-status-text'),
  ariaBoard: $('#aria-board'), chat: $('#chat'), form: $('#chat-form'), input: $('#chat-input'), send: $('#chat-send'),
  chips: [...document.querySelectorAll('[data-quick]')], undo: $('#btn-undo'), trash: $('#btn-trash'),
  report: $('#report'), reportBody: $('#report-body'), reportClose: $('#report-close'),
  confirm: $('#confirm'), confirmText: $('#confirm-text'), confirmYes: $('#confirm-yes'), confirmNo: $('#confirm-no'),
};

// ---------- helpers ----------
const clock = (ms) => {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};
const stamp = (ms) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};
const question = () => state.questions.find((q) => q.id === state.qid);
const session = () => state.sessions.get(state.qid);
const totalMs = (q = question()) => q.minutes * 60_000;
const elapsedMs = (s = session(), q = question()) => totalMs(q) - s.remainingMs;

function greeting(q) {
  return [{
    role: 'assistant', at: 0, kind: 'greeting',
    content: `Hi, I am Aria, your interviewer today. We will work on "${q.title}". Ask me about the requirements first. When you press Start, I will watch your board as you build.`,
  }];
}

function newSession(q) {
  return {
    status: 'idle', remainingMs: totalMs(q), board: emptyBoard(), messages: greeting(q), evaluation: null,
    lastChangeAt: null, lastAriaAt: null, lastWatchedSig: boardSignature(emptyBoard()), nudges: 0, milestones: {},
  };
}

function save(qid = state.qid) {
  const s = state.sessions.get(qid);
  if (!s) return;
  try {
    localStorage.setItem(STORE + qid, JSON.stringify({
      status: s.status === 'running' ? 'paused' : s.status, remainingMs: s.remainingMs, board: s.board,
      messages: s.messages.slice(-80), evaluation: s.evaluation, nudges: s.nudges, milestones: s.milestones,
    }));
  } catch { /* storage unavailable (private mode, quota): the app still works without persistence */ }
}

function load(q) {
  const s = newSession(q);
  try {
    const raw = JSON.parse(localStorage.getItem(STORE + q.id) || 'null');
    if (raw && typeof raw === 'object') {
      s.status = ['idle', 'paused', 'ended'].includes(raw.status) ? raw.status : 'idle';
      s.remainingMs = Number.isFinite(raw.remainingMs) ? Math.max(0, Math.min(totalMs(q), raw.remainingMs)) : s.remainingMs;
      s.board = sanitizeBoard(raw.board);
      if (Array.isArray(raw.messages) && raw.messages.length) {
        s.messages = raw.messages.filter((m) => m && typeof m.content === 'string' && ['user', 'assistant', 'system'].includes(m.role));
      }
      s.evaluation = raw.evaluation && typeof raw.evaluation === 'object' ? raw.evaluation : null;
      s.nudges = Number.isInteger(raw.nudges) ? raw.nudges : 0;
      s.milestones = raw.milestones && typeof raw.milestones === 'object' ? raw.milestones : {};
      s.lastWatchedSig = boardSignature(s.board);
    }
  } catch { /* ignore corrupt saved data */ }
  return s;
}

function askConfirm(text, yesLabel = 'Confirm') {
  return new Promise((resolve) => {
    els.confirmText.textContent = text;
    els.confirmYes.textContent = yesLabel;
    const done = (v) => { els.confirm.close(); resolve(v); };
    els.confirmYes.onclick = () => done(true);
    els.confirmNo.onclick = () => done(false);
    els.confirm.oncancel = () => resolve(false);
    els.confirm.showModal();
  });
}

// ---------- whiteboard ----------
const wb = new Whiteboard($('#board'), {
  onChange: (board) => {
    const s = session();
    if (!s) return;
    s.board = board;
    s.lastChangeAt = Date.now();
    save();
    renderStatus();
    scheduleWatch();
  },
  onSelect: () => renderToolbar(),
  onTool: (tool) => {
    document.querySelectorAll('.tool[data-tool]').forEach((b) => b.classList.toggle('on', b.dataset.tool === tool));
  },
});

document.querySelectorAll('.tool[data-tool]').forEach((b) => b.addEventListener('click', () => wb.setTool(b.dataset.tool)));
els.undo.addEventListener('click', () => wb.undo());
els.trash.addEventListener('click', async () => {
  if (wb.hasSelection()) { wb.deleteSelected(); return; }
  if (!wb.getBoard().items.length) return;
  if (await askConfirm('Clear the whole board?', 'Clear board')) wb.clear();
});
function renderToolbar() {
  els.undo.disabled = !wb.canUndo();
  els.trash.title = wb.hasSelection() ? 'Delete selected (Del)' : 'Clear the whole board';
}
setInterval(renderToolbar, 400);

// ---------- question list / prompt ----------
function renderList() {
  const t = state.filter.text.trim().toLowerCase();
  const items = state.questions.filter((q) =>
    (state.filter.diff === 'All' || q.difficulty === state.filter.diff) &&
    (!t || q.title.toLowerCase().includes(t) || q.category.toLowerCase().includes(t)));
  els.list.replaceChildren();
  if (!items.length) {
    const p = document.createElement('p');
    p.className = 'q-empty';
    p.textContent = 'No questions match.';
    els.list.appendChild(p);
    return;
  }
  for (const q of items) {
    const b = document.createElement('button');
    b.type = 'button';
    b.className = 'q-item' + (q.id === state.qid ? ' on' : '');
    b.setAttribute('role', 'option');
    b.setAttribute('aria-selected', String(q.id === state.qid));
    const title = document.createElement('b');
    title.textContent = q.title;
    const meta = document.createElement('span');
    meta.textContent = `${q.difficulty} · ${q.minutes} min · ${q.category}`;
    const st = state.sessions.get(q.id)?.status;
    if (st === 'ended') { const badge = document.createElement('i'); badge.className = 'badge'; badge.textContent = 'done'; meta.appendChild(badge); }
    b.append(title, meta);
    b.addEventListener('click', () => selectQuestion(q.id));
    els.list.appendChild(b);
  }
}

function renderPrompt() {
  const q = question();
  els.title.textContent = q.title;
  els.sub.textContent = `High-level design · ${q.difficulty} · ${q.minutes} min`;
  els.prompt.textContent = q.prompt;
  els.assume.replaceChildren(...q.assumptions.map((a) => { const li = document.createElement('li'); li.textContent = a; return li; }));
  document.title = `${q.title} - HLD Room`;
}

async function selectQuestion(id) {
  if (id === state.qid) return;
  if (state.streaming) return;
  if (state.qid && session()?.status === 'running') pause();
  save();
  state.qid = id;
  if (!state.sessions.has(id)) state.sessions.set(id, load(question()));
  try { localStorage.setItem(STORE + 'last', id); } catch { /* ignore */ }
  wb.setBoard(session().board);
  renderAll();
}

// ---------- timer / lifecycle ----------
function renderTimer() {
  const s = session();
  els.timer.textContent = clock(s.remainingMs);
  els.timer.classList.toggle('low', s.status !== 'idle' && s.remainingMs < 5 * 60_000);
}

function renderControls() {
  const s = session();
  const running = s.status === 'running';
  const active = running || s.status === 'paused';
  els.start.hidden = active;
  els.start.textContent = s.status === 'ended' ? 'Try again' : 'Start';
  els.pause.hidden = !active;
  els.pauseLabel.textContent = running ? 'Pause' : 'Resume';
  els.end.hidden = !active;
  els.feedback.hidden = !(s.status === 'ended' && s.evaluation);
  els.chips.forEach((c) => { c.disabled = state.streaming || !state.health.ai || state.health.ai === 'off'; });
  els.send.disabled = state.streaming;
}

function renderStatus() {
  const s = session();
  const st = els.ariaStatus;
  st.classList.toggle('live', s.status === 'running' && !state.streaming);
  st.classList.toggle('thinking', state.streaming);
  let text;
  if (state.streaming) text = 'Thinking…';
  else if (s.status === 'running') text = s.lastChangeAt ? `Watching · last change ${Math.max(0, Math.round((Date.now() - s.lastChangeAt) / 1000))}s ago` : 'Watching · no changes yet';
  else if (s.status === 'paused') text = 'Paused';
  else if (s.status === 'ended') text = 'Interview ended';
  else text = 'Ready when you are';
  els.ariaStatusText.textContent = text;
  const b = boardStats(s.board);
  els.ariaBoard.textContent = `${b.components} component${b.components === 1 ? '' : 's'} · ${b.connections} connection${b.connections === 1 ? '' : 's'} in view`;
  els.pill.classList.toggle('off', s.status !== 'running');
  els.pillText.textContent = s.status === 'running' ? 'Aria is watching this board'
    : s.status === 'paused' ? 'Paused - Aria is not watching'
      : s.status === 'ended' ? 'Interview ended' : 'Press Start so Aria can watch';
}

function start() {
  const s = session();
  const q = question();
  if (s.status === 'ended') {
    Object.assign(s, newSession(q));
    wb.setBoard(s.board);
    renderChat();
  }
  s.status = 'running';
  s.lastChangeAt = null;
  state.lastTick = performance.now();
  if (!s.messages.some((m) => m.kind === 'started')) {
    addMessage({ role: 'assistant', kind: 'started', content: 'The timer is running. Spend the first few minutes on requirements. I will stay quiet unless you ask me something or I spot something worth raising.' });
  }
  save();
  renderAll();
}

function pause() {
  const s = session();
  if (s.status === 'running') s.status = 'paused';
  else if (s.status === 'paused') { s.status = 'running'; state.lastTick = performance.now(); }
  save();
  renderAll();
}

async function end({ timeUp = false } = {}) {
  const s = session();
  if (s.status !== 'running' && s.status !== 'paused') return;
  if (!timeUp && !(await askConfirm('End the interview now and get feedback from Aria?', 'End and get feedback'))) return;
  s.status = 'ended';
  clearTimeout(state.watchTimer);
  save();
  renderAll();
  await showFeedback(true);
}

setInterval(() => {
  if (!state.qid) return;
  const s = session();
  if (s.status === 'running') {
    const now = performance.now();
    s.remainingMs = Math.max(0, s.remainingMs - (now - state.lastTick));
    state.lastTick = now;
    if (now - state.lastSave > 5000) { state.lastSave = now; save(); }
    checkMilestones();
    if (s.remainingMs <= 0) {
      addMessage({ role: 'assistant', kind: 'system-note', content: 'Time is up. Let me put together your feedback.' });
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
      const li = document.createElement('li');
      li.textContent = bullet[1];
      list.appendChild(li);
    } else if (line.trim()) {
      list = null;
      const p = document.createElement('p');
      p.textContent = line;
      container.appendChild(p);
    } else list = null;
  }
}

function messageNode(m) {
  const wrap = document.createElement('div');
  wrap.className = `msg ${m.role}${m.kind === 'nudge' ? ' nudge' : ''}`;
  const bubble = document.createElement('div');
  bubble.className = 'bubble';
  renderText(bubble, m.content);
  wrap.appendChild(bubble);
  if (m.role !== 'system') {
    const t = document.createElement('div');
    t.className = 'stamp';
    t.textContent = stamp(m.at || 0);
    wrap.appendChild(t);
  }
  return { wrap, bubble };
}

function renderChat() {
  els.chat.replaceChildren(...session().messages.map((m) => messageNode(m).wrap));
  els.chat.scrollTop = els.chat.scrollHeight;
}

function addMessage(m) {
  const s = session();
  const msg = { at: elapsedMs(), ...m };
  s.messages.push(msg);
  if (m.role === 'assistant' && m.kind !== 'greeting') s.lastAriaAt = Date.now();
  els.chat.appendChild(messageNode(msg).wrap);
  els.chat.scrollTop = els.chat.scrollHeight;
  save();
  return msg;
}

function apiMessages(s) {
  return s.messages.filter((m) => m.role !== 'system').map((m) => ({ role: m.role, content: m.content }));
}

function payload(extra = {}) {
  const s = session();
  return { questionId: state.qid, board: wb.getBoard(), elapsedMs: elapsedMs(), status: s.status, messages: apiMessages(s), ...extra };
}

async function readError(res) {
  try { return (await res.json()).error || `Request failed (${res.status}).`; } catch { return `Request failed (${res.status}).`; }
}

async function askAria(text, mode = 'chat', shown = text) {
  if (state.streaming || !text.trim()) return;
  const s = session();
  addMessage({ role: 'user', content: shown });
  // the model sees the quick-action text as the user's turn; the display text is just the button label
  const body = payload({ mode });
  body.messages[body.messages.length - 1].content = text;
  state.streaming = true;
  renderControls();
  renderStatus();
  const typing = document.createElement('div');
  typing.className = 'msg assistant';
  typing.innerHTML = '<div class="bubble"><span class="typing"><i></i><i></i><i></i></span></div>';
  els.chat.appendChild(typing);
  els.chat.scrollTop = els.chat.scrollHeight;
  let reply = '';
  let node = null;
  try {
    const res = await fetch('/api/chat', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
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
          if (!node) {
            node = messageNode({ role: 'assistant', content: reply, at: elapsedMs() });
            typing.replaceWith(node.wrap);
          } else renderText(node.bubble, reply);
          els.chat.scrollTop = els.chat.scrollHeight;
        } else if (ev === 'error') throw new Error(data.message || 'Aria could not answer.');
      }
    }
    if (!reply.trim()) throw new Error('Aria sent an empty reply. Try again.');
    typing.remove();
    node?.wrap.remove();
    addMessage({ role: 'assistant', content: reply.trim() });
  } catch (err) {
    typing.remove();
    node?.wrap.remove();
    if (reply.trim()) addMessage({ role: 'assistant', content: reply.trim() });
    addMessage({ role: 'system', content: err.message });
  } finally {
    state.streaming = false;
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
const QUICK = {
  clarify: ['Clarify requirements', 'I would like to clarify the requirements.'],
  hint: ['Give me a hint', 'Could you give me a hint?'],
  review: ['Review my board', 'Please review my board.'],
};
els.chips.forEach((c) => c.addEventListener('click', () => {
  const [label, text] = QUICK[c.dataset.quick];
  askAria(text, c.dataset.quick, label);
}));

// ---------- Aria watching the board ----------
function scheduleWatch() {
  clearTimeout(state.watchTimer);
  const s = session();
  if (!s || s.status !== 'running') return;
  state.watchTimer = setTimeout(() => watchNow(false), WATCH_SETTLE_MS);
}

function checkMilestones() {
  const s = session();
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
  const s = session();
  if (!s || s.status !== 'running' || state.streaming || state.watching) return;
  if (state.health.ai === 'off' || s.nudges >= MAX_NUDGES) return;
  const sig = boardSignature(wb.getBoard());
  if (!force && sig === s.lastWatchedSig) return;
  const sinceAria = s.lastAriaAt == null ? null : Date.now() - s.lastAriaAt;
  if (sinceAria != null && sinceAria < WATCH_MIN_GAP_MS) { state.watchTimer = setTimeout(() => watchNow(force), WATCH_MIN_GAP_MS - sinceAria + 500); return; }
  s.lastWatchedSig = sig;
  state.watching = true;
  const qid = state.qid;
  try {
    const res = await fetch('/api/watch', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(payload({ sinceAriaMs: sinceAria })) });
    if (!res.ok) return;
    const out = await res.json();
    if (qid === state.qid && session().status === 'running' && out.speak && out.message) {
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
  const h = document.createElement('h3');
  h.textContent = title;
  const ul = document.createElement('ul');
  for (const t of items.length ? items : ['Nothing to list.']) { const li = document.createElement('li'); li.textContent = t; ul.appendChild(li); }
  box.append(h, ul);
  return box;
}

function renderReport(ev) {
  const body = els.reportBody;
  body.replaceChildren();
  const v = document.createElement('div');
  v.className = 'verdict';
  const tag = document.createElement('span');
  tag.className = 'tag' + (ev.verdict === 'Needs work' ? ' bad' : '');
  tag.textContent = ev.verdict;
  const b = document.createElement('b');
  const total = ev.areas.reduce((a, x) => a + x.score, 0);
  b.textContent = `${total} / ${ev.areas.length * 5}`;
  v.append(b, tag);
  const summary = document.createElement('p');
  summary.textContent = ev.summary;
  const areas = document.createElement('div');
  areas.className = 'areas';
  for (const a of ev.areas) {
    const row = document.createElement('div');
    row.className = 'area';
    const top = document.createElement('div');
    top.className = 'area-top';
    const name = document.createElement('span');
    name.textContent = a.name;
    const score = document.createElement('span');
    score.textContent = `${a.score} / 5`;
    top.append(name, score);
    const bar = document.createElement('div');
    bar.className = 'bar';
    const fill = document.createElement('i');
    fill.style.width = `${(a.score / 5) * 100}%`;
    bar.appendChild(fill);
    const c = document.createElement('p');
    c.textContent = a.comment;
    row.append(top, bar, c);
    areas.appendChild(row);
  }
  const cols = document.createElement('div');
  cols.className = 'cols';
  cols.append(section('What went well', ev.strengths), section('Gaps', ev.gaps), section('Next steps', ev.next_steps));
  body.append(v, summary, areas, cols);
}

async function showFeedback(generate) {
  const s = session();
  if (!els.report.open) els.report.showModal();
  if (s.evaluation) return renderReport(s.evaluation);
  if (!generate) return undefined;
  els.reportBody.innerHTML = '<div class="loading"><span class="typing"><i></i><i></i><i></i></span><p>Aria is writing your feedback…</p></div>';
  try {
    const res = await fetch('/api/evaluate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(payload({ status: 'ended' })) });
    if (!res.ok) throw new Error(await readError(res));
    s.evaluation = await res.json();
    save();
    renderControls();
    renderReport(s.evaluation);
  } catch (err) {
    els.reportBody.replaceChildren();
    const p = document.createElement('p');
    p.className = 'error-text';
    p.textContent = err.message;
    const retry = document.createElement('button');
    retry.className = 'btn primary';
    retry.type = 'button';
    retry.textContent = 'Try again';
    retry.addEventListener('click', () => showFeedback(true));
    els.reportBody.append(p, retry);
  }
  return undefined;
}

// ---------- wiring ----------
els.start.addEventListener('click', start);
els.pause.addEventListener('click', pause);
els.end.addEventListener('click', () => end());
els.feedback.addEventListener('click', () => showFeedback(false));
els.reportClose.addEventListener('click', () => els.report.close());
els.search.addEventListener('input', () => { state.filter.text = els.search.value; renderList(); });
els.diff.addEventListener('click', (e) => {
  const b = e.target.closest('button[data-diff]');
  if (!b) return;
  state.filter.diff = b.dataset.diff;
  els.diff.querySelectorAll('button').forEach((x) => x.classList.toggle('on', x === b));
  renderList();
});
window.addEventListener('beforeunload', () => { if (state.qid) save(); });

function renderAll() {
  renderList();
  renderPrompt();
  renderTimer();
  renderControls();
  renderStatus();
  renderChat();
  renderToolbar();
}

function renderBanner() {
  const ai = state.health.ai;
  const b = els.banner;
  b.classList.remove('info');
  if (ai === 'off') {
    b.hidden = false;
    b.innerHTML = 'Aria is not connected. Copy <code>hld/.env.example</code> to <code>hld/.env</code>, set <code>ANTHROPIC_API_KEY</code>, and restart the server. The whiteboard and timer work meanwhile.';
  } else if (ai === 'mock') {
    b.hidden = false;
    b.classList.add('info');
    b.textContent = 'Demo mode: Aria gives canned replies and no API calls are made. Set ANTHROPIC_API_KEY for real interviews.';
  } else b.hidden = true;
}

async function init() {
  try {
    const [qs, health] = await Promise.all([fetch('/api/questions').then((r) => r.json()), fetch('/api/health').then((r) => r.json())]);
    state.questions = qs;
    state.health = health;
  } catch {
    els.title.textContent = 'Could not reach the HLD Room server';
    return;
  }
  renderBanner();
  let last = null;
  try { last = localStorage.getItem(STORE + 'last'); } catch { /* ignore */ }
  const id = state.questions.some((q) => q.id === last) ? last : state.questions[0].id;
  state.qid = id;
  state.sessions.set(id, load(question()));
  wb.setBoard(session().board);
  renderAll();
}

init();

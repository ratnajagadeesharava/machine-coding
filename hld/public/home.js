// Home: choose a question from the bank, write your own, or open a blank whiteboard. Nothing here talks to the AI.
import {
  LEVELS, LIMITS, STYLES, deleteCustom, getCustoms, getLaunch, listSessions, saveCustom, setLaunch, validateCustom,
} from '/store.js';

const $ = (sel) => document.querySelector(sel);
const state = { questions: [], diff: 'All', category: 'All', text: '' };

function node(tag, cls, text) {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (text != null) n.textContent = text;
  return n;
}
function fillSelect(select, items, value) {
  select.replaceChildren(...items.map((i) => { const o = node('option', '', i.label); o.value = i.id; return o; }));
  select.value = value;
}
const clock = (ms) => {
  const s = Math.max(0, Math.ceil(ms / 1000));
  return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
};
const levelLabel = (id) => LEVELS.find((l) => l.id === id)?.label || id;
const styleLabel = (id) => STYLES.find((s) => s.id === id)?.label || id;

function askConfirm(text, yesLabel) {
  const dlg = $('#confirm');
  return new Promise((resolve) => {
    $('#confirm-text').textContent = text;
    $('#confirm-yes').textContent = yesLabel;
    const done = (v) => { dlg.close(); resolve(v); };
    $('#confirm-yes').onclick = () => done(true);
    $('#confirm-no').onclick = () => done(false);
    dlg.oncancel = () => resolve(false);
    dlg.showModal();
  });
}

// ---------- health notice ----------
async function showHealth() {
  const banner = $('#banner');
  const notice = $('#notice');
  try {
    const health = await (await fetch('/api/health')).json();
    if (health.ai === 'off') {
      banner.hidden = false;
      banner.textContent = 'Aria (the AI interviewer) is not connected, so chat and feedback are unavailable. The whiteboard and timer still work. Add ANTHROPIC_API_KEY to hld/.env and restart, or run "npm run demo".';
    } else if (health.ai === 'mock') {
      notice.hidden = false;
      notice.textContent = 'Demo mode: Aria gives canned replies and no API calls are made.';
    }
  } catch {
    banner.hidden = false;
    banner.textContent = 'Could not reach the HLD Room server, so the AI interviewer is unavailable. The whiteboard and timer still work.';
  }
}

// ---------- continue strip ----------
function renderContinue() {
  const sessions = listSessions().slice(0, 6);
  $('#continue').hidden = !sessions.length;
  $('#continue-list').replaceChildren(...sessions.map((s) => {
    const a = node('a', 'hm-resume');
    a.href = s.href;
    a.append(node('b', '', s.title), node('span', '', s.status === 'ended' ? 'Finished - open feedback and board' : `In progress - ${clock(s.remainingMs)} left`));
    return a;
  }));
}

// ---------- question bank ----------
function chip(text, cls = '') { return node('span', `chip-pill ${cls}`.trim(), text); }

function renderGrid() {
  const t = state.text.trim().toLowerCase();
  const items = state.questions.filter((q) =>
    (state.diff === 'All' || q.difficulty === state.diff)
    && (state.category === 'All' || q.category === state.category)
    && (!t || q.title.toLowerCase().includes(t) || q.category.toLowerCase().includes(t) || q.prompt.toLowerCase().includes(t)));
  $('#q-count').textContent = `${items.length} of ${state.questions.length} questions`;
  const grid = $('#q-grid');
  if (!items.length) { grid.replaceChildren(node('p', 'hm-empty', 'No questions match your filters.')); return; }
  grid.replaceChildren(...items.map((q) => {
    const b = node('button', 'bq-card');
    b.type = 'button';
    b.dataset.id = q.id;
    const meta = node('span', 'bq-meta');
    meta.append(chip(q.difficulty, q.difficulty), chip(`${q.minutes} min`, 'plain'), chip(q.category, 'plain'));
    b.append(node('span', 'bq-title', q.title), node('span', 'bq-prompt', q.prompt), meta);
    b.addEventListener('click', () => openStart(q));
    return b;
  }));
}

// ---------- pre-start dialog ----------
const dlg = $('#start-dialog');
let pending = null;

function openStart(q) {
  pending = q;
  const launch = getLaunch(q.id, { minutes: q.minutes });
  $('#sd-title').textContent = q.title;
  $('#sd-chips').replaceChildren(chip(q.difficulty, q.difficulty), chip(q.category, 'plain'), chip(`suggested ${q.minutes} min`, 'plain'));
  $('#sd-prompt').textContent = q.prompt;
  $('#sd-assume').replaceChildren(...q.assumptions.map((a) => node('li', '', a)));
  fillSelect($('#sd-level'), LEVELS, launch.level);
  fillSelect($('#sd-style'), STYLES, launch.style);
  $('#sd-minutes').value = String(launch.minutes);
  hints();
  dlg.showModal();
  $('#sd-start').focus();
}
function hints() {
  $('#sd-level-hint').textContent = LEVELS.find((l) => l.id === $('#sd-level').value)?.hint || '';
  $('#sd-style-hint').textContent = STYLES.find((s) => s.id === $('#sd-style').value)?.hint || '';
}
$('#sd-level').addEventListener('change', hints);
$('#sd-style').addEventListener('change', hints);
const closeStart = () => dlg.close();
$('#sd-close').addEventListener('click', closeStart);
$('#sd-cancel').addEventListener('click', closeStart);
$('#sd-form').addEventListener('submit', (e) => {
  e.preventDefault();
  if (!pending) return;
  const minutes = Number($('#sd-minutes').value);
  const field = $('#sd-minutes');
  if (!Number.isFinite(minutes) || minutes < LIMITS.minutes[0] || minutes > LIMITS.minutes[1]) {
    field.setCustomValidity(`Choose ${LIMITS.minutes[0]}-${LIMITS.minutes[1]} minutes.`);
    field.reportValidity();
    field.addEventListener('input', () => field.setCustomValidity(''), { once: true });
    return;
  }
  setLaunch(pending.id, { level: $('#sd-level').value, style: $('#sd-style').value, minutes });
  location.href = `/room.html?q=${encodeURIComponent(pending.id)}`;
});

// ---------- custom question form ----------
const form = $('#custom-form');
const F = {
  id: $('#c-id'), title: $('#c-title'), prompt: $('#c-prompt'), assumptions: $('#c-assumptions'), time: $('#c-time'),
  minutes: $('#c-minutes'), level: $('#c-level'), style: $('#c-style'), focus: $('#c-focus'),
};
fillSelect(F.level, LEVELS, 'senior');
fillSelect(F.style, STYLES, 'balanced');

function syncTime() {
  const custom = F.time.value === 'custom';
  $('#c-minutes-wrap').hidden = !custom;
  if (!custom) F.minutes.value = F.time.value;
}
F.time.addEventListener('change', syncTime);

function formValues() {
  return {
    title: F.title.value, prompt: F.prompt.value, assumptions: F.assumptions.value, minutes: F.minutes.value,
    level: F.level.value, style: F.style.value, focus: F.focus.value,
  };
}
function showErrors(errors) {
  for (const key of ['title', 'prompt', 'assumptions', 'minutes', 'focus']) {
    const msg = errors[key] || '';
    $(`#e-${key}`).textContent = msg;
    const input = F[key];
    if (input) input.setAttribute('aria-invalid', String(!!msg));
  }
  const first = ['title', 'prompt', 'assumptions', 'minutes', 'focus'].find((k) => errors[k]);
  if (first) (first === 'minutes' && F.time.value !== 'custom' ? F.time : F[first]).focus();
}
function resetForm() {
  form.reset();
  F.id.value = '';
  fillSelect(F.level, LEVELS, 'senior');
  fillSelect(F.style, STYLES, 'balanced');
  F.time.value = '45';
  syncTime();
  showErrors({});
  $('#c-reset').hidden = true;
  $('#write-title').textContent = 'WRITE YOUR OWN QUESTION';
}
function editCustom(c) {
  resetForm();
  F.id.value = c.id;
  F.title.value = c.title;
  F.prompt.value = c.prompt;
  F.assumptions.value = c.assumptions.join('\n');
  F.focus.value = c.focus;
  fillSelect(F.level, LEVELS, c.level);
  fillSelect(F.style, STYLES, c.style);
  F.minutes.value = String(c.minutes);
  F.time.value = TIME_OPTIONS.has(String(c.minutes)) ? String(c.minutes) : 'custom';
  syncTime();
  F.minutes.value = String(c.minutes);
  $('#c-reset').hidden = false;
  $('#write-title').textContent = 'EDIT YOUR QUESTION';
  $('#write').scrollIntoView({ block: 'start' });
  F.title.focus();
}
const TIME_OPTIONS = new Set([...F.time.options].map((o) => o.value).filter((v) => v !== 'custom'));

function submit(start) {
  const { errors, question } = validateCustom(formValues());
  showErrors(errors);
  if (Object.keys(errors).length) return;
  const saved = saveCustom({ ...question, id: F.id.value || undefined });
  if (!saved) {
    $('#e-prompt').textContent = 'Could not save in this browser (storage is full or blocked).';
    return;
  }
  if (start) { location.href = `/room.html?custom=${encodeURIComponent(saved.id)}`; return; }
  resetForm();
  renderCustoms();
}
form.addEventListener('submit', (e) => { e.preventDefault(); submit(true); });
$('#c-save').addEventListener('click', () => submit(false));
$('#c-reset').addEventListener('click', resetForm);

function renderCustoms() {
  const list = getCustoms();
  const box = $('#customs-list');
  if (!list.length) { box.replaceChildren(node('p', 'hm-empty', 'Questions you write are saved here, in this browser.')); return; }
  box.replaceChildren(...list.map((c) => {
    const row = node('div', 'hm-custom');
    row.dataset.id = c.id;
    const chips = node('div', 'hm-chips');
    chips.append(chip(`${c.minutes} min`, 'plain'), chip(levelLabel(c.level), 'plain'), chip(styleLabel(c.style), 'plain'));
    const actions = node('div', 'hm-custom-actions');
    const start = node('a', 'btn small primary', 'Start');
    start.href = `/room.html?custom=${encodeURIComponent(c.id)}`;
    const edit = node('button', 'btn small', 'Edit');
    edit.type = 'button';
    edit.addEventListener('click', () => editCustom(c));
    const del = node('button', 'btn small danger', 'Delete');
    del.type = 'button';
    del.addEventListener('click', async () => {
      if (!(await askConfirm(`Delete "${c.title}" and its saved board and chat?`, 'Delete'))) return;
      deleteCustom(c.id);
      if (F.id.value === c.id) resetForm();
      renderCustoms();
      renderContinue();
    });
    actions.append(start, edit, del);
    row.append(node('b', '', c.title), node('p', '', c.prompt), chips, actions);
    return row;
  }));
}

// ---------- wiring ----------
$('#q-search').addEventListener('input', (e) => { state.text = e.target.value; renderGrid(); });
$('#q-category').addEventListener('change', (e) => { state.category = e.target.value; renderGrid(); });
$('#q-diff').addEventListener('click', (e) => {
  const b = e.target.closest('button[data-diff]');
  if (!b) return;
  state.diff = b.dataset.diff;
  $('#q-diff').querySelectorAll('button').forEach((x) => { x.classList.toggle('on', x === b); x.setAttribute('aria-pressed', String(x === b)); });
  renderGrid();
});
$('#entry-write').addEventListener('click', () => setTimeout(() => F.title.focus({ preventScroll: true }), 300));
$('#entry-pick').addEventListener('click', () => setTimeout(() => $('#q-search').focus({ preventScroll: true }), 300));

async function init() {
  syncTime();
  renderContinue();
  renderCustoms();
  showHealth();
  try {
    const res = await fetch('/api/questions');
    if (!res.ok) throw new Error('bad status');
    state.questions = await res.json();
  } catch {
    $('#q-grid').replaceChildren(node('p', 'hm-empty', 'Could not load the question bank. You can still write your own question or open the whiteboard.'));
    return;
  }
  const cats = [...new Set(state.questions.map((q) => q.category))].sort();
  $('#q-category').append(...cats.map((c) => { const o = node('option', '', c); o.value = c; return o; }));
  renderGrid();
}

window.addEventListener('pageshow', (e) => { if (e.persisted) { renderContinue(); renderCustoms(); } });
init();

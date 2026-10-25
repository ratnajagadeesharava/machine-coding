// HLD Room quiz page: pick a quiz, answer with instant (or end-of-quiz) feedback, ask Claude to explain, review results,
// and generate new quizzes with Claude. Progress lives in this browser's localStorage.
import './apikey.js';
import {
  isCorrect, numberFor, mulberry32, pickQuestions, presentQuestion, scoreAttempt, validateQuiz,
} from '/shared/quiz.js';

const $ = (sel) => document.querySelector(sel);
const STORE = 'hld-quiz:v1:';
const QUICK_COUNT = 10;

const store = {
  get(key, fallback) {
    try { const v = JSON.parse(localStorage.getItem(STORE + key)); return v ?? fallback; } catch { return fallback; }
  },
  set(key, value) {
    try { localStorage.setItem(STORE + key, JSON.stringify(value)); } catch { /* storage unavailable: the page still works for this visit */ }
  },
};

const S = { history: [], summaries: [], custom: [], attempts: {}, health: { ai: 'off' }, full: new Map(), run: null, view: 'home' };

const els = {
  title: $('#page-title'), sub: $('#page-sub'), quit: $('#btn-quit'), banner: $('#banner'),
  views: { home: $('#view-home'), run: $('#view-run'), result: $('#view-result') },
  cards: $('#cards'),
  genForm: $('#gen-form'), genTopic: $('#gen-topic'), genNotes: $('#gen-notes'), genCount: $('#gen-count'), genLevel: $('#gen-level'), genBtn: $('#gen-btn'), genStatus: $('#gen-status'),
  fill: $('#progress-fill'), progress: document.querySelector('.progress'), count: $('#run-count'), instant: $('#instant'),
  concept: $('#q-concept'), diff: $('#q-diff'), prompt: $('#q-prompt'), hint: $('#q-hint'), options: $('#q-options'),
  feedback: $('#q-feedback'), verdict: $('#q-verdict'), explain: $('#q-explain'), tutorBtn: $('#btn-tutor'), tutor: $('#tutor'),
  tutorText: $('#tutor-text'), tutorForm: $('#tutor-form'), tutorInput: $('#tutor-input'),
  check: $('#btn-check'), next: $('#btn-next'),
  ring: $('#ring'), ringPct: $('#ring-pct'), resultTitle: $('#result-title'), resultSub: $('#result-sub'), clock: $('#run-clock'), timer: $('#run-timer'), limit: $('#limit'), limitLabel: $('#limit-label'),
  retryMissed: $('#btn-retry-missed'), retake: $('#btn-retake'), home: $('#btn-home'),
  byConcept: $('#by-concept'), review: $('#review'), onlyWrong: $('#only-wrong'),
  confirm: $('#confirm'), confirmText: $('#confirm-text'), confirmYes: $('#confirm-yes'), confirmNo: $('#confirm-no'),
};

// ---------- tiny DOM helper (no innerHTML: quiz text can come from an AI) ----------
function h(tag, props = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? '' : String(v));
  }
  for (const kid of kids.flat()) if (kid != null) el.append(kid);
  return el;
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

async function streamSse(url, body, onDelta) {
  const res = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
  if (!res.ok) {
    let msg = `Request failed (${res.status}).`;
    try { msg = (await res.json()).error || msg; } catch { /* keep default */ }
    throw new Error(msg);
  }
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
      if (ev === 'delta') onDelta(data.text);
      else if (ev === 'error') throw new Error(data.message || 'The tutor could not answer.');
    }
  }
}

// ---------- views ----------
function show(view) {
  S.view = view;
  for (const [name, el] of Object.entries(els.views)) el.hidden = name !== view;
  els.quit.hidden = view !== 'run';
  if (view === 'home') { els.title.textContent = 'Quizzes'; els.sub.textContent = 'Check what you know, then see why.'; document.title = 'Quizzes - HLD Room'; }
  window.scrollTo({ top: 0 });
}

// ---------- timer ----------
// Counts up from the first question; with "Time limit" on it counts down from the quiz's estimated time and the quiz
// ends by itself at zero (unanswered questions score as wrong). Ticks are capped so a hidden tab does not rack up time.
const fmtClock = (sec) => { const s = Math.max(0, Math.round(sec)); return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`; };
const limitSec = (run) => Math.max(60, run.questions.length * 45); // 45 seconds per question
let tickId = null;
function renderClock() {
  const run = S.run;
  if (!run) return;
  const limited = els.limit.checked;
  const left = limitSec(run) - run.elapsed;
  els.clock.textContent = limited ? fmtClock(left) : fmtClock(run.elapsed);
  els.timer.classList.toggle('low', limited && left <= 60);
  els.limitLabel.textContent = `Time limit (${fmtClock(limitSec(run))})`;
}
function startTimer() {
  stopTimer();
  let last = Date.now();
  tickId = setInterval(() => {
    const run = S.run;
    if (!run || S.view !== 'run') return stopTimer();
    const now = Date.now();
    run.elapsed += Math.min(2, (now - last) / 1000);
    last = now;
    renderClock();
    if (els.limit.checked && run.elapsed >= limitSec(run)) { stopTimer(); flash('Time is up - here are your results.'); finish(); }
  }, 250);
  renderClock();
}
function stopTimer() { if (tickId) clearInterval(tickId); tickId = null; }

const modeLabel = (m) => (m === 'quick' ? `Quick ${QUICK_COUNT}` : m === 'missed' ? 'Retry missed' : 'Full quiz');
const fmtMinutes = (n) => `~${n} min`;

function allQuizzes() {
  const bank = S.summaries.map((s) => ({ ...s, custom: false }));
  const mine = S.custom.map((q) => ({
    id: q.id, title: q.title, topic: q.topic || '', description: q.description || '', source: q.source || '',
    estimatedMinutes: q.estimatedMinutes || Math.max(1, Math.round(q.questions.length * 0.7)), questionCount: q.questions.length, custom: true,
  }));
  return [...mine, ...bank];
}

function renderCards() {
  els.cards.replaceChildren();
  const list = allQuizzes();
  if (!list.length) { els.cards.append(h('p', { class: 'empty', text: 'No quizzes yet. Create one with AI on the right.' })); return; }
  for (const q of list) {
    const att = S.attempts[q.id];
    const missed = att?.last?.missed?.length || 0;
    const meta = h('div', { class: 'meta' },
      h('span', { text: `${q.questionCount} questions` }),
      h('span', { text: fmtMinutes(q.estimatedMinutes) }),
      q.topic ? h('span', { text: q.topic }) : null,
      att?.best != null ? h('span', { class: 'badge', text: `Best ${att.best}%` }) : null,
      q.custom ? h('span', { class: 'badge ai', text: 'AI generated' }) : null);
    const actions = h('div', { class: 'card-actions' },
      h('button', { class: 'btn primary small', type: 'button', text: 'Start', onclick: () => startQuiz(q.id, 'full') }),
      q.questionCount > QUICK_COUNT + 2 ? h('button', { class: 'btn small', type: 'button', text: `Quick ${QUICK_COUNT}`, onclick: () => startQuiz(q.id, 'quick') }) : null,
      missed ? h('button', { class: 'btn small', type: 'button', text: `Retry missed (${missed})`, onclick: () => startQuiz(q.id, 'missed') }) : null,
      q.custom ? h('button', { class: 'btn ghost small', type: 'button', text: 'Delete', onclick: () => deleteCustom(q) }) : null);
    els.cards.append(h('article', { class: 'card' },
      h('h3', { text: q.title }),
      q.description ? h('p', { class: 'muted', text: q.description }) : null,
      meta,
      q.source ? h('p', { class: 'muted', text: `Source: ${q.source}` }) : null,
      actions));
  }
}

async function deleteCustom(q) {
  if (!(await askConfirm(`Delete "${q.title}"? This removes it from this browser.`, 'Delete'))) return;
  S.custom = S.custom.filter((x) => x.id !== q.id);
  delete S.attempts[q.id];
  store.set('custom', S.custom);
  store.set('attempts', S.attempts);
  renderCards();
}

// ---------- starting a quiz ----------
async function loadFull(id) {
  const mine = S.custom.find((q) => q.id === id);
  if (mine) return mine;
  if (S.full.has(id)) return S.full.get(id);
  const res = await fetch(`/api/quizzes/${encodeURIComponent(id)}`);
  if (!res.ok) throw new Error('Could not load that quiz.');
  const quiz = await res.json();
  const errs = validateQuiz(quiz);
  if (errs.length) throw new Error(`That quiz is malformed: ${errs[0]}`);
  S.full.set(id, quiz);
  return quiz;
}

async function startQuiz(id, mode) {
  let quiz;
  try { quiz = await loadFull(id); } catch (err) { flash(err.message); return; }
  const seed = (Math.random() * 2 ** 32) >>> 0;
  const rand = mulberry32(seed);
  let questions = quiz.questions;
  if (mode === 'quick') questions = pickQuestions(quiz, QUICK_COUNT, rand);
  if (mode === 'missed') {
    const ids = new Set(S.attempts[id]?.last?.missed || []);
    questions = quiz.questions.filter((q) => ids.has(q.id));
    if (!questions.length) { flash('Nothing missed to retry.'); return; }
  }
  S.run = { quiz, mode, questions: questions.map((q) => presentQuestion(q, rand)), i: 0, answers: {}, checked: {}, tutor: {}, elapsed: 0 };
  els.title.textContent = quiz.title;
  els.sub.textContent = `${modeLabel(mode)} · ${S.run.questions.length} questions`;
  document.title = `${quiz.title} - Quizzes - HLD Room`;
  show('run');
  renderQuestion();
  startTimer();
}

function flash(message) {
  els.banner.hidden = false;
  els.banner.classList.remove('info');
  els.banner.textContent = message;
  setTimeout(() => { if (els.banner.textContent === message) renderBanner(); }, 4000);
}

// ---------- answering ----------
const cur = () => S.run.questions[S.run.i];
const selectedIds = () => S.run.answers[cur().id] || [];

function renderQuestion() {
  const run = S.run;
  const q = cur();
  const total = run.questions.length;
  const checked = !!run.checked[q.id];
  const sel = new Set(selectedIds());
  els.fill.style.width = `${(run.i / total) * 100}%`;
  els.progress.setAttribute('aria-valuenow', String(Math.round((run.i / total) * 100)));
  els.count.textContent = `Question ${run.i + 1} of ${total}`;
  els.concept.textContent = q.concept;
  els.diff.replaceChildren(...[1, 2, 3].map((n) => h('i', { class: n <= q.difficulty ? 'on' : '' })));
  els.diff.setAttribute('aria-label', `Difficulty ${q.difficulty} of 3`);
  els.prompt.textContent = q.prompt;
  els.hint.textContent = q.type === 'multi' ? 'Select all that apply.' : q.type === 'truefalse' ? 'True or false?' : '';
  els.hint.hidden = !els.hint.textContent;

  els.options.replaceChildren(...q.options.map((o, idx) => {
    const isRight = q.correct.includes(o.id);
    const picked = sel.has(o.id);
    let cls = 'opt' + (q.type === 'multi' ? ' multi' : '') + (picked && !checked ? ' on' : '');
    if (checked) cls += isRight && picked ? ' right' : isRight ? ' missed' : picked ? ' wrong' : '';
    const label = checked ? (isRight ? '✓' : picked ? '✕' : numberFor(idx)) : numberFor(idx);
    return h('button', {
      class: cls, type: 'button', 'data-id': o.id, role: q.type === 'multi' ? 'checkbox' : 'radio', 'aria-checked': String(picked), disabled: checked,
      onclick: () => choose(o.id),
    }, h('span', { class: 'key', text: label }),
    h('span', { class: 'txt' }, o.text, checked && o.why ? h('span', { class: 'why', text: o.why }) : null));
  }));

  els.feedback.hidden = !checked;
  els.check.hidden = checked;
  els.check.disabled = sel.size === 0;
  els.check.textContent = 'Check answer';
  els.next.hidden = !checked && els.instant.checked;
  els.next.disabled = !checked && sel.size === 0;
  els.next.textContent = run.i === total - 1 ? 'See results' : 'Next';
  if (!els.instant.checked && !checked) { els.check.hidden = true; }
  if (checked) {
    const ok = isCorrect(q, [...sel]);
    els.verdict.textContent = ok ? 'Correct!' : q.type === 'multi' && [...sel].some((id) => q.correct.includes(id)) ? 'Partly right - you need exactly the right set.' : 'Not quite.';
    els.verdict.className = `verdict-line ${ok ? 'ok' : 'no'}`;
    els.explain.textContent = q.explanation;
    renderTutor();
  }
}

function choose(id) {
  const q = cur();
  if (S.run.checked[q.id]) return;
  const now = new Set(selectedIds());
  if (q.type === 'multi') { if (now.has(id)) now.delete(id); else now.add(id); } else { now.clear(); now.add(id); }
  S.run.answers[q.id] = [...now];
  renderQuestion();
  els.options.querySelector(`[data-id="${id}"]`)?.focus();
}

function check() {
  const q = cur();
  if (!selectedIds().length || S.run.checked[q.id]) return;
  S.run.checked[q.id] = true;
  renderQuestion();
  els.next.focus();
}

function next() {
  const run = S.run;
  if (!els.instant.checked && !selectedIds().length) return;
  if (run.i < run.questions.length - 1) { run.i += 1; renderQuestion(); els.options.querySelector('.opt')?.focus(); } else finish();
}

els.check.addEventListener('click', check);
els.next.addEventListener('click', next);
els.limit.addEventListener('change', () => { store.set('limit', els.limit.checked); renderClock(); });
els.instant.addEventListener('change', () => { store.set('instant', els.instant.checked); if (S.run) renderQuestion(); });
els.quit.addEventListener('click', async () => {
  if (await askConfirm('Quit this quiz? Your answers so far will not be saved.', 'Quit quiz')) { stopTimer(); S.run = null; show('home'); renderCards(); }
});

document.addEventListener('keydown', (e) => {
  if (S.view !== 'run' || e.ctrlKey || e.metaKey || e.altKey) return;
  if (document.querySelector('dialog[open]')) return;
  const t = e.target;
  if (t && (t.tagName === 'INPUT' && t.type !== 'checkbox' || t.tagName === 'TEXTAREA')) return;
  const q = cur();
  const pick = /^[1-6]$/.test(e.key) ? Number(e.key) - 1 : -1;
  if (pick >= 0 && pick < q.options.length) { e.preventDefault(); choose(q.options[pick].id); return; }
  // Enter on an answer option (focus stays there after choosing) still checks / moves on; on the other buttons it presses them.
  if (e.key === 'Enter' && (t?.tagName !== 'BUTTON' || t.classList.contains('opt'))) {
    e.preventDefault();
    if (!S.run.checked[q.id] && els.instant.checked) check(); else if (!els.next.hidden) next();
  }
});

// ---------- tutor ----------
function renderTutor() {
  const q = cur();
  const t = S.run.tutor[q.id];
  const aiOff = S.health.ai === 'off';
  els.tutorBtn.hidden = !!t;
  els.tutorBtn.disabled = aiOff;
  els.tutorBtn.title = aiOff ? 'Needs the AI connection (add your key with the key button, or run demo mode)' : '';
  els.tutor.hidden = !t;
  els.tutorText.replaceChildren();
  if (!t) return;
  for (const m of t.log) {
    if (m.role === 'user') els.tutorText.append(h('span', { class: 'q', text: `You: ${m.content}` }));
    else els.tutorText.append(h('span', { class: m.error ? 'err' : '', text: m.content }));
  }
}

async function askTutor(followup) {
  const q = cur();
  const run = S.run;
  const t = run.tutor[q.id] || (run.tutor[q.id] = { log: [], busy: false });
  if (t.busy) return;
  t.busy = true;
  const history = t.log.filter((m) => !m.error).map((m) => ({ role: m.role, content: m.content }));
  if (followup) { t.log.push({ role: 'user', content: followup }); history.push({ role: 'user', content: followup }); }
  const reply = { role: 'assistant', content: '' };
  t.log.push(reply);
  renderTutor();
  els.tutorInput.disabled = true;
  try {
    await streamSse('/api/quiz/explain', {
      question: { id: q.id, concept: q.concept, type: q.type, difficulty: q.difficulty, prompt: q.prompt, options: q.options, correct: q.correct, explanation: q.explanation },
      selected: selectedIds(), history,
    }, (d) => { reply.content += d; if (S.run === run && cur() === q) renderTutor(); });
    if (!reply.content.trim()) throw new Error('The tutor sent an empty reply. Try again.');
  } catch (err) {
    if (reply.content.trim()) { reply.content = reply.content.trim(); t.log.push({ role: 'assistant', content: err.message, error: true }); } else { reply.content = err.message; reply.error = true; }
  } finally {
    t.busy = false;
    els.tutorInput.disabled = false;
    if (S.run === run && cur() === q) renderTutor();
  }
}

els.tutorBtn.addEventListener('click', () => askTutor(null));
els.tutorForm.addEventListener('submit', (e) => {
  e.preventDefault();
  const text = els.tutorInput.value.trim();
  if (!text) return;
  els.tutorInput.value = '';
  askTutor(text);
});

// ---------- results ----------
function recordAttempt(result) {
  const run = S.run;
  const prev = S.attempts[run.quiz.id] || { best: null, count: 0, last: null };
  const asked = new Set(run.questions.map((q) => q.id));
  const carried = (prev.last?.missed || []).filter((id) => !asked.has(id));
  const missed = [...carried, ...result.missed];
  S.attempts[run.quiz.id] = {
    best: run.mode === 'full' ? Math.max(prev.best ?? 0, result.percent) : prev.best,
    count: prev.count + 1,
    last: { seconds: result.seconds, percent: result.percent, score: result.score, total: result.total, mode: run.mode, at: Date.now(), missed },
  };
  store.set('attempts', S.attempts);
  // Every finished attempt is kept (newest first, last 100) so results survive reloads and can be reviewed later.
  S.history = [{ quizId: run.quiz.id, title: run.quiz.title, at: Date.now(), seconds: result.seconds, mode: run.mode, percent: result.percent, score: result.score, total: result.total,
    weak: result.byConcept.filter((c) => c.correct < c.total).map((c) => c.concept || c.name || c.label).filter(Boolean).slice(0, 5) }, ...S.history].slice(0, 100);
  store.set('history', S.history);
  renderHistory();
}

function renderHistory() {
  const box = document.getElementById('history');
  if (!box) return;
  box.hidden = S.history.length === 0;
  const rows = S.history.slice(0, 8).map((r) => h('li', { class: 'hist-row' },
    h('span', { class: 'hist-score' + (r.percent >= 70 ? ' good' : r.percent < 50 ? ' low' : ''), text: `${r.percent}%` }),
    h('span', { class: 'hist-main' }, h('b', { text: r.title }), h('span', { class: 'muted', text: `${r.score}/${r.total} · ${modeLabel(r.mode)}${r.seconds != null ? ` · ${fmtClock(r.seconds)}` : ''} · ${new Date(r.at).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' })}` }))));
  box.replaceChildren(
    h('div', { class: 'hist-head' }, h('h2', { class: 'eyebrow', text: 'RECENT RESULTS' }),
      h('button', { class: 'btn ghost small', type: 'button', text: 'Clear', onclick: () => { S.history = []; store.set('history', []); renderHistory(); } })),
    h('ul', { class: 'hist-list' }, ...rows));
}

function finish() {
  const run = S.run;
  stopTimer();
  const result = scoreAttempt({ questions: run.questions }, run.answers);
  run.result = result;
  result.seconds = Math.round(run.elapsed);
  recordAttempt(result);
  els.fill.style.width = '100%';
  els.title.textContent = run.quiz.title;
  els.sub.textContent = 'Results';
  const p = result.percent;
  els.ring.style.setProperty('--p', String(p));
  els.ring.setAttribute('aria-label', `Score ${p} percent`);
  els.ringPct.textContent = `${p}%`;
  els.resultTitle.textContent = p >= 90 ? 'Excellent' : p >= 70 ? 'Solid' : p >= 50 ? 'Getting there' : 'Keep practising';
  els.resultSub.textContent = `${result.score} of ${result.total} correct · ${modeLabel(run.mode)} · ${fmtClock(result.seconds)}`;
  const missedAll = S.attempts[run.quiz.id].last.missed;
  els.retryMissed.hidden = missedAll.length === 0;
  els.retryMissed.textContent = `Retry missed (${missedAll.length})`;
  els.byConcept.replaceChildren(...result.byConcept.map((c) => {
    const pct = Math.round((c.correct / c.total) * 100);
    return h('div', { class: 'bc-row' },
      h('div', { class: 'bc-top' }, h('span', { text: c.concept }), h('span', { class: 'muted', text: `${c.correct}/${c.total}` })),
      h('div', { class: 'bc-bar' }, h('i', { class: pct < 50 ? 'low' : pct < 80 ? 'mid' : '', style: `width:${pct}%` })));
  }));
  els.onlyWrong.checked = result.missed.length > 0 && result.missed.length < result.total;
  renderReview();
  show('result');
  els.title.textContent = run.quiz.title;
  els.sub.textContent = 'Results';
}

function textFor(q, ids) {
  const picked = q.options.map((o, i) => ({ o, i })).filter(({ o }) => ids.includes(o.id));
  return picked.length ? picked.map(({ o, i }) => `${numberFor(i)}. ${o.text}`).join('  |  ') : '(no answer)';
}

function renderReview() {
  const run = S.run;
  const onlyWrong = els.onlyWrong.checked;
  const rows = run.questions.map((q, n) => ({ q, n, ok: isCorrect(q, run.answers[q.id]) })).filter((r) => !onlyWrong || !r.ok);
  els.review.replaceChildren();
  if (!rows.length) { els.review.append(h('p', { class: 'empty', text: 'Nothing wrong here. Nice.' })); return; }
  for (const { q, n, ok } of rows) {
    els.review.append(h('article', { class: `rv${ok ? '' : ' bad'}` },
      h('h4', { text: `${n + 1}. ${q.prompt}` }),
      h('p', { class: 'ans' }, h('b', { text: 'Your answer: ' }), textFor(q, run.answers[q.id] || [])),
      ok ? null : h('p', { class: 'ans' }, h('b', { text: 'Correct: ' }), textFor(q, q.correct)),
      h('p', { class: 'ex', text: q.explanation })));
  }
}

els.onlyWrong.addEventListener('change', () => { if (S.run) renderReview(); });
els.retake.addEventListener('click', () => startQuiz(S.run.quiz.id, S.run.mode === 'missed' ? 'full' : S.run.mode));
els.retryMissed.addEventListener('click', () => startQuiz(S.run.quiz.id, 'missed'));
els.home.addEventListener('click', () => { S.run = null; show('home'); renderCards(); });

// ---------- generator ----------
els.genForm.addEventListener('submit', async (e) => {
  e.preventDefault();
  const status = els.genStatus;
  const setStatus = (text, cls = '') => { status.textContent = text; status.className = `gen-status ${cls}`; };
  if (S.health.ai === 'off') { setStatus('Aria is not connected. Add your Claude key with the key button (top right), or use demo mode.', 'err'); return; }
  els.genBtn.disabled = true;
  setStatus('Claude is writing your quiz… this can take up to a minute.');
  try {
    const res = await fetch('/api/quiz/generate', {
      method: 'POST', headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ topic: els.genTopic.value, notes: els.genNotes.value, count: Number(els.genCount.value), level: els.genLevel.value }),
    });
    const body = await res.json().catch(() => ({}));
    if (!res.ok) throw new Error(body.error || `Request failed (${res.status}).`);
    const errs = validateQuiz(body.quiz);
    if (errs.length) throw new Error(`The generated quiz was malformed (${errs[0]}). Please try again.`);
    S.custom = [body.quiz, ...S.custom].slice(0, 20);
    store.set('custom', S.custom);
    renderCards();
    setStatus(`Created "${body.quiz.title}" with ${body.quiz.questions.length} questions${body.dropped ? ` (${body.dropped} weak question${body.dropped > 1 ? 's were' : ' was'} discarded)` : ''}.`, 'ok');
    els.genTopic.value = '';
  } catch (err) {
    setStatus(err.message, 'err');
  } finally {
    els.genBtn.disabled = false;
  }
});

// ---------- boot ----------
function renderBanner() {
  const ai = S.health.ai;
  els.banner.classList.remove('info');
  if (ai === 'off') {
    els.banner.hidden = false;
    els.banner.textContent = 'AI is not connected, so "Explain more" and "Create a quiz with AI" are disabled. The quizzes themselves work fine. See hld/README.md to add your API key.';
  } else if (ai === 'mock') {
    els.banner.hidden = false;
    els.banner.classList.add('info');
    els.banner.textContent = 'Demo mode: AI explanations and generated quizzes are canned. Add your Claude key (top right) for the real thing.';
  } else els.banner.hidden = true;
}

async function init() {
  S.attempts = store.get('attempts', {});
  S.history = (store.get('history', []) || []).filter((r) => r && typeof r.title === 'string' && Number.isFinite(r.percent));
  els.instant.checked = store.get('instant', true);
  els.limit.checked = store.get('limit', false) === true;
  S.custom = (store.get('custom', []) || []).filter((q) => validateQuiz(q).length === 0);
  try {
    const [list, health] = await Promise.all([fetch('/api/quizzes').then((r) => r.json()), fetch('/api/health').then((r) => r.json())]);
    S.summaries = list;
    S.health = health;
  } catch {
    els.banner.hidden = false;
    els.banner.textContent = 'Could not reach the HLD Room server. Start it with "npm start" in the hld folder.';
  }
  renderBanner();
  els.genBtn.title = S.health.ai === 'off' ? 'Needs the AI connection' : '';
  show('home');
  renderCards();
  renderHistory();
}

init();

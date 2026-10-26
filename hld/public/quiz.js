// HLD Room quiz page: pick a quiz, answer with instant (or end-of-quiz) feedback, ask Claude to explain, review results,
// and generate new quizzes with Claude. Progress lives in this browser's localStorage.
import './apikey.js';
import {
  isCorrect, numberFor, mulberry32, pickQuestions, presentQuestion, scoreAttempt, shuffle, validateQuiz,
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
  cards: $('#cards'), toc: $('#toc'), stats: $('#stats'), search: $('#quiz-search'), noMatch: $('#no-match'), heading: $('#list-heading'),
  genForm: $('#gen-form'), genTopic: $('#gen-topic'), genNotes: $('#gen-notes'), genCount: $('#gen-count'), genLevel: $('#gen-level'), genBtn: $('#gen-btn'), genStatus: $('#gen-status'),
  fill: $('#progress-fill'), progress: document.querySelector('.progress'), count: $('#run-count'), instant: $('#instant'),
  concept: $('#q-concept'), diff: $('#q-diff'), prompt: $('#q-prompt'), hint: $('#q-hint'), options: $('#q-options'),
  feedback: $('#q-feedback'), verdict: $('#q-verdict'), explain: $('#q-explain'), tutorBtn: $('#btn-tutor'), tutor: $('#tutor'),
  tutorText: $('#tutor-text'), tutorForm: $('#tutor-form'), tutorInput: $('#tutor-input'),
  check: $('#btn-check'), next: $('#btn-next'),
  ring: $('#ring'), ringPct: $('#ring-pct'), resultTitle: $('#result-title'), resultSub: $('#result-sub'), clock: $('#run-clock'), timer: $('#run-timer'), limit: $('#limit'), limitLabel: $('#limit-label'),
  retryMissed: $('#btn-retry-missed'), retake: $('#btn-retake'), home: $('#btn-home'),
  byConcept: $('#by-concept'), review: $('#review'), onlyWrong: $('#only-wrong'),
  map: $('#q-map'), prev: $('#btn-prev'), flag: $('#btn-flag'), finish: $('#btn-finish'), upnext: $('#upnext'),
  confirm: $('#confirm'), confirmText: $('#confirm-text'), confirmYes: $('#confirm-yes'), confirmNo: $('#confirm-no'),
};

// ---------- tiny DOM helper (no innerHTML: quiz text can come from an AI) ----------
function h(tag, props = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k === 'dataset') Object.assign(el.dataset, v);
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

const modeLabel = (m) => (m === 'weak' ? 'Weak spots' : m === 'quick' ? `Quick ${QUICK_COUNT}` : m === 'missed' ? 'Retry missed' : 'Full quiz');
const ago = (t) => {
  const m = Math.round((Date.now() - t) / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m} min ago`;
  const hrs = Math.round(m / 60);
  if (hrs < 24) return `${hrs} h ago`;
  const d = Math.round(hrs / 24);
  return d === 1 ? 'yesterday' : `${d} days ago`;
};
const fmtMinutes = (n) => `~${n} min`;

function allQuizzes() {
  const bank = S.summaries.map((s) => ({ ...s, custom: false }));
  const mine = S.custom.map((q) => ({
    id: q.id, title: q.title, topic: q.topic || '', description: q.description || '', source: q.source || '',
    estimatedMinutes: q.estimatedMinutes || Math.max(1, Math.round(q.questions.length * 0.7)), questionCount: q.questions.length, custom: true,
  }));
  return [...mine, ...bank];
}

function quizCard(q, inSeries) {
  const att = S.attempts[q.id];
  const last = att?.last;
  const run = savedRun();
  const inProgress = run && run.quizId === q.id;
  const missed = att?.last?.missed?.length || 0;
  const meta = h('div', { class: 'meta' },
    h('span', { text: `${q.questionCount} questions` }),
    h('span', { text: fmtMinutes(q.estimatedMinutes) }),
    q.topic && !inSeries ? h('span', { text: q.topic }) : null,
    att?.best != null ? h('span', { class: 'badge', text: `Best ${att.best}%` }) : null,
    q.custom ? h('span', { class: 'badge ai', text: 'AI generated' }) : null);
  const actions = h('div', { class: 'card-actions' },
    h('button', { class: inProgress || !(att?.best != null || last) ? 'btn primary small' : 'btn small', type: 'button', text: inProgress ? 'Resume' : att?.best != null || last ? 'Retake' : 'Start', onclick: () => (inProgress ? resumeRun() : startQuiz(q.id, 'full')) }),
    q.questionCount > QUICK_COUNT + 2 ? h('button', { class: 'btn small', type: 'button', text: `Quick ${QUICK_COUNT}`, onclick: () => startQuiz(q.id, 'quick') }) : null,
    missed ? h('button', { class: 'btn small', type: 'button', text: `Retry missed (${missed})`, onclick: () => startQuiz(q.id, 'missed') }) : null,
    q.custom ? h('button', { class: 'btn ghost small', type: 'button', text: 'Delete', onclick: () => deleteCustom(q) }) : null);
  const status = inProgress ? h('span', { class: 'status prog', text: 'In progress' })
    : att?.best != null || last ? h('span', { class: 'status done', title: last ? `Last attempt ${new Date(last.at).toLocaleString()}` : '' }, '✓ Completed',
      last ? h('span', { class: 'status-detail', text: ` · last ${last.percent}% · ${ago(last.at)}` }) : null)
    : h('span', { class: 'status new', text: 'Not started' });
  return h('article', { class: 'card' + (inSeries ? ' in-series' : '') + (inProgress ? ' is-progress' : att?.best != null || last ? ' is-done' : ''), id: `quiz-${q.id}`, dataset: { quiz: q.id, search: `${q.title} ${q.description} ${q.topic}`.toLowerCase() } },
    h('div', { class: 'card-top' }, h('h3', { text: q.title }), status),
    q.description ? h('p', { class: 'muted', text: q.description }) : null,
    meta,
    q.source && !inSeries ? h('p', { class: 'muted', text: `Source: ${q.source}` }) : null,
    actions);
}

// ---------- topics ----------
// The home page is an index of topics (a series of sub-topic quizzes, a single quiz, or your own generated quizzes).
// Opening a topic (#/topic/<id>) shows just its quizzes, so the page stays small however many quizzes exist.
function buildTopics(list) {
  const map = new Map();
  for (const q of list) {
    let id; let title; let description;
    if (q.custom) { id = 'mine'; title = 'My generated quizzes'; description = 'Quizzes you created with AI. They are saved in this browser.'; }
    else if (q.series) { id = q.series; title = q.seriesTitle; description = q.seriesDescription || ''; }
    else { id = `quiz-${q.id}`; title = q.title; description = q.description; }
    if (!map.has(id)) map.set(id, { id, title, description, items: [], single: !q.custom && !q.series });
    map.get(id).items.push(q);
  }
  const topics = [...map.values()];
  for (const t of topics) t.items.sort((x, y) => (x.seriesOrder || 0) - (y.seriesOrder || 0));
  return topics.sort((x, y) => (x.id === 'mine') - (y.id === 'mine') || x.title.localeCompare(y.title));
}
const topicStats = (t) => ({
  done: t.items.filter((q) => S.attempts[q.id]?.best != null).length,
  questions: t.items.reduce((n, q) => n + q.questionCount, 0),
  minutes: t.items.reduce((n, q) => n + q.estimatedMinutes, 0),
  running: t.items.some((q) => savedRun()?.quizId === q.id),
});
const routeTopic = () => { const m = /^#\/topic\/(.+)$/.exec(location.hash); return m ? decodeURIComponent(m[1]) : null; };
const goTopic = (id) => { location.hash = id ? `#/topic/${encodeURIComponent(id)}` : ''; if (!id && !location.hash) renderCards(); };
const searchTerm = () => (els.search?.value || '').trim().toLowerCase();
const matches = (q, t) => !searchTerm() || `${q.title} ${q.description} ${q.topic} ${t?.title || ''}`.toLowerCase().includes(searchTerm());

function topicCard(t) {
  const st = topicStats(t);
  const pct = Math.round((st.done / t.items.length) * 100);
  return h('article', { class: 'card topic-card', dataset: { topic: t.id } },
    h('div', { class: 'card-top' }, h('h3', { text: t.title }),
      st.running ? h('span', { class: 'status prog', text: 'In progress' }) : st.done === t.items.length ? h('span', { class: 'status done', text: '✓ All done' }) : st.done ? h('span', { class: 'status prog', text: `${st.done}/${t.items.length} done` }) : null),
    t.description ? h('p', { class: 'muted', text: t.description }) : null,
    h('div', { class: 'meta' }, h('span', { text: `${t.items.length} quizzes` }), h('span', { text: `${st.questions} questions` }), h('span', { text: fmtMinutes(st.minutes) })),
    h('div', { class: 'topic-progress' }, h('span', { class: 'progress' }, h('i', { style: `width:${pct}%` })), h('span', { class: 'muted', text: `${st.done}/${t.items.length}` })),
    h('div', { class: 'card-actions' }, h('button', { class: 'btn primary small', type: 'button', text: st.done ? 'Continue' : 'Open topic', onclick: () => goTopic(t.id) })));
}

function renderCards() {
  els.cards.replaceChildren();
  const list = allQuizzes();
  const topics = buildTopics(list);
  const open = routeTopic();
  const topic = open ? topics.find((t) => t.id === open) : null;
  if (open && !topic) { location.hash = ''; return; }
  document.body.classList.toggle('in-topic', !!topic);
  els.search.placeholder = topic ? `Search in ${topic.title}` : 'Search topics and quizzes';
  els.heading.textContent = topic ? 'QUIZZES IN THIS TOPIC' : searchTerm() ? 'SEARCH RESULTS' : 'TOPICS';
  let shown = 0;

  if (topic) {
    const st = topicStats(topic);
    els.cards.append(h('div', { class: 'topic-head' },
      h('a', { class: 'back', href: '#', onclick: (e) => { e.preventDefault(); goTopic(null); } }, '← All topics'),
      h('h2', { text: topic.title }),
      topic.description ? h('p', { class: 'muted', text: topic.description }) : null,
      h('div', { class: 'topic-progress' }, h('span', { class: 'progress' }, h('i', { style: `width:${Math.round((st.done / topic.items.length) * 100)}%` })), h('span', { class: 'muted', text: `${st.done}/${topic.items.length} done · ${st.questions} questions` }))));
    topic.items.forEach((q, i) => {
      if (!matches(q, topic)) return;
      shown += 1;
      els.cards.append(h('div', { class: 'series-item' }, h('span', { class: 'series-num' + (badgeFor(q) != null ? ' done' : ''), text: badgeFor(q) != null ? '✓' : String(i + 1) }), quizCard(q, true)));
    });
  } else if (searchTerm()) {
    for (const t of topics) {
      for (const q of t.items) {
        if (!matches(q, t)) continue;
        shown += 1;
        els.cards.append(h('div', { class: 'result-item' },
          t.single ? null : h('a', { class: 'crumb', href: `#/topic/${encodeURIComponent(t.id)}`, text: `${t.title} ›` }), quizCard(q, true)));
      }
    }
  } else {
    for (const t of topics) { shown += 1; els.cards.append(t.single ? quizCard(t.items[0], false) : topicCard(t)); }
  }
  if (!list.length) els.cards.append(h('p', { class: 'empty', text: 'No quizzes yet. Create one with AI on the right.' }));
  els.noMatch.hidden = shown > 0 || !list.length;
  renderToc(topics, topic);
  els.stats.hidden = !!topic || !!searchTerm();
  els.upnext.dataset.hide = topic || searchTerm() ? '1' : '';
  renderStats(list);
  renderUpNext();
  const t = topic ? `${topic.title} - Quizzes` : 'Quizzes';
  if (S.view === 'home') { document.title = `${t} - HLD Room`; els.title.textContent = topic ? topic.title : 'Quizzes'; els.sub.textContent = topic ? 'Pick a sub-topic quiz.' : 'Check what you know, then see why.'; }
}

// ---------- index (table of contents), stats and search ----------
const badgeFor = (q) => { const b = S.attempts[q.id]?.best; return b == null ? null : b; };
function tocLink(q, num) {
  const best = badgeFor(q);
  return h('a', { class: 'toc-item' + (best != null ? ' done' : ''), href: `#quiz-${q.id}`, dataset: { target: q.id }, onclick: (e) => { e.preventDefault(); gotoQuiz(q.id); } },
    h('span', { class: 'toc-mark', text: best != null ? '✓' : num != null ? String(num) : '' }),
    h('span', { class: 'toc-title', text: (t => t.charAt(0).toUpperCase() + t.slice(1))(q.title.replace(/^[A-Za-z ]+:\s*/, '')) }),
    best != null ? h('span', { class: 'toc-score', text: `${best}%` }) : null);
}
function renderToc(topics, topic) {
  const nav = els.toc;
  nav.replaceChildren();
  if (topic) {
    nav.append(h('a', { class: 'toc-back', href: '#', onclick: (e) => { e.preventDefault(); goTopic(null); } }, '← All topics'),
      h('div', { class: 'toc-head', text: topic.title.toUpperCase() }),
      h('div', { class: 'toc-group' }, ...topic.items.map((q, i) => tocLink(q, i + 1))));
    return;
  }
  nav.append(h('div', { class: 'toc-head', text: 'TOPICS' }));
  for (const t of topics) {
    const st = topicStats(t);
    nav.append(h('a', { class: 'toc-topic' + (st.done === t.items.length ? ' done' : ''), href: `#/topic/${encodeURIComponent(t.id)}`, onclick: (e) => { e.preventDefault(); if (t.single) gotoQuiz(t.items[0].id); else goTopic(t.id); } },
      h('span', { class: 'toc-mark', text: st.done === t.items.length ? '✓' : String(t.items.length) }),
      h('span', { class: 'toc-title', text: t.title }),
      h('span', { class: 'toc-count', text: `${st.done}/${t.items.length}` })));
  }
}
function gotoQuiz(id) {
  const q = allQuizzes().find((x) => x.id === id);
  const t = q && buildTopics(allQuizzes()).find((x) => x.items.some((i) => i.id === id));
  const scroll = () => {
    const card = document.getElementById(`quiz-${id}`);
    if (!card) return;
    card.scrollIntoView({ behavior: 'smooth', block: 'start' });
    card.classList.remove('flash'); void card.offsetWidth; card.classList.add('flash');
  };
  if (t && !t.single && routeTopic() !== t.id) { location.hash = `#/topic/${encodeURIComponent(t.id)}`; setTimeout(scroll, 60); return; }
  if (t && t.single && routeTopic()) { goTopic(null); setTimeout(scroll, 60); return; }
  scroll();
}
// One smart suggestion at the top: resume, practise weak spots, continue the series, or revisit an old quiz.
function renderUpNext() {
  const box = els.upnext;
  if (!box) return;
  const list = allQuizzes();
  const card = (kicker, title, text, actions) => box.replaceChildren(h('div', { class: 'upnext-card' },
    h('div', { class: 'upnext-body' }, h('span', { class: 'kicker', text: kicker }), h('h3', { text: title }), h('p', { class: 'muted', text })), h('div', { class: 'upnext-actions' }, ...actions)));
  const run = savedRun();
  if (run) {
    const done = Object.values(run.answers || {}).filter((a) => a.length).length;
    card('PICK UP WHERE YOU LEFT OFF', run.title || 'Quiz in progress', `${done} of ${run.order.length} answered · ${fmtClock(run.elapsed || 0)} elapsed`, [
      h('button', { class: 'btn primary', type: 'button', text: 'Resume', onclick: resumeRun }),
      h('button', { class: 'btn ghost', type: 'button', text: 'Discard', onclick: () => { clearRun(); renderUpNext(); } })]);
    box.hidden = false; return;
  }
  if (box.dataset.hide) { box.hidden = true; return; } // suggestions only on the main index; a saved run always shows
  const weak = weakRefs().length;
  if (weak >= 3) {
    card('WEAK SPOTS', `${weak} questions to revisit`, 'A mixed round of up to 15 questions you got wrong, across every quiz. Answer one right and it leaves the list.', [
      h('button', { class: 'btn primary', type: 'button', text: 'Practise weak spots', onclick: startWeak })]);
    box.hidden = false; return;
  }
  const nextInSeries = list.filter((q) => q.series && !q.custom).sort((a, b) => (a.seriesOrder || 0) - (b.seriesOrder || 0)).find((q) => badgeFor(q) == null);
  const anyDone = list.some((q) => badgeFor(q) != null);
  if (nextInSeries && anyDone) {
    card('UP NEXT', nextInSeries.title, `${nextInSeries.questionCount} questions · ${fmtMinutes(nextInSeries.estimatedMinutes)}. Next sub-topic in ${nextInSeries.seriesTitle}.`, [
      h('button', { class: 'btn primary', type: 'button', text: 'Start', onclick: () => startQuiz(nextInSeries.id, 'full') }),
      h('button', { class: 'btn', type: 'button', text: 'Show in list', onclick: () => gotoQuiz(nextInSeries.id) })]);
    box.hidden = false; return;
  }
  const stale = list.filter((q) => S.attempts[q.id]?.last && Date.now() - S.attempts[q.id].last.at > 3 * 864e5 && (S.attempts[q.id].best ?? 100) < 90)[0];
  if (stale) {
    card('TIME TO REVIEW', stale.title, `Your best was ${S.attempts[stale.id].best}% and it has been a few days. A quick round keeps it fresh.`, [
      h('button', { class: 'btn primary', type: 'button', text: `Quick ${QUICK_COUNT}`, onclick: () => startQuiz(stale.id, stale.questionCount > QUICK_COUNT + 2 ? 'quick' : 'full') })]);
    box.hidden = false; return;
  }
  box.hidden = true;
}
function streakDays() {
  const days = new Set(S.history.map((r) => new Date(r.at).toDateString()));
  let d = new Date();
  if (!days.has(d.toDateString())) d = new Date(d - 864e5); // today may not have a quiz yet; the streak is still alive
  let n = 0;
  while (days.has(d.toDateString())) { n += 1; d = new Date(d - 864e5); }
  return n;
}
function renderStats(list) {
  if (!els.stats) return;
  const done = list.filter((q) => badgeFor(q) != null);
  const avg = done.length ? Math.round(done.reduce((n, q) => n + badgeFor(q), 0) / done.length) : null;
  const questions = list.reduce((n, q) => n + q.questionCount, 0);
  const tile = (value, label) => h('div', { class: 'stat' }, h('b', { text: value }), h('span', { class: 'muted', text: label }));
  els.stats.replaceChildren(
    tile(`${done.length}/${list.length}`, 'quizzes completed'),
    tile(avg == null ? '-' : `${avg}%`, 'average best score'),
    tile(String(questions), 'questions available'),
    tile(String(streakDays()), 'day streak'));
}
const applySearch = () => renderCards();
// Highlight the index entry for the quiz nearest the top third of the screen.
function watchToc() {
  let queued = false;
  const update = () => {
    queued = false;
    if (S.view !== 'home') return;
    const line = window.innerHeight * 0.4;
    let best = null;
    for (const c of els.cards.querySelectorAll('.card')) {
      if (c.hidden || !c.offsetParent) continue;
      const r = c.getBoundingClientRect();
      if (r.top <= line && r.bottom > 80) best = c; else if (!best) best = c;
    }
    for (const l of document.querySelectorAll('.toc-item')) l.classList.toggle('on', !!best && l.dataset.target === best.dataset.quiz);
  };
  window.addEventListener('scroll', () => { if (!queued) { queued = true; requestAnimationFrame(update); } }, { passive: true });
  update();
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

function beginRun(quiz, mode, presented, extra = {}) {
  S.run = { quiz, mode, questions: presented, i: 0, answers: {}, checked: {}, flagged: {}, tutor: {}, elapsed: 0, ...extra };
  els.title.textContent = quiz.title;
  els.sub.textContent = `${modeLabel(mode)} · ${presented.length} questions`;
  document.title = `${quiz.title} - Quizzes - HLD Room`;
  show('run');
  renderQuestion();
  startTimer();
  persistRun();
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
  beginRun(quiz, mode, questions.map((q) => presentQuestion(q, rand)));
}

// ---------- weak spots: everything you missed, across all quizzes, in one practice round ----------
const WEAK_ID = 'weak-spots';
const weakRefs = () => Object.entries(S.attempts).flatMap(([quizId, a]) => (a?.last?.missed || []).map((qid) => [quizId, qid]));
async function buildWeak(refs) {
  const byQuiz = new Map();
  for (const [quizId] of refs) if (!byQuiz.has(quizId)) { try { byQuiz.set(quizId, await loadFull(quizId)); } catch { /* quiz removed */ } }
  const questions = [];
  for (const [quizId, qid] of refs) {
    const src = byQuiz.get(quizId)?.questions.find((q) => q.id === qid);
    if (src) questions.push({ ...src, id: `${quizId}::${qid}`, src: [quizId, qid] });
  }
  return { id: WEAK_ID, title: 'Weak spots practice', questions };
}
async function startWeak() {
  const quiz = await buildWeak(weakRefs());
  if (!quiz.questions.length) { flash('No missed questions to practise yet.'); return; }
  const rand = mulberry32((Math.random() * 2 ** 32) >>> 0);
  const picked = shuffle(quiz.questions, rand).slice(0, 15);
  beginRun(quiz, 'weak', picked.map((q) => presentQuestion(q, rand)));
}

// ---------- resume: the running quiz is saved after every action ----------
function persistRun() {
  const run = S.run;
  if (!run || run.result) return;
  store.set('run', {
    quizId: run.quiz.id, mode: run.mode, i: run.i, answers: run.answers, checked: run.checked, flagged: run.flagged, elapsed: Math.round(run.elapsed), at: Date.now(),
    order: run.questions.map((q) => [q.id, q.options.map((o) => o.id)]), refs: run.mode === 'weak' ? run.questions.map((q) => q.src) : undefined, title: run.quiz.title,
  });
}
const clearRun = () => store.set('run', null);
const savedRun = () => { const r = store.get('run', null); return r && typeof r === 'object' && r.quizId && Array.isArray(r.order) ? r : null; };
async function resumeRun() {
  const r = savedRun();
  if (!r) return;
  try {
    const quiz = r.mode === 'weak' ? await buildWeak(r.refs || []) : await loadFull(r.quizId);
    const byId = new Map(quiz.questions.map((q) => [q.id, q]));
    const presented = r.order.map(([qid, optIds]) => {
      const q = byId.get(qid);
      if (!q) return null;
      const opts = optIds.map((id) => q.options.find((o) => o.id === id)).filter(Boolean);
      return opts.length === q.options.length ? { ...q, options: opts } : q;
    }).filter(Boolean);
    if (!presented.length) throw new Error('empty');
    beginRun(quiz, r.mode, presented, { i: Math.min(r.i, presented.length - 1), answers: r.answers || {}, checked: r.checked || {}, flagged: r.flagged || {}, elapsed: r.elapsed || 0 });
  } catch { clearRun(); flash('That saved quiz could not be restored.'); renderUpNext(); }
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
  renderMap();
  els.flag.setAttribute('aria-pressed', String(!!run.flagged[q.id]));
  els.flag.classList.toggle('on', !!run.flagged[q.id]);
  els.prev.disabled = run.i === 0;
  els.fill.style.width = `${(Object.keys(run.answers).filter((id) => run.answers[id].length).length / total) * 100}%`;
  els.progress.setAttribute('aria-valuenow', String(Math.round((Object.keys(run.answers).length / total) * 100)));
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

function renderMap() {
  const run = S.run;
  els.map.replaceChildren(...run.questions.map((q, n) => {
    const answered = (run.answers[q.id] || []).length > 0;
    let state = answered ? 'answered' : '';
    if (run.checked[q.id]) state = isCorrect(q, run.answers[q.id]) ? 'right' : 'wrong';
    return h('button', {
      type: 'button', class: `map-dot ${state}${n === run.i ? ' cur' : ''}${run.flagged[q.id] ? ' flag' : ''}`, text: String(n + 1),
      title: `Question ${n + 1}${run.flagged[q.id] ? ' (flagged)' : ''}`, 'aria-label': `Go to question ${n + 1}${answered ? ', answered' : ''}${run.flagged[q.id] ? ', flagged' : ''}`,
      'aria-current': n === run.i ? 'step' : null, onclick: () => goto(n),
    });
  }));
  els.map.querySelector('.cur')?.scrollIntoView({ block: 'nearest', inline: 'center' });
}
function goto(n) {
  const run = S.run;
  if (n < 0 || n >= run.questions.length || n === run.i) return;
  run.i = n;
  renderQuestion();
  persistRun();
}
function toggleFlag() {
  const run = S.run; const q = cur();
  if (run.flagged[q.id]) delete run.flagged[q.id]; else run.flagged[q.id] = true;
  renderQuestion();
  persistRun();
}
async function finishEarly() {
  const run = S.run;
  const blank = run.questions.filter((q) => !(run.answers[q.id] || []).length).length;
  const flagged = Object.keys(run.flagged).length;
  if (blank || flagged) {
    const parts = [blank ? `${blank} unanswered question${blank > 1 ? 's' : ''}` : '', flagged ? `${flagged} flagged for review` : ''].filter(Boolean).join(' and ');
    if (!(await askConfirm(`You still have ${parts}. Finish anyway? Unanswered questions count as wrong.`, 'Finish quiz'))) return;
  }
  finish();
}

function choose(id) {
  const q = cur();
  if (S.run.checked[q.id]) return;
  const now = new Set(selectedIds());
  if (q.type === 'multi') { if (now.has(id)) now.delete(id); else now.add(id); } else { now.clear(); now.add(id); }
  S.run.answers[q.id] = [...now];
  renderQuestion();
  persistRun();
  els.options.querySelector(`[data-id="${id}"]`)?.focus();
}

function check() {
  const q = cur();
  if (!selectedIds().length || S.run.checked[q.id]) return;
  S.run.checked[q.id] = true;
  renderQuestion();
  persistRun();
  els.next.focus();
}

function next() {
  const run = S.run;
  if (!els.instant.checked && !selectedIds().length) return;
  if (run.i < run.questions.length - 1) { run.i += 1; renderQuestion(); persistRun(); els.options.querySelector('.opt')?.focus(); } else finish();
}

els.prev.addEventListener('click', () => goto(S.run.i - 1));
els.flag.addEventListener('click', toggleFlag);
els.finish.addEventListener('click', finishEarly);
window.addEventListener('beforeunload', persistRun);
els.check.addEventListener('click', check);
els.next.addEventListener('click', next);
els.limit.addEventListener('change', () => { store.set('limit', els.limit.checked); renderClock(); });
els.instant.addEventListener('change', () => { store.set('instant', els.instant.checked); if (S.run) renderQuestion(); });
els.quit.addEventListener('click', async () => {
  if (await askConfirm('Quit this quiz? Your answers so far will not be saved.', 'Quit quiz')) { stopTimer(); clearRun(); S.run = null; show('home'); renderCards(); renderUpNext(); }
});

document.addEventListener('keydown', (e) => {
  if (S.view !== 'run' || e.ctrlKey || e.metaKey || e.altKey) return;
  if (document.querySelector('dialog[open]')) return;
  const t = e.target;
  if (t && (t.tagName === 'INPUT' && t.type !== 'checkbox' || t.tagName === 'TEXTAREA')) return;
  const q = cur();
  if (e.key === 'ArrowLeft') { e.preventDefault(); goto(S.run.i - 1); return; }
  if (e.key === 'ArrowRight') { e.preventDefault(); goto(S.run.i + 1); return; }
  if (e.key.toLowerCase() === 'f') { e.preventDefault(); toggleFlag(); return; }
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
  if (run.mode === 'weak') {
    // Questions answered correctly leave the weak list; the others stay for next time.
    for (const q of run.questions) {
      if (!q.src || !isCorrect(q, run.answers[q.id])) continue;
      const a = S.attempts[q.src[0]];
      if (a?.last) a.last.missed = a.last.missed.filter((id) => id !== q.src[1]);
    }
    store.set('attempts', S.attempts);
    S.history = [{ quizId: WEAK_ID, title: run.quiz.title, at: Date.now(), seconds: result.seconds, mode: 'weak', percent: result.percent, score: result.score, total: result.total }, ...S.history].slice(0, 100);
    store.set('history', S.history);
    renderHistory();
    return;
  }
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

// Progress lives only in this browser, so offer a backup file you can move to another browser.
function exportProgress() {
  const data = { app: 'hld-room-quizzes', version: 1, at: new Date().toISOString(), attempts: S.attempts, history: S.history, custom: S.custom };
  const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
  const a = h('a', { href: url, download: 'hld-quiz-progress.json' });
  document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function importProgress() {
  const input = h('input', { type: 'file', accept: 'application/json,.json' });
  input.addEventListener('change', async () => {
    try {
      const data = JSON.parse(await input.files[0].text());
      if (data?.app !== 'hld-room-quizzes') throw new Error('not a progress file');
      S.attempts = { ...S.attempts, ...(data.attempts && typeof data.attempts === 'object' ? data.attempts : {}) };
      S.history = [...(Array.isArray(data.history) ? data.history : []), ...S.history].filter((r) => r && typeof r.title === 'string' && Number.isFinite(r.percent))
        .sort((x, y) => y.at - x.at).filter((r, i, all) => all.findIndex((o) => o.at === r.at && o.title === r.title) === i).slice(0, 100);
      const known = new Set(S.custom.map((q) => q.id));
      S.custom = [...S.custom, ...(Array.isArray(data.custom) ? data.custom.filter((q) => validateQuiz(q).length === 0 && !known.has(q.id)) : [])];
      store.set('attempts', S.attempts); store.set('history', S.history); store.set('custom', S.custom);
      renderCards(); renderHistory();
      flash('Progress imported.');
    } catch { flash('That file is not a quiz progress export.'); }
  });
  input.click();
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
      h('span', { class: 'hist-tools' },
        h('button', { class: 'btn ghost small', type: 'button', text: 'Export progress', onclick: exportProgress }),
        h('button', { class: 'btn ghost small', type: 'button', text: 'Import', onclick: importProgress }),
        h('button', { class: 'btn ghost small', type: 'button', text: 'Clear', onclick: () => { S.history = []; store.set('history', []); renderHistory(); } }))),
    h('ul', { class: 'hist-list' }, ...rows));
}

function finish() {
  const run = S.run;
  stopTimer();
  const result = scoreAttempt({ questions: run.questions }, run.answers);
  run.result = result;
  result.seconds = Math.round(run.elapsed);
  clearRun();
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
  const missedAll = run.mode === 'weak' ? [] : S.attempts[run.quiz.id].last.missed;
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
els.retake.addEventListener('click', () => (S.run.mode === 'weak' ? startWeak() : startQuiz(S.run.quiz.id, S.run.mode === 'missed' ? 'full' : S.run.mode)));
els.retryMissed.addEventListener('click', () => startQuiz(S.run.quiz.id, 'missed'));
els.home.addEventListener('click', () => { S.run = null; show('home'); renderCards(); renderUpNext(); });

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
  watchToc();
  els.search.addEventListener('input', applySearch);
  window.addEventListener('hashchange', () => { els.search.value = ''; renderCards(); window.scrollTo({ top: 0 }); });
}

init();

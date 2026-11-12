// Subjects -> topics -> quizzes, plus search. Every list is cards or rows you can click; the path is always visible as breadcrumbs.
import { S, emit } from './store.js';
import { allQuizzes, best, buildSubjects, clearRun, modeLabel, savedRun, stats, streakDays, subjectQuizzes, topicStats, weakRefs } from './data.js';
import { deleteCustom, go, openGenerator, openProgressDialog, resume, startFlow, startMissed, startWeak } from './actions.js';
import { ago, h, icon, menuButton, plural } from './dom.js';

const norm = (s) => String(s || '').toLowerCase();
const tone = (pct) => (pct >= 80 ? 'good' : pct >= 50 ? 'mid' : 'low');

function bar(pct, cls = '') { return h('div', { class: `bar ${cls}`, role: 'progressbar', 'aria-valuenow': String(pct), 'aria-valuemin': '0', 'aria-valuemax': '100' }, h('i', { style: { width: `${pct}%` } })); }
function crumbs(parts) {
  return h('nav', { class: 'crumbs', 'aria-label': 'You are here' }, ...parts.flatMap((p, i) => [i ? h('span', { class: 'sep', 'aria-hidden': 'true' }, icon('chevron', 14)) : null,
    p.href ? h('a', { href: p.href, text: p.label }) : h('span', { class: 'here', 'aria-current': 'page', text: p.label })]));
}
function searchBox(placeholder, onInput, value = '') {
  const input = h('input', { type: 'search', placeholder, 'aria-label': placeholder, autocomplete: 'off', value, id: 'qsearch' });
  input.addEventListener('input', () => onInput(input.value));
  return { input, el: h('label', { class: 'searchbox' }, icon('search', 18), input, h('kbd', { text: '/' })) };
}

// ---------- cards and rows ----------
function subjectCard(sj) {
  const st = stats(subjectQuizzes(sj));
  return h('a', { class: 'card subject', href: `#/subject/${encodeURIComponent(sj.id)}`, 'aria-label': `${sj.title}: ${plural(sj.topics.length, 'topic')}` },
    h('div', { class: 'subject-icon', 'aria-hidden': 'true', text: sj.icon }),
    h('div', { class: 'card-main' }, h('h3', { text: sj.title }), h('p', { class: 'desc', text: sj.description }),
      h('div', { class: 'meta' }, h('span', { text: plural(sj.topics.length, 'topic') }), h('span', { text: plural(st.total, 'quiz', 'quizzes') }), h('span', { text: `${st.questions} questions` }))),
    h('div', { class: 'card-foot' }, bar(st.pct), h('span', { class: 'muted small', text: st.running ? 'In progress' : `${st.done}/${st.total} done` })));
}
function topicCard(t, sj) {
  const st = topicStats(t);
  return h('a', { class: 'card topic', href: `#/topic/${encodeURIComponent(t.id)}` },
    h('div', { class: 'card-main' }, h('h3', { text: t.title }), t.description ? h('p', { class: 'desc', text: t.description.replace(/ JEE-Advanced style:.*$/, '') }) : null,
      h('div', { class: 'meta' }, h('span', { text: plural(t.items.length, 'quiz', 'quizzes') }), h('span', { text: `${st.questions} questions` }), h('span', { text: `~${st.minutes} min` }))),
    h('div', { class: 'card-foot' }, bar(st.pct), h('span', { class: 'muted small', text: st.running ? 'In progress' : `${st.done}/${st.total}` })));
}
function quizRow(q, n, { showTopic = null } = {}) {
  const att = S.attempts[q.id];
  const run = savedRun();
  const inProgress = !!run && run.quizId === q.id;
  const missed = att?.last?.missed?.length || 0;
  const b = best(q);
  const items = [{ label: 'Quick 10: a random sample', onclick: () => startFlow(q, { defaultMode: 'quick' }) }];
  if (missed) items.push({ label: `Retry missed (${missed})`, onclick: () => startMissed(q.id) });
  if (q.custom) items.push({ label: 'Delete this quiz', danger: true, onclick: () => deleteCustom(q) });
  return h('article', { class: 'qrow' + (b != null ? ' done' : '') + (inProgress ? ' running' : ''), id: `quiz-${q.id}` },
    h('div', { class: 'qnum', 'aria-hidden': 'true' }, b != null ? icon('check', 16) : String(n)),
    h('div', { class: 'qbody' },
      showTopic ? h('a', { class: 'crumb-link', href: showTopic.href, text: showTopic.label }) : null,
      h('h3', { text: q.title }),
      q.description ? h('p', { class: 'desc one', text: q.description.replace(/ JEE-Advanced style:.*$/, '') }) : null,
      h('div', { class: 'meta' }, h('span', { text: `${q.questionCount} questions` }), h('span', { text: `~${q.estimatedMinutes} min` }),
        inProgress ? h('span', { class: 'pill prog', text: 'In progress' }) : att?.last ? h('span', { class: 'muted', text: `Last ${att.last.percent}% · ${ago(att.last.at)}` }) : h('span', { class: 'muted', text: 'Not started' }))),
    b != null ? h('div', { class: `score ${tone(b)}`, title: 'Best score' }, h('b', { text: `${b}%` }), h('small', { text: 'best' })) : null,
    h('div', { class: 'qactions' },
      h('button', { class: 'btn primary', type: 'button', onclick: () => (inProgress ? resume() : startFlow(q)) }, icon('play', 14), inProgress ? ' Resume' : b != null || att?.last ? ' Retake' : ' Start'),
      menuButton('More options', items)));
}

// ---------- pages ----------
function heroCard() {
  const run = savedRun();
  if (run) {
    const done = Object.values(run.answers || {}).filter((a) => a.length).length;
    return h('section', { class: 'hero resume' }, h('div', {}, h('span', { class: 'eyebrow', text: 'Pick up where you left off' }), h('h2', { text: run.title || 'Quiz in progress' }),
      h('p', { text: `${done} of ${run.order.length} answered · ${Math.round((run.elapsed || 0) / 60)} min so far` })),
    h('div', { class: 'row-btns' }, h('button', { class: 'btn primary big', type: 'button', onclick: resume }, icon('play', 16), ' Resume'),
      h('button', { class: 'btn ghost', type: 'button', text: 'Discard', onclick: () => { clearRun(); emit(); } })));
  }
  const weak = weakRefs().length;
  if (weak) return h('section', { class: 'hero weak' }, h('div', {}, h('span', { class: 'eyebrow', text: 'Recommended' }), h('h2', { text: 'Practise your weak spots' }), h('p', { text: `${plural(weak, 'question')} you missed, mixed from all your quizzes.` })),
    h('button', { class: 'btn primary big', type: 'button', onclick: startWeak }, icon('target', 16), ' Start practice'));
  return null;
}
function statsRow() {
  const list = allQuizzes();
  const st = stats(list);
  const tile = (value, label) => h('div', { class: 'stat' }, h('b', { text: value }), h('span', { text: label }));
  return h('div', { class: 'stats' }, tile(`${st.done}/${st.total}`, 'quizzes done'), tile(st.avg == null ? '-' : `${st.avg}%`, 'average best'), tile(String(S.history.length), 'attempts'), tile(String(streakDays()), 'day streak'));
}
function recent() {
  if (!S.history.length) return null;
  return h('section', { class: 'recent' }, h('h2', { class: 'section-title', text: 'Recent results' }),
    h('ul', { class: 'hist' }, ...S.history.slice(0, 5).map((r) => h('li', {}, h('span', { class: `hist-score ${tone(r.percent)}`, text: `${r.percent}%` }),
      h('span', { class: 'hist-main' }, h('b', { text: r.title }), h('small', { class: 'muted', text: `${r.score}/${r.total} · ${modeLabel(r.mode)} · ${ago(r.at)}` }))))));
}
function toolbar() {
  return h('div', { class: 'toolbar' },
    h('button', { class: 'btn', type: 'button', onclick: openGenerator }, icon('spark', 16), ' Create a quiz with AI'),
    h('button', { class: 'btn', type: 'button', onclick: openProgressDialog }, icon('db', 16), ' Your progress'));
}

// Search results: subjects, topics and quizzes that match, each with where it lives.
function searchResults(term, scope) {
  const t = norm(term);
  const subjects = buildSubjects().filter((s) => !scope || s.id === scope);
  const out = [];
  for (const sj of subjects) {
    if (!scope && norm(`${sj.title} ${sj.description}`).includes(t)) out.push(subjectCard(sj));
    for (const tp of sj.topics) {
      if (!tp.single && norm(`${tp.title} ${tp.description}`).includes(t)) out.push(h('div', { class: 'result' }, h('a', { class: 'crumb-link', href: `#/subject/${encodeURIComponent(sj.id)}`, text: sj.title }), topicCard(tp, sj)));
      for (const q of tp.items) {
        if (!norm(`${q.title} ${q.description} ${q.topic} ${tp.title}`).includes(t)) continue;
        out.push(h('div', { class: 'result' }, quizRow(q, '·', { showTopic: { href: tp.single ? `#/subject/${encodeURIComponent(sj.id)}` : `#/topic/${encodeURIComponent(tp.id)}`, label: tp.single ? sj.title : `${sj.title} › ${tp.title}` } })));
      }
    }
  }
  return out.length ? h('div', { class: 'results' }, ...out) : h('div', { class: 'empty' }, h('b', { text: 'Nothing matches that.' }), h('p', { class: 'muted', text: 'Try a shorter word, like "hashmap", "sql" or "kafka".' }));
}

export function renderHome(app) {
  const subjects = buildSubjects();
  const area = h('div', { class: 'home-area' });
  const draw = (term) => {
    if (term.trim()) { area.replaceChildren(searchResults(term, null)); return; }
    area.replaceChildren(
      heroCard(), statsRow(),
      h('section', {}, h('h2', { class: 'section-title', text: 'Subjects' }), h('div', { class: 'grid subjects' }, ...subjects.map(subjectCard))),
      h('div', { class: 'two-col' }, recent(), toolbar()));
  };
  const sb = searchBox('Search subjects, topics and quizzes', draw);
  app.replaceChildren(h('div', { class: 'page' },
    h('header', { class: 'page-head' }, h('div', {}, h('h1', { text: 'Quizzes' }), h('p', { class: 'muted', text: 'Pick a subject, then a topic, then a quiz. Or just search.' })), sb.el),
    area));
  draw('');
}

export function renderSubject(app, id) {
  const sj = buildSubjects().find((s) => s.id === id);
  if (!sj) { go('#/'); return; }
  const st = stats(subjectQuizzes(sj));
  const area = h('div', { class: 'home-area' });
  const draw = (term) => {
    if (term.trim()) { area.replaceChildren(searchResults(term, sj.id)); return; }
    area.replaceChildren(h('div', { class: 'grid topics' }, ...sj.topics.map((t) => (t.single ? h('div', { class: 'single-wrap' }, quizRow(t.items[0], '·')) : topicCard(t, sj)))));
  };
  const sb = searchBox(`Search in ${sj.title}`, draw);
  app.replaceChildren(h('div', { class: 'page' }, crumbs([{ label: 'Quizzes', href: '#/' }, { label: sj.title }]),
    h('header', { class: 'page-head' }, h('div', { class: 'head-main' }, h('div', { class: 'subject-icon big', 'aria-hidden': 'true', text: sj.icon }),
      h('div', {}, h('h1', { text: sj.title }), h('p', { class: 'muted', text: sj.description }), h('div', { class: 'head-progress' }, bar(st.pct), h('span', { class: 'muted small', text: `${st.done}/${st.total} quizzes · ${st.questions} questions` })))), sb.el),
    area));
  draw('');
}

export function renderTopic(app, id) {
  const sj = buildSubjects().find((s) => s.topics.some((t) => t.id === id));
  const t = sj?.topics.find((x) => x.id === id);
  if (!t) { go('#/'); return; }
  const st = topicStats(t);
  const area = h('div', { class: 'rows' });
  const draw = (term) => {
    area.replaceChildren(...t.items.map((q, i) => ({ q, i })).filter(({ q }) => !term.trim() || norm(`${q.title} ${q.description}`).includes(norm(term))).map(({ q, i }) => quizRow(q, i + 1)));
    if (!area.children.length) area.append(h('div', { class: 'empty' }, h('b', { text: 'No quiz matches that.' })));
  };
  const sb = searchBox(`Search in ${t.title}`, draw);
  app.replaceChildren(h('div', { class: 'page' }, crumbs([{ label: 'Quizzes', href: '#/' }, { label: sj.title, href: `#/subject/${encodeURIComponent(sj.id)}` }, { label: t.title }]),
    h('header', { class: 'page-head' }, h('div', {}, h('h1', { text: t.title }), t.description ? h('p', { class: 'muted', text: t.description.replace(/ JEE-Advanced style:.*$/, '') }) : null,
      h('div', { class: 'head-progress' }, bar(st.pct), h('span', { class: 'muted small', text: `${st.done}/${st.total} done · ${st.questions} questions · ~${st.minutes} min` }))), sb.el),
    area));
  draw('');
}

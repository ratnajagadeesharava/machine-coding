// The results page after a quiz: score, what to review (weakest concepts first), and every question with your answer and the explanation.
import { isCorrect, numberFor } from '/shared/quiz.js';
import { richText } from '../rich.js';
import { S } from './store.js';
import { allQuizzes, buildSubjects, modeLabel } from './data.js';
import { go, startFlow, startMissed, startWeak } from './actions.js';
import { answerText, correctText } from './player.js';
import { fmtClock, h, icon } from './dom.js';

const RICH = new Set(['integer', 'matrix', 'fill', 'code']);
function shortPrompt(p) { return p.replace(/```[\s\S]*?```/g, ' … ').replace(/`/g, '').replace(/\s+/g, ' ').trim().slice(0, 130); }

function optionAnswer(q, ids) {
  const picked = (q.options || []).map((o, i) => ({ o, i })).filter(({ o }) => ids.includes(o.id));
  return picked.length ? picked.map(({ o, i }) => `${numberFor(i)}. ${o.text}`) : ['(no answer)'];
}

export function renderResults(app) {
  const last = S.last;
  if (!last) { go('#/'); return; }
  const { run, result } = last;
  const p = result.percent;
  const quizSummary = allQuizzes().find((q) => q.id === run.quiz.id);
  const missedAll = run.mode === 'weak' ? [] : (S.attempts[run.quiz.id]?.last?.missed || []);
  const flaggedIds = new Set(Object.keys(run.flagged || {}));
  let filter = result.missed.length && result.missed.length < result.total ? 'wrong' : 'all';

  // where to go next: the following quiz in the same topic
  const subjects = buildSubjects();
  let topic = null; let nextQuiz = null; let subject = null;
  for (const sj of subjects) for (const t of sj.topics) { const i = t.items.findIndex((q) => q.id === run.quiz.id); if (i >= 0) { topic = t; subject = sj; nextQuiz = t.items[i + 1] || null; } }

  const list = h('div', { class: 'rv-list' });
  const pills = h('div', { class: 'pills', role: 'tablist' });
  const drawList = () => {
    const rows = run.questions.map((q, n) => ({ q, n, ok: isCorrect(q, run.answers[q.id]), flag: flaggedIds.has(q.id) }))
      .filter((r) => filter === 'all' || (filter === 'wrong' && !r.ok) || (filter === 'flagged' && r.flag));
    list.replaceChildren(...(rows.length ? rows.map(({ q, n, ok, flag }) => {
      const mine = RICH.has(q.type) ? [answerText(q, run.answers[q.id] || [])] : q.type === 'assertion' ? (run.answers[q.id] || []).map((id) => `Option ${id.toUpperCase()}`) : optionAnswer(q, run.answers[q.id] || []);
      const right = RICH.has(q.type) ? [correctText(q)] : q.type === 'assertion' ? q.correct.map((id) => `Option ${id.toUpperCase()}`) : q.correct.map((id) => `${numberFor(q.options.findIndex((o) => o.id === id))}. ${q.options.find((o) => o.id === id).text}`);
      return h('details', { class: `rv ${ok ? 'ok' : 'bad'}`, open: !ok && filter === 'wrong' && rows.length <= 3 },
        h('summary', {}, h('span', { class: 'rv-mark' }, icon(ok ? 'check' : 'x', 15)), h('span', { class: 'rv-title' }, h('b', { text: `${n + 1}. ` }), shortPrompt(q.prompt || q.assertion || '')), flag ? h('span', { class: 'rv-flag' }, icon('flag', 14)) : null),
        h('div', { class: 'rv-body' }, h('div', { class: 'rv-q' }, ...richText(h, q.prompt)),
          h('div', { class: 'rv-ans' }, h('span', { class: 'eyebrow', text: 'Your answer' }), ...mine.map((t) => h('div', { class: ok ? 'good' : 'bad', text: t }))),
          ok ? null : h('div', { class: 'rv-ans' }, h('span', { class: 'eyebrow', text: 'Correct answer' }), ...right.map((t) => h('div', { class: 'good', text: t }))),
          h('div', { class: 'rv-ex' }, h('span', { class: 'eyebrow', text: 'Explanation' }), ...richText(h, q.explanation))));
    }) : [h('div', { class: 'empty' }, h('b', { text: filter === 'wrong' ? 'Nothing wrong here. Nice.' : 'Nothing flagged.' }))]));
  };
  const drawPills = () => pills.replaceChildren(...[['all', `All ${result.total}`], ['wrong', `Wrong ${result.missed.length}`], ['flagged', `Flagged ${flaggedIds.size}`]].map(([k, label]) =>
    h('button', { class: 'pill-btn' + (filter === k ? ' on' : ''), type: 'button', role: 'tab', 'aria-selected': String(filter === k), text: label, onclick: () => { filter = k; drawPills(); drawList(); } })));
  drawPills(); drawList();

  const actions = [];
  if (run.mode === 'weak') actions.push(h('button', { class: 'btn primary big', type: 'button', onclick: startWeak }, icon('target', 16), ' Practise more'));
  else {
    if (missedAll.length) actions.push(h('button', { class: 'btn primary big', type: 'button', onclick: () => startMissed(run.quiz.id) }, icon('retry', 16), ` Retry missed (${missedAll.length})`));
    if (quizSummary) actions.push(h('button', { class: 'btn' + (missedAll.length ? '' : ' primary big'), type: 'button', onclick: () => startFlow(quizSummary) }, icon('retry', 16), ' Retake'));
    if (nextQuiz) actions.push(h('button', { class: 'btn', type: 'button', onclick: () => startFlow(nextQuiz) }, 'Next quiz ', icon('next', 16)));
  }
  actions.push(h('a', { class: 'btn ghost', href: topic ? `#/topic/${encodeURIComponent(topic.id)}` : '#/' }, topic ? `Back to ${topic.title.replace(/^(Java|Python|Core): /, '')}` : 'All quizzes'));

  app.replaceChildren(h('div', { class: 'page results-page' },
    h('section', { class: 'result-hero' },
      h('div', { class: 'ring', style: { '--p': String(p) }, role: 'img', 'aria-label': `Score ${p} percent` }, h('span', { text: `${p}%` })),
      h('div', { class: 'result-text' }, h('span', { class: 'eyebrow', text: run.quiz.title }),
        h('h1', { text: p >= 90 ? 'Excellent!' : p >= 70 ? 'Solid work' : p >= 50 ? 'Getting there' : 'Keep practising' }),
        h('p', { class: 'muted', text: `${result.score} of ${result.total} correct · ${modeLabel(run.mode)} · ${fmtClock(result.seconds)}` }),
        h('div', { class: 'row-btns' }, ...actions))),
    h('div', { class: 'result-cols' },
      h('section', { class: 'card flat' }, h('h2', { class: 'section-title', text: 'By concept' }), h('p', { class: 'muted small', text: 'Weakest first: start your review at the top.' }),
        ...result.byConcept.map((c) => { const pct = Math.round((c.correct / c.total) * 100); return h('div', { class: 'bc' }, h('div', { class: 'bc-top' }, h('span', { text: c.concept }), h('span', { class: 'muted', text: `${c.correct}/${c.total}` })), h('div', { class: 'bar ' + (pct < 50 ? 'low' : pct < 80 ? 'mid' : 'good') }, h('i', { style: { width: `${pct}%` } }))); })),
      h('section', { class: 'card flat' }, h('div', { class: 'rv-head' }, h('h2', { class: 'section-title', text: 'Review' }), pills), list))));
}

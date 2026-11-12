// Derived data: subjects -> topics -> quizzes, stats, weak spots, and loading full quizzes.
import { mulberry32, pickQuestions, presentQuestion, shuffle, validateQuiz } from '/shared/quiz.js';
import { SUBJECTS, subjectIdForTopic, topicRank } from '/shared/subjects.js';
import { S, store } from './store.js';

export const QUICK_COUNT = 10;
export const WEAK_ID = 'weak-spots';

export function allQuizzes() {
  const bank = S.summaries.map((s) => ({ ...s, custom: false }));
  const mine = S.custom.map((q) => ({
    id: q.id, title: q.title, topic: q.topic || '', description: q.description || '', source: q.source || '',
    estimatedMinutes: q.estimatedMinutes || Math.max(1, Math.round(q.questions.length * 0.7)), questionCount: q.questions.length, custom: true,
  }));
  return [...mine, ...bank];
}

export function buildTopics(list) {
  const map = new Map();
  for (const q of list) {
    let id; let title; let description;
    if (q.custom) { id = 'mine'; title = 'My generated quizzes'; description = 'Quizzes you created with AI. Saved in this browser.'; }
    else if (q.series) { id = q.series; title = q.seriesTitle; description = q.seriesDescription || ''; }
    else { id = `quiz-${q.id}`; title = q.title; description = q.description; }
    if (!map.has(id)) map.set(id, { id, title, description, items: [], single: !q.custom && !q.series });
    map.get(id).items.push(q);
  }
  const topics = [...map.values()];
  for (const t of topics) t.items.sort((x, y) => (x.seriesOrder || 0) - (y.seriesOrder || 0));
  return topics;
}

export function buildSubjects() {
  const topics = buildTopics(allQuizzes());
  const map = new Map(SUBJECTS.map((s) => [s.id, { ...s, topics: [] }]));
  for (const t of topics) map.get(subjectIdForTopic(t.id)).topics.push(t);
  for (const sj of map.values()) sj.topics.sort((a, b) => topicRank(a.id) - topicRank(b.id) || a.title.localeCompare(b.title));
  return [...map.values()].filter((s) => s.topics.length);
}

export const best = (q) => (S.attempts[q.id]?.best ?? null);
export const isDone = (q) => best(q) != null;
export const savedRun = () => { const r = store.get('run', null); return r && typeof r === 'object' && r.quizId && Array.isArray(r.order) ? r : null; };

export function stats(quizzes) {
  const done = quizzes.filter(isDone);
  const run = savedRun();
  return {
    total: quizzes.length, done: done.length, pct: quizzes.length ? Math.round((done.length / quizzes.length) * 100) : 0,
    avg: done.length ? Math.round(done.reduce((n, q) => n + best(q), 0) / done.length) : null,
    questions: quizzes.reduce((n, q) => n + q.questionCount, 0), minutes: quizzes.reduce((n, q) => n + q.estimatedMinutes, 0),
    running: !!run && quizzes.some((q) => q.id === run.quizId),
  };
}
export const subjectQuizzes = (s) => s.topics.flatMap((t) => t.items);
export const topicStats = (t) => stats(t.items);

export function streakDays() {
  const days = new Set(S.history.map((r) => new Date(r.at).toDateString()));
  let d = new Date();
  if (!days.has(d.toDateString())) d = new Date(d - 864e5);
  let n = 0;
  while (days.has(d.toDateString())) { n += 1; d = new Date(d - 864e5); }
  return n;
}

export const weakRefs = () => Object.entries(S.attempts).flatMap(([quizId, a]) => (a?.last?.missed || []).map((qid) => [quizId, qid]));

export async function loadFull(id) {
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

export async function buildWeak(refs) {
  const byQuiz = new Map();
  for (const [quizId] of refs) if (!byQuiz.has(quizId)) { try { byQuiz.set(quizId, await loadFull(quizId)); } catch { /* quiz removed */ } }
  const questions = [];
  for (const [quizId, qid] of refs) {
    const src = byQuiz.get(quizId)?.questions.find((q) => q.id === qid);
    if (src) questions.push({ ...src, id: `${quizId}::${qid}`, src: [quizId, qid] });
  }
  return { id: WEAK_ID, title: 'Weak spots practice', questions };
}

// ---------- building a run ----------
export const settings = () => ({ instant: store.get('instant', true) !== false, limit: store.get('limit', false) === true });
export function newRun(quiz, mode, presented, extra = {}) {
  const run = { quiz, mode, questions: presented, i: 0, answers: {}, checked: {}, flagged: {}, tutor: {}, elapsed: 0, ...settings(), ...extra };
  S.run = run;
  persistRun();
  return run;
}
export async function buildRun(quizId, mode) {
  const quiz = await loadFull(quizId);
  const rand = mulberry32((Math.random() * 2 ** 32) >>> 0);
  let questions = quiz.questions;
  if (mode === 'quick') questions = pickQuestions(quiz, QUICK_COUNT, rand);
  if (mode === 'missed') {
    const ids = new Set(S.attempts[quizId]?.last?.missed || []);
    questions = quiz.questions.filter((q) => ids.has(q.id));
    if (!questions.length) throw new Error('Nothing missed to retry.');
  }
  return newRun(quiz, mode, questions.map((q) => presentQuestion(q, rand)));
}
export async function buildWeakRun() {
  const quiz = await buildWeak(weakRefs());
  if (!quiz.questions.length) throw new Error('No missed questions to practise yet.');
  const rand = mulberry32((Math.random() * 2 ** 32) >>> 0);
  return newRun(quiz, 'weak', shuffle(quiz.questions, rand).slice(0, 15).map((q) => presentQuestion(q, rand)));
}

// The running quiz is saved after every action so it can be resumed after a reload.
export function persistRun() {
  const run = S.run;
  if (!run || run.result) return;
  store.set('run', {
    quizId: run.quiz.id, mode: run.mode, i: run.i, answers: run.answers, checked: run.checked, flagged: run.flagged, elapsed: Math.round(run.elapsed), at: Date.now(),
    instant: run.instant, limit: run.limit,
    order: run.questions.map((q) => [q.id, (q.options || []).map((o) => o.id)]), refs: run.mode === 'weak' ? run.questions.map((q) => q.src) : undefined, title: run.quiz.title,
  });
}
export const clearRun = () => store.set('run', null);
export async function resumeSaved() {
  const r = savedRun();
  if (!r) throw new Error('No saved quiz.');
  const quiz = r.mode === 'weak' ? await buildWeak(r.refs || []) : await loadFull(r.quizId);
  const byId = new Map(quiz.questions.map((q) => [q.id, q]));
  const presented = r.order.map(([qid, optIds]) => {
    const q = byId.get(qid);
    if (!q) return null;
    if (!q.options) return presentQuestion(q, mulberry32(Date.now()));
    const opts = optIds.map((id) => q.options.find((o) => o.id === id)).filter(Boolean);
    return opts.length === q.options.length ? { ...q, options: opts } : presentQuestion(q, mulberry32(Date.now()));
  }).filter(Boolean);
  if (!presented.length) throw new Error('empty');
  return newRun(quiz, r.mode, presented, { i: Math.min(r.i, presented.length - 1), answers: r.answers || {}, checked: r.checked || {}, flagged: r.flagged || {}, elapsed: r.elapsed || 0,
    ...(r.instant != null ? { instant: r.instant } : {}), ...(r.limit != null ? { limit: r.limit } : {}) });
}

// ---------- recording a finished attempt ----------
export function recordAttempt(run, result, isCorrectFn) {
  if (run.mode === 'weak') {
    for (const q of run.questions) {
      if (!q.src || !isCorrectFn(q, run.answers[q.id])) continue;
      const a = S.attempts[q.src[0]];
      if (a?.last) a.last.missed = a.last.missed.filter((id) => id !== q.src[1]);
    }
    store.set('attempts', S.attempts);
    S.history = [{ quizId: WEAK_ID, title: run.quiz.title, at: Date.now(), seconds: result.seconds, mode: 'weak', percent: result.percent, score: result.score, total: result.total }, ...S.history].slice(0, 100);
    store.set('history', S.history);
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
  S.history = [{ quizId: run.quiz.id, title: run.quiz.title, at: Date.now(), seconds: result.seconds, mode: run.mode, percent: result.percent, score: result.score, total: result.total,
    weak: result.byConcept.filter((c) => c.correct < c.total).map((c) => c.concept).filter(Boolean).slice(0, 5) }, ...S.history].slice(0, 100);
  store.set('history', S.history);
}
export const modeLabel = (m) => (m === 'weak' ? 'Weak spots' : m === 'quick' ? `Quick ${QUICK_COUNT}` : m === 'missed' ? 'Retry missed' : 'Full quiz');

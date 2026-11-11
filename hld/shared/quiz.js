// Quiz model shared by the browser and the server: validation, grading and a deterministic shuffle.
//
// quiz     = { id, title, topic, description, source?, estimatedMinutes, questions: Question[] }
// question = { id, concept, type: 'single'|'multi'|'truefalse', difficulty: 1|2|3, prompt,
//              options: [{ id, text, why }], correct: [optionId...], explanation }

// Option-based types: single, multi, truefalse. JEE-Advanced style types: integer (type a number), assertion (assertion-reason,
// four fixed options), matrix (match column I to column II), fill (drag words into blanks) and code (write a short snippet).
//   integer   { answer: number, tolerance?: number, unit?: string }
//   assertion { assertion, reason, correct: ['a'|'b'|'c'|'d'], whys?: { a, b, c, d } }   a = both true and R explains A, b = both true but R does not
//             explain A, c = A true and R false, d = A false and R true
//   matrix    { left: [{id,text}], right: [{id,text}], pairs: { [leftId]: rightId } }   (right may hold extra distractors)
//   fill      { text: 'a {1} and a {2}', blanks: { 1: 'word', 2: 'word' }, bank: ['word', ...] }   (bank holds every answer plus distractors)
//   code      { language: 'java'|'sql'|'python'|..., starter?, checks: [{ label, pattern, flags? }], model }   (regex checks + a model answer)
export const QUESTION_TYPES = ['single', 'multi', 'truefalse', 'integer', 'assertion', 'matrix', 'fill', 'code'];
export const OPTION_TYPES = ['single', 'multi', 'truefalse'];
export const ASSERTION_OPTIONS = [
  { id: 'a', text: 'Both A and R are true, and R is the correct explanation of A.' },
  { id: 'b', text: 'Both A and R are true, but R is not the correct explanation of A.' },
  { id: 'c', text: 'A is true, but R is false.' },
  { id: 'd', text: 'A is false, but R is true.' },
];
export const LIMITS = { questions: 40, options: 6, prompt: 600, option: 300, explanation: 900, concept: 60, title: 100 };

const isText = (v, max, min = 1) => typeof v === 'string' && v.trim().length >= min && v.length <= max;
const BANNED_OPTION = /^\s*(all|none|both|neither)\s+of\s+the\s+(above|these|following)\b/i;

// Returns a list of human-readable problems (empty = valid).
export function validateQuestion(q, label = 'question') {
  const errs = [];
  const bad = (m) => errs.push(`${label}: ${m}`);
  if (!q || typeof q !== 'object') return [`${label}: not an object`];
  if (!isText(q.id, 80) || !/^[a-z0-9-]+$/.test(q.id)) bad('id must be lowercase letters, digits or dashes');
  if (!QUESTION_TYPES.includes(q.type)) bad(`type must be one of ${QUESTION_TYPES.join(', ')}`);
  if (![1, 2, 3].includes(q.difficulty)) bad('difficulty must be 1, 2 or 3');
  if (!isText(q.concept, LIMITS.concept)) bad('concept is required');
  if (!isText(q.prompt, LIMITS.prompt, 15)) bad('prompt must be a real question (15+ characters)');
  if (!isText(q.explanation, LIMITS.explanation, 10)) bad('explanation is required');
  if (!OPTION_TYPES.includes(q.type)) { errs.push(...validateRich(q, bad)); return errs; }
  if (!Array.isArray(q.options)) { bad('options must be an array'); return errs; }
  const n = q.options.length;
  if (q.type === 'truefalse' ? n !== 2 : n < 3 || n > LIMITS.options) bad(q.type === 'truefalse' ? 'true/false needs exactly 2 options' : 'needs 3-6 options');
  const ids = new Set();
  const texts = new Set();
  for (const o of q.options) {
    if (!o || typeof o !== 'object') { bad('option is not an object'); continue; }
    if (!isText(o.id, 4) || ids.has(o.id)) bad('option ids must be unique');
    ids.add(o.id);
    if (!isText(o.text, LIMITS.option)) bad('option text is required');
    else {
      const key = o.text.trim().toLowerCase();
      if (texts.has(key)) bad(`duplicate option "${o.text.slice(0, 30)}"`);
      texts.add(key);
      if (q.type !== 'truefalse' && BANNED_OPTION.test(o.text)) bad('"all/none of the above" options are not allowed');
    }
    if (o.why != null && !isText(o.why, LIMITS.option)) bad('option "why" must be short text');
  }
  if (!Array.isArray(q.correct) || !q.correct.length) bad('correct must list at least one option id');
  else {
    if (new Set(q.correct).size !== q.correct.length) bad('correct has duplicates');
    for (const c of q.correct) if (!ids.has(c)) bad(`correct id "${c}" is not an option`);
    if (q.type !== 'multi' && q.correct.length !== 1) bad(`${q.type} questions need exactly one correct option`);
    if (q.type === 'multi' && (q.correct.length < 2 || q.correct.length >= n)) bad('multi questions need 2+ correct options and at least one wrong one');
  }
  return errs;
}

const TOKEN = /\{([A-Za-z0-9]+)\}/g;

function validateRich(q, bad) {
  const errs = [];
  const add = (m) => bad(m);
  const idsOf = (list) => (Array.isArray(list) ? list.map((x) => x?.id) : []);
  if (q.type === 'integer') {
    if (typeof q.answer !== 'number' || !Number.isFinite(q.answer)) add('integer questions need a numeric "answer"');
    if (q.tolerance != null && !(typeof q.tolerance === 'number' && q.tolerance >= 0)) add('tolerance must be a number >= 0');
  } else if (q.type === 'assertion') {
    if (!isText(q.assertion, 300, 10)) add('assertion questions need an "assertion" statement');
    if (!isText(q.reason, 300, 10)) add('assertion questions need a "reason" statement');
    if (!Array.isArray(q.correct) || q.correct.length !== 1 || !ASSERTION_OPTIONS.some((o) => o.id === q.correct[0])) add('assertion "correct" must be exactly one of a, b, c, d');
    if (q.whys != null) for (const o of ASSERTION_OPTIONS) if (q.whys[o.id] != null && !isText(q.whys[o.id], LIMITS.option)) add(`whys.${o.id} must be short text`);
  } else if (q.type === 'matrix') {
    const l = idsOf(q.left); const r = idsOf(q.right);
    if (!Array.isArray(q.left) || q.left.length < 3 || q.left.length > 6) add('matrix needs 3-6 left items');
    if (!Array.isArray(q.right) || q.right.length < (q.left?.length || 0) || q.right.length > 8) add('matrix needs at least as many right items as left (max 8)');
    for (const it of [...(q.left || []), ...(q.right || [])]) if (!it || !isText(it.id, 4) || !isText(it.text, LIMITS.option)) add('matrix items need an id and text');
    if (new Set(l).size !== l.length || new Set(r).size !== r.length) add('matrix item ids must be unique');
    if (!q.pairs || typeof q.pairs !== 'object') add('matrix needs "pairs"');
    else {
      for (const id of l) if (!r.includes(q.pairs[id])) add(`pair for "${id}" is missing or not a right item`);
      if (new Set(Object.values(q.pairs)).size !== Object.values(q.pairs).length) add('each right item can be used once');
    }
  } else if (q.type === 'fill') {
    if (!isText(q.text, 700, 10)) add('fill questions need "text" with {1}, {2} ... blanks');
    const tokens = [...String(q.text || '').matchAll(TOKEN)].map((m) => m[1]);
    if (!tokens.length || tokens.length > 6) add('fill text needs 1-6 blanks');
    if (!q.blanks || typeof q.blanks !== 'object') add('fill needs "blanks"');
    else {
      for (const t of tokens) if (!isText(q.blanks[t], 60)) add(`blank {${t}} has no answer`);
      for (const k of Object.keys(q.blanks)) if (!tokens.includes(k)) add(`answer "${k}" has no {${k}} in the text`);
    }
    if (!Array.isArray(q.bank) || q.bank.length < tokens.length || q.bank.length > 10) add('fill needs a "bank" of words (answers plus distractors, max 10)');
    else {
      if (new Set(q.bank.map((w) => String(w).toLowerCase())).size !== q.bank.length) add('bank words must be unique');
      for (const t of tokens) if (q.blanks && !q.bank.includes(q.blanks[t])) add(`bank is missing the answer for {${t}}`);
    }
  } else if (q.type === 'code') {
    if (!isText(q.language, 20)) add('code questions need a "language"');
    if (!isText(q.model, 2000, 5)) add('code questions need a "model" answer');
    if (q.starter != null && typeof q.starter !== 'string') add('starter must be text');
    if (!Array.isArray(q.checks) || q.checks.length < 1 || q.checks.length > 8) add('code needs 1-8 checks');
    else {
      for (const c of q.checks) {
        if (!c || !isText(c.label, 120) || !isText(c.pattern, 300)) { add('each check needs a label and a pattern'); continue; }
        try { new RegExp(c.pattern, c.flags || 'i'); } catch { add(`check "${c.label}" has an invalid pattern`); }
      }
      if (q.checks.every((c) => c?.pattern) && Array.isArray(q.checks)) {
        // the model answer must pass its own checks
        const failing = q.checks.filter((c) => { try { return !new RegExp(c.pattern, c.flags || 'i').test(q.model); } catch { return false; } });
        if (failing.length) add(`the model answer fails its own check "${failing[0].label}"`);
      }
    }
  }
  return errs;
}

export function validateQuiz(quiz) {
  const errs = [];
  if (!quiz || typeof quiz !== 'object') return ['quiz: not an object'];
  if (!isText(quiz.id, 80) || !/^[a-z0-9-]+$/.test(quiz.id)) errs.push('quiz: id must be lowercase letters, digits or dashes');
  if (!isText(quiz.title, LIMITS.title)) errs.push('quiz: title is required');
  if (quiz.description != null && (typeof quiz.description !== 'string' || quiz.description.length > 400)) errs.push('quiz: description must be text of at most 400 characters');
  if (!Array.isArray(quiz.questions) || quiz.questions.length < 1 || quiz.questions.length > LIMITS.questions) {
    errs.push(`quiz: needs 1-${LIMITS.questions} questions`);
    return errs;
  }
  const seen = new Set();
  quiz.questions.forEach((q, i) => {
    errs.push(...validateQuestion(q, `question ${i + 1} (${q?.id ?? '?'})`));
    if (q && seen.has(q.id)) errs.push(`question ${i + 1}: duplicate id "${q.id}"`);
    if (q) seen.add(q.id);
  });
  return errs;
}

// A question is correct only if the selection equals the correct set exactly.
export function isCorrect(question, selected) {
  if (!OPTION_TYPES.includes(question.type) && question.type !== 'assertion') return gradeRich(question, selected || []);
  const want = [...question.correct].sort();
  const got = [...new Set(selected || [])].sort();
  return want.length === got.length && want.every((v, i) => v === got[i]);
}

// Answers for the rich types are string arrays too:
//   integer ['42']   matrix ['l1=r3', ...]   fill ['1=word', ...]   code ['source text']
function pairsOf(selected) {
  const m = new Map();
  for (const s of selected) { const i = String(s).indexOf('='); if (i > 0) m.set(s.slice(0, i), s.slice(i + 1)); }
  return m;
}

export function codeChecks(question, source) {
  return question.checks.map((c) => {
    let ok = false;
    try { ok = new RegExp(c.pattern, c.flags || 'i').test(source); } catch { ok = false; }
    return { label: c.label, ok };
  });
}

function gradeRich(q, selected) {
  if (q.type === 'integer') {
    const raw = String(selected[0] ?? '').trim();
    if (!/^[-+]?\d+(\.\d+)?$/.test(raw)) return false;
    return Math.abs(Number(raw) - q.answer) <= (q.tolerance || 0);
  }
  if (q.type === 'matrix') {
    const got = pairsOf(selected);
    return Object.entries(q.pairs).every(([l, r]) => got.get(l) === r);
  }
  if (q.type === 'fill') {
    const got = pairsOf(selected);
    return Object.entries(q.blanks).every(([k, w]) => got.get(k) === w);
  }
  if (q.type === 'code') {
    const src = String(selected[0] ?? '');
    return src.trim().length > 0 && codeChecks(q, src).every((c) => c.ok);
  }
  return false;
}

// answers: { [questionId]: string[] }  ->  { score, total, percent, byConcept: [{concept, correct, total}], missed: [questionId] }
export function scoreAttempt(quiz, answers) {
  const byConcept = new Map();
  const missed = [];
  let score = 0;
  for (const q of quiz.questions) {
    const ok = isCorrect(q, answers[q.id]);
    if (ok) score += 1; else missed.push(q.id);
    const c = byConcept.get(q.concept) || { concept: q.concept, correct: 0, total: 0 };
    c.total += 1;
    if (ok) c.correct += 1;
    byConcept.set(q.concept, c);
  }
  const total = quiz.questions.length;
  return {
    score, total, percent: total ? Math.round((score / total) * 100) : 0,
    byConcept: [...byConcept.values()].sort((a, b) => a.correct / a.total - b.correct / b.total || a.concept.localeCompare(b.concept)),
    missed,
  };
}

// Small deterministic PRNG so a shuffle can be reproduced from a seed.
export function mulberry32(seed) {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

export function shuffle(list, rand) {
  const a = [...list];
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

// Present a question for one attempt: options shuffled (true/false keeps its natural order).
export function presentQuestion(question, rand) {
  if (question.type === 'truefalse') return { ...question, options: question.options };
  if (question.type === 'assertion') {
    const options = ASSERTION_OPTIONS.map((o) => ({ ...o, why: question.whys?.[o.id] }));
    return { ...question, options, correct: question.correct };
  }
  if (question.type === 'matrix') return { ...question, options: [], correct: [], right: shuffle(question.right, rand) };
  if (question.type === 'fill') return { ...question, options: [], correct: [], bank: shuffle(question.bank, rand) };
  if (question.type === 'integer' || question.type === 'code') return { ...question, options: [], correct: [] };
  return { ...question, options: shuffle(question.options, rand) };
}

// Options are labelled 1, 2, 3 ... (keys 1-6 select them; a-f still work as a fallback).
export function numberFor(index) {
  return String(index + 1);
}

// Pick n questions at random (stable order within the pick follows the original order).
export function pickQuestions(quiz, n, rand) {
  if (n >= quiz.questions.length) return quiz.questions;
  const idx = shuffle(quiz.questions.map((_, i) => i), rand).slice(0, n).sort((a, b) => a - b);
  return idx.map((i) => quiz.questions[i]);
}

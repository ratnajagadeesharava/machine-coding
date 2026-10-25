// Quiz model shared by the browser and the server: validation, grading and a deterministic shuffle.
//
// quiz     = { id, title, topic, description, source?, estimatedMinutes, questions: Question[] }
// question = { id, concept, type: 'single'|'multi'|'truefalse', difficulty: 1|2|3, prompt,
//              options: [{ id, text, why }], correct: [optionId...], explanation }

export const QUESTION_TYPES = ['single', 'multi', 'truefalse'];
export const LIMITS = { questions: 40, options: 6, prompt: 600, option: 300, explanation: 900, concept: 60, title: 100 };
const LETTERS = 'abcdef';

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
  const want = [...question.correct].sort();
  const got = [...new Set(selected || [])].sort();
  return want.length === got.length && want.every((v, i) => v === got[i]);
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
  return { ...question, options: shuffle(question.options, rand) };
}

export function letterFor(index) {
  return LETTERS[index].toUpperCase();
}

// Pick n questions at random (stable order within the pick follows the original order).
export function pickQuestions(quiz, n, rand) {
  if (n >= quiz.questions.length) return quiz.questions;
  const idx = shuffle(quiz.questions.map((_, i) => i), rand).slice(0, n).sort((a, b) => a - b);
  return idx.map((i) => quiz.questions[i]);
}

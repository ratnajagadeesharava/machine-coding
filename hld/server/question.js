// The interview question as the AI sees it. The browser sends it with every AI request; the server validates and
// normalises it here and never looks anything up. Treat every field as untrusted data.

export const QUESTION_LIMITS = {
  title: [3, 120], prompt: [10, 2500], assumptions: 8, assumption: 240, minutes: [10, 120], focus: 300,
};
export const LEVELS = ['junior', 'mid', 'senior', 'staff'];
export const STYLES = ['supportive', 'balanced', 'tough'];
export const DIFFICULTIES = ['Easy', 'Medium', 'Hard'];

// Control characters other than newline and tab are dropped; CRLF becomes LF.
// eslint-disable-next-line no-control-regex
const CONTROL = /[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F-\u009F​-‏‪-‮⁦-⁩]/g;

function text(v, { multiline = false } = {}) {
  if (typeof v !== 'string') return '';
  let s = v.replace(/\r\n?/g, '\n').replace(CONTROL, '');
  s = multiline ? s.replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n') : s.replace(/\s+/g, ' ');
  return s.trim();
}

// -> { question } | { error }
export function cleanInterviewQuestion(raw) {
  if (!raw || typeof raw !== 'object' || Array.isArray(raw)) return { error: 'question must be an object with a title and a prompt.' };
  const L = QUESTION_LIMITS;

  const title = text(raw.title);
  if (title.length < L.title[0] || title.length > L.title[1]) return { error: `question.title must be ${L.title[0]}-${L.title[1]} characters.` };

  const prompt = text(raw.prompt, { multiline: true });
  if (prompt.length < L.prompt[0] || prompt.length > L.prompt[1]) return { error: `question.prompt must be ${L.prompt[0]}-${L.prompt[1]} characters.` };

  let assumptions = [];
  if (raw.assumptions != null) {
    if (!Array.isArray(raw.assumptions)) return { error: 'question.assumptions must be a list of strings.' };
    if (raw.assumptions.length > L.assumptions) return { error: `question.assumptions can have at most ${L.assumptions} items.` };
    for (const a of raw.assumptions) {
      if (typeof a !== 'string') return { error: 'question.assumptions must be a list of strings.' };
      const t = text(a);
      if (t.length > L.assumption) return { error: `Each assumption can be at most ${L.assumption} characters.` };
      if (t) assumptions.push(t);
    }
  }

  // minutes are clamped rather than rejected: a slightly-off time limit is not worth failing the request
  const minutes = Number.isFinite(raw.minutes) ? Math.max(L.minutes[0], Math.min(L.minutes[1], Math.round(raw.minutes))) : 45;

  const question = { title, prompt, assumptions, minutes, level: 'senior', style: 'balanced' };
  if (raw.difficulty != null && raw.difficulty !== '') {
    if (!DIFFICULTIES.includes(raw.difficulty)) return { error: 'question.difficulty must be Easy, Medium or Hard.' };
    question.difficulty = raw.difficulty;
  }
  if (raw.level != null && raw.level !== '') {
    if (!LEVELS.includes(raw.level)) return { error: `question.level must be one of ${LEVELS.join(', ')}.` };
    question.level = raw.level;
  }
  if (raw.style != null && raw.style !== '') {
    if (!STYLES.includes(raw.style)) return { error: `question.style must be one of ${STYLES.join(', ')}.` };
    question.style = raw.style;
  }
  if (raw.focus != null && raw.focus !== '') {
    if (typeof raw.focus !== 'string') return { error: 'question.focus must be text.' };
    const focus = text(raw.focus);
    if (focus.length > L.focus) return { error: `question.focus can be at most ${L.focus} characters.` };
    if (focus) question.focus = focus;
  }
  return { question };
}

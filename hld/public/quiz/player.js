// The quiz player: a full-screen, distraction-free view. A slim top bar (exit, one segment per question, timer, flag, Claude), the question in the
// middle, and a bottom dock that always holds the one button you need next (Check / Continue). Feedback appears in the dock; the explanation lives
// in the Ask Claude drawer on the right.
import { isCorrect, numberFor, scoreAttempt } from '/shared/quiz.js';
import { assertionBlock, codeWidget, fillWidget, integerWidget, matrixWidget } from '../qtypes.js';
import { richText } from '../rich.js';
import { S } from './store.js';
import { clearRun, modeLabel, persistRun, recordAttempt } from './data.js';
import { createChat } from './chat.js';
import { dialog, fmtClock, h, icon } from './dom.js';

const RICH = { integer: integerWidget, matrix: matrixWidget, fill: fillWidget, code: codeWidget };
const HINTS = { multi: 'Select all that apply', truefalse: 'True or false', integer: 'Type a number', assertion: 'Assertion and Reason: pick the best option', matrix: 'Match the columns', fill: 'Fill in the blanks', code: 'Write a short piece of code' };
const limitSec = (run) => Math.max(60, run.questions.length * 45); // 45 seconds per question

const answerText = (q, ans) => {
  if (q.type === 'integer' || q.type === 'code') return ans.length ? ans[0] : '(no answer)';
  const m = new Map(ans.map((a) => [a.slice(0, a.indexOf('=')), a.slice(a.indexOf('=') + 1)]));
  if (q.type === 'matrix') return q.left.map((l) => `${l.id.toUpperCase()} → ${q.right.find((r) => r.id === m.get(l.id))?.text ?? '?'}`).join('   ');
  return Object.keys(q.blanks).map((k) => `(${k}) ${m.get(k) ?? '?'}`).join('   ');
};
export const correctText = (q) => (q.type === 'integer' ? String(q.answer) : q.type === 'code' ? q.model
  : q.type === 'matrix' ? q.left.map((l) => `${l.id.toUpperCase()} → ${q.right.find((r) => r.id === q.pairs[l.id]).text}`).join('   ')
    : Object.entries(q.blanks).map(([k, w]) => `(${k}) ${w}`).join('   '));
export { answerText };

export function openPlayer(host, { onExit, onFinish }) {
  const run = S.run;
  let tick = null;
  const cur = () => run.questions[run.i];
  const sel = () => run.answers[cur().id] || [];

  // ---------- skeleton ----------
  const segs = h('div', { class: 'p-segs', role: 'group', 'aria-label': 'Questions' });
  const time = h('span', { class: 'p-time', role: 'timer' });
  const flagBtn = h('button', { class: 'icon-btn', type: 'button', title: 'Flag this question to come back to it (F)', 'aria-label': 'Flag question', 'aria-pressed': 'false' }, icon('flag', 18));
  const aiBtn = h('button', { class: 'btn small p-ai', type: 'button', title: 'Ask Claude (C)' }, icon('spark', 16), h('span', { text: 'Ask Claude' }));
  const exitBtn = h('button', { class: 'icon-btn', type: 'button', title: 'Leave the quiz (progress is saved)', 'aria-label': 'Leave the quiz' }, icon('x', 20));
  const meta = h('div', { class: 'p-meta' });
  const prompt = h('div', { class: 'p-prompt', role: 'heading', 'aria-level': '2' });
  const hint = h('p', { class: 'p-hint' });
  const answer = h('div', { class: 'p-answer', role: 'group', 'aria-labelledby': 'p-prompt' });
  prompt.id = 'p-prompt';
  const feedback = h('div', { class: 'p-feedback', hidden: true, role: 'status' });
  const backBtn = h('button', { class: 'btn', type: 'button' }, icon('back', 16), ' Back');
  const skipBtn = h('button', { class: 'btn ghost', type: 'button', text: 'Skip' });
  const mainBtn = h('button', { class: 'btn primary big', type: 'button' });
  const keys = h('span', { class: 'p-keys' });
  const scroller = h('div', { class: 'p-scroll' }, h('div', { class: 'p-col' }, meta, prompt, hint, answer));
  const dock = h('div', { class: 'p-dock' }, feedback, h('div', { class: 'p-actions' }, backBtn, keys, skipBtn, mainBtn));

  const chat = createChat({ getState: () => ({ run, q: run.questions[run.i] }), onChange: (open) => { root.classList.toggle('chat-open', open); aiBtn.classList.toggle('on', open); } });
  const root = h('div', { class: 'player-root' },
    h('div', { class: 'p-bar' }, exitBtn, segs, time, flagBtn, aiBtn),
    h('div', { class: 'p-main' }, scroller), dock, chat.el);
  host.replaceChildren(root);
  host.hidden = false;
  document.body.classList.add('playing');

  // ---------- rendering ----------
  function renderSegs() {
    segs.replaceChildren(...run.questions.map((q, n) => {
      const answered = (run.answers[q.id] || []).length > 0;
      let state = answered ? 'answered' : '';
      if (run.checked[q.id]) state = isCorrect(q, run.answers[q.id]) ? 'ok' : 'bad';
      return h('button', { type: 'button', class: `seg ${state}${n === run.i ? ' cur' : ''}${run.flagged[q.id] ? ' flag' : ''}`, title: `Question ${n + 1}${run.flagged[q.id] ? ' (flagged)' : ''}`,
        'aria-label': `Go to question ${n + 1}${answered ? ', answered' : ''}`, 'aria-current': n === run.i ? 'step' : null, onclick: () => goto(n) });
    }));
    segs.querySelector('.cur')?.scrollIntoView({ block: 'nearest', inline: 'center' });
  }
  function renderClock() {
    const left = limitSec(run) - run.elapsed;
    time.replaceChildren(icon('clock', 15), h('b', { text: run.limit ? fmtClock(left) : fmtClock(run.elapsed) }));
    time.classList.toggle('warn', run.limit && left <= 60);
  }
  function renderAnswer(q, checked) {
    const rich = RICH[q.type];
    if (rich) { answer.replaceChildren(rich(h, q, [...sel()], checked, setAnswer)); return; }
    const picked = new Set(sel());
    const pre = q.type === 'assertion' ? [assertionBlock(h, q)] : [];
    answer.replaceChildren(...pre, ...q.options.map((o, idx) => {
      const right = q.correct.includes(o.id);
      const isPicked = picked.has(o.id);
      let cls = 'opt' + (q.type === 'multi' ? ' multi' : '') + (isPicked && !checked ? ' on' : '');
      if (checked) cls += right && isPicked ? ' right' : right ? ' missed' : isPicked ? ' wrong' : '';
      const badge = checked ? (right ? icon('check', 15) : isPicked ? icon('x', 15) : numberFor(idx)) : numberFor(idx);
      return h('button', { class: cls, type: 'button', 'data-id': o.id, role: q.type === 'multi' ? 'checkbox' : 'radio', 'aria-checked': String(isPicked), disabled: checked, onclick: () => choose(o.id) },
        h('span', { class: 'key' }, badge),
        h('span', { class: 'txt' }, ...richText(h, o.text), checked && o.why ? h('span', { class: 'why' }, ...richText(h, o.why)) : null));
    }));
  }
  function renderFeedback(q, checked) {
    feedback.hidden = !(checked && run.instant);
    if (feedback.hidden) return;
    const ok = isCorrect(q, sel());
    const partly = !ok && q.type === 'multi' && sel().some((id) => q.correct.includes(id));
    const detail = RICH[q.type] && !ok ? h('p', { class: 'fb-detail' }, h('b', { text: 'Answer: ' }), correctText(q).length > 160 ? 'see the explanation' : correctText(q))
      : !ok && !RICH[q.type] && q.type !== 'assertion' ? h('p', { class: 'fb-detail', text: `Correct: ${q.correct.map((id) => numberFor(q.options.findIndex((o) => o.id === id))).join(' and ')}` }) : null;
    feedback.className = `p-feedback ${ok ? 'ok' : 'no'}`;
    feedback.replaceChildren(h('div', { class: 'fb-icon' }, icon(ok ? 'check' : 'x', 20)),
      h('div', { class: 'fb-text' }, h('b', { text: ok ? 'Correct!' : partly ? 'Partly right: you need exactly the right set.' : 'Not quite.' }), detail),
      h('button', { class: 'btn small', type: 'button', text: chat.isOpen() ? 'Hide explanation' : 'Show explanation', onclick: () => chat.toggle() }));
  }
  function renderQuestion() {
    const q = cur();
    const checked = !!run.checked[q.id];
    const has = sel().length > 0;
    renderSegs(); renderClock();
    flagBtn.setAttribute('aria-pressed', String(!!run.flagged[q.id])); flagBtn.classList.toggle('on', !!run.flagged[q.id]);
    meta.replaceChildren(h('span', { class: 'chip', text: q.concept }), h('span', { class: 'dots', 'aria-label': `Difficulty ${q.difficulty} of 3` }, ...[1, 2, 3].map((n) => h('i', { class: n <= q.difficulty ? 'on' : '' }))),
      h('span', { class: 'p-count', text: `Question ${run.i + 1} of ${run.questions.length}` }));
    prompt.replaceChildren(...richText(h, q.prompt));
    hint.textContent = HINTS[q.type] || '';
    hint.hidden = !hint.textContent;
    renderAnswer(q, checked);
    renderFeedback(q, checked);
    backBtn.disabled = run.i === 0;
    const last = run.i === run.questions.length - 1;
    if (run.instant) {
      mainBtn.textContent = checked ? (last ? 'See results' : 'Continue') : 'Check';
      mainBtn.disabled = !checked && !has;
      skipBtn.hidden = checked || !has && false;
      skipBtn.hidden = checked || last;
    } else {
      mainBtn.textContent = last ? 'Finish' : has ? 'Next' : 'Skip';
      mainBtn.disabled = false;
      skipBtn.hidden = true;
    }
    mainBtn.classList.toggle('ok', checked && isCorrect(q, sel()) && run.instant);
    keys.textContent = RICH[q.type] ? 'Enter to continue · ← → move · F flag · C Claude' : '1-6 choose · Enter check / continue · ← → move · F flag · C Claude';
    chat.render();
    persistRun();
  }

  function setAnswer(arr) {
    const q = cur();
    if (run.checked[q.id]) return;
    if (arr.length) run.answers[q.id] = arr; else delete run.answers[q.id];
    if (run.instant) mainBtn.disabled = arr.length === 0; else mainBtn.textContent = run.i === run.questions.length - 1 ? 'Finish' : arr.length ? 'Next' : 'Skip';
    renderSegs();
    persistRun();
  }
  function choose(id) {
    const q = cur();
    if (run.checked[q.id]) return;
    const now = new Set(sel());
    if (q.type === 'multi') { if (now.has(id)) now.delete(id); else now.add(id); } else { now.clear(); now.add(id); }
    run.answers[q.id] = [...now];
    if (!run.answers[q.id].length) delete run.answers[q.id];
    renderAnswer(q, false);
    setAnswer(run.answers[q.id] || []);
    answer.querySelector(`[data-id="${id}"]`)?.focus({ preventScroll: true });
  }
  function check() {
    const q = cur();
    if (!sel().length || run.checked[q.id]) return;
    run.checked[q.id] = true;
    renderQuestion();
    chat.autoOpenOnCheck();
    renderFeedback(q, true);
    mainBtn.focus({ preventScroll: true });
  }
  function goto(n) {
    if (n < 0 || n >= run.questions.length || n === run.i) return;
    run.i = n;
    renderQuestion();
    scroller.scrollTop = 0;
  }
  function next() {
    if (run.i < run.questions.length - 1) { goto(run.i + 1); } else finish();
  }
  function toggleFlag() {
    const q = cur();
    if (run.flagged[q.id]) delete run.flagged[q.id]; else run.flagged[q.id] = true;
    renderQuestion();
  }
  function onMain() {
    const q = cur();
    if (run.instant && !run.checked[q.id]) check(); else next();
  }

  function finish() {
    stop();
    const result = scoreAttempt({ questions: run.questions }, run.answers);
    result.seconds = Math.round(run.elapsed);
    run.result = result;
    clearRun();
    recordAttempt(run, result, isCorrect);
    S.last = { run, result };
    close();
    onFinish();
  }
  async function leave() {
    const blank = run.questions.filter((q) => !(run.answers[q.id] || []).length).length;
    const choice = await dialog({
      title: 'Leave this quiz?',
      body: h('p', { text: `Your progress is saved, so you can resume later from the Quizzes page.${blank ? ` ${blank} question${blank > 1 ? 's are' : ' is'} still unanswered.` : ''}` }),
      actions: [{ label: 'Keep going', value: 'stay' }, { label: 'Finish now and see results', value: 'finish' }, { label: 'Save and exit', value: 'exit', kind: 'primary' }],
    });
    if (choice === 'finish') finish();
    else if (choice === 'exit') { persistRun(); stop(); close(); onExit(); }
  }

  // ---------- timer, keyboard, lifecycle ----------
  function start() {
    let last = Date.now();
    tick = setInterval(() => {
      const now = Date.now();
      run.elapsed += Math.min(2, (now - last) / 1000);
      last = now;
      renderClock();
      if (run.limit && run.elapsed >= limitSec(run)) { stop(); import('./dom.js').then((d) => d.toast('Time is up: here are your results.')); finish(); }
    }, 500);
  }
  function stop() { if (tick) clearInterval(tick); tick = null; }
  function onKey(e) {
    if (e.ctrlKey || e.metaKey || e.altKey || document.querySelector('dialog[open]')) return;
    const t = e.target;
    if (t && (t.tagName === 'INPUT' && t.type !== 'checkbox' || t.tagName === 'TEXTAREA' || t.tagName === 'SELECT')) return;
    const q = cur();
    const k = e.key;
    if (k === 'Escape') { e.preventDefault(); if (chat.isOpen()) chat.close({ user: true }); else leave(); return; }
    if (k === 'ArrowLeft') { e.preventDefault(); goto(run.i - 1); return; }
    if (k === 'ArrowRight') { e.preventDefault(); goto(run.i + 1); return; }
    if (k.toLowerCase() === 'f') { e.preventDefault(); toggleFlag(); return; }
    if (k.toLowerCase() === 'c') { e.preventDefault(); chat.toggle(); return; }
    if (/^[1-9]$/.test(k) && !RICH[q.type] && q.type !== 'assertion' ? Number(k) - 1 < q.options.length : /^[1-4]$/.test(k) && q.type === 'assertion') {
      e.preventDefault(); choose(q.options[Number(k) - 1].id); return;
    }
    if (k === 'Enter' && !(t && t.tagName === 'BUTTON' && !t.classList.contains('opt') && t !== mainBtn)) { e.preventDefault(); if (!mainBtn.disabled) onMain(); }
  }
  function close() {
    stop();
    document.removeEventListener('keydown', onKey);
    document.body.classList.remove('playing');
    host.hidden = true;
    host.replaceChildren();
  }

  exitBtn.addEventListener('click', leave);
  flagBtn.addEventListener('click', toggleFlag);
  aiBtn.addEventListener('click', () => chat.toggle());
  backBtn.addEventListener('click', () => goto(run.i - 1));
  skipBtn.addEventListener('click', next);
  mainBtn.addEventListener('click', onMain);
  document.addEventListener('keydown', onKey);
  renderQuestion();
  start();
  return { close, stop, leave, modeLabel: () => modeLabel(run.mode) };
}

// What the buttons do: start / resume a quiz, the start dialog, AI quiz generation, the progress dialog, deleting a custom quiz.
import { validateQuiz } from '/shared/quiz.js';
import { DB, S, declineSync, exportProgress, importProgressFile, resetAll, store, syncNow } from './store.js';
import { QUICK_COUNT, buildRun, buildWeakRun, clearRun, resumeSaved, savedRun, settings, weakRefs } from './data.js';
import { confirmDialog, dialog, h, icon, plural, toast } from './dom.js';

export const go = (hash) => { location.hash = hash; };
const play = () => { go('#/play'); };

async function begin(promise) {
  try { await promise; play(); } catch (err) { toast(err.message || 'Could not start that quiz.', 'err'); }
}
export const resume = () => begin(resumeSaved().catch((e) => { clearRun(); throw new Error(e.message === 'empty' || e.message === 'No saved quiz.' ? 'That saved quiz could not be restored.' : e.message); }));
export const startWeak = () => begin(buildWeakRun());
export const startMissed = (quizId) => begin(settingsThen(() => buildRun(quizId, 'missed')));
const settingsThen = (fn) => fn();

// "Start" opens a small dialog: feedback style, timer and how many questions. The choices are remembered.
export async function startFlow(quiz, { defaultMode = 'full' } = {}) {
  const cur = settings();
  const full = quiz.questionCount;
  let mode = defaultMode === 'quick' && full > QUICK_COUNT + 2 ? 'quick' : 'full';
  let instant = cur.instant;
  let limit = cur.limit;
  const radio = (name, value, label, desc, checked, onPick) => h('label', { class: 'choice' },
    h('input', { type: 'radio', name, value, checked, onchange: () => onPick(value) }), h('span', { class: 'choice-body' }, h('b', { text: label }), desc ? h('small', { text: desc }) : null));
  const body = h('div', { class: 'start-opts' },
    h('p', { class: 'muted', text: `${quiz.title} · ${plural(full, 'question')} · about ${quiz.estimatedMinutes} min` }),
    h('fieldset', {}, h('legend', { text: 'Feedback' }),
      radio('fb', 'instant', 'Check each answer', 'See right or wrong after every question, with the explanation.', instant, () => { instant = true; }),
      radio('fb', 'exam', 'Exam style', 'No feedback until the end, then a full review.', !instant, () => { instant = false; })),
    full > QUICK_COUNT + 2 ? h('fieldset', {}, h('legend', { text: 'Length' }),
      radio('len', 'full', `All ${full} questions`, '', mode === 'full', (v) => { mode = v; }),
      radio('len', 'quick', `Quick ${QUICK_COUNT}`, 'A random sample for a fast round.', mode === 'quick', (v) => { mode = v; })) : null,
    h('label', { class: 'choice solo' }, h('input', { type: 'checkbox', checked: limit, onchange: (e) => { limit = e.target.checked; } }), h('span', { class: 'choice-body' }, h('b', { text: 'Time limit' }), h('small', { text: 'About 45 seconds per question.' }))));
  const ok = await dialog({ title: 'Start quiz', body, actions: [{ label: 'Cancel', value: false }, { label: 'Start', value: true, kind: 'primary' }] });
  if (!ok) return;
  store.set('instant', instant); store.set('limit', limit);
  await begin(buildRun(quiz.id, mode));
}

export async function deleteCustom(q) {
  if (!(await confirmDialog(`Delete "${q.title}"? It is removed from your saved quizzes.`, 'Delete', { danger: true, title: 'Delete quiz' }))) return;
  S.custom = S.custom.filter((x) => x.id !== q.id);
  delete S.attempts[q.id];
  store.set('custom', S.custom);
  store.set('attempts', S.attempts);
  window.dispatchEvent(new Event('hashchange'));
}

// ---------- create a quiz with AI ----------
export function openGenerator() {
  const topic = h('input', { type: 'text', maxlength: 200, placeholder: 'e.g. Database sharding and consistent hashing', required: true });
  const notes = h('textarea', { rows: 5, maxlength: 12000, placeholder: 'Paste an article or your notes. Questions will be based on them.' });
  const count = h('select', {}, ...[5, 10, 15, 20, 25].map((n) => h('option', { value: n, text: String(n), selected: n === 10 })));
  const level = h('select', {}, h('option', { value: 'beginner', text: 'Beginner' }), h('option', { value: 'intermediate', text: 'Intermediate', selected: true }), h('option', { value: 'advanced', text: 'Advanced' }));
  const status = h('p', { class: 'gen-status', role: 'status' });
  const off = S.health.ai === 'off';
  const body = h('form', { class: 'gen-form', autocomplete: 'off' },
    off ? h('p', { class: 'notice' }, 'Claude is not connected. Click "Add key" at the top, or start the server with demo mode.') : null,
    h('label', {}, 'Topic', topic), h('label', {}, h('span', {}, 'Study notes ', h('small', { class: 'muted', text: '(optional)' })), notes),
    h('div', { class: 'row2' }, h('label', {}, 'Questions', count), h('label', {}, 'Level', level)), status);
  return dialog({
    title: 'Create a quiz with Claude', body, wide: true,
    actions: [{ label: 'Close', value: null }, { label: 'Generate quiz', kind: 'primary', onclick: async (finish) => {
      if (!topic.value.trim()) { topic.focus(); return; }
      if (S.health.ai === 'off') { status.textContent = 'Claude is not connected.'; status.className = 'gen-status err'; return; }
      status.textContent = 'Claude is writing your quiz… this can take up to a minute.'; status.className = 'gen-status';
      try {
        const res = await fetch('/api/quiz/generate', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ topic: topic.value, notes: notes.value, count: Number(count.value), level: level.value }) });
        const data = await res.json().catch(() => ({}));
        if (!res.ok) throw new Error(data.error || `Request failed (${res.status}).`);
        const errs = validateQuiz(data.quiz);
        if (errs.length) throw new Error(`The generated quiz was malformed (${errs[0]}). Please try again.`);
        S.custom = [data.quiz, ...S.custom].slice(0, 20);
        store.set('custom', S.custom);
        finish(data.quiz);
        toast(`Created "${data.quiz.title}" with ${data.quiz.questions.length} questions.`, 'ok');
        go('#/topic/mine');
      } catch (err) { status.textContent = err.message; status.className = 'gen-status err'; }
    } }],
  });
}

// ---------- progress: where it is saved, sync, backup, reset ----------
export function openProgressDialog() {
  const body = h('div', { class: 'prog' });
  const draw = () => {
    body.replaceChildren(
      h('p', {}, DB.on ? ['Your progress is saved in ', h('code', { text: DB.file || 'hld/data/progress.db' }), DB.branch ? [' on branch ', h('b', { text: DB.branch })] : '', '. Commit that file on your own branch to keep it. ']
        : 'Your progress is saved in this browser only (the progress database is not available).'),
      DB.offer ? h('div', { class: 'notice' }, h('b', { text: 'Progress found in this browser. ' }), DB.info?.empty === false ? 'Merge it into the progress database?' : 'Save it into the progress database so it travels with your branch?',
        h('div', { class: 'row-btns' }, h('button', { class: 'btn small primary', type: 'button', text: 'Sync to database', onclick: async () => { try { await syncNow(); toast('Synced to the database.', 'ok'); draw(); } catch (e) { toast(e.message, 'err'); } } }),
          h('button', { class: 'btn small', type: 'button', text: 'No thanks', onclick: () => { declineSync(); draw(); } }))) : null,
      h('div', { class: 'row-btns' },
        h('button', { class: 'btn', type: 'button', text: 'Export backup', onclick: exportProgress }),
        h('button', { class: 'btn', type: 'button', text: 'Import backup', onclick: () => { const input = h('input', { type: 'file', accept: 'application/json,.json' }); input.addEventListener('change', async () => { try { await importProgressFile(input.files[0]); toast('Progress imported.', 'ok'); } catch { toast('That file is not a quiz progress export.', 'err'); } }); input.click(); } })),
      h('div', { class: 'danger-zone' }, h('b', { text: 'Reset' }), h('p', { class: 'muted', text: 'Deletes every result, best score, generated quiz and any quiz in progress. Everything goes back to zero.' }),
        h('button', { class: 'btn danger', type: 'button', text: 'Reset all progress', onclick: async () => {
          if (!(await confirmDialog(`Reset ALL quiz progress to zero in ${DB.on ? 'the database (hld/data/progress.db) and this browser' : 'this browser'}? This cannot be undone.`, 'Reset everything', { danger: true, title: 'Reset progress' }))) return;
          try { await resetAll(); toast('All progress was reset to zero.', 'ok'); draw(); } catch (e) { toast(`Reset failed: ${e.message}`, 'err'); }
        } })));
  };
  draw();
  return dialog({ title: 'Your progress', body, actions: [{ label: 'Done', value: true, kind: 'primary' }] });
}
export { savedRun, weakRefs };

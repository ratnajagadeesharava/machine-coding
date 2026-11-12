// The "Ask Claude" drawer inside the quiz player. One conversation per question. Before the learner answers Claude explains and nudges but does not give
// the answer away; after answering it shows the question's explanation as its first message and can discuss anything. The server gets the whole
// conversation, the question and the learner's answer on every message (see /api/quiz/explain).
import { isCorrect } from '/shared/quiz.js';
import { richText } from '../rich.js';
import { S, store } from './store.js';
import { h, icon, streamSse } from './dom.js';

const AUTO_TEXT = 'Explain this question more clearly.';
const RICH = new Set(['integer', 'matrix', 'fill', 'code']);

// Claude ends each reply with "SUGGESTIONS: a | b | c" (see server/quiz.js). The learner sees the answer without that line and the three follow-ups
// become the quick replies. While a reply streams the unfinished tail is hidden too.
export function splitSuggestions(raw) {
  const m = /(?:^|\n)\s*SUGGESTIONS:\s*([\s\S]*)$/.exec(raw);
  if (!m) { const partial = /(?:^|\n)\s*S(?:U(?:G(?:G(?:E(?:S(?:T(?:I(?:O(?:N(?:S)?)?)?)?)?)?)?)?)?)?:?\s*$/.exec(raw); return { body: partial ? raw.slice(0, partial.index) : raw, suggestions: [] }; }
  const suggestions = m[1].split('|').map((x) => x.replace(/^[\s"'-]+|[\s"']+$/g, '').trim()).filter((x) => x && x.length <= 90).slice(0, 3);
  return { body: raw.slice(0, m.index), suggestions };
}

const tutorQuestion = (q) => {
  const { id, concept, type, difficulty, prompt, explanation, options, correct, answer, tolerance, unit, assertion, reason, whys, left, right, pairs, text, blanks, bank, language, starter, checks, model } = q;
  return { id, concept, type, difficulty, prompt, explanation, options, correct, answer, tolerance, unit, assertion, reason, whys, left, right, pairs, text, blanks, bank, language, starter, checks, model };
};

export function createChat({ getState, onChange }) {
  const log = h('div', { class: 'dr-log', 'aria-live': 'polite' });
  const state = h('div', { class: 'dr-state' });
  const chips = h('div', { class: 'dr-chips' });
  const input = h('textarea', { rows: 2, maxlength: 800, placeholder: 'Ask about this question, or just chat...', 'aria-label': 'Message to Claude' });
  const send = h('button', { class: 'btn primary', type: 'submit', text: 'Send' });
  const form = h('form', { class: 'dr-form', autocomplete: 'off' }, input, send);
  const clear = h('button', { class: 'btn small ghost', type: 'button', text: 'Clear', title: 'Start this chat over' });
  const close = h('button', { class: 'icon-btn', type: 'button', 'aria-label': 'Close the chat' }, icon('x', 18));
  const el = h('aside', { class: 'drawer', 'aria-label': 'Ask Claude about this question', hidden: true },
    h('header', { class: 'dr-head' }, h('span', { class: 'dr-title' }, icon('spark', 18), ' Ask Claude'), h('span', { class: 'dr-tools' }, clear, close)),
    state, log, chips, form, h('p', { class: 'dr-note', text: 'Claude can make mistakes. Until you answer, it will not give the answer away.' }));

  let open = false;
  const chipsHidden = () => store.get('chipsHidden', false) === true;
  const aiOff = () => S.health.ai === 'off';

  function openingChips(q, checked) {
    const c = q.concept;
    return checked
      ? [`Explain "${c}" in simpler words.`, 'Where could a learner go wrong here?', 'How would I say this in an interview?']
      : [AUTO_TEXT, `What is "${c}" in simple terms?`, 'Give me a hint without the answer.'];
  }
  function bubble(m, last, busy) {
    if (m.role === 'user') return h('div', { class: 'msg user' }, h('div', { class: 'bubble' }, ...richText(h, m.content)));
    const body = m.error ? [m.content] : richText(h, splitSuggestions(m.content || '').body.trim());
    return h('div', { class: 'msg bot' + (m.error ? ' err' : '') }, h('div', { class: 'avatar', 'aria-hidden': 'true' }, icon('spark', 14)),
      h('div', { class: 'bubble' }, ...body, busy && last ? h('span', { class: 'typing', 'aria-label': 'Claude is typing' }, h('i'), h('i'), h('i')) : null));
  }
  function renderChips(q, checked, t) {
    chips.replaceChildren(); chips.hidden = true;
    if (aiOff() || t.busy) return;
    const lastBot = [...t.log].reverse().find((m) => m.role === 'assistant' && !m.error);
    const list = t.log.length ? (lastBot ? splitSuggestions(lastBot.content).suggestions : []) : openingChips(q, checked);
    const used = new Set(t.log.filter((m) => m.role === 'user').map((m) => m.content));
    const items = list.filter((c) => !used.has(c));
    if (!items.length) return;
    chips.hidden = false;
    if (chipsHidden()) { chips.append(h('button', { class: 'link-btn', type: 'button', text: 'Show suggestions', onclick: () => { store.setLocal('chipsHidden', false); render(); } })); return; }
    chips.append(h('div', { class: 'dr-chips-head' }, h('span', { class: 'eyebrow', text: t.log.length ? 'You could ask next' : 'Try asking' }),
      h('button', { class: 'link-btn', type: 'button', title: 'Hide the suggestions', text: 'Hide', onclick: () => { store.setLocal('chipsHidden', true); render(); } })),
    ...items.map((text) => h('button', { class: 'chip-btn', type: 'button', text, onclick: () => sendChat(text, { auto: text === AUTO_TEXT && !t.log.length }) })));
  }

  function render() {
    el.hidden = !open;
    if (!open) return;
    const { run, q } = getState();
    if (!run || !q) return;
    const checked = !!run.checked[q.id];
    const t = run.tutor[q.id] || { log: [], busy: false };
    state.textContent = aiOff() ? '' : checked ? 'You answered: Claude can discuss the answer freely.' : 'Not answered yet: Claude will help without giving the answer away.';
    const keep = log.scrollTop;
    log.replaceChildren();
    if (checked) {
      log.append(h('div', { class: 'msg bot explain' }, h('div', { class: 'avatar', 'aria-hidden': 'true' }, icon('spark', 14)),
        h('div', { class: 'bubble' }, h('div', { class: 'eyebrow', text: 'Explanation' }), ...richText(h, q.explanation))));
    }
    if (aiOff()) {
      log.append(h('div', { class: 'dr-empty' }, h('b', { text: 'Claude chat is not connected.' }), h('p', { class: 'muted' }, 'Click ', h('button', { class: 'link-btn', type: 'button', text: 'Add key', onclick: () => document.querySelector('[data-keybtn]')?.click() }), ' at the top and paste your Anthropic API key (it stays in this browser) to ask follow-up questions, or start the server with demo mode.')));
    } else if (!t.log.length && !checked) {
      log.append(h('div', { class: 'dr-empty' }, h('b', { text: 'Stuck, or want to go deeper?' }), h('p', { class: 'muted', text: 'Ask me to explain this question in simpler words, give a hint, compare the options, or just discuss it.' })));
    } else t.log.forEach((m, i) => log.append(bubble(m, i === t.log.length - 1, t.busy)));
    log.scrollTop = keep;
    renderChips(q, checked, t);
    input.disabled = aiOff() || t.busy; send.disabled = aiOff() || t.busy;
  }

  // While a reply streams only that bubble is rebuilt: the rest of the drawer, the scroll position and any text selection stay put.
  function updateStreaming(reply) {
    const last = log.lastElementChild;
    const bub = last?.classList.contains('bot') ? last.querySelector('.bubble') : null;
    if (!bub) return render();
    bub.replaceChildren(...richText(h, splitSuggestions(reply.content).body.trim()), h('span', { class: 'typing', 'aria-label': 'Claude is typing' }, h('i'), h('i'), h('i')));
  }

  async function sendChat(text, { auto = false } = {}) {
    const { run, q } = getState();
    if (!run || !q || aiOff()) return;
    const t = run.tutor[q.id] || (run.tutor[q.id] = { log: [], busy: false });
    if (t.busy || !text.trim()) return;
    t.busy = true;
    t.log.push({ role: 'user', content: text.trim() });
    const history = t.log.filter((m) => !m.error && m.content).map((m) => ({ role: m.role, content: m.role === 'assistant' ? splitSuggestions(m.content).body.trim() : m.content }));
    const reply = { role: 'assistant', content: '' };
    t.log.push(reply);
    render();
    const users = log.querySelectorAll('.msg.user');
    const lastUser = users[users.length - 1];
    if (lastUser) log.scrollTop = Math.max(0, lastUser.offsetTop - log.offsetTop - 8); // read the reply from its start; no auto-scroll after this
    const answered = !!run.checked[q.id];
    const sel = run.answers[q.id] || [];
    const optionLike = !RICH.has(q.type);
    let acc = 0;
    try {
      await streamSse('/api/quiz/explain', { question: tutorQuestion(q), selected: optionLike ? sel : [], answer: optionLike ? [] : sel, checked: answered, auto, history }, (d) => {
        reply.content += d;
        const now = Date.now();
        if (now - acc > 60 && getState().q === q) { acc = now; updateStreaming(reply); }
      });
      if (!splitSuggestions(reply.content).body.trim()) throw new Error('Claude sent an empty reply. Try again.');
    } catch (err) {
      const body = splitSuggestions(reply.content).body.trim();
      if (body) { reply.content = body; t.log.push({ role: 'assistant', content: err.message, error: true }); } else { reply.content = err.message; reply.error = true; }
    } finally {
      t.busy = false;
      if (getState().q === q) render();
    }
  }

  const setOpen = (v, { user = false } = {}) => {
    open = v;
    if (user) store.setLocal('chatAuto', v); // closing it by hand turns off the auto-open after each answer
    render();
    onChange?.(open);
  };
  form.addEventListener('submit', (e) => { e.preventDefault(); const text = input.value.trim(); if (!text) return; input.value = ''; sendChat(text); });
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter' && !e.shiftKey && !e.isComposing) { e.preventDefault(); form.requestSubmit(); } e.stopPropagation(); });
  close.addEventListener('click', () => setOpen(false, { user: true }));
  clear.addEventListener('click', () => { const { run, q } = getState(); const t = run?.tutor[q?.id]; if (t && !t.busy) { t.log = []; render(); } });

  return {
    el, render,
    isOpen: () => open,
    open: (opts) => setOpen(true, opts),
    close: (opts) => setOpen(false, opts),
    toggle: () => setOpen(!open, { user: true }),
    autoOpenOnCheck: () => { if (store.get('chatAuto', true) !== false && !open) setOpen(true); else render(); },
  };
}

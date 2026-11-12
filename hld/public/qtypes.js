// Widgets for the JEE-Advanced style question types: integer, assertion-reason (rendered as options by quiz.js, so only the
// statement block lives here), matrix match, drag-the-words fill, and short code answers. Each widget gets the answer as a
// string[] (same shape the grader in /shared/quiz.js expects) and reports changes through onChange.
import { codeChecks } from '/shared/quiz.js';
import { richText } from './rich.js';

const TOKEN = /\{([A-Za-z0-9]+)\}/g;

function pairs(answer) {
  const m = new Map();
  for (const s of answer) { const i = s.indexOf('='); if (i > 0) m.set(s.slice(0, i), s.slice(i + 1)); }
  return m;
}
const toAnswer = (m) => [...m].map(([k, v]) => `${k}=${v}`);

export function assertionBlock(h, q) {
  return h('div', { class: 'ar' },
    h('div', { class: 'ar-row' }, h('b', { text: 'Assertion (A): ' }), ...richText(h, q.assertion)),
    h('div', { class: 'ar-row' }, h('b', { text: 'Reason (R): ' }), ...richText(h, q.reason)));
}

export function integerWidget(h, q, answer, checked, onChange) {
  const input = h('input', {
    class: 'int-input', type: 'text', inputmode: 'decimal', autocomplete: 'off', spellcheck: 'false', 'aria-label': 'Your numeric answer',
    placeholder: 'Type a number', value: answer[0] ?? '', disabled: checked,
  });
  input.addEventListener('input', () => onChange(input.value.trim() ? [input.value.trim()] : []));
  const kids = [input, q.unit ? h('span', { class: 'int-unit', text: q.unit }) : null];
  if (checked) {
    const ok = Math.abs(Number(answer[0]) - q.answer) <= (q.tolerance || 0);
    kids.push(h('p', { class: `int-ans ${ok ? 'ok' : 'no'}` }, ok ? 'Correct.' : `Answer: ${q.answer}${q.unit ? ' ' + q.unit : ''}`));
  }
  return h('div', { class: 'int' }, ...kids);
}

export function matrixWidget(h, q, answer, checked, onChange) {
  const got = pairs(answer);
  const rows = q.left.map((l) => {
    const sel = h('select', { 'aria-label': `Match for ${l.text}`, disabled: checked },
      h('option', { value: '', text: 'Choose...' }),
      ...q.right.map((r, i) => h('option', { value: r.id, text: `${String.fromCharCode(80 + i)}. ${r.text.replace(/`/g, '')}`, selected: got.get(l.id) === r.id })));
    sel.addEventListener('change', () => { if (sel.value) got.set(l.id, sel.value); else got.delete(l.id); onChange(toAnswer(got)); });
    const right = q.pairs[l.id];
    const state = checked ? (got.get(l.id) === right ? ' right' : ' wrong') : '';
    const rr = q.right.find((r) => r.id === right);
    return h('div', { class: `mx-row${state}` },
      h('span', { class: 'mx-l' }, `${l.id.toUpperCase()}. `, ...richText(h, l.text)), sel,
      checked && got.get(l.id) !== right ? h('span', { class: 'mx-fix', text: `Correct: ${rr.text}` }) : null);
  });
  const legend = h('div', { class: 'mx-right' }, ...q.right.map((r, i) => h('span', {}, `${String.fromCharCode(80 + i)}. `, ...richText(h, r.text))));
  return h('div', { class: 'mx' }, h('p', { class: 'mx-note', text: 'Match each item in Column I with one item in Column II. Each Column II item is used at most once.' }), ...rows, h('div', { class: 'mx-colhead', text: 'Column II' }), legend);
}

export function fillWidget(h, q, answer, checked, onChange) {
  const got = pairs(answer); // blankId -> word
  let armed = null; // word picked up by click, waiting for a blank
  const root = h('div', { class: 'fill' });
  const redraw = () => {
    const used = new Set(got.values());
    const parts = [];
    let last = 0;
    const text = q.text;
    for (const m of text.matchAll(TOKEN)) {
      parts.push(text.slice(last, m.index));
      const id = m[1];
      const word = got.get(id);
      const right = q.blanks[id];
      const cls = `blank${word ? ' filled' : ''}${checked ? (word === right ? ' right' : ' wrong') : ''}`;
      const slot = h('button', { type: 'button', class: cls, 'data-blank': id, disabled: checked, 'aria-label': word ? `Blank ${id}: ${word}. Press to clear.` : `Blank ${id}: empty` }, word || `(${id})`);
      slot.addEventListener('click', () => {
        if (word) { got.delete(id); } else if (armed) { got.set(id, armed); armed = null; } else return;
        onChange(toAnswer(got)); redraw();
      });
      slot.addEventListener('dragover', (e) => { if (!checked) e.preventDefault(); });
      slot.addEventListener('drop', (e) => {
        e.preventDefault();
        const w = e.dataTransfer.getData('text/plain');
        if (!w || !q.bank.includes(w)) return;
        for (const [k, v] of got) if (v === w) got.delete(k);
        got.set(id, w); armed = null; onChange(toAnswer(got)); redraw();
      });
      parts.push(slot);
      if (checked && word !== right) parts.push(h('span', { class: 'fix', text: ` [${right}]` }));
      last = m.index + m[0].length;
    }
    parts.push(text.slice(last));
    const bank = q.bank.map((w) => {
      const chip = h('button', { type: 'button', class: `chip${used.has(w) ? ' used' : ''}${armed === w ? ' armed' : ''}`, draggable: checked || used.has(w) ? 'false' : 'true', disabled: checked || used.has(w), text: w });
      chip.addEventListener('dragstart', (e) => { e.dataTransfer.setData('text/plain', w); e.dataTransfer.effectAllowed = 'move'; });
      chip.addEventListener('click', () => { armed = armed === w ? null : w; redraw(); });
      return chip;
    });
    root.replaceChildren(
      h('p', { class: 'fill-text' }, ...parts),
      h('p', { class: 'fill-note', text: checked ? '' : 'Drag a word into a blank, or click a word and then a blank. Click a filled blank to clear it.' }),
      h('div', { class: 'bank' }, ...bank));
  };
  redraw();
  return root;
}

export function codeWidget(h, q, answer, checked, onChange) {
  const ta = h('textarea', { class: 'code-input', rows: Math.max(6, (q.starter || '').split('\n').length + 3), spellcheck: 'false', autocapitalize: 'off', 'aria-label': `Your ${q.language} code`, disabled: checked });
  ta.value = answer[0] ?? q.starter ?? '';
  ta.addEventListener('input', () => onChange(ta.value.trim() && ta.value !== (q.starter ?? '') ? [ta.value] : []));
  ta.addEventListener('keydown', (e) => {
    if (e.key === 'Tab') { e.preventDefault(); const s = ta.selectionStart; ta.setRangeText('    ', s, ta.selectionEnd, 'end'); ta.dispatchEvent(new Event('input')); }
  });
  const kids = [h('div', { class: 'code-lang', text: q.language }), ta];
  if (checked) {
    const results = codeChecks(q, answer[0] ?? '');
    kids.push(h('ul', { class: 'code-checks' }, ...results.map((r) => h('li', { class: r.ok ? 'ok' : 'no', text: `${r.ok ? '✓' : '✕'} ${r.label}` }))));
    kids.push(h('p', { class: 'code-note', text: 'Checked by pattern, not by running your code: a different but correct solution may fail a check. Compare with the model answer:' }));
    kids.push(h('pre', { class: 'code-model' }, q.model));
  } else {
    kids.push(h('p', { class: 'code-note', text: 'Write the code from memory. It is checked for the key elements; you will see a model answer afterwards.' }));
  }
  return h('div', { class: 'code' }, ...kids);
}

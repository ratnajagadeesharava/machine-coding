// Keyboard shortcuts dialog. openHelp(wb?) opens it; mountHelp(wb, host) wires `request {name:'help'}` and adds a "?" button.
import { h, icon, ensureChromeCss, followTheme } from './icons.js';

const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/i.test(navigator.platform || '');

// Key spec: space separated tokens form one combo, ' | ' separates alternatives. Mod = Ctrl/Cmd, Alt = Alt/Option.
export const SHORTCUTS = {
  Tools: [
    ['Hand (panning tool)', 'H'], ['Selection', 'V | 1'], ['Rectangle', 'R | 2'], ['Diamond', 'D | 3'], ['Ellipse', 'O | 4'], ['Arrow', 'A | 5'], ['Line', 'L | 6'],
    ['Draw', 'P | 7'], ['Text', 'T | 8'], ['Insert image', '9'], ['Eraser', 'E | 0'], ['Database (cylinder)', 'C'], ['Sticky note', 'N'], ['Frame', 'F'],
    ['Laser pointer', 'K'], ['Bucket fill', 'B'], ['Eyedropper', 'I'], ['Keep selected tool active', 'Q'],
  ],
  View: [
    ['Zoom in', 'Mod +'], ['Zoom out', 'Mod -'], ['Reset zoom', 'Mod 0'], ['Zoom to fit', 'Shift 1'], ['Zoom to selection', 'Shift 2'],
    ['Pan canvas', 'Space drag | Middle-drag | Scroll'], ['Zoom with wheel', 'Mod Wheel'], ['Toggle grid', "Mod '"], ['Object snapping', 'Alt S'],
    ['Zen mode', 'Alt Z'], ['View mode', 'Alt R'], ['Toggle theme', 'Alt Shift D'], ['Stats', 'Alt /'], ['Command palette', 'Mod / | Mod Shift P'],
    ['Find on canvas', 'Mod F'], ['Shortcuts', '?'],
  ],
  Editor: [
    ['Edit text or label', 'Enter'], ['Finish / deselect', 'Esc'], ['Delete', 'Delete | Backspace'], ['Select all', 'Mod A'], ['Duplicate', 'Mod D | Alt drag'],
    ['Copy', 'Mod C'], ['Cut', 'Mod X'], ['Paste', 'Mod V'], ['Paste as plain text', 'Mod Shift V'], ['Undo', 'Mod Z'], ['Redo', 'Mod Shift Z | Ctrl Y'],
    ['Group', 'Mod G'], ['Ungroup', 'Mod Shift G'], ['Lock / unlock', 'Mod Shift L'], ['Send backward', 'Mod ['], ['Bring forward', 'Mod ]'],
    ['Send to back', 'Mod Shift ['], ['Bring to front', 'Mod Shift ]'], ['Align', 'Mod Shift Arrow'], ['Flip horizontal', 'Shift H'], ['Flip vertical', 'Shift V'],
    ['Copy style', 'Mod Alt C'], ['Paste style', 'Mod Alt V'], ['Smaller / larger text', 'Mod Shift < | Mod Shift >'], ['Add link', 'Mod K'],
    ['Add connected shape', 'Mod Arrow'], ['Nudge', 'Arrow | Shift Arrow'], ['Select several', 'Shift click'], ['Marquee select', 'drag on empty canvas'],
    ['Resize from centre', 'Alt drag handle'], ['Keep aspect ratio / 45°', 'Shift drag'], ['Snap rotation to 15°', 'Shift rotate'],
    ['Do not bind arrow end', 'Mod drag end'], ['Add arrow point', 'click click click'], ['New text', 'double-click'],
    ['Open file', 'Mod O'], ['Save to file', 'Mod S'], ['Export image', 'Mod Shift E'],
  ],
};

const NICE = { Arrow: '←↑→↓' };
function keyLabel(tok) {
  if (tok === 'Mod') return isMac ? 'Cmd' : 'Ctrl';
  if (tok === 'Alt' && isMac) return 'Option';
  return NICE[tok] || tok;
}
const WORDS = new Set(['drag', 'click', 'double-click', 'Middle-drag', 'Scroll', 'Wheel', 'rotate', 'handle', 'end', 'on', 'empty', 'canvas']);

/** Plain-text form of a key spec, e.g. 'Mod Shift Z | Ctrl Y' -> 'Cmd+Shift+Z or Ctrl+Y'. Used for search and tests. */
export function keysText(spec) {
  return spec.split(' | ').map((combo) => combo.split(' ').map(keyLabel).join('+')).join(' or ');
}

function keyEls(spec) {
  const out = [];
  spec.split(' | ').forEach((combo, i) => {
    if (i) out.push(h('span', { class: 'wbui-or', text: 'or' }));
    const toks = combo.split(' ');
    const wrap = h('span', { class: 'wbui-combo' });
    for (const t of toks) {
      if (WORDS.has(t)) wrap.append(h('span', { class: 'wbui-kbd-text', text: t }));
      else wrap.append(h('kbd', { class: 'wbui-key', text: keyLabel(t) }));
    }
    out.push(wrap);
  });
  return out;
}

let current = null;

export function openHelp(wb) {
  ensureChromeCss();
  if (current && current.open) { current.querySelector('input')?.focus(); return current; }
  const search = h('input', { type: 'search', class: 'wbui-input wbui-help-search', placeholder: 'Search shortcuts', 'aria-label': 'Search shortcuts', dataset: { helpSearch: '' }, autocomplete: 'off', spellcheck: 'false' });
  const closeBtn = h('button', { type: 'button', class: 'wbui-btn', title: 'Close (Esc)', 'aria-label': 'Close', dataset: { action: 'help-close' } }, icon('close', 18));
  const cols = h('div', { class: 'wbui-help-cols' });
  const sections = [];
  for (const [name, list] of Object.entries(SHORTCUTS)) {
    const ul = h('ul', { class: 'wbui-help-list' });
    const rows = list.map(([label, spec]) => {
      const li = h('li', { class: 'wbui-help-row', dataset: { shortcut: label } }, h('span', { class: 'wbui-help-label', text: label }), h('span', { class: 'wbui-help-keys' }, keyEls(spec)));
      li.dataset.search = (label + ' ' + keysText(spec)).toLowerCase();
      ul.append(li);
      return li;
    });
    const sec = h('section', { class: 'wbui-help-col', dataset: { section: name } }, h('h3', { class: 'wbui-help-h', text: name }), ul);
    sections.push({ sec, rows });
    cols.append(sec);
  }
  const empty = h('p', { class: 'wbui-help-empty', hidden: true, text: 'No shortcuts match your search.' });
  search.addEventListener('input', () => {
    const q = search.value.trim().toLowerCase();
    let total = 0;
    for (const { sec, rows } of sections) {
      let n = 0;
      for (const li of rows) { const on = !q || li.dataset.search.includes(q); li.hidden = !on; if (on) n++; }
      sec.hidden = n === 0;
      total += n;
    }
    empty.hidden = total > 0;
  });
  const dlg = h('dialog', { class: 'wbui-dialog wbui-help', 'aria-labelledby': 'wbui-help-title', dataset: { dialog: 'help' } },
    h('header', { class: 'wbui-dlg-head' }, h('h2', { id: 'wbui-help-title', class: 'wbui-dlg-title', text: 'Keyboard shortcuts' }), search, closeBtn),
    h('div', { class: 'wbui-help-body' }, cols, empty));
  closeBtn.addEventListener('click', () => dlg.close());
  dlg.addEventListener('click', (e) => { if (e.target === dlg) dlg.close(); });
  dlg.addEventListener('keydown', (e) => e.stopPropagation());
  let off = null;
  if (wb && wb.on) off = followTheme(wb, dlg);
  const prev = document.activeElement;
  dlg.addEventListener('close', () => {
    if (off) off();
    dlg.remove();
    if (current === dlg) current = null;
    if (prev && prev.isConnected && prev.focus) prev.focus({ preventScroll: true });
  });
  document.body.append(dlg);
  dlg.showModal();
  search.focus();
  current = dlg;
  return dlg;
}

export function mountHelp(wb, host) {
  ensureChromeCss();
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  const root = h('div', { class: 'wbui-root wbui-help-root' });
  const btn = h('button', { type: 'button', class: 'wbui-btn wbui-help-btn', dataset: { action: 'help' }, title: 'Keyboard shortcuts (?)', 'aria-label': 'Keyboard shortcuts' }, icon('help', 20));
  btn.addEventListener('click', () => openHelp(wb));
  root.append(btn);
  host.append(root);
  const offs = [
    wb.on('request', (e) => { if (e && e.name === 'help') openHelp(wb); }),
    wb.on('request:help', () => openHelp(wb)),
    wb.on('mode', () => { root.hidden = !!(wb.mode && wb.mode.zen); }),
    followTheme(wb, root),
  ];
  return { destroy() { offs.forEach((o) => o && o()); root.remove(); if (current) current.close(); } };
}

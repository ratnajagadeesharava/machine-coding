// In-place text editing for the Whiteboard: free text, container labels, arrow labels and frame names (mixed into its prototype).
import { FONT_FAMILIES, FONT_SIZES, LIMITS, createElement, findItem, isContainer, isLinear, newId, resolvePoints } from '../../shared/board.js';
import { frameNameBox, innerBox, linearLabelLayout, polylineMidpoint, textSize } from './geometry.js';
import { themeColor } from './render.js';
import * as ops from './ops.js';
import { creationProps } from './engine-util.js';

function kindOf(el) {
  if (el.type === 'text') return 'text';
  if (el.type === 'frame') return 'frame';
  if (isLinear(el)) return 'linear';
  if (isContainer(el)) return 'container';
  return null;
}

// Grows a shape (around its centre, never shrinking it) until its label fits: wide enough for the longest word, then tall enough for the lines.
function fitContainer(el, v) {
  if (!v) return;
  const opts = { fontFamily: el.fontFamily, fontSize: el.fontSize };
  const longest = Math.max(...v.split(/\s+/).map((wd) => textSize(wd || ' ', opts).w));
  for (let i = 0; i < 10; i++) {
    const ib = innerBox(el);
    const s = textSize(v, { ...opts, maxWidth: ib.w });
    let dw = 0; let dh = 0;
    if (longest > ib.w) dw = Math.ceil((longest - ib.w + 2) * (el.w / ib.w));
    else if (s.h > ib.h) dh = Math.ceil((s.h - ib.h + 2) * (el.h / ib.h));
    if (!dw && !dh) break;
    el.x -= dw / 2; el.y -= dh / 2; el.w += dw; el.h += dh;
  }
}

const el0 = (wb, id) => (findItem(wb._board, id) || {}).type;

// Names people usually give boxes in a system design; shown as one-click suggestions while a shape is being named.
const COMPONENT_NAMES = [
  'Client', 'Web app', 'Mobile app', 'API Gateway', 'Load Balancer', 'Service', 'Auth Service', 'Cache (Redis)', 'Database', 'Read Replica',
  'Message Queue', 'Kafka', 'Worker', 'CDN', 'Object Storage (S3)', 'Search (Elasticsearch)', 'Rate Limiter', 'Notification Service',
];

export function installText(P) {
  P.startEditing = function startEditing(id) { this._beginEdit(id); };

  // Creates an empty text element at pt and opens the editor on it.
  P._newText = function _newText(pt) {
    if (!this._canEdit()) return;
    const b = this._board;
    const props = creationProps('text', this._style, { textAlignSet: this._textAlignSet });
    const fs = props.fontSize || 20;
    const el = createElement('text', { id: newId(b), ...props, x: pt.x, y: pt.y - (fs * 1.25) / 2, w: 8, h: fs * 1.25, text: '' });
    b.items.push(el);
    this._setSelection([el.id]);
    this._beginEdit(el.id, { isNew: true });
  };

  P._editSelected = function _editSelected() {
    if (this._selection.size === 1) this._beginEdit([...this._selection][0]);
  };

  P._beginEdit = function _beginEdit(id, { initial = '', isNew = false } = {}) {
    if (!this._canEdit()) return;
    const el = findItem(this._board, id);
    if (!el) return;
    const kind = kindOf(el);
    if (!kind) return;
    if (this._editor) this._finishEditor();
    if (this._g && this._g.type === 'multi') this._finishMulti();
    const ta = document.createElement('textarea');
    ta.className = `wb-editor wb-editor-${kind}`;
    ta.spellcheck = false;
    ta.setAttribute('autocapitalize', 'off');
    ta.setAttribute('autocomplete', 'off');
    ta.setAttribute('aria-label', kind === 'frame' ? 'Frame name' : 'Text');
    ta.placeholder = kind === 'text' ? 'Type here…' : kind === 'frame' ? 'Frame name' : kind === 'linear' ? 'Label' : 'Type a name';
    ta.wrap = kind === 'text' || kind === 'frame' ? 'off' : 'soft';
    const field = kind === 'frame' ? 'name' : 'text';
    ta.value = (el[field] || '') + initial;
    this._editor = { id, ta, kind, field, isNew, before: el[field] || '', composing: false, done: false };
    this.root.appendChild(ta);
    this._buildFormatBar(this._editor);
    this._renderNow();
    this._placeEditor();
    this._emit('editing', { id });
    const ed = this._editor;
    ta.addEventListener('compositionstart', () => { ed.composing = true; });
    ta.addEventListener('compositionend', () => { ed.composing = false; this._onEditorInput(); });
    ta.addEventListener('input', () => this._onEditorInput());
    ta.addEventListener('keydown', (e) => {
      e.stopPropagation();
      if (e.isComposing || ed.composing || e.keyCode === 229) return;
      if (e.key === 'Escape') { e.preventDefault(); this._finishEditor(); return; }
      // short labels: Enter finishes, Shift+Enter adds a line. Notes and free text keep Enter for a new line (Ctrl+Enter finishes).
      const shortLabel = kind === 'frame' || kind === 'linear' || (kind === 'container' && el.type !== 'sticky');
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey || (shortLabel && !e.shiftKey))) { e.preventDefault(); this._finishEditor(); return; }
      if (e.key === 'Tab') {
        e.preventDefault();
        const first = ed.sugg && ed.sugg[0];
        if (kind === 'container' && el.type !== 'sticky') { if (first) this._pickSuggestion(ed, first); } else if (kind === 'text' || kind === 'container') this._indentEditor(e.shiftKey);
      }
    });
    ta.addEventListener('blur', () => { if (this._editor === ed) this._finishEditor(); });
    ta.addEventListener('pointerdown', (e) => e.stopPropagation());
    ta.addEventListener('wheel', (e) => e.stopPropagation(), { passive: true });
    const focus = () => {
      if (this._editor !== ed) return;
      ta.focus({ preventScroll: true });
      const n = ta.value.length;
      ta.setSelectionRange(n, n);
    };
    focus();
    setTimeout(focus, 0);
    this._onEditorInput(true);
    this._placeConnectors();
  };

  // A small bar above the text being edited: font, size and alignment without leaving the keyboard flow.
  // Its buttons never take focus, so the caret stays in the text and nothing closes the editor.
  P._buildFormatBar = function _buildFormatBar(ed) {
    if (ed.kind === 'frame') return;
    const bar = document.createElement('div');
    bar.className = 'wb-fmtbar';
    bar.setAttribute('role', 'toolbar');
    bar.setAttribute('aria-label', 'Text formatting');
    const btns = [];
    const group = (label) => { const g = document.createElement('span'); g.className = 'wb-fmt-group'; g.setAttribute('role', 'group'); g.setAttribute('aria-label', label); bar.appendChild(g); return g; };
    const add = (g, text, title, prop, value) => {
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'wb-fmt-btn'; b.textContent = text; b.title = title; b.setAttribute('aria-label', title);
      b.addEventListener('mousedown', (e) => e.preventDefault());
      b.addEventListener('pointerdown', (e) => e.stopPropagation());
      b.addEventListener('click', () => { this.setStyle({ [prop]: value }); this._refreshFormatBar(ed); if (ed.ta.isConnected) ed.ta.focus({ preventScroll: true }); });
      g.appendChild(b); btns.push({ b, prop, value });
    };
    const sizes = group('Size');
    for (const [k, v] of Object.entries(FONT_SIZES)) add(sizes, k, `Font size ${k} (${v}px)`, 'fontSize', v);
    if (ed.kind !== 'linear') {
      const fam = group('Font');
      add(fam, 'Aa', 'Hand-drawn font', 'fontFamily', 1); add(fam, 'Sans', 'Clean sans-serif font', 'fontFamily', 2); add(fam, '</>', 'Code font', 'fontFamily', 3);
      const al = group('Align');
      add(al, 'L', 'Align left', 'textAlign', 'left'); add(al, 'C', 'Align centre', 'textAlign', 'center'); add(al, 'R', 'Align right', 'textAlign', 'right');
    }
    const done = document.createElement('button');
    done.type = 'button'; done.className = 'wb-fmt-done'; done.textContent = 'Done'; done.title = 'Finish editing (Esc or Ctrl+Enter)';
    done.addEventListener('mousedown', (e) => e.preventDefault());
    done.addEventListener('click', () => this._finishEditor());
    bar.appendChild(done);
    if (ed.kind === 'container' && el0(this, ed.id) !== 'sticky') {
      ed.suggBox = document.createElement('div');
      ed.suggBox.className = 'wb-fmt-sugg';
      ed.suggBox.setAttribute('aria-label', 'Suggested names');
      bar.appendChild(ed.suggBox);
    }
    bar.addEventListener('pointerdown', (e) => e.stopPropagation());
    bar.addEventListener('mousedown', (e) => e.preventDefault());
    ed.bar = bar; ed.barBtns = btns;
    this.root.appendChild(bar);
  };

  P._pickSuggestion = function _pickSuggestion(ed, name) {
    ed.ta.value = name;
    this._onEditorInput();
    ed.ta.focus({ preventScroll: true });
    ed.ta.setSelectionRange(name.length, name.length);
  };

  P._refreshSuggestions = function _refreshSuggestions(ed) {
    if (!ed.suggBox) return;
    const q = ed.ta.value.trim().toLowerCase();
    const words = (n) => n.toLowerCase().split(/[\s(]+/);
    const rank = (n) => (n.toLowerCase().startsWith(q) ? 0 : words(n).some((w) => w.startsWith(q)) ? 1 : q.length > 2 && n.toLowerCase().includes(q) ? 2 : 9);
    const list = (q ? COMPONENT_NAMES.filter((n) => rank(n) < 9 && n.toLowerCase() !== q).sort((a, b) => rank(a) - rank(b)) : COMPONENT_NAMES).slice(0, 5);
    const same = list.length === (ed.sugg || []).length && list.every((n, i) => n === ed.sugg[i]);
    ed.sugg = list;
    if (same) return;
    ed.suggBox.replaceChildren();
    ed.suggBox.hidden = !list.length;
    if (!list.length) return;
    const lead = document.createElement('span');
    lead.className = 'wb-fmt-lead';
    lead.textContent = 'Tab \u21B9';
    lead.title = 'Press Tab to use the first suggestion';
    ed.suggBox.appendChild(lead);
    for (const n of list) {
      const b = document.createElement('button');
      b.type = 'button'; b.className = 'wb-fmt-chip'; b.textContent = n;
      b.addEventListener('mousedown', (e) => e.preventDefault());
      b.addEventListener('pointerdown', (e) => e.stopPropagation());
      b.addEventListener('click', () => this._pickSuggestion(ed, n));
      ed.suggBox.appendChild(b);
    }
  };

  P._refreshFormatBar = function _refreshFormatBar(ed) {
    const el = findItem(this._board, ed.id);
    if (!el || !ed.barBtns) return;
    for (const { b, prop, value } of ed.barBtns) {
      const cur = prop === 'textAlign' ? (el.textAlign || (ed.kind === 'container' ? 'center' : 'left')) : el[prop];
      b.setAttribute('aria-pressed', String(cur === value));
    }
  };

  P._placeFormatBar = function _placeFormatBar(ed, el, sx, sy, sw) {
    if (!ed.bar) return;
    this._refreshFormatBar(ed);
    this._refreshSuggestions(ed);
    const rw = this.root.clientWidth || 800;
    const bw = ed.bar.offsetWidth || 280;
    let x = sx + sw / 2 - bw / 2;
    x = Math.max(8, Math.min(rw - bw - 8, x));
    const z = this._view.zoom;
    const bh = ed.bar.offsetHeight || 46;
    const rh = this.root.clientHeight || 600;
    // the whole shape (not just the text box) is kept visible: the bar goes under it, or above it when there is no room below
    const top = ed.kind === 'container' ? this._view.y + el.y * z : sy;
    const bottom = ed.kind === 'container' ? this._view.y + (el.y + el.h) * z : sy + (parseFloat(ed.ta.style.height) || 24);
    let y = bottom + 14;
    if (y + bh > rh - 8) y = top - bh - 14;
    y = Math.max(8, y);
    ed.bar.style.left = `${x}px`;
    ed.bar.style.top = `${y}px`;
  };

  // Tab inserts two spaces (at the start of every selected line when several lines are selected); Shift+Tab removes them.
  P._indentEditor = function _indentEditor(outdent) {
    const ed = this._editor;
    if (!ed) return;
    const ta = ed.ta;
    const IND = '  ';
    const { selectionStart: a, selectionEnd: b, value } = ta;
    const multi = value.slice(a, b).includes('\n');
    if (!outdent && !multi) {
      ta.setRangeText(IND, a, b, 'end');
    } else {
      const start = value.lastIndexOf('\n', a - 1) + 1;
      const end = b;
      const block = value.slice(start, end).split('\n');
      const out = block.map((line) => (outdent ? line.replace(/^ {1,2}|^\t/, '') : IND + line));
      const next = out.join('\n');
      ta.setRangeText(next, start, end, 'preserve');
      const delta = next.length - (end - start);
      ta.setSelectionRange(Math.max(start, a + (outdent ? Math.min(0, out[0].length - block[0].length) : IND.length)), Math.max(start, b + delta));
    }
    this._onEditorInput();
  };

  P._onEditorInput = function _onEditorInput(silent = false) {
    const ed = this._editor;
    if (!ed) return;
    const el = findItem(this._board, ed.id);
    if (!el) return;
    let v = ed.ta.value;
    const max = ed.kind === 'text' ? LIMITS.text : ed.kind === 'frame' ? LIMITS.frameName : LIMITS.label;
    if (v.length > max) { v = v.slice(0, max); ed.ta.value = v; }
    if (ed.kind === 'frame') v = v.replace(/\n/g, ' ');
    if (ed.kind === 'text') {
      el.text = v;
      const s = textSize(v || ' ', { fontFamily: el.fontFamily, fontSize: el.fontSize });
      el.w = Math.max(s.w, el.fontSize * 0.4);
      el.h = Math.max(s.h, el.fontSize * 1.25);
    } else if (ed.kind === 'frame') el[ed.field] = v;
    else { el.text = v; if (ed.kind === 'container') fitContainer(el, v); }
    this._placeEditor();
    if (!silent) this._drawOverlay();
  };

  P._placeEditor = function _placeEditor() {
    const ed = this._editor;
    if (!ed) return;
    const el = findItem(this._board, ed.id);
    const ta = ed.ta;
    if (!el) { ta.style.display = 'none'; return; }
    const z = this._view.zoom;
    const dark = this.dark;
    const fs = (el.fontSize || (ed.kind === 'frame' ? 14 : 20));
    const family = ed.kind === 'frame' ? 2 : el.fontFamily;
    const css = (FONT_FAMILIES[family] || FONT_FAMILIES[1]).css;
    const v = ta.value || '';
    let box; let align = el.textAlign || 'left'; let rotate = 0; let rotC = null; let bg = 'transparent';
    let size;
    if (ed.kind === 'text') {
      size = textSize(v || ' ', { fontFamily: el.fontFamily, fontSize: el.fontSize });
      box = { x: el.x, y: el.y, w: Math.max(size.w + el.fontSize * 0.6, v ? 4 : 130 / z), h: Math.max(size.h, el.fontSize * 1.25) };
      rotate = el.angle || 0; rotC = { x: el.x + el.w / 2, y: el.y + el.h / 2 };
    } else if (ed.kind === 'container') {
      const ib = innerBox(el);
      size = textSize(v || ' ', { fontFamily: el.fontFamily, fontSize: el.fontSize, maxWidth: ib.w });
      const h = Math.max(size.h, el.fontSize * 1.25);
      const valign = el.verticalAlign || 'middle';
      const top = valign === 'top' ? ib.y : valign === 'bottom' ? ib.y + ib.h - h : ib.y + (ib.h - h) / 2;
      box = { x: ib.x, y: top, w: ib.w, h };
      rotate = el.angle || 0; rotC = { x: el.x + el.w / 2, y: el.y + el.h / 2 };
      align = el.textAlign || 'center';
    } else if (ed.kind === 'linear') {
      const lay = el.text ? linearLabelLayout(this._board, el) : null;
      const pts = resolvePoints(this._board, el);
      const mid = lay ? { x: lay.anchorX, y: lay.box.y + lay.box.h / 2 } : polylineMidpoint(pts);
      size = textSize(v || ' ', { fontFamily: el.fontFamily, fontSize: el.fontSize, maxWidth: 220 });
      const w = Math.max(size.w + el.fontSize, 64);
      box = { x: mid.x - w / 2, y: mid.y - size.h / 2, w, h: Math.max(size.h, el.fontSize * 1.25) };
      align = 'center';
      bg = themeColor(this._bg, dark);
    } else {
      const nb = frameNameBox(el);
      size = textSize(v || ' ', { fontFamily: 2, fontSize: 14 });
      box = { x: nb.x, y: el.y - 24, w: Math.max(size.w + 24, 120), h: 20 };
      align = 'left';
    }
    const lh = ed.kind === 'frame' ? 20 : fs * 1.25;
    const sx = this._view.x + box.x * z;
    const sy = this._view.y + box.y * z;
    const st = ta.style;
    st.display = '';
    st.left = `${sx}px`;
    st.top = `${sy}px`;
    st.width = `${box.w * z}px`;
    st.height = `${box.h * z}px`;
    st.fontFamily = css;
    st.fontSize = `${fs * z}px`;
    st.lineHeight = `${lh * z}px`;
    st.textAlign = align;
    st.color = themeColor(el.strokeColor || '#1e1e1e', dark);
    st.background = bg;
    st.opacity = String((el.opacity ?? 100) / 100);
    this._placeFormatBar(ed, el, sx, sy, box.w * z);
    if (rotate && rotC) {
      st.transformOrigin = `${(rotC.x - box.x) * z}px ${(rotC.y - box.y) * z}px`;
      st.transform = `rotate(${rotate}rad)`;
    } else { st.transform = ''; st.transformOrigin = ''; }
  };

  // Closes the editor, applying (or discarding) the typed text and recording one undo step.
  P._finishEditor = function _finishEditor({ discard = false } = {}) {
    const ed = this._editor;
    if (!ed || ed.done) return;
    ed.done = true;
    this._editor = null;
    if (ed.bar) ed.bar.remove();
    const el = findItem(this._board, ed.id);
    const value = ed.ta.value;
    ed.ta.remove();
    if (!el) { this._emit('editing', { id: null }); return; }
    let drop = false;
    if (discard) {
      if (ed.isNew) drop = true; else el[ed.field] = ed.before;
    } else if (ed.kind === 'text') {
      if (!value.trim()) drop = true;
      else {
        el.text = value;
        const s = textSize(value, { fontFamily: el.fontFamily, fontSize: el.fontSize });
        el.w = Math.max(s.w, 4);
        el.h = Math.max(s.h, 4);
      }
    } else if (ed.kind === 'frame') el.name = value.replace(/\n/g, ' ').slice(0, LIMITS.frameName);
    else el.text = value.replace(/\s+$/g, '').slice(0, LIMITS.label);
    if (drop) {
      if (ed.isNew) this._board.items = this._board.items.filter((i) => i.id !== el.id);
      else ops.deleteElements(this._board, [el.id]);
      this._setSelection(this._selIds().filter((id) => id !== el.id));
    }
    this._emit('editing', { id: null });
    this._placeConnectors();
    if (this._tool === 'text' && !this._toolLocked) this._afterCreate();
    if (ed.isNew && drop) this._renderNow(); else this._commit();
    this._renderNow();
    try { this.root.focus({ preventScroll: true }); } catch { /* ignore */ }
  };
}

// In-place text editing for the Whiteboard: free text, container labels, arrow labels and frame names (mixed into its prototype).
import { FONT_FAMILIES, LIMITS, createElement, findItem, isContainer, isLinear, newId, resolvePoints } from '../../shared/board.js';
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
    ta.wrap = kind === 'text' || kind === 'frame' ? 'off' : 'soft';
    const field = kind === 'frame' ? 'name' : 'text';
    ta.value = (el[field] || '') + initial;
    this._editor = { id, ta, kind, field, isNew, before: el[field] || '', composing: false, done: false };
    this.root.appendChild(ta);
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
      if (e.key === 'Enter' && (e.ctrlKey || e.metaKey || (kind === 'frame' && !e.shiftKey))) { e.preventDefault(); this._finishEditor(); return; }
      if (e.key === 'Tab') { e.preventDefault(); }
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
    else el.text = v;
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
      box = { x: el.x, y: el.y, w: Math.max(size.w, 4) + el.fontSize * 0.6, h: Math.max(size.h, el.fontSize * 1.25) };
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
    if (this._tool === 'text' && !this._toolLocked) this._afterCreate();
    if (ed.isNew && drop) this._renderNow(); else this._commit();
    this._renderNow();
    try { this.root.focus({ preventScroll: true }); } catch { /* ignore */ }
  };
}

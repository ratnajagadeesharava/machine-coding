// SVG whiteboard for system-design sketches: boxes, ovals, databases, arrows that stick to shapes, and free text.
import {
  DEFAULT_LABELS, DEFAULT_SIZES, arrowEnds, center, edgePoint, emptyBoard, findItem, isArrow, isShape, sanitizeBoard,
} from '/shared/board.js';

const NS = 'http://www.w3.org/2000/svg';
const HISTORY_LIMIT = 100;
const TOOL_KEYS = { v: 'select', r: 'rect', o: 'ellipse', d: 'database', a: 'arrow', t: 'text', e: 'eraser' };

function svgEl(name, attrs = {}, parent) {
  const n = document.createElementNS(NS, name);
  for (const [k, v] of Object.entries(attrs)) n.setAttribute(k, String(v));
  if (parent) parent.appendChild(n);
  return n;
}

function wrapLines(text, maxChars) {
  const out = [];
  for (const para of String(text).split('\n')) {
    let line = '';
    for (const word of para.split(/\s+/).filter(Boolean)) {
      if (!line) line = word;
      else if ((line + ' ' + word).length <= maxChars) line += ' ' + word;
      else { out.push(line); line = word; }
    }
    out.push(line);
  }
  return out.length ? out : [''];
}

function distToSegment(p, a, b) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  let t = len2 ? ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2 : 0;
  t = Math.max(0, Math.min(1, t));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

export class Whiteboard {
  constructor(container, { onChange, onSelect, onTool } = {}) {
    this.onChange = onChange;
    this.onSelect = onSelect;
    this.onTool = onTool;
    this.board = emptyBoard();
    this.tool = 'select';
    this.selection = new Set();
    this.history = [];
    this.view = { x: 0, y: 0, k: 1 };
    this.drag = null;
    this.editing = null;
    this.keyboard = true;

    this.root = document.createElement('div');
    this.root.className = 'wb tool-select';
    container.appendChild(this.root);
    this.svg = svgEl('svg', { role: 'img', 'aria-label': 'System design whiteboard' }, this.root);
    const defs = svgEl('defs', {}, this.svg);
    const marker = svgEl('marker', { id: 'hld-arrow', viewBox: '0 0 10 10', refX: 9, refY: 5, markerWidth: 9, markerHeight: 9, orient: 'auto-start-reverse', markerUnits: 'userSpaceOnUse' }, defs);
    svgEl('path', { d: 'M0,0 L10,5 L0,10 z', class: 'head' }, marker);
    this.gView = svgEl('g', {}, this.svg);
    this.gItems = svgEl('g', {}, this.gView);
    this.gOverlay = svgEl('g', {}, this.gView);
    this.ta = document.createElement('textarea');
    this.ta.hidden = true;
    this.ta.rows = 1;
    this.ta.setAttribute('aria-label', 'Edit label');
    this.root.appendChild(this.ta);

    this.svg.addEventListener('pointerdown', (e) => this.pointerDown(e));
    this.svg.addEventListener('pointermove', (e) => this.pointerMove(e));
    this.svg.addEventListener('pointerup', (e) => this.pointerUp(e));
    this.svg.addEventListener('pointercancel', (e) => this.pointerUp(e));
    this.svg.addEventListener('dblclick', (e) => this.doubleClick(e));
    this.svg.addEventListener('wheel', (e) => this.wheel(e), { passive: false });
    window.addEventListener('keydown', (e) => this.keyDown(e));
    this.render();
  }

  // ---- public API ----
  getBoard() {
    return JSON.parse(JSON.stringify(this.board));
  }

  setBoard(board) {
    this.cancelEdit();
    this.board = sanitizeBoard(board);
    this.selection.clear();
    this.history = [];
    this.render();
    this.emitSelect();
  }

  setTool(tool) {
    this.finishEdit();
    this.tool = tool;
    this.root.className = `wb tool-${tool}`;
    if (tool !== 'select') this.selection.clear();
    this.render();
    this.emitSelect();
    this.onTool?.(tool);
  }

  undo() {
    this.finishEdit();
    const snap = this.history.pop();
    if (!snap) return false;
    this.board = JSON.parse(snap);
    this.selection.clear();
    this.render();
    this.emitChange();
    this.emitSelect();
    return true;
  }

  canUndo() {
    return this.history.length > 0;
  }

  clear() {
    if (!this.board.items.length) return;
    this.pushHistory();
    this.board = emptyBoard();
    this.selection.clear();
    this.render();
    this.emitChange();
    this.emitSelect();
  }

  deleteSelected() {
    if (!this.selection.size) return false;
    this.pushHistory();
    for (const id of [...this.selection]) this.removeItem(id);
    this.selection.clear();
    this.render();
    this.emitChange();
    this.emitSelect();
    return true;
  }

  hasSelection() {
    return this.selection.size > 0;
  }

  resetView() {
    this.view = { x: 0, y: 0, k: 1 };
    this.render();
  }

  // ---- bookkeeping ----
  pushHistory() {
    this.history.push(JSON.stringify(this.board));
    if (this.history.length > HISTORY_LIMIT) this.history.shift();
  }

  emitChange() {
    this.onChange?.(this.getBoard());
  }

  emitSelect() {
    this.onSelect?.(this.selection.size);
  }

  nextId() {
    return this.board.nextId++;
  }

  addShape(type, x, y, w, h, text) {
    const item = { id: this.nextId(), type, x, y, w, h, text: text ?? DEFAULT_LABELS[type] };
    this.board.items.push(item);
    return item;
  }

  // Remove an item; arrows that were attached to a removed shape keep their drawn position but lose the binding.
  removeItem(id) {
    const item = findItem(this.board, id);
    if (!item) return;
    if (isShape(item)) {
      for (const a of this.board.items.filter(isArrow)) {
        if (a.from === id || a.to === id) {
          const e = arrowEnds(this.board, a);
          if (a.from === id) { a.from = null; a.x1 = e.x1; a.y1 = e.y1; }
          if (a.to === id) { a.to = null; a.x2 = e.x2; a.y2 = e.y2; }
        }
      }
    }
    this.board.items = this.board.items.filter((i) => i.id !== id);
  }

  // ---- geometry ----
  toWorld(e) {
    const r = this.svg.getBoundingClientRect();
    return { x: (e.clientX - r.left - this.view.x) / this.view.k, y: (e.clientY - r.top - this.view.y) / this.view.k };
  }

  inside(shape, p, tol) {
    if (shape.type === 'ellipse') {
      const c = center(shape);
      return ((p.x - c.x) / (shape.w / 2 + tol)) ** 2 + ((p.y - c.y) / (shape.h / 2 + tol)) ** 2 <= 1;
    }
    return p.x >= shape.x - tol && p.x <= shape.x + shape.w + tol && p.y >= shape.y - tol && p.y <= shape.y + shape.h + tol;
  }

  shapeAt(p, { includeText = false } = {}) {
    const tol = 4 / this.view.k;
    for (let i = this.board.items.length - 1; i >= 0; i--) {
      const it = this.board.items[i];
      if (isShape(it) && (includeText || it.type !== 'text') && this.inside(it, p, tol)) return it;
    }
    return null;
  }

  itemAt(p) {
    const s = this.shapeAt(p, { includeText: true });
    if (s) return s;
    const tol = 8 / this.view.k;
    for (let i = this.board.items.length - 1; i >= 0; i--) {
      const it = this.board.items[i];
      if (isArrow(it)) {
        const e = arrowEnds(this.board, it);
        if (distToSegment(p, { x: e.x1, y: e.y1 }, { x: e.x2, y: e.y2 }) <= tol) return it;
      }
    }
    return null;
  }

  handleAt(p) {
    if (this.selection.size !== 1) return null;
    const item = findItem(this.board, [...this.selection][0]);
    if (!item) return null;
    const r = 10 / this.view.k;
    if (isShape(item) && item.type !== 'text') {
      if (Math.hypot(p.x - (item.x + item.w), p.y - (item.y + item.h)) <= r) return { kind: 'resize', item };
    }
    if (isArrow(item)) {
      const e = arrowEnds(this.board, item);
      if (Math.hypot(p.x - e.x1, p.y - e.y1) <= r) return { kind: 'end', item, end: 1 };
      if (Math.hypot(p.x - e.x2, p.y - e.y2) <= r) return { kind: 'end', item, end: 2 };
    }
    return null;
  }

  // ---- pointer handling ----
  pointerDown(e) {
    if (e.button !== 0 && e.button !== 1) return;
    this.finishEdit();
    const p = this.toWorld(e);
    this.svg.setPointerCapture?.(e.pointerId);
    if (e.button === 1) return this.startPan(e);

    switch (this.tool) {
      case 'select': return this.downSelect(e, p);
      case 'rect':
      case 'ellipse':
      case 'database':
        this.drag = { kind: 'create', type: this.tool, start: p, cur: p };
        return this.renderOverlay();
      case 'text': {
        this.pushHistory();
        const [w, h] = DEFAULT_SIZES.text;
        const item = this.addShape('text', p.x, p.y - h / 2, w, h, '');
        this.freshId = item.id;
        this.setTool('select');
        this.selection = new Set([item.id]);
        this.render();
        this.emitChange();
        this.startEdit(item);
        return undefined;
      }
      case 'arrow': {
        const from = this.shapeAt(p);
        this.drag = { kind: 'arrow', from, start: p, cur: p };
        return this.renderOverlay();
      }
      case 'eraser':
        this.drag = { kind: 'erase', pushed: false };
        return this.eraseAt(p);
      default: return undefined;
    }
  }

  downSelect(e, p) {
    const handle = this.handleAt(p);
    if (handle) {
      this.pushHistory();
      this.drag = { ...handle, moved: true };
      return;
    }
    const item = this.itemAt(p);
    if (!item) {
      if (this.selection.size) { this.selection.clear(); this.render(); this.emitSelect(); }
      return this.startPan(e);
    }
    if (e.shiftKey) {
      if (this.selection.has(item.id)) this.selection.delete(item.id); else this.selection.add(item.id);
    } else if (!this.selection.has(item.id)) {
      this.selection = new Set([item.id]);
    }
    this.render();
    this.emitSelect();
    const movable = [...this.selection].map((id) => findItem(this.board, id)).filter(Boolean)
      .filter((i) => isShape(i) || (isArrow(i) && i.from == null && i.to == null));
    this.drag = {
      kind: 'move', start: p, pushed: false,
      origins: movable.map((i) => (isShape(i) ? { i, x: i.x, y: i.y } : { i, x1: i.x1, y1: i.y1, x2: i.x2, y2: i.y2 })),
    };
  }

  startPan(e) {
    this.drag = { kind: 'pan', sx: e.clientX, sy: e.clientY, vx: this.view.x, vy: this.view.y };
    this.root.classList.add('panning');
  }

  pointerMove(e) {
    const d = this.drag;
    if (!d) return;
    const p = this.toWorld(e);
    switch (d.kind) {
      case 'pan':
        this.view.x = d.vx + (e.clientX - d.sx);
        this.view.y = d.vy + (e.clientY - d.sy);
        return this.applyView();
      case 'move': {
        const dx = p.x - d.start.x;
        const dy = p.y - d.start.y;
        if (!d.pushed) {
          if (Math.hypot(dx, dy) * this.view.k < 3) return;
          this.pushHistory();
          d.pushed = true;
        }
        for (const o of d.origins) {
          if (isShape(o.i)) { o.i.x = o.x + dx; o.i.y = o.y + dy; }
          else { o.i.x1 = o.x1 + dx; o.i.y1 = o.y1 + dy; o.i.x2 = o.x2 + dx; o.i.y2 = o.y2 + dy; }
        }
        return this.render();
      }
      case 'resize':
        d.item.w = Math.max(48, p.x - d.item.x);
        d.item.h = Math.max(36, p.y - d.item.y);
        return this.render();
      case 'end': {
        const a = d.item;
        if (d.end === 1) { a.from = null; a.x1 = p.x; a.y1 = p.y; } else { a.to = null; a.x2 = p.x; a.y2 = p.y; }
        d.hover = this.shapeAt(p);
        this.render();
        return this.renderOverlay(d.hover);
      }
      case 'create':
      case 'arrow':
        d.cur = p;
        d.hover = d.kind === 'arrow' ? this.shapeAt(p) : null;
        return this.renderOverlay(d.hover);
      case 'erase':
        return this.eraseAt(p);
      default: return undefined;
    }
  }

  pointerUp(e) {
    const d = this.drag;
    this.drag = null;
    this.root.classList.remove('panning');
    if (!d) return;
    const p = this.toWorld(e);
    switch (d.kind) {
      case 'move':
        if (d.pushed) this.emitChange();
        break;
      case 'resize':
        this.emitChange();
        break;
      case 'end': {
        const target = this.shapeAt(p);
        const a = d.item;
        const other = d.end === 1 ? a.to : a.from;
        if (target && target.id !== other) {
          if (d.end === 1) a.from = target.id; else a.to = target.id;
        }
        this.render();
        this.emitChange();
        break;
      }
      case 'create': {
        const [dw, dh] = DEFAULT_SIZES[d.type];
        const dragged = Math.hypot(p.x - d.start.x, p.y - d.start.y) * this.view.k > 10;
        let x; let y; let w; let h;
        if (dragged) {
          x = Math.min(d.start.x, p.x); y = Math.min(d.start.y, p.y);
          w = Math.max(60, Math.abs(p.x - d.start.x)); h = Math.max(44, Math.abs(p.y - d.start.y));
        } else {
          w = dw; h = dh; x = p.x - w / 2; y = p.y - h / 2;
        }
        this.pushHistory();
        const item = this.addShape(d.type, x, y, w, h);
        this.freshId = item.id;
        this.setTool('select');
        this.selection = new Set([item.id]);
        this.render();
        this.emitChange();
        this.emitSelect();
        this.startEdit(item);
        break;
      }
      case 'arrow': {
        const to = this.shapeAt(p);
        const far = Math.hypot(p.x - d.start.x, p.y - d.start.y) * this.view.k > 14;
        if ((!far && !(d.from && to && d.from.id !== to.id)) || (d.from && to && d.from.id === to.id)) {
          this.render();
          break;
        }
        this.pushHistory();
        this.board.items.push({
          id: this.nextId(), type: 'arrow',
          from: d.from ? d.from.id : null, to: to ? to.id : null,
          x1: d.start.x, y1: d.start.y, x2: p.x, y2: p.y, text: '',
        });
        this.render();
        this.emitChange();
        break;
      }
      case 'erase':
        if (d.pushed) this.emitChange();
        this.render();
        break;
      default: break;
    }
  }

  eraseAt(p) {
    const d = this.drag;
    const item = this.itemAt(p);
    if (!item) return;
    if (d && !d.pushed) { this.pushHistory(); d.pushed = true; }
    this.removeItem(item.id);
    this.selection.delete(item.id);
    this.render();
  }

  doubleClick(e) {
    if (this.tool !== 'select') return;
    const p = this.toWorld(e);
    const item = this.itemAt(p);
    if (item) {
      this.selection = new Set([item.id]);
      this.render();
      this.emitSelect();
      this.startEdit(item);
    }
  }

  wheel(e) {
    e.preventDefault();
    if (e.ctrlKey || e.metaKey) {
      const r = this.svg.getBoundingClientRect();
      const mx = e.clientX - r.left;
      const my = e.clientY - r.top;
      const k = Math.max(0.4, Math.min(2.5, this.view.k * Math.exp(-e.deltaY * 0.01)));
      this.view.x = mx - ((mx - this.view.x) / this.view.k) * k;
      this.view.y = my - ((my - this.view.y) / this.view.k) * k;
      this.view.k = k;
    } else {
      this.view.x -= e.deltaX;
      this.view.y -= e.deltaY;
    }
    this.applyView();
  }

  keyDown(e) {
    if (!this.keyboard || this.editing != null) return;
    const t = e.target;
    if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
    if (document.querySelector('dialog[open]')) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === 'z') { e.preventDefault(); this.undo(); return; }
    if (mod || e.altKey) return;
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (this.deleteSelected()) e.preventDefault();
    } else if (e.key === 'Escape') {
      this.selection.clear();
      this.setTool('select');
    } else if (TOOL_KEYS[e.key.toLowerCase()]) {
      this.setTool(TOOL_KEYS[e.key.toLowerCase()]);
    }
  }

  // ---- text editing ----
  startEdit(item) {
    this.editing = item.id;
    const k = this.view.k;
    let box;
    if (isArrow(item)) {
      const e = arrowEnds(this.board, item);
      box = { x: (e.x1 + e.x2) / 2 - 70, y: (e.y1 + e.y2) / 2 - 16, w: 140, h: 34 };
    } else if (item.type === 'text') {
      box = { x: item.x, y: item.y, w: Math.max(item.w, 160), h: Math.max(item.h, 36) };
    } else {
      box = { x: item.x + 6, y: item.y + item.h / 2 - 20, w: Math.max(item.w - 12, 60), h: 40 };
    }
    const s = this.ta.style;
    s.left = `${box.x * k + this.view.x}px`;
    s.top = `${box.y * k + this.view.y}px`;
    s.width = `${box.w * k}px`;
    s.height = `${box.h * k}px`;
    s.fontSize = `${Math.max(12, 18 * k)}px`;
    s.textAlign = item.type === 'text' ? 'left' : 'center';
    this.ta.value = item.text || '';
    this.ta.hidden = false;
    this.ta.onkeydown = (ev) => {
      if (ev.key === 'Enter' && !ev.shiftKey) { ev.preventDefault(); this.finishEdit(); }
      else if (ev.key === 'Escape') { ev.preventDefault(); this.cancelEdit(); }
      ev.stopPropagation();
    };
    this.ta.onblur = () => this.finishEdit();
    this.ta.focus();
    this.ta.select();
    this.render();
  }

  finishEdit() {
    if (this.editing == null) return;
    const id = this.editing;
    this.editing = null;
    this.ta.hidden = true;
    this.ta.onblur = null;
    const item = findItem(this.board, id);
    if (!item) return;
    const text = this.ta.value.replace(/\s+$/g, '').slice(0, 120);
    const fresh = this.freshId === id;
    this.freshId = null;
    if (item.type === 'text' && !text.trim()) {
      if (!fresh) this.pushHistory();
      this.removeItem(id);
      this.selection.delete(id);
      if (fresh) this.history.pop();
    } else if (text !== (item.text || '')) {
      if (!fresh) this.pushHistory();
      item.text = text;
      if (item.type === 'text') this.fitText(item);
    } else if (item.type === 'text') {
      this.fitText(item);
    }
    this.render();
    this.emitChange();
    this.emitSelect();
  }

  cancelEdit() {
    if (this.editing == null) return;
    const id = this.editing;
    this.editing = null;
    this.ta.hidden = true;
    this.ta.onblur = null;
    const item = findItem(this.board, id);
    if (item && this.freshId === id && item.type === 'text') {
      this.removeItem(id);
      this.history.pop();
      this.selection.delete(id);
    }
    this.freshId = null;
    this.render();
    this.emitChange();
  }

  fitText(item) {
    const lines = String(item.text || '').split('\n');
    item.w = Math.max(60, Math.max(...lines.map((l) => l.length)) * 10 + 16);
    item.h = lines.length * 24 + 10;
  }

  // ---- rendering ----
  applyView() {
    this.gView.setAttribute('transform', `translate(${this.view.x} ${this.view.y}) scale(${this.view.k})`);
    this.root.parentElement?.style.setProperty('background-position', `${this.view.x}px ${this.view.y}px`);
    this.root.parentElement?.style.setProperty('background-size', `${22 * this.view.k}px ${22 * this.view.k}px`);
  }

  render() {
    this.applyView();
    this.gItems.replaceChildren();
    for (const item of this.board.items) {
      const g = svgEl('g', { 'data-id': item.id }, this.gItems);
      if (this.selection.has(item.id)) g.setAttribute('class', 'selected');
      if (isArrow(item)) this.drawArrow(g, item); else this.drawShape(g, item);
    }
    this.renderOverlay();
  }

  drawShape(g, s) {
    const cx = s.x + s.w / 2;
    let textY = s.y + s.h / 2;
    if (s.type === 'rect') {
      svgEl('rect', { class: 'shape', x: s.x, y: s.y, width: s.w, height: s.h, rx: 12 }, g);
    } else if (s.type === 'ellipse') {
      svgEl('ellipse', { class: 'shape', cx, cy: s.y + s.h / 2, rx: s.w / 2, ry: s.h / 2 }, g);
    } else if (s.type === 'database') {
      const ry = Math.min(14, s.h * 0.18);
      const rx = s.w / 2;
      svgEl('path', { class: 'shape', d: `M${s.x},${s.y + ry} L${s.x},${s.y + s.h - ry} A${rx},${ry} 0 0 0 ${s.x + s.w},${s.y + s.h - ry} L${s.x + s.w},${s.y + ry}` }, g);
      svgEl('ellipse', { class: 'shape', cx, cy: s.y + ry, rx, ry }, g);
      textY = s.y + s.h / 2 + ry / 2;
    } else {
      svgEl('rect', { x: s.x, y: s.y, width: s.w, height: s.h, fill: 'transparent' }, g);
    }
    if (this.editing === s.id) return;
    const text = s.text || '';
    if (!text) return;
    const lh = 22;
    if (s.type === 'text') {
      const lines = text.split('\n');
      const t = svgEl('text', { class: 'label note', x: s.x + 8, y: s.y + 4 + lh / 2 }, g);
      lines.forEach((line, i) => { const ts = svgEl('tspan', { x: s.x + 8, dy: i ? lh : 0 }, t); ts.textContent = line; });
      return;
    }
    const lines = wrapLines(text, Math.max(6, Math.floor((s.w - 16) / 9.5)));
    const t = svgEl('text', { class: 'label', x: cx, y: textY - ((lines.length - 1) * lh) / 2 }, g);
    lines.forEach((line, i) => { const ts = svgEl('tspan', { x: cx, dy: i ? lh : 0 }, t); ts.textContent = line; });
  }

  drawArrow(g, a) {
    const e = arrowEnds(this.board, a);
    const attrs = { x1: e.x1, y1: e.y1, x2: e.x2, y2: e.y2 };
    svgEl('line', { ...attrs, class: 'arrow-hit' }, g);
    svgEl('line', { ...attrs, class: 'arrow-line' }, g);
    if (a.text && this.editing !== a.id) {
      const t = svgEl('text', { class: 'arrow-label', x: (e.x1 + e.x2) / 2, y: (e.y1 + e.y2) / 2 - 6 }, g);
      t.textContent = a.text;
    }
  }

  renderOverlay(hover = null) {
    this.gOverlay.replaceChildren();
    const k = this.view.k;
    const d = this.drag;
    if (hover) {
      this.outline(hover, 'drop');
    }
    if (d && d.kind === 'create') {
      const x = Math.min(d.start.x, d.cur.x); const y = Math.min(d.start.y, d.cur.y);
      svgEl('rect', { class: 'ghost', x, y, width: Math.abs(d.cur.x - d.start.x), height: Math.abs(d.cur.y - d.start.y), rx: 12 }, this.gOverlay);
    }
    if (d && d.kind === 'arrow') {
      const from = d.from ? edgePoint(d.from, d.cur.x, d.cur.y) : d.start;
      svgEl('line', { class: 'ghost-line', x1: from.x, y1: from.y, x2: d.cur.x, y2: d.cur.y }, this.gOverlay);
    }
    if (this.tool !== 'select') return;
    for (const id of this.selection) {
      const item = findItem(this.board, id);
      if (!item) continue;
      if (isShape(item)) {
        svgEl('rect', { class: 'sel', x: item.x - 5, y: item.y - 5, width: item.w + 10, height: item.h + 10, rx: 8 }, this.gOverlay);
        if (this.selection.size === 1 && item.type !== 'text') {
          svgEl('rect', { class: 'handle resize', x: item.x + item.w - 6 / k, y: item.y + item.h - 6 / k, width: 12 / k, height: 12 / k, rx: 3 / k }, this.gOverlay);
        }
      } else if (this.selection.size === 1) {
        const e = arrowEnds(this.board, item);
        svgEl('circle', { class: 'handle end', cx: e.x1, cy: e.y1, r: 6 / k }, this.gOverlay);
        svgEl('circle', { class: 'handle end', cx: e.x2, cy: e.y2, r: 6 / k }, this.gOverlay);
      }
    }
  }

  outline(shape, cls) {
    if (shape.type === 'ellipse') {
      svgEl('ellipse', { class: cls, cx: shape.x + shape.w / 2, cy: shape.y + shape.h / 2, rx: shape.w / 2 + 4, ry: shape.h / 2 + 4 }, this.gOverlay);
    } else {
      svgEl('rect', { class: cls, x: shape.x - 4, y: shape.y - 4, width: shape.w + 8, height: shape.h + 8, rx: 14 }, this.gOverlay);
    }
  }
}

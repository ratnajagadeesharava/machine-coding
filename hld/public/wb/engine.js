// The whiteboard engine: scene, history, selection, view, rendering, actions and export. Pointer/keyboard handling lives in
// engine-input.js, text editing in engine-text.js, clipboard in engine-clipboard.js (all mixed into the class below).
import {
  DEFAULT_STYLE, GRID_SIZE, TOOLS, ZOOM, canRotate, cloneBoard, cloneElements, createElement, emptyBoard, findItem, hasPoints, isLinear,
  newId, sanitizeBoard, selectionBounds,
} from '../../shared/board.js';
import { boardToSvgString, elementToSvg, exportBounds, frameClipPath, themeColor } from './render.js';
import { transformHandles } from './geometry.js';
import * as ops from './ops.js';
import { HISTORY_BYTES, HISTORY_LIMIT, STYLE_KEYS, clamp, cloneEl, fin, svgEl } from './engine-util.js';
import { svgToPngBlob } from './engine-image.js';
import { installInput } from './engine-input.js';
import { installText } from './engine-text.js';
import { installClipboard } from './engine-clipboard.js';

let uid = 0;
const TOOL_IDS = TOOLS.map((t) => t.id);
const ACCENT = '#6965db';
const ACCENT_DARK = '#a8a5ff';

export class Whiteboard {
  constructor(container, options = {}) {
    if (!container) throw new Error('Whiteboard needs a container element');
    this.container = container;
    this.options = { keyboard: true, ...options };
    this._uid = ++uid;
    this._listeners = new Map();
    this._cleanups = [];
    this._destroyed = false;
    this._board = options.board ? sanitizeBoard(options.board) : emptyBoard();
    this._selection = new Set();
    this._tool = 'select';
    this._toolLocked = false;
    this._style = { ...DEFAULT_STYLE };
    this._textAlignSet = false;
    this._styleClip = null;
    this._view = { x: 0, y: 0, zoom: 1 };
    this._mode = {
      grid: !!options.grid, gridSize: GRID_SIZE, snap: false, zen: false, view: false,
      theme: options.theme === 'dark' ? 'dark' : 'light', background: options.background || null,
    };
    this._readOnly = !!options.readOnly;
    this._keyboard = this.options.keyboard !== false;
    this._editor = null;
    this._g = null; // current pointer gesture
    this._nodes = new Map();
    this._clipKey = '';
    this._clipIds = new Map();
    this._hist = [];
    this._histIdx = -1;
    this._histBytes = 0;
    this._pointer = null;
    this._space = false;
    this._touches = new Map();
    this._guides = [];
    this._bindHL = null;
    this._erasing = new Set();
    this._justCreated = null;
    this._renderQueued = false;
    this._size = { w: 0, h: 0 };
    this._buildDom();
    this._pushHistory(true);
    this._applyView();
    this._applyMode();
    this._renderNow();
    this._bindInput();
    this._observe();
    container.__wb = this;
  }

  // ---------- DOM ----------
  _buildDom() {
    const cs = getComputedStyle(this.container);
    if (cs.position === 'static') { this.container.style.position = 'relative'; this._restorePosition = true; }
    const root = document.createElement('div');
    root.className = 'wb-root';
    root.tabIndex = -1;
    this.root = root;
    const svg = svgEl('svg', { class: 'wb-svg', 'data-wb-canvas': '', width: '100%', height: '100%', role: 'application', 'aria-label': 'Whiteboard canvas' });
    this.svg = svg;
    const defs = svgEl('defs', {}, svg);
    this._gridId = `wb-grid-${this._uid}`;
    this._gridPattern = svgEl('pattern', { id: this._gridId, patternUnits: 'userSpaceOnUse', width: GRID_SIZE, height: GRID_SIZE }, defs);
    this._gridDot = svgEl('circle', { class: 'wb-grid-dot', r: 1, cx: 0, cy: 0 }, this._gridPattern);
    this._clipDefs = svgEl('defs', {}, svg);
    this._gridRect = svgEl('rect', { class: 'wb-grid', width: '100%', height: '100%', fill: `url(#${this._gridId})`, visibility: 'hidden' }, svg);
    this._vp = svgEl('g', { class: 'wb-viewport' }, svg);
    this._scene = svgEl('g', { class: 'wb-scene' }, this._vp);
    this._overlay = svgEl('g', { class: 'wb-overlay' }, this._vp);
    this._trail = svgEl('g', { class: 'wb-trail' }, svg);
    root.appendChild(svg);
    this.container.appendChild(root);
    void defs;
  }

  _observe() {
    this._measure();
    if (typeof ResizeObserver === 'function') {
      this._ro = new ResizeObserver(() => { this._measure(); this._placeEditor(); this._drawGrid(); });
      this._ro.observe(this.container);
    }
  }

  _measure() {
    const r = this.root.getBoundingClientRect();
    this._size = { w: r.width, h: r.height };
  }

  // ---------- events ----------
  on(type, fn) {
    if (!this._listeners.has(type)) this._listeners.set(type, new Set());
    this._listeners.get(type).add(fn);
    return () => this._listeners.get(type)?.delete(fn);
  }

  _emit(type, payload) {
    const set = this._listeners.get(type);
    if (!set) return;
    for (const fn of [...set]) {
      try { fn(payload); } catch (err) { console.error(`[whiteboard] listener for "${type}" failed`, err); }
    }
  }

  _toast(message, kind = 'info') { this._emit('toast', { message, kind }); }

  _fail(err) {
    console.error('[whiteboard]', err);
    this._toast((err && err.message) || 'Something went wrong', 'error');
  }

  // ---------- getters ----------
  get board() { return this._board; }
  get tool() { return this._tool; }
  get style() { return this._style; }
  get selection() { return this._selection; }
  get view() { return this._view; }
  get mode() { return this._mode; }
  get editingId() { return this._editor ? this._editor.id : null; }
  get toolLocked() { return this._toolLocked; }
  get readOnly() { return this._readOnly || this._mode.view; }
  get dark() { return this._mode.theme === 'dark'; }

  get _bg() { return this._mode.background || '#ffffff'; }

  // ---------- coordinate helpers ----------
  clientToWorld(cx, cy) {
    const r = this.root.getBoundingClientRect();
    const z = this._view.zoom;
    return { x: (cx - r.left - this._view.x) / z, y: (cy - r.top - this._view.y) / z };
  }

  worldToClient(x, y) {
    const r = this.root.getBoundingClientRect();
    const z = this._view.zoom;
    return { x: r.left + this._view.x + x * z, y: r.top + this._view.y + y * z };
  }

  _viewportCenter() {
    const { w, h } = this._size;
    const z = this._view.zoom;
    return { x: (w / 2 - this._view.x) / z, y: (h / 2 - this._view.y) / z };
  }

  // ---------- history ----------
  _snapshot() { return JSON.stringify({ version: this._board.version, items: this._board.items, nextId: this._board.nextId }); }

  _pushHistory(reset = false) {
    const snap = this._snapshot();
    if (reset) { this._hist = [snap]; this._histIdx = 0; this._histBytes = snap.length; return true; }
    if (snap === this._hist[this._histIdx]) return false;
    for (let i = this._hist.length - 1; i > this._histIdx; i--) this._histBytes -= this._hist.pop().length;
    this._hist.push(snap);
    this._histBytes += snap.length;
    while (this._hist.length > HISTORY_LIMIT + 1 || (this._hist.length > 2 && this._histBytes > HISTORY_BYTES)) this._histBytes -= this._hist.shift().length;
    this._histIdx = this._hist.length - 1;
    return true;
  }

  // Records the current board as one undo step (no-op when nothing changed) and notifies the host.
  _commit() {
    const changed = this._pushHistory(false);
    this._pruneSelection();
    this._renderNow();
    if (changed) { this._emit('change', { board: this._board }); this._emitHistory(); }
    return changed;
  }

  _emitHistory() { this._emit('history', { canUndo: this.canUndo(), canRedo: this.canRedo() }); }

  _restore(idx) {
    this._finishEditor();
    this._cancelGesture();
    const data = JSON.parse(this._hist[idx]);
    this._board = sanitizeBoard(data);
    this._board.nextId = Math.max(this._board.nextId, data.nextId || 1);
    this._histIdx = idx;
    this._pruneSelection();
    this._renderNow();
    this._emit('change', { board: this._board });
    this._emitHistory();
  }

  canUndo() { return this._histIdx > 0; }
  canRedo() { return this._histIdx < this._hist.length - 1; }
  undo() { if (this.canUndo() && !this._readOnly) this._restore(this._histIdx - 1); }
  redo() { if (this.canRedo() && !this._readOnly) this._restore(this._histIdx + 1); }

  // ---------- selection ----------
  _setSelection(ids, { silent = false } = {}) {
    const next = new Set(ids.filter((id) => findItem(this._board, id)));
    const same = next.size === this._selection.size && [...next].every((id) => this._selection.has(id));
    this._selection = next;
    if (!same) {
      this._justCreated = next.size === 1 && this._justCreated === [...next][0] ? this._justCreated : null;
      if (!silent) this._emit('selection', { ids: [...next] });
    }
    this._drawOverlay();
  }

  _pruneSelection() {
    if (![...this._selection].some((id) => !findItem(this._board, id))) return;
    this._setSelection([...this._selection]);
  }

  _selEls() { return [...this._selection].map((id) => findItem(this._board, id)).filter(Boolean); }
  _selIds() { return [...this._selection].filter((id) => findItem(this._board, id)); }
  _canEdit() { return !this._readOnly && !this._mode.view; }

  selectAll() {
    if (this.readOnly) return;
    this._finishEditor();
    this._setSelection(this._board.items.filter((e) => !e.locked).map((e) => e.id));
  }

  clearSelection() { this._setSelection([]); }

  // Selects ids (public, for UI modules).
  select(ids) { this._setSelection(Array.isArray(ids) ? ids : [ids]); }

  // ---------- scene ----------
  getBoard() { return cloneBoard(this._board); }
  isEmpty() { return this._board.items.length === 0; }

  setBoard(board, { resetHistory = true, fit = false } = {}) {
    this._finishEditor({ discard: true });
    this._cancelGesture();
    const next = sanitizeBoard(board);
    this._board = next;
    this._selection = new Set();
    if (resetHistory) this._pushHistory(true); else this._pushHistory(false);
    this._renderNow();
    if (fit) this.zoomToFit(); 
    this._emit('selection', { ids: [] });
    this._emit('change', { board: this._board });
    this._emitHistory();
  }

  clear() {
    if (!this._canEdit() || !this._board.items.length) return;
    this._finishEditor();
    this._board.items = [];
    this._setSelection([]);
    this._commit();
  }

  find(query) { return ops.findText(this._board, String(query ?? '')); }

  focusElement(id) {
    const el = findItem(this._board, id);
    if (!el) return;
    this._setSelection([id]);
    const b = selectionBounds(this._board, [el]);
    const { w, h } = this._size;
    const z = this._view.zoom;
    const sx = this._view.x + b.x * z; const sy = this._view.y + b.y * z;
    const fits = b.w * z <= w && b.h * z <= h;
    const visible = sx >= 0 && sy >= 0 && sx + b.w * z <= w && sy + b.h * z <= h;
    if (visible) return;
    const nz = fits ? z : clamp(Math.min((w * 0.8) / Math.max(b.w, 1), (h * 0.8) / Math.max(b.h, 1)), ZOOM.min, ZOOM.max);
    this._setView({ zoom: nz, x: w / 2 - (b.x + b.w / 2) * nz, y: h / 2 - (b.y + b.h / 2) * nz });
  }

  // Clones elements into the board with fresh ids; `at` is the centre of the pasted bounds ({x,y} or 'viewport-center').
  insertElements(elements, { at, select = true } = {}) {
    if (!this._canEdit() || !Array.isArray(elements) || !elements.length) return [];
    this._finishEditor();
    const src = elements.filter((e) => e && typeof e === 'object').map((e) => JSON.parse(JSON.stringify(e)));
    const tmpBoard = sanitizeBoard({ version: 2, items: src, nextId: 1 });
    const keep = tmpBoard.items;
    if (!keep.length) return [];
    const clones = cloneElements(this._board, keep, { source: tmpBoard });
    let dx = 0; let dy = 0;
    if (at) {
      const target = at === 'viewport-center' ? this._viewportCenter() : at;
      const b = selectionBounds({ items: clones }, clones);
      if (b && Number.isFinite(target.x) && Number.isFinite(target.y)) { dx = target.x - (b.x + b.w / 2); dy = target.y - (b.y + b.h / 2); }
    }
    for (const c of clones) this._shiftClone(c, dx, dy);
    this._board.items.push(...clones);
    const ids = clones.map((c) => c.id);
    ops.assignFrames(this._board, clones.filter((c) => c.frameId == null && c.type !== 'frame').map((c) => c.id));
    this._setSelection(select ? ids : this._selIds());
    this._commit();
    return ids;
  }

  _shiftClone(c, dx, dy) { c.x += dx; c.y += dy; }

  toTemplate() {
    const ids = ops.expandSelection(this._board, this._selIds());
    const els = ids.map((id) => findItem(this._board, id)).filter(Boolean);
    if (!els.length) return { elements: [] };
    const tmp = { version: 2, items: [], nextId: 1 };
    const clones = cloneElements(tmp, els, { source: this._board });
    const b = selectionBounds({ items: clones }, clones);
    for (const c of clones) this._shiftClone(c, -b.x, -b.y);
    return { elements: clones };
  }

  // ---------- tools ----------
  setTool(id, { lock } = {}) {
    if (!TOOL_IDS.includes(id)) return;
    if (id === 'image') { this._emit('request', { name: 'image' }); return; }
    if (this._mode.view && id !== 'hand' && id !== 'laser' && id !== 'select') return;
    this._finishEditor();
    this._cancelGesture({ keepMulti: false });
    if (lock !== undefined) this._toolLocked = !!lock;
    const prev = this._tool;
    this._tool = id;
    if (id !== 'select' && id !== 'hand' && this._selection.size) this._setSelection([]);
    this._applyCursor();
    if (prev !== id || lock !== undefined) this._emit('tool', { tool: id, locked: this._toolLocked });
    this._drawOverlay();
  }

  toggleToolLock() {
    this._toolLocked = !this._toolLocked;
    this._emit('tool', { tool: this._tool, locked: this._toolLocked });
  }

  // After a one-shot creation tool is used.
  _afterCreate() {
    if (!this._toolLocked && this._tool !== 'freedraw' && this._tool !== 'eraser') {
      this._tool = 'select';
      this._applyCursor();
      this._emit('tool', { tool: 'select', locked: this._toolLocked });
    }
  }

  // ---------- style ----------
  setStyle(patch) {
    if (!patch || typeof patch !== 'object') return;
    const clean = {};
    for (const k of STYLE_KEYS) if (Object.prototype.hasOwnProperty.call(patch, k) && patch[k] !== undefined) clean[k] = patch[k];
    if (!Object.keys(clean).length) return;
    if (clean.fontSize !== undefined && !Number.isFinite(clean.fontSize)) delete clean.fontSize;
    if (clean.opacity !== undefined && !Number.isFinite(clean.opacity)) delete clean.opacity;
    if ('textAlign' in clean) this._textAlignSet = true;
    Object.assign(this._style, clean);
    let changed = [];
    if (this._canEdit() && this._selection.size) {
      changed = ops.applyStyle(this._board, this._selIds(), clean);
      if (this._editor) this._placeEditor();
    }
    this._emit('style', { style: this._style });
    if (changed.length) this._commit(); else this._drawOverlay();
  }

  styleSummary() { return ops.styleSummary(this._board, this._selIds(), this._style, this._selection.size ? null : this._tool); }
  copyStyle() {
    const el = this._selEls()[0];
    if (!el) return null;
    this._styleClip = ops.copyStyle(el);
    this._toast('Style copied');
    return this._styleClip;
  }

  pasteStyle() {
    if (!this._canEdit() || !this._styleClip || !this._selection.size) return;
    const changed = ops.pasteStyle(this._board, this._selIds(), this._styleClip);
    if (changed.length) this._commit();
  }

  // ---------- actions ----------
  _act(fn) {
    if (!this._canEdit()) return false;
    this._finishEditor();
    const r = fn();
    this._commit();
    return r;
  }

  duplicate() {
    if (!this._selection.size) return [];
    let ids = [];
    this._act(() => {
      ids = ops.duplicate(this._board, this._selIds(), { dx: 20, dy: 20 });
      if (ids.length) {
        ops.assignFrames(this._board, ids);
        this._setSelection(ids.filter((id) => findItem(this._board, id)?.type !== 'frame' || true));
      }
    });
    return ids;
  }

  deleteSelected() {
    if (!this._selection.size) return;
    this._act(() => {
      ops.deleteElements(this._board, ops.expandSelection(this._board, this._selIds()));
      this._setSelection([]);
    });
  }

  group() { this._act(() => { const g = ops.group(this._board, this._selIds()); if (g != null) this._setSelection(this._selIds()); }); }
  ungroup() { this._act(() => ops.ungroup(this._board, this._selIds())); }

  toggleLock() {
    const els = this._selEls();
    if (!els.length || !this._canEdit()) return;
    const all = els.every((e) => e.locked);
    this._act(() => { ops.setLocked(this._board, this._selIds(), !all); });
    if (!all) this._setSelection([]);
  }

  unlockAll() { this._act(() => { ops.unlockAll(this._board); }); }
  bringToFront() { this._act(() => ops.reorder(this._board, this._selIds(), 'front')); }
  bringForward() { this._act(() => ops.reorder(this._board, this._selIds(), 'forward')); }
  sendBackward() { this._act(() => ops.reorder(this._board, this._selIds(), 'backward')); }
  sendToBack() { this._act(() => ops.reorder(this._board, this._selIds(), 'back')); }
  align(dir) { this._act(() => ops.align(this._board, this._selIds(), dir)); }
  distribute(axis) { this._act(() => ops.distribute(this._board, this._selIds(), axis)); }
  flip(axis) { this._act(() => ops.flip(this._board, this._selIds(), axis)); }

  _fontStep(delta) {
    const els = this._selEls().filter((e) => e.fontSize !== undefined);
    if (!els.length) {
      this.setStyle({ fontSize: clamp((this._style.fontSize || 20) + delta, 8, 200) });
      return;
    }
    this._act(() => { for (const e of els) ops.applyStyle(this._board, [e.id], { fontSize: clamp(e.fontSize + delta, 8, 200) }); });
    this._placeEditor();
  }

  increaseFontSize() { this._fontStep(2); }
  decreaseFontSize() { this._fontStep(-2); }

  setLink(url) {
    const els = this._selEls();
    if (!els.length) return;
    let link = null;
    if (typeof url === 'string' && url.trim()) {
      let u = url.trim();
      if (!/^[a-z][a-z0-9+.-]*:/i.test(u)) u = `https://${u}`;
      if (!/^https?:\/\//i.test(u)) { this._toast('Only http(s) links are allowed', 'error'); return; }
      link = u.slice(0, 500);
    }
    this._act(() => { for (const e of els) e.link = link; });
  }

  // Ctrl+Arrow: adds a connected copy of the selected shape and an arrow to it.
  flowchartAdd(direction) {
    const el = this._selEls()[0];
    if (!el || this._selection.size !== 1 || !this._canEdit()) return null;
    if (!['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text', 'image'].includes(el.type)) return null;
    const dir = String(direction || 'right').replace('Arrow', '').toLowerCase();
    const gap = 70;
    const [vx, vy] = { right: [1, 0], left: [-1, 0], up: [0, -1], down: [0, 1] }[dir] || [1, 0];
    let newId2 = null;
    this._act(() => {
      const id = newId(this._board);
      const copy = { ...cloneEl(el), id, seed: Math.floor(Math.random() * 2 ** 31), groupIds: [], from: null, to: null };
      if (el.type !== 'text') copy.text = '';
      copy.x = el.x + vx * (el.w + gap);
      copy.y = el.y + vy * (el.h + gap);
      this._board.items.push(copy);
      const a = createElement('arrow', {
        id: newId(this._board), strokeColor: this._style.strokeColor, strokeWidth: this._style.strokeWidth, roughness: this._style.roughness,
        arrowType: this._style.arrowType, from: el.id, to: id, endArrowhead: this._style.endArrowhead || 'arrow',
        x: el.x + el.w / 2, y: el.y + el.h / 2, points: [[0, 0], [copy.x + copy.w / 2 - (el.x + el.w / 2), copy.y + copy.h / 2 - (el.y + el.h / 2)]],
      });
      a.w = Math.abs(a.points[1][0]); a.h = Math.abs(a.points[1][1]);
      this._board.items.push(a);
      ops.assignFrames(this._board, [id]);
      this._setSelection([id]);
      newId2 = id;
    });
    return newId2;
  }

  // ---------- view ----------
  _setView(patch, { silent = false } = {}) {
    const z = clamp(fin(patch.zoom, this._view.zoom), ZOOM.min, ZOOM.max);
    const next = { x: fin(patch.x, this._view.x), y: fin(patch.y, this._view.y), zoom: z };
    if (next.x === this._view.x && next.y === this._view.y && next.zoom === this._view.zoom) return;
    this._view = next;
    this._applyView();
    if (!silent) this._emit('view', { ...next });
  }

  _applyView() {
    const { x, y, zoom } = this._view;
    this._vp.setAttribute('transform', `translate(${x} ${y}) scale(${zoom})`);
    this._drawGrid();
    this._placeEditor();
    this._drawOverlay();
  }

  setZoom(z, center) {
    const nz = clamp(fin(z, this._view.zoom), ZOOM.min, ZOOM.max);
    const c = center && Number.isFinite(center.x) && Number.isFinite(center.y) ? center : { x: this._size.w / 2, y: this._size.h / 2 };
    const k = nz / this._view.zoom;
    this._setView({ zoom: nz, x: c.x - (c.x - this._view.x) * k, y: c.y - (c.y - this._view.y) * k });
  }

  zoomIn() { this.setZoom(this._view.zoom * ZOOM.step * 1.1); }
  zoomOut() { this.setZoom(this._view.zoom / (ZOOM.step * 1.1)); }
  resetZoom() { this.setZoom(1); }
  pan(dx, dy) { this._setView({ x: this._view.x + fin(dx), y: this._view.y + fin(dy) }); }

  _fitBox(b, maxZoom, pad = 60) {
    const { w, h } = this._size;
    if (!b || !(w > 0) || !(h > 0)) return;
    const z = clamp(Math.min((w - pad * 2) / Math.max(b.w, 1), (h - pad * 2) / Math.max(b.h, 1), maxZoom), ZOOM.min, ZOOM.max);
    this._setView({ zoom: z, x: w / 2 - (b.x + b.w / 2) * z, y: h / 2 - (b.y + b.h / 2) * z });
  }

  zoomToFit() {
    const b = exportBounds(this._board);
    if (!b) { this._setView({ x: 0, y: 0, zoom: 1 }); return; }
    this._fitBox(b, 1.5);
  }

  zoomToSelection() {
    const ids = this._selIds();
    if (!ids.length) { this.zoomToFit(); return; }
    this._fitBox(exportBounds(this._board, ids), 2);
  }

  // ---------- modes ----------
  setMode(patch) {
    if (!patch || typeof patch !== 'object') return;
    const m = this._mode;
    const prev = { ...m };
    if ('grid' in patch) m.grid = !!patch.grid;
    if ('snap' in patch) m.snap = !!patch.snap;
    if ('zen' in patch) m.zen = !!patch.zen;
    if ('view' in patch) m.view = !!patch.view;
    if ('theme' in patch) m.theme = patch.theme === 'dark' ? 'dark' : 'light';
    if ('background' in patch) m.background = typeof patch.background === 'string' && patch.background ? patch.background : null;
    if ('gridSize' in patch && Number.isFinite(patch.gridSize) && patch.gridSize >= 2) m.gridSize = patch.gridSize;
    if (JSON.stringify(prev) === JSON.stringify(m)) return;
    if (m.view && !prev.view) {
      this._finishEditor();
      this._cancelGesture();
      this._setSelection([]);
      if (this._tool !== 'laser') { this._tool = 'hand'; this._emit('tool', { tool: 'hand', locked: this._toolLocked }); }
    } else if (!m.view && prev.view && this._tool === 'hand') {
      this._tool = 'select';
      this._emit('tool', { tool: 'select', locked: this._toolLocked });
    }
    this._applyMode();
    this._renderNow();
    this._emit('mode', { mode: { ...m } });
  }

  toggleGrid() { this.setMode({ grid: !this._mode.grid }); }
  toggleSnap() { this.setMode({ snap: !this._mode.snap }); }
  toggleZen() { this.setMode({ zen: !this._mode.zen }); }
  toggleViewMode() { this.setMode({ view: !this._mode.view }); }
  toggleTheme() { this.setMode({ theme: this._mode.theme === 'dark' ? 'light' : 'dark' }); }
  setKeyboard(on) { this._keyboard = !!on; }
  setReadOnly(on) { this._readOnly = !!on; this._applyMode(); }

  _applyMode() {
    const m = this._mode;
    const dark = m.theme === 'dark';
    this.root.classList.toggle('wb-dark', dark);
    this.root.classList.toggle('wb-zen', m.zen);
    this.root.classList.toggle('wb-viewmode', m.view || this._readOnly);
    this.root.style.background = themeColor(m.background || '#ffffff', dark);
    this.root.style.setProperty('--wb-bg', themeColor(m.background || '#ffffff', dark));
    this.root.style.setProperty('--wb-accent', dark ? ACCENT_DARK : ACCENT);
    this._applyCursor();
    this._drawGrid();
  }

  _drawGrid() {
    const m = this._mode;
    const z = this._view.zoom;
    let g = m.gridSize * z;
    while (g < 8) g *= 5;
    if (!m.grid) { this._gridRect.setAttribute('visibility', 'hidden'); return; }
    this._gridRect.setAttribute('visibility', 'visible');
    const ox = this._view.x % g;
    const oy = this._view.y % g;
    this._gridPattern.setAttribute('width', g);
    this._gridPattern.setAttribute('height', g);
    this._gridPattern.setAttribute('x', ox - g / 2);
    this._gridPattern.setAttribute('y', oy - g / 2);
    this._gridDot.setAttribute('cx', g / 2);
    this._gridDot.setAttribute('cy', g / 2);
    this._gridDot.setAttribute('r', Math.max(0.8, Math.min(1.6, z * 1.2)));
    this._gridDot.setAttribute('fill', this.dark ? 'rgba(255,255,255,0.22)' : 'rgba(0,0,0,0.2)');
  }

  _applyCursor() {
    const t = this._g && this._g.type === 'pan' ? 'grabbing' : null;
    this.root.dataset.tool = this._mode.view && this._tool !== 'laser' ? 'hand' : this._tool;
    this.svg.style.cursor = t || (this._space ? 'grab' : '');
  }

  // ---------- rendering ----------
  _scheduleRender() {
    if (this._renderQueued || this._destroyed) return;
    this._renderQueued = true;
    const run = () => { if (this._renderQueued) { this._renderQueued = false; this._renderNow(); } };
    if (typeof requestAnimationFrame === 'function') { this._raf = requestAnimationFrame(run); } else setTimeout(run, 16);
  }

  _keyOf(el, byId, dark, editing) {
    let k;
    if (el.type === 'image') k = `i|${el.x}|${el.y}|${el.w}|${el.h}|${el.angle}|${el.opacity}|${el.link}|${el.dataUrl.length}|${el.dataUrl.slice(-48)}`;
    else k = JSON.stringify(el);
    if (el.type === 'arrow') {
      for (const id of [el.from, el.to]) {
        const s = id != null ? byId.get(id) : null;
        k += s ? `|${s.type},${s.x},${s.y},${s.w},${s.h},${s.angle}` : '|-';
      }
    }
    return `${k}|${dark ? 'd' : 'l'}|${editing ? 'e' : ''}`;
  }

  _renderNow() {
    if (this._destroyed) return;
    this._renderQueued = false;
    const board = this._board;
    const dark = this.dark;
    const items = board.items;
    const byId = new Map();
    for (const e of items) byId.set(e.id, e);
    // frame clip paths
    const frames = items.filter((e) => e.type === 'frame');
    const clipKey = frames.map((f) => `${f.id}:${f.x},${f.y},${f.w},${f.h}`).join('|');
    if (clipKey !== this._clipKey) {
      this._clipKey = clipKey;
      this._clipDefs.innerHTML = frames.map((f) => frameClipPath(board, f)).join('');
      this._clipIds = new Map(frames.map((f) => [f.id, `wb-fclip-${f.id}`]));
    }
    const editingId = this._editor ? this._editor.id : null;
    const bg = this._bg;
    const seen = new Set();
    const scene = this._scene;
    for (let i = 0; i < items.length; i++) {
      const el = items[i];
      seen.add(el.id);
      let ent = this._nodes.get(el.id);
      if (!ent) { ent = { slot: svgEl('g', { class: 'wb-slot' }), key: '', clip: '' }; this._nodes.set(el.id, ent); }
      const editing = editingId === el.id;
      const key = `${this._keyOf(el, byId, dark, editing)}|${bg}`;
      if (ent.key !== key) {
        ent.slot.innerHTML = elementToSvg(board, el, { dark, skipLabel: editing, background: bg });
        ent.key = key;
        ent.slot.style.visibility = editing && el.type === 'text' ? 'hidden' : '';
      }
      const clip = el.frameId != null && this._clipIds.has(el.frameId) ? this._clipIds.get(el.frameId) : '';
      if (clip !== ent.clip) {
        if (clip) ent.slot.setAttribute('clip-path', `url(#${clip})`); else ent.slot.removeAttribute('clip-path');
        ent.clip = clip;
      }
      const er = this._erasing.has(el.id);
      if (er !== ent.er) { ent.slot.classList.toggle('wb-erasing', er); ent.er = er; }
      if (scene.children[i] !== ent.slot) scene.insertBefore(ent.slot, scene.children[i] || null);
    }
    if (this._nodes.size > items.length) {
      for (const [id, ent] of this._nodes) if (!seen.has(id)) { ent.slot.remove(); this._nodes.delete(id); }
    }
    this._drawOverlay();
  }

  // ---------- overlay ----------
  // Box + handles for the current selection: { kind:'box', box, angle, handles } | { kind:'linear', el } | null
  _selectionInfo() {
    if (this._mode.view || this._readOnly || this._editor) return null;
    const els = this._selEls();
    if (!els.length) return null;
    const z = this._view.zoom;
    const pad = 6 / z;
    if (els.length === 1) {
      const el = els[0];
      if (isLinear(el)) return { kind: 'linear', el };
      if (!hasPoints(el)) {
        const box = { x: el.x - pad, y: el.y - pad, w: el.w + pad * 2, h: el.h + pad * 2 };
        const handles = transformHandles(box, el.angle || 0, z, { rotation: canRotate(el) && !el.locked });
        return { kind: 'box', box, angle: el.angle || 0, handles, single: el };
      }
    }
    const b = selectionBounds(this._board, els);
    const box = { x: b.x - pad, y: b.y - pad, w: b.w + pad * 2, h: b.h + pad * 2 };
    return { kind: 'box', box, angle: 0, handles: transformHandles(box, 0, z, { rotation: false }), els };
  }

  _drawOverlay() {
    if (this._destroyed) return;
    const o = this._overlay;
    o.replaceChildren();
    const z = this._view.zoom;
    const lw = 1 / z;
    const col = this.dark ? ACCENT_DARK : ACCENT;
    const info = this._selectionInfo();
    const g = this._g;
    if (info && !(g && (g.type === 'shape' || g.type === 'free' || g.type === 'erase'))) {
      if (info.kind === 'linear') {
        const el = info.el;
        const pts = this._absPoints(el);
        const hs = 5 / z;
        if (!(g && g.type === 'linear')) {
          for (let i = 0; i < pts.length; i++) {
            svgEl('circle', { cx: pts[i].x, cy: pts[i].y, r: hs, fill: this.dark ? '#1e1e2e' : '#fff', stroke: col, 'stroke-width': lw * 1.5, class: 'wb-handle wb-point' }, o);
          }
          if (el.arrowType !== 'elbow') {
            for (let i = 0; i < pts.length - 1; i++) {
              const mx = (pts[i].x + pts[i + 1].x) / 2; const my = (pts[i].y + pts[i + 1].y) / 2;
              if (Math.hypot(pts[i].x - pts[i + 1].x, pts[i].y - pts[i + 1].y) * z < 28) continue;
              svgEl('circle', { cx: mx, cy: my, r: hs * 0.6, fill: col, 'fill-opacity': 0.35, stroke: col, 'stroke-width': lw, class: 'wb-handle wb-mid' }, o);
            }
          }
        }
      } else {
        const { box, angle, handles } = info;
        const c = { x: box.x + box.w / 2, y: box.y + box.h / 2 };
        if (info.els && info.els.length > 1) {
          for (const el of info.els) {
            const b = selectionBounds(this._board, [el]);
            svgEl('rect', { x: b.x, y: b.y, width: b.w, height: b.h, fill: 'none', stroke: col, 'stroke-width': lw, 'stroke-opacity': 0.55, 'stroke-dasharray': `${4 / z} ${3 / z}`,
              transform: el.angle ? `rotate(${(el.angle * 180) / Math.PI} ${el.x + el.w / 2} ${el.y + el.h / 2})` : null }, o);
          }
        }
        const r = svgEl('rect', { x: box.x, y: box.y, width: box.w, height: box.h, fill: 'none', stroke: col, 'stroke-width': lw * 1.2, class: 'wb-sel-box' }, o);
        if (angle) r.setAttribute('transform', `rotate(${(angle * 180) / Math.PI} ${c.x} ${c.y})`);
        const sz = 8 / z;
        for (const [name, p] of Object.entries(handles)) {
          if (name === 'rotate') {
            svgEl('circle', { cx: p.x, cy: p.y, r: sz * 0.6, fill: this.dark ? '#1e1e2e' : '#fff', stroke: col, 'stroke-width': lw * 1.5, class: 'wb-handle wb-rotate', 'data-handle': name }, o);
            continue;
          }
          const h = svgEl('rect', { x: p.x - sz / 2, y: p.y - sz / 2, width: sz, height: sz, rx: 2 / z, fill: this.dark ? '#1e1e2e' : '#fff', stroke: col, 'stroke-width': lw * 1.5, class: 'wb-handle', 'data-handle': name }, o);
          if (angle) h.setAttribute('transform', `rotate(${(angle * 180) / Math.PI} ${p.x} ${p.y})`);
        }
      }
    }
    // binding highlight
    if (this._bindHL) {
      const s = this._bindHL;
      const pad = 5 / z;
      const r = svgEl('rect', { x: s.x - pad, y: s.y - pad, width: s.w + pad * 2, height: s.h + pad * 2, rx: 6 / z, fill: col, 'fill-opacity': 0.08, stroke: col, 'stroke-width': lw * 2, class: 'wb-bind' }, o);
      if (s.angle) r.setAttribute('transform', `rotate(${(s.angle * 180) / Math.PI} ${s.x + s.w / 2} ${s.y + s.h / 2})`);
    }
    // marquee
    if (g && g.type === 'marquee' && g.moved) {
      const b = g.box;
      svgEl('rect', { x: b.x, y: b.y, width: b.w, height: b.h, fill: col, 'fill-opacity': 0.08, stroke: col, 'stroke-width': lw, class: 'wb-marquee' }, o);
    }
    // snap guides
    for (const gd of this._guides) {
      const a = gd.axis === 'x' ? { x1: gd.pos, x2: gd.pos, y1: gd.from, y2: gd.to } : { x1: gd.from, x2: gd.to, y1: gd.pos, y2: gd.pos };
      svgEl('line', { ...a, stroke: '#ff5d8f', 'stroke-width': lw * 1.2, 'stroke-dasharray': `${5 / z} ${3 / z}`, class: 'wb-guide' }, o);
    }
  }

  // Absolute points of a linear element (bound ends resolved).
  _absPoints(el) {
    return _resolve(this._board, el);
  }

  // ---------- export ----------
  _exportIds(selectionOnly) {
    if (!selectionOnly) return undefined;
    const ids = ops.expandSelection(this._board, this._selIds());
    return ids.length ? ids : undefined;
  }

  toSVG({ selectionOnly = false, background = true, dark = this.dark, padding = 10, scale = 1 } = {}) {
    return boardToSvgString(this._board, { ids: this._exportIds(selectionOnly), background: background ? this._bg : null, dark, padding, scale });
  }

  async toPNG({ selectionOnly = false, background = true, dark = this.dark, scale = 2, padding = 10 } = {}) {
    const ids = this._exportIds(selectionOnly);
    if (!exportBounds(this._board, ids)) throw new Error('Nothing to export');
    const svg = boardToSvgString(this._board, { ids, background: background ? this._bg : null, dark, padding, scale: 1 });
    return svgToPngBlob(svg, scale);
  }

  async copyAsPng(opts = {}) {
    try {
      const sel = this._selection.size > 0;
      const blob = await this.toPNG({ selectionOnly: sel, ...opts });
      if (typeof ClipboardItem === 'undefined' || !navigator.clipboard?.write) throw new Error('Clipboard is not available');
      await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
      this._toast('Copied to clipboard as PNG');
      return blob;
    } catch (err) {
      this._toast(`Could not copy as PNG: ${err.message || err}`, 'error');
      return null;
    }
  }

  async copyAsSvg(opts = {}) {
    try {
      const svg = this.toSVG({ selectionOnly: this._selection.size > 0, ...opts });
      if (!navigator.clipboard?.writeText) throw new Error('Clipboard is not available');
      await navigator.clipboard.writeText(svg);
      this._toast('Copied to clipboard as SVG');
      return svg;
    } catch (err) {
      this._toast(`Could not copy as SVG: ${err.message || err}`, 'error');
      return null;
    }
  }

  // ---------- teardown ----------
  destroy() {
    if (this._destroyed) return;
    this._finishEditor({ discard: true });
    this._destroyed = true;
    this._cancelGesture();
    for (const off of this._cleanups) { try { off(); } catch { /* ignore */ } }
    this._cleanups = [];
    if (this._ro) this._ro.disconnect();
    if (this._raf && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(this._raf);
    if (this._trailRaf && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(this._trailRaf);
    this.root.remove();
    if (this.container.__wb === this) delete this.container.__wb;
    if (this._restorePosition) this.container.style.position = '';
    this._listeners.clear();
    this._nodes.clear();
  }
}

import { resolvePoints } from '../../shared/board.js';
function _resolve(board, el) {
  if (el.type === 'arrow') return resolvePoints(board, el);
  return (el.points || []).map(([px, py]) => ({ x: el.x + px, y: el.y + py }));
}

installInput(Whiteboard.prototype);
installText(Whiteboard.prototype);
installClipboard(Whiteboard.prototype);

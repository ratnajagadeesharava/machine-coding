// Pointer, wheel, touch and keyboard handling for the Whiteboard (mixed into its prototype by engine.js).
import {
  BINDABLE_TYPES, MIN_SIZE, TOOLS, ZOOM, createElement, findItem, isContainer, isLinear, newId, normalizePoints,
  resolvePoints, selectionBounds,
} from '../../shared/board.js';
import {
  elementsInBox, handleAt, hitTestTop, transformHandles, nearestBindTarget, resizeSelection, rotateAngle, simplifyStroke, snapMove, snapToGrid,
} from './geometry.js';
import * as ops from './ops.js';
import {
  clamp, cloneEl, creationProps, dragBox, fin, normAngle, setAbsPoints, snapAngle45, snapRotation, svgEl,
} from './engine-util.js';

const DRAG_PX = 3;
const CURSORS = { nw: 'nwse-resize', se: 'nwse-resize', ne: 'nesw-resize', sw: 'nesw-resize', n: 'ns-resize', s: 'ns-resize', e: 'ew-resize', w: 'ew-resize', rotate: 'grab' };
const IGNORE_KEY_TARGETS = 'input,textarea,select,[contenteditable=""],[contenteditable="true"],dialog,[role="dialog"],[role="menu"],[role="listbox"]';

export function installInput(P) {
  // ---------- wiring ----------
  P._bindInput = function _bindInput() {
    const on = (target, type, fn, opts) => {
      const h = (e) => { try { fn.call(this, e); } catch (err) { this._fail(err); } };
      target.addEventListener(type, h, opts);
      this._cleanups.push(() => target.removeEventListener(type, h, opts));
    };
    const svg = this.svg;
    on(svg, 'pointerdown', this._onDown);
    for (const t of ['mousedown', 'mouseup', 'auxclick']) on(svg, t, (e) => { if (e.button === 1) e.preventDefault(); }); // no X11 middle-click paste
    on(svg, 'pointermove', this._onMove);
    on(svg, 'pointerup', this._onUp);
    on(svg, 'pointercancel', this._onUp);
    on(svg, 'pointerleave', () => { this._pointerIn = false; });
    on(svg, 'pointerenter', () => { this._pointerIn = true; });
    on(svg, 'dblclick', this._onDblClick);
    on(svg, 'contextmenu', this._onContext);
    on(svg, 'wheel', this._onWheel, { passive: false });
    on(window, 'keydown', this._onKeyDown);
    on(window, 'keyup', this._onKeyUp);
    on(window, 'blur', () => { this._space = false; this._clipPriority = false; this._applyCursor(); });
    on(document, 'paste', this._onPaste);
    on(this.container, 'dragover', this._onDragOver);
    on(this.container, 'drop', this._onDrop);
  };

  P._rel = function _rel(e) {
    const r = this.root.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  P._pt = function _pt(e) {
    const rel = this._rel(e);
    const z = this._view.zoom;
    return { x: (rel.x - this._view.x) / z, y: (rel.y - this._view.y) / z };
  };

  // ---------- snapping helpers ----------
  P._gridOn = function _gridOn() { return this._mode.grid || this._mode.snap; };
  P._gs = function _gs(v) { return this._gridOn() ? snapToGrid(v, this._mode.gridSize) : v; };
  P._sp = function _sp(p) { return { x: this._gs(p.x), y: this._gs(p.y) }; };

  P._bindTarget = function _bindTarget(pt, excludeId, e) {
    if (e && (e.ctrlKey || e.metaKey)) return null;
    const t = nearestBindTarget(this._board, pt, 10 / this._view.zoom, { excludeId });
    return t && !t.locked && BINDABLE_TYPES.includes(t.type) ? t : null;
  };

  P._setHL = function _setHL(t) {
    this._bindHL = t ? { x: t.x, y: t.y, w: t.w, h: t.h, angle: t.angle || 0 } : null;
  };

  // ---------- gestures ----------
  P._cancelGesture = function _cancelGesture() {
    const g = this._g;
    if (!g) { this._clearTransient(); return; }
    this._g = null;
    const b = this._board;
    switch (g.type) {
      case 'move': case 'resize': case 'rotate': case 'point':
        if (g.base) b.items = g.base.map(cloneEl);
        break;
      case 'shape': case 'free':
        if (g.el) b.items = b.items.filter((e) => e.id !== g.el.id);
        break;
      case 'linear':
        if (g.el) b.items = b.items.filter((e) => e.id !== g.el.id);
        break;
      case 'multi': this._g = g; this._finishMulti(); break;
      default: break;
    }
    this._clearTransient();
    this._renderNow();
  };

  P._clearTransient = function _clearTransient() {
    this._guides = [];
    this._bindHL = null;
    if (this._erasing.size) this._erasing = new Set();
    this._applyCursor();
    this._drawOverlay();
  };

  P._endGesture = function _endGesture() {
    this._g = null;
    this._clearTransient();
  };

  // ---------- pointer down ----------
  P._onDown = function _onDown(e) {
    if (this._destroyed) return;
    const active = document.activeElement;
    if (active && active !== document.body && active !== this.root && /^(INPUT|TEXTAREA|SELECT)$/.test(active.tagName) && active !== (this._editor && this._editor.ta)) active.blur();
    if (this._editor) this._finishEditor();
    try { this.svg.setPointerCapture(e.pointerId); } catch { /* synthetic events */ }
    if (e.pointerType === 'touch') {
      this._touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
      if (this._touches.size === 2) { this._startPinch(); e.preventDefault(); return; }
    }
    const rel = this._rel(e);
    this._pointer = { ...rel, ...this._pt(e) };
    // multi-click polyline in progress: this click adds a point
    if (this._g && this._g.type === 'multi') { if (e.button === 0) this._multiClick(e); e.preventDefault(); return; }
    if (this._g) return;
    if (e.button === 1) this._midAt = Date.now();
    const panning = e.button === 1 || (e.button === 0 && (this._space || this._tool === 'hand'));
    if (panning) { this._startPan(e); e.preventDefault(); return; }
    if (e.button !== 0) return;
    e.preventDefault();
    const pt = this._pt(e);
    if (this._tool === 'laser') { this._g = { type: 'laser' }; this._laserPush(e); return; }
    if (!this._canEdit()) { this._startPan(e); return; }
    this._justCreated = null;
    switch (this._tool) {
      case 'select': this._downSelect(e, pt); break;
      case 'rect': case 'diamond': case 'ellipse': case 'database': case 'sticky': case 'frame':
        this._g = { type: 'shape', tool: this._tool, start: this._sp(pt), startClient: rel, el: null };
        break;
      case 'arrow': case 'line': this._downLinear(e, pt); break;
      case 'freedraw': this._downFree(e, pt); break;
      case 'text': this._g = { type: 'textclick', start: pt, startClient: rel, hit: hitTestTop(this._board, pt, 6 / this._view.zoom, { skipLocked: true }) }; break;
      case 'eraser': this._g = { type: 'erase', last: pt, ids: new Set() }; this._eraseAt(pt, pt); break;
      case 'bucket': case 'eyedropper': this._g = { type: 'pick', start: pt, alt: e.altKey || e.shiftKey }; break;
      default: break;
    }
  };

  P._startPan = function _startPan(e) {
    this._g = { type: 'pan', sx: e.clientX, sy: e.clientY, vx: this._view.x, vy: this._view.y };
    this._applyCursor();
  };

  P._startPinch = function _startPinch() {
    if (this._g) { const keep = this._g.type === 'laser'; if (!keep) this._cancelGesture(); }
    const [a, b] = [...this._touches.values()];
    this._g = { type: 'pinch', d0: Math.hypot(a.x - b.x, a.y - b.y) || 1, c0: { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 }, view0: { ...this._view } };
  };

  // ---------- select tool ----------
  P._downSelect = function _downSelect(e, pt) {
    const z = this._view.zoom;
    const tol = 6 / z;
    const rel = this._rel(e);
    const info = this._selectionInfo();
    if (info && info.kind === 'linear') {
      const el = info.el;
      const pts = resolvePoints(this._board, el);
      const hr = 9 / z;
      for (let i = 0; i < pts.length; i++) {
        if (Math.hypot(pts[i].x - pt.x, pt.y - pts[i].y) <= hr) { this._startPoint(el, i, pt); return; }
      }
      if (el.arrowType !== 'elbow') {
        for (let i = 0; i < pts.length - 1; i++) {
          const mx = (pts[i].x + pts[i + 1].x) / 2; const my = (pts[i].y + pts[i + 1].y) / 2;
          if (Math.hypot(pts[i].x - pts[i + 1].x, pts[i].y - pts[i + 1].y) * z < 28) continue;
          if (Math.hypot(mx - pt.x, my - pt.y) <= 7 / z) {
            const abs = pts.map((p) => ({ x: p.x, y: p.y }));
            abs.splice(i + 1, 0, { x: mx, y: my });
            this._startPoint(el, i + 1, pt, abs);
            return;
          }
        }
      }
    } else if (info && info.kind === 'box') {
      const h = handleAt(info.handles, pt, z);
      if (h) { this._startTransform(h, info, pt, e); return; }
    }
    const hit = hitTestTop(this._board, pt, tol, { skipLocked: true });
    const deep = e.ctrlKey || e.metaKey;
    if (hit) {
      const ids = this._groupIds(hit, deep);
      const already = ids.every((id) => this._selection.has(id));
      let toggledOff = false;
      if (e.shiftKey) {
        if (already) { this._setSelection([...this._selection].filter((id) => !ids.includes(id))); toggledOff = true; } else this._setSelection([...this._selection, ...ids]);
      } else if (!already) this._setSelection(ids);
      this._g = { type: 'press', start: pt, startClient: rel, hit, ids, wasSelected: already, toggledOff, shift: e.shiftKey, deep };
      return;
    }
    this._g = { type: 'marquee', start: pt, startClient: rel, box: { x: pt.x, y: pt.y, w: 0, h: 0 }, moved: false, additive: e.shiftKey, base: new Set(this._selection) };
  };

  P._groupIds = function _groupIds(el, deep) {
    if (!deep && el.groupIds && el.groupIds.length) {
      const gid = el.groupIds[el.groupIds.length - 1];
      return this._board.items.filter((i) => i.groupIds.includes(gid) && !i.locked).map((i) => i.id);
    }
    return [el.id];
  };

  P._startTransform = function _startTransform(handle, info, pt, e) {
    const base = this._board.items.map(cloneEl);
    if (handle === 'rotate') {
      const el = info.single;
      const c = { x: el.x + el.w / 2, y: el.y + el.h / 2 };
      this._g = { type: 'rotate', base, el: el.id, c, a0: rotateAngle(c, pt), angle0: el.angle || 0, moved: false };
    } else {
      const els = this._selEls();
      let real;
      if (info.single) real = { x: info.single.x, y: info.single.y, w: info.single.w, h: info.single.h };
      else real = selectionBounds(this._board, els);
      const ph = info.handles[handle];
      const th = transformHandles(real, info.angle || 0, this._view.zoom, { rotation: false })[handle];
      this._g = { type: 'resize', base, handle, ids: this._selIds(), moved: false, startClient: this._rel(e), off: ph && th ? { x: th.x - ph.x, y: th.y - ph.y } : { x: 0, y: 0 } };
    }
  };

  P._startPoint = function _startPoint(el, idx, pt, abs) {
    const base = this._board.items.map(cloneEl);
    const g = { type: 'point', base, id: el.id, idx, moved: false, inserted: !!abs, abs: abs || resolvePoints(this._board, el).map((p) => ({ x: p.x, y: p.y })), startClient: null };
    g.n = g.abs.length;
    this._g = g;
  };

  // ---------- pointer move ----------
  P._onMove = function _onMove(e) {
    if (this._destroyed) return;
    const rel = this._rel(e);
    if (e.pointerType === 'touch' && this._touches.has(e.pointerId)) {
      this._touches.set(e.pointerId, { x: e.clientX, y: e.clientY });
      const g0 = this._g;
      if (g0 && g0.type === 'pinch' && this._touches.size >= 2) { this._pinchMove(); return; }
    }
    const pt = this._pt(e);
    this._pointer = { ...rel, x: pt.x, y: pt.y };
    const g = this._g;
    if (!g) { this._hover(e, pt); return; }
    const far = (s) => Math.hypot(rel.x - s.x, rel.y - s.y) > DRAG_PX;
    switch (g.type) {
      case 'pan': this._setView({ x: g.vx + (e.clientX - g.sx), y: g.vy + (e.clientY - g.sy) }); break;
      case 'laser': this._laserPush(e); break;
      case 'press':
        if (far(g.startClient)) this._beginMove(g, e);
        if (this._g && this._g.type === 'move') this._doMove(pt, e);
        break;
      case 'move': this._doMove(pt, e); break;
      case 'resize': if (g.moved || far(g.startClient)) { g.moved = true; this._doResize(pt, e); } break;
      case 'rotate': g.moved = true; this._doRotate(pt, e); break;
      case 'point': this._doPoint(pt, e, rel); break;
      case 'marquee': this._doMarquee(pt, e, rel); break;
      case 'shape': this._doShape(pt, e, rel); break;
      case 'linear': this._doLinear(pt, e, rel); break;
      case 'multi': this._doMulti(pt, e); break;
      case 'free': this._doFree(pt, e); break;
      case 'erase': this._eraseAt(g.last, pt); g.last = pt; break;
      default: break;
    }
    this._scheduleRender();
    this._drawOverlay();
  };

  P._hover = function _hover(e, pt) {
    if (this._tool !== 'select' || e.pointerType === 'touch' || this._space) return;
    let cur = '';
    if (this._canEdit()) {
      const info = this._selectionInfo();
      const z = this._view.zoom;
      if (info && info.kind === 'box') {
        const h = handleAt(info.handles, pt, z);
        if (h) cur = CURSORS[h];
      } else if (info && info.kind === 'linear') {
        const pts = resolvePoints(this._board, info.el);
        if (pts.some((p) => Math.hypot(p.x - pt.x, p.y - pt.y) <= 9 / z)) cur = 'pointer';
      }
      if (!cur && hitTestTop(this._board, pt, 6 / z, { skipLocked: true })) cur = 'move';
    }
    if (this.svg.style.cursor !== cur) this.svg.style.cursor = cur;
  };

  P._pinchMove = function _pinchMove() {
    const g = this._g;
    const [a, b] = [...this._touches.values()];
    const d = Math.hypot(a.x - b.x, a.y - b.y) || 1;
    const c = { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
    const r = this.root.getBoundingClientRect();
    const z = clamp(g.view0.zoom * (d / g.d0), ZOOM.min, ZOOM.max);
    const wx = (g.c0.x - r.left - g.view0.x) / g.view0.zoom;
    const wy = (g.c0.y - r.top - g.view0.y) / g.view0.zoom;
    this._setView({ zoom: z, x: c.x - r.left - wx * z, y: c.y - r.top - wy * z });
  };

  // ---------- moving ----------
  P._beginMove = function _beginMove(g, e) {
    if (!g.wasSelected && !this._selection.has(g.hit.id)) this._setSelection(g.ids);
    if (g.toggledOff) { this._g = null; return; }
    const b = this._board;
    if (e.altKey) {
      const dup = ops.duplicate(b, this._selIds(), { dx: 0, dy: 0 });
      if (dup.length) this._setSelection(dup);
    }
    const ids = this._selIds();
    const moving = ops.expandSelection(b, ids);
    this._g = {
      type: 'move', start: g.start, base: b.items.map(cloneEl), ids, moving,
      bounds: selectionBounds(b, moving.map((id) => findItem(b, id)).filter(Boolean)),
    };
    this._applyCursor();
  };

  P._doMove = function _doMove(pt, e) {
    const g = this._g;
    const b = this._board;
    let dx = pt.x - g.start.x;
    let dy = pt.y - g.start.y;
    if (e.shiftKey) { if (Math.abs(dx) > Math.abs(dy)) dy = 0; else dx = 0; }
    b.items = g.base.map(cloneEl);
    let gx = false; let gy = false;
    this._guides = [];
    if (this._mode.snap && g.moving.length) {
      const r = snapMove(b, g.moving, dx, dy, { threshold: 6 / this._view.zoom });
      gx = r.guides.some((q) => q.axis === 'x');
      gy = r.guides.some((q) => q.axis === 'y');
      dx = r.dx; dy = r.dy;
      this._guides = r.guides;
    }
    if (this._gridOn() && g.bounds) {
      if (!gx) dx = snapToGrid(g.bounds.x + dx, this._mode.gridSize) - g.bounds.x;
      if (!gy) dy = snapToGrid(g.bounds.y + dy, this._mode.gridSize) - g.bounds.y;
    }
    ops.moveElements(b, g.ids, dx, dy);
    g.dx = dx; g.dy = dy;
  };

  P._endMove = function _endMove() {
    const g = this._g;
    const b = this._board;
    const moved = g.moving.filter((id) => findItem(b, id)?.type !== 'frame');
    ops.assignFrames(b, moved);
    this._endGesture();
    this._commit();
  };

  // ---------- resize / rotate ----------
  P._doResize = function _doResize(pt, e) {
    const g = this._g;
    const b = this._board;
    b.items = g.base.map(cloneEl);
    const p = this._sp({ x: pt.x + g.off.x, y: pt.y + g.off.y });
    const patches = resizeSelection(b, g.ids, g.handle, p, { keepAspect: e.shiftKey, fromCenter: e.altKey });
    for (const [id, patch] of patches) {
      const el = findItem(b, id);
      if (!el) continue;
      for (const k of Object.keys(patch)) if (typeof patch[k] === 'number' ? Number.isFinite(patch[k]) : true) el[k] = patch[k];
    }
  };

  P._doRotate = function _doRotate(pt, e) {
    const g = this._g;
    const b = this._board;
    const el = findItem(b, g.el);
    if (!el) return;
    let a = normAngle(g.angle0 + rotateAngle(g.c, pt) - g.a0);
    if (e.shiftKey) a = normAngle(snapRotation(a));
    el.angle = a;
  };

  // ---------- linear point editing ----------
  P._doPoint = function _doPoint(pt, e, rel) {
    const g = this._g;
    const b = this._board;
    if (!g.startClient) g.startClient = rel;
    if (!g.moved && !g.inserted && Math.hypot(rel.x - g.startClient.x, rel.y - g.startClient.y) <= DRAG_PX) return;
    g.moved = true;
    b.items = g.base.map(cloneEl);
    const el = findItem(b, g.id);
    if (!el) return;
    const abs = g.abs.map((p) => ({ ...p }));
    let p = this._sp(pt);
    const isEnd = el.type === 'arrow' && (g.idx === 0 || g.idx === abs.length - 1);
    if (e.shiftKey && abs.length > 1) {
      const ref = abs[g.idx === 0 ? 1 : g.idx - 1];
      p = snapAngle45(ref, p);
    }
    abs[g.idx] = p;
    // an end that is being dragged is detached while it moves
    if (isEnd) {
      if (g.idx === 0) el.from = null; else el.to = null;
    }
    // drop the stale binding of an end that is not being dragged: it stays bound, its stored position is the resolved one
    setAbsPoints(el, abs);
    let target = null;
    if (isEnd) {
      target = this._bindTarget(pt, el.id, e);
      const other = g.idx === 0 ? el.to : el.from;
      if (target && target.id === other) target = null;
    }
    g.target = target;
    this._setHL(target);
    this._reanchorElbow(el);
  };

  P._reanchorElbow = function _reanchorElbow() { /* elbow routes derive from the two end points */ };

  P._endPoint = function _endPoint() {
    const g = this._g;
    const b = this._board;
    const el = findItem(b, g.id);
    if (el && g.moved) {
      if (el.type === 'arrow' && (g.idx === 0 || g.idx === g.abs.length - 1)) {
        const t = g.target;
        if (g.idx === 0) el.from = t ? t.id : null; else el.to = t ? t.id : null;
        this._tidyBound(el);
      }
    } else if (el && g.inserted) {
      b.items = g.base.map(cloneEl);
    }
    this._endGesture();
    if (g.moved) this._commit();
  };

  // Stored points of a bound (non-elbow) arrow equal where it is drawn, so unbinding later keeps the shape.
  P._tidyBound = function _tidyBound(el) {
    if (el.type !== 'arrow' || el.arrowType === 'elbow' || (el.from == null && el.to == null)) return;
    const abs = resolvePoints(this._board, el);
    if (abs.length >= 2) setAbsPoints(el, abs);
  };

  // ---------- marquee ----------
  P._doMarquee = function _doMarquee(pt, e, rel) {
    const g = this._g;
    if (!g.moved && Math.hypot(rel.x - g.startClient.x, rel.y - g.startClient.y) <= DRAG_PX) return;
    g.moved = true;
    g.box = { x: Math.min(g.start.x, pt.x), y: Math.min(g.start.y, pt.y), w: Math.abs(pt.x - g.start.x), h: Math.abs(pt.y - g.start.y) };
    const found = elementsInBox(this._board, g.box, { contain: !e.altKey })
      .map((id) => findItem(this._board, id)).filter((el) => el && !el.locked);
    const set = new Set(found.map((el) => el.id));
    for (const el of found) { // a touched group is selected whole
      if (el.groupIds && el.groupIds.length) {
        const gid = el.groupIds[el.groupIds.length - 1];
        for (const m of this._board.items) if (!m.locked && m.groupIds.includes(gid)) set.add(m.id);
      }
    }
    const ids = g.additive ? [...new Set([...g.base, ...set])] : [...set];
    this._setSelection(this._board.items.filter((el) => ids.includes(el.id)).map((el) => el.id));
  };

  // ---------- shapes ----------
  P._doShape = function _doShape(pt, e, rel) {
    const g = this._g;
    const b = this._board;
    if (!g.el) {
      if (Math.hypot(rel.x - g.startClient.x, rel.y - g.startClient.y) <= DRAG_PX) return;
      const el = createElement(g.tool, { id: newId(b), ...creationProps(g.tool, this._style, { textAlignSet: this._textAlignSet }), x: g.start.x, y: g.start.y, w: 1, h: 1 });
      if (g.tool === 'frame') el.name = `Frame ${b.items.filter((i) => i.type === 'frame').length + 1}`;
      if (g.tool === 'sticky' && !el.text) el.text = '';
      b.items.push(el);
      g.el = el;
    }
    const box = dragBox(g.start, this._sp(pt), { square: e.shiftKey, centre: e.altKey });
    Object.assign(g.el, { x: box.x, y: box.y, w: Math.max(box.w, 1), h: Math.max(box.h, 1) });
  };

  P._endShape = function _endShape() {
    const g = this._g;
    const b = this._board;
    let el = g.el;
    if (!el) {
      const [dw, dh] = { rect: [160, 90], diamond: [140, 100], ellipse: [140, 90], database: [110, 110], sticky: [160, 160], frame: [360, 240] }[g.tool];
      el = createElement(g.tool, { id: newId(b), ...creationProps(g.tool, this._style, { textAlignSet: this._textAlignSet }) });
      el.w = dw; el.h = dh;
      el.x = this._gs(g.start.x - dw / 2);
      el.y = this._gs(g.start.y - dh / 2);
      if (g.tool === 'frame') el.name = `Frame ${b.items.filter((i) => i.type === 'frame').length + 1}`;
      b.items.push(el);
    } else {
      if (el.w < MIN_SIZE.w) el.w = MIN_SIZE.w;
      if (el.h < MIN_SIZE.h) el.h = MIN_SIZE.h;
    }
    this._finishCreated(el);
  };

  // Shared tail of every one-shot creation.
  P._finishCreated = function _finishCreated(el, { select = true } = {}) {
    const b = this._board;
    if (el.type === 'frame') {
      const kids = b.items.filter((i) => i.type !== 'frame' && i.frameId == null && !i.locked);
      ops.assignFrames(b, kids.map((k) => k.id));
      // frames sit below their children
      const first = b.items.findIndex((i) => i.frameId === el.id);
      const mine = b.items.indexOf(el);
      if (first !== -1 && first < mine) { b.items.splice(mine, 1); b.items.splice(first, 0, el); }
    } else ops.assignFrames(b, [el.id]);
    this._endGesture();
    const keep = this._toolLocked;
    this._setSelection(keep && !isLinear(el) ? [] : (select ? [el.id] : []));
    this._justCreated = this._selection.size === 1 ? el.id : null;
    this._afterCreate();
    this._commit();
  };

  // ---------- arrows / lines ----------
  P._downLinear = function _downLinear(e, pt) {
    const from = this._tool === 'arrow' ? this._bindTarget(pt, null, e) : null;
    this._g = { type: 'linear', tool: this._tool, start: this._sp(pt), raw: pt, startClient: this._rel(e), from, el: null };
    this._setHL(from);
  };

  P._makeLinear = function _makeLinear(type, start, from) {
    const b = this._board;
    const el = createElement(type, { id: newId(b), ...creationProps(type, this._style), x: start.x, y: start.y, points: [[0, 0], [0, 0]], w: 0, h: 0 });
    if (type === 'arrow' && from) el.from = from.id;
    b.items.push(el);
    return el;
  };

  P._linearEnd = function _linearEnd(el, base, pt, e) {
    let end = this._sp(pt);
    if (e.shiftKey) end = snapAngle45(base, end);
    let target = el.type === 'arrow' ? this._bindTarget(pt, el.id, e) : null;
    if (target && target.id === el.from) target = null;
    if (target && !e.shiftKey) end = { x: pt.x, y: pt.y };
    return { end, target };
  };

  P._doLinear = function _doLinear(pt, e, rel) {
    const g = this._g;
    if (!g.el) {
      if (Math.hypot(rel.x - g.startClient.x, rel.y - g.startClient.y) <= DRAG_PX) return;
      g.el = this._makeLinear(g.tool, g.start, g.from);
    }
    const { end, target } = this._linearEnd(g.el, g.start, pt, e);
    setAbsPoints(g.el, [g.start, end]);
    g.el.to = target ? target.id : null;
    this._setHL(target || g.from);
    g.target = target;
  };

  P._endLinear = function _endLinear(e) {
    const g = this._g;
    if (!g.el) {
      // a click: start a multi-point line that follows the pointer
      const el = this._makeLinear(g.tool, g.start, g.from);
      this._g = { type: 'multi', tool: g.tool, el, pts: [g.start], cur: null };
      this._pointerDownCount = 1;
      this._bindHL = g.from ? { x: g.from.x, y: g.from.y, w: g.from.w, h: g.from.h, angle: g.from.angle || 0 } : null;
      return;
    }
    const el = g.el;
    this._tidyBound(el);
    this._g = null;
    this._finishCreated(el);
    void e;
  };

  P._multiAbs = function _multiAbs(g, withCur = true) {
    const pts = g.pts.map((p) => ({ x: p.x, y: p.y }));
    if (withCur && g.cur) pts.push({ x: g.cur.x, y: g.cur.y });
    return pts;
  };

  P._doMulti = function _doMulti(pt, e) {
    const g = this._g;
    const last = g.pts[g.pts.length - 1];
    const { end, target } = this._linearEnd(g.el, last, pt, e);
    g.cur = end;
    g.target = target;
    const abs = this._multiAbs(g);
    if (abs.length >= 2) {
      setAbsPoints(g.el, abs);
      g.el.to = target ? target.id : null;
    }
    this._setHL(target);
  };

  P._multiClick = function _multiClick(e) {
    const g = this._g;
    const z = this._view.zoom;
    const pt = this._pt(e);
    const last = g.pts[g.pts.length - 1];
    const { end, target } = this._linearEnd(g.el, last, pt, e);
    const same = Math.hypot(end.x - last.x, end.y - last.y) * z < 5;
    if (same) { this._finishMulti(); return; }
    g.pts.push(end);
    g.cur = null;
    if (g.el.type === 'arrow' && g.el.arrowType === 'elbow') { g.target = target; this._finishMulti(); return; }
    setAbsPoints(g.el, this._multiAbs(g, false).concat([{ x: end.x + 0.01, y: end.y + 0.01 }]));
    g.target = target;
  };

  P._finishMulti = function _finishMulti() {
    const g = this._g;
    if (!g || g.type !== 'multi') return;
    const el = g.el;
    const abs = this._multiAbs(g, false);
    this._g = null;
    if (abs.length < 2) {
      this._board.items = this._board.items.filter((i) => i.id !== el.id);
      this._clearTransient();
      this._renderNow();
      return;
    }
    setAbsPoints(el, abs);
    if (el.type === 'arrow') {
      const t = g.target || null;
      el.to = t ? t.id : null;
      if (el.to === el.from) el.to = null;
    }
    this._tidyBound(el);
    this._lastLinearEnd = Date.now();
    this._finishCreated(el);
  };

  // ---------- freedraw ----------
  P._downFree = function _downFree(e, pt) {
    const b = this._board;
    const el = createElement('freedraw', { id: newId(b), ...creationProps('freedraw', this._style), x: pt.x, y: pt.y, points: [[0, 0]], w: 0, h: 0 });
    b.items.push(el);
    this._g = { type: 'free', el, last: pt };
  };

  P._doFree = function _doFree(pt) {
    const g = this._g;
    const z = this._view.zoom;
    if (Math.hypot(pt.x - g.last.x, pt.y - g.last.y) * z < 1.2) return;
    if (g.el.points.length >= 3900) return;
    g.last = pt;
    g.el.points.push([pt.x - g.el.x, pt.y - g.el.y]);
    normalizePoints(g.el);
  };

  P._endFree = function _endFree() {
    const g = this._g;
    const el = g.el;
    const z = this._view.zoom;
    if (el.points.length > 2) el.points = simplifyStroke(el.points, 0.6 / z);
    if (el.points.length === 1) el.points = [[0, 0], [0.1, 0.1]];
    normalizePoints(el);
    this._endGesture();
    this._setSelection([]);
    this._commit();
  };

  // ---------- eraser ----------
  P._eraseAt = function _eraseAt(a, b2) {
    const z = this._view.zoom;
    const dist = Math.hypot(b2.x - a.x, b2.y - a.y);
    const steps = Math.max(1, Math.ceil((dist * z) / 4));
    const ids = [];
    for (let i = 0; i <= steps; i++) {
      const p = { x: a.x + ((b2.x - a.x) * i) / steps, y: a.y + ((b2.y - a.y) * i) / steps };
      const hit = hitTestTop(this._board, p, 4 / z, { skipLocked: true });
      if (hit && !this._erasing.has(hit.id)) { this._erasing.add(hit.id); ids.push(hit.id); }
    }
    if (ids.length) this._scheduleRender();
  };

  P._endErase = function _endErase() {
    const ids = [...this._erasing];
    this._erasing = new Set();
    this._g = null;
    if (ids.length) {
      ops.deleteElements(this._board, ids);
      this._setSelection([]);
    }
    this._clearTransient();
    this._commit();
  };

  // ---------- bucket / eyedropper / text click ----------
  P._endPick = function _endPick(pt) {
    const g = this._g;
    const hit = hitTestTop(this._board, pt, 6 / this._view.zoom, { skipLocked: true });
    this._g = null;
    if (!hit) return;
    if (this._tool === 'bucket') {
      const col = this._style.backgroundColor;
      if (ops.applyStyle(this._board, [hit.id], { backgroundColor: col }).length) this._commit();
      return;
    }
    const useBg = g.alt;
    const col = useBg ? hit.backgroundColor : hit.strokeColor;
    if (!col || (useBg && col === 'transparent')) { this._toast('Nothing to pick there'); return; }
    this.setStyle(useBg ? { backgroundColor: col } : { strokeColor: col });
    this._toast(`${useBg ? 'Background' : 'Stroke'} colour ${col}`);
    if (!this._toolLocked) { this._tool = 'select'; this._applyCursor(); this._emit('tool', { tool: 'select', locked: this._toolLocked }); }
  };

  // ---------- pointer up ----------
  P._onUp = function _onUp(e) {
    if (this._destroyed) return;
    this._touches.delete(e.pointerId);
    try { this.svg.releasePointerCapture(e.pointerId); } catch { /* not captured */ }
    const g = this._g;
    if (!g) return;
    const pt = this._pt(e);
    switch (g.type) {
      case 'pinch': if (this._touches.size < 2) this._g = null; return;
      case 'pan': this._g = null; this._applyCursor(); return;
      case 'laser': this._g = null; return;
      case 'multi': return;
      case 'press':
        if (!g.shift && g.wasSelected && this._selection.size > 1) this._setSelection(g.ids);
        this._endGesture();
        return;
      case 'move': this._endMove(); return;
      case 'resize': {
        if (!g.moved) { this._endGesture(); return; }
        const b = this._board;
        ops.assignFrames(b, g.ids.filter((id) => findItem(b, id)?.type !== 'frame'));
        this._endGesture();
        this._commit();
        return;
      }
      case 'rotate': this._endGesture(); if (g.moved) this._commit(); return;
      case 'point': this._endPoint(); return;
      case 'marquee':
        if (!g.moved && !g.additive) this._setSelection([]);
        this._endGesture();
        return;
      case 'shape': if (e.type === 'pointercancel' && !g.el) { this._endGesture(); return; } this._endShape(); return;
      case 'linear': if (e.type === 'pointercancel') { this._cancelGesture(); return; } this._endLinear(e); return;
      case 'free': this._endFree(); return;
      case 'erase': this._endErase(); return;
      case 'pick': this._endPick(pt); return;
      case 'textclick': this._endTextClick(g, pt); return;
      default: this._endGesture();
    }
  };

  P._endTextClick = function _endTextClick(g, pt) {
    this._g = null;
    const hit = g.hit;
    if (hit && (hit.type === 'text' || isContainer(hit) || (hit.type === 'arrow'))) {
      this._setSelection([hit.id]);
      this._beginEdit(hit.id);
      return;
    }
    this._newText(pt);
  };

  // ---------- double click / context menu ----------
  P._onDblClick = function _onDblClick(e) {
    if (!this._canEdit()) return;
    if (this._lastLinearEnd && Date.now() - this._lastLinearEnd < 450) return;
    if (this._g && this._g.type === 'multi') { this._finishMulti(); return; }
    if (this._tool !== 'select' && this._tool !== 'text') return;
    const pt = this._pt(e);
    const hit = hitTestTop(this._board, pt, 6 / this._view.zoom, { skipLocked: true });
    if (hit) {
      if (hit.type === 'text' || isContainer(hit) || hit.type === 'arrow' || hit.type === 'frame') {
        this._setSelection([hit.id]);
        this._beginEdit(hit.id);
      }
      return;
    }
    this._newText(pt);
  };

  P._onContext = function _onContext(e) {
    e.preventDefault();
    if (this._g && this._g.type !== 'multi') return;
    const pt = this._pt(e);
    const hit = hitTestTop(this._board, pt, 6 / this._view.zoom, {});
    if (hit && this._canEdit() && !hit.locked && !this._selection.has(hit.id)) this._setSelection(this._groupIds(hit, false));
    else if (!hit && this._canEdit() && this._selection.size) this._setSelection([]);
    this._emit('contextmenu', { clientX: e.clientX, clientY: e.clientY, targetId: hit ? hit.id : null });
  };

  // ---------- wheel ----------
  P._onWheel = function _onWheel(e) {
    e.preventDefault();
    const k = e.deltaMode === 1 ? 16 : e.deltaMode === 2 ? 400 : 1;
    const dx = fin(e.deltaX) * k;
    const dy = fin(e.deltaY) * k;
    if (e.ctrlKey || e.metaKey) {
      const rel = this._rel(e);
      this.setZoom(this._view.zoom * Math.exp(-dy * 0.01), rel);
    } else if (e.shiftKey && !dx) this.pan(-dy, 0);
    else this.pan(-dx, -dy);
  };

  // ---------- laser ----------
  P._laserPush = function _laserPush(e) {
    const rel = this._rel(e);
    (this._trailPts || (this._trailPts = [])).push({ x: rel.x, y: rel.y, t: performance.now() });
    if (!this._trailRaf) this._laserStep();
  };

  P._laserStep = function _laserStep() {
    const reduced = typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
    const life = reduced ? 300 : 900;
    const now = performance.now();
    this._trailPts = (this._trailPts || []).filter((p) => now - p.t < life);
    const g = this._trail;
    g.replaceChildren();
    const pts = this._trailPts;
    for (let i = 1; i < pts.length; i++) {
      const age = 1 - (now - pts[i].t) / life;
      svgEl('line', { x1: pts[i - 1].x, y1: pts[i - 1].y, x2: pts[i].x, y2: pts[i].y, stroke: '#ff2d55', 'stroke-width': 3 + 4 * age, 'stroke-linecap': 'round', 'stroke-opacity': Math.max(0, age) }, g);
    }
    if (pts.length) {
      const h = pts[pts.length - 1];
      svgEl('circle', { cx: h.x, cy: h.y, r: 6, fill: '#ff2d55', 'fill-opacity': Math.max(0, 1 - (now - h.t) / life) }, g);
    }
    if (pts.length && typeof requestAnimationFrame === 'function') this._trailRaf = requestAnimationFrame(() => { this._trailRaf = 0; this._laserStep(); });
    else this._trailRaf = 0;
  };

  // ---------- keyboard ----------
  P._onKeyUp = function _onKeyUp(e) {
    if (e.code === 'Space' || e.key === ' ') {
      if (this._space) { this._space = false; this._applyCursor(); }
    }
  };

  P._nudge = function _nudge(dx, dy) {
    if (!this._selection.size || !this._canEdit()) return;
    this._act(() => {
      const ids = this._selIds();
      ops.moveElements(this._board, ids, dx, dy);
      ops.assignFrames(this._board, ids.filter((id) => findItem(this._board, id)?.type !== 'frame'));
    });
  };

  P._onKeyDown = function _onKeyDown(e) {
    if (!this._keyboard || this._destroyed || e.defaultPrevented) return;
    if (e.isComposing || e.keyCode === 229) return;
    const t = e.target;
    if (this._editor) return;
    if (t && t.closest && t.closest(IGNORE_KEY_TARGETS)) return;
    const onButton = !!(t && t.closest && t.closest('button,a,[role="button"],[role="tab"],[role="menuitem"]'));
    const mod = e.ctrlKey || e.metaKey;
    const { shiftKey: shift, altKey: alt, code, key } = e;
    const editable = this._canEdit();
    const prevent = () => e.preventDefault();

    if (code === 'Space' || key === ' ') {
      if (onButton || mod || alt) return;
      prevent();
      if (!this._space) { this._space = true; this._applyCursor(); }
      return;
    }

    if (mod) {
      if (alt) {
        if (code === 'KeyC') { prevent(); this.copyStyle(); } else if (code === 'KeyV') { prevent(); this.pasteStyle(); }
        return;
      }
      switch (code) {
        case 'KeyA': prevent(); this.selectAll(); return;
        case 'KeyD': prevent(); if (editable) this.duplicate(); return;
        case 'KeyC': prevent(); this.copy(); return;
        case 'KeyX': prevent(); if (editable) this.cut(); return;
        case 'KeyV': if (shift) this._plainNext = true; return; // the browser then fires a paste event
        case 'KeyZ': prevent(); if (shift) this.redo(); else this.undo(); return;
        case 'KeyY': prevent(); this.redo(); return;
        case 'KeyG': prevent(); if (shift) this.ungroup(); else this.group(); return;
        case 'KeyL': if (shift) { prevent(); this.toggleLock(); } return;
        case 'BracketLeft': prevent(); if (shift) this.sendToBack(); else this.sendBackward(); return;
        case 'BracketRight': prevent(); if (shift) this.bringToFront(); else this.bringForward(); return;
        case 'ArrowLeft': case 'ArrowRight': case 'ArrowUp': case 'ArrowDown':
          prevent();
          if (shift) this.align({ ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'top', ArrowDown: 'bottom' }[code]);
          else this.flowchartAdd(code);
          return;
        case 'Comma': case 'Period':
          if (shift) { prevent(); if (code === 'Comma') this.decreaseFontSize(); else this.increaseFontSize(); }
          return;
        case 'KeyK': prevent(); this._emit('request', { name: 'link' }); return;
        case 'Equal': case 'NumpadAdd': prevent(); this.zoomIn(); return;
        case 'Minus': case 'NumpadSubtract': prevent(); this.zoomOut(); return;
        case 'Digit0': case 'Numpad0': prevent(); this.resetZoom(); return;
        case 'Quote': prevent(); this.toggleGrid(); return;
        case 'KeyF': prevent(); this._emit('request', { name: 'search' }); return;
        case 'Slash': prevent(); this._emit('request', { name: 'palette' }); return;
        case 'KeyP': if (shift) { prevent(); this._emit('request', { name: 'palette' }); } return;
        case 'KeyE': if (shift) { prevent(); this._emit('request', { name: 'export' }); } return;
        case 'KeyO': prevent(); this._emit('request', { name: 'open' }); return;
        case 'KeyS': prevent(); this._emit('request', { name: 'save' }); return;
        default: return;
      }
    }

    if (alt) {
      switch (code) {
        case 'KeyS': prevent(); this.toggleSnap(); break;
        case 'KeyZ': prevent(); this.toggleZen(); break;
        case 'KeyR': prevent(); this.toggleViewMode(); break;
        case 'KeyD': if (shift) { prevent(); this.toggleTheme(); } break;
        case 'Slash': prevent(); this._emit('request', { name: 'stats' }); break;
        default: break;
      }
      return;
    }

    if (key === 'Escape') {
      if (this._g && this._g.type === 'multi') { this._finishMulti(); return; }
      if (this._g) { this._cancelGesture(); return; }
      if (this._selection.size) { this.clearSelection(); return; }
      if (this._tool !== 'select') this.setTool('select');
      return;
    }
    if (key === 'Enter') {
      if (this._g && this._g.type === 'multi') { prevent(); this._finishMulti(); return; }
      if (onButton) return;
      if (editable && this._selection.size === 1) { prevent(); this._beginEdit([...this._selection][0]); }
      return;
    }
    if (key === 'Delete' || key === 'Backspace') { if (editable && this._selection.size) { prevent(); this.deleteSelected(); } return; }
    if (key.startsWith('Arrow')) {
      if (!this._selection.size || !editable || onButton) return;
      prevent();
      const s = shift ? 10 : 1;
      this._nudge(key === 'ArrowLeft' ? -s : key === 'ArrowRight' ? s : 0, key === 'ArrowUp' ? -s : key === 'ArrowDown' ? s : 0);
      return;
    }
    if (key === '?') { this._emit('request', { name: 'help' }); return; }
    if (shift) {
      if (code === 'KeyH') { prevent(); this.flip('h'); return; }
      if (code === 'KeyV') { prevent(); this.flip('v'); return; }
      if (code === 'Digit1') { prevent(); this.zoomToFit(); return; }
      if (code === 'Digit2') { prevent(); this.zoomToSelection(); return; }
      if (key.length === 1 && editable && this._justCreated != null) { /* shifted character typed into a fresh shape */ } else return;
    }
    // typing straight after creating a shape starts its label
    if (key.length === 1 && editable && this._justCreated != null && this._selection.size === 1 && this._selection.has(this._justCreated)) {
      const el = findItem(this._board, this._justCreated);
      if (el && isContainer(el)) { prevent(); this._beginEdit(el.id, { initial: key }); return; }
    }
    if (key.length !== 1) return;
    const k = key.toLowerCase();
    if (k === 'q') { this.toggleToolLock(); return; }
    const tool = TOOLS.find((tl) => tl.key === k || tl.num === k);
    if (tool) { prevent(); this.setTool(tool.id); }
  };
}


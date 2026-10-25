// Hit testing, boxes, handles, resize maths, snapping and text metrics. DOM-free (numbers only).
import {
  BINDABLE_TYPES, LIMITS, MIN_SIZE, center, elementBounds, findItem, hasPoints, isLinear, normalizePoints, pointsBounds, resolvePoints, rotatePoint,
  selectionBounds,
} from '../../shared/board.js';
import { curveSamples, databaseCap } from './rough.js';

const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
const isFilled = (el) => !!el.backgroundColor && el.backgroundColor !== 'transparent';

// ---------- text metrics ----------
const AVG = { 1: 0.55, 2: 0.52, 3: 0.6 };
const NARROW = new Set("iljtfI.,:;'|!()[]`".split(''));
const WIDE = new Set('mwMW@%&#'.split(''));

function glyphWidth(ch, family) {
  const avg = AVG[family] || AVG[2];
  if (family === 3) return avg;
  if (ch === ' ') return avg * 0.6;
  if (NARROW.has(ch)) return avg * 0.55;
  if (WIDE.has(ch)) return avg * 1.45;
  if (ch >= 'A' && ch <= 'Z') return avg * 1.15;
  if (ch.charCodeAt(0) > 0x2e80) return 1; // CJK and friends are full-width
  return avg * 0.98;
}

function measure(str, family, size) {
  let w = 0;
  for (const ch of str) w += glyphWidth(ch, family);
  return w * size;
}

export function textSize(text, { fontFamily = 1, fontSize = 20, maxWidth } = {}) {
  const lineHeight = fontSize * 1.25;
  const hard = String(text ?? '').replace(/\r/g, '').replace(/\t/g, '  ').split('\n');
  let lines = [];
  if (!maxWidth || maxWidth <= 0) lines = hard;
  else {
    for (const para of hard) {
      if (para === '') { lines.push(''); continue; }
      let cur = '';
      const words = para.split(/(?<= )/); // keep the trailing space with each word
      for (const word of words) {
        const trial = cur + word;
        if (measure(trial.trimEnd(), fontFamily, fontSize) <= maxWidth) { cur = trial; continue; }
        if (cur) { lines.push(cur.trimEnd()); cur = ''; }
        let rest = word;
        while (measure(rest.trimEnd(), fontFamily, fontSize) > maxWidth && rest.length > 1) { // break a long word
          let k = 1;
          while (k < rest.length && measure(rest.slice(0, k + 1), fontFamily, fontSize) <= maxWidth) k++;
          lines.push(rest.slice(0, k)); rest = rest.slice(k);
        }
        cur = rest;
      }
      lines.push(cur.trimEnd());
    }
  }
  let w = 0;
  for (const l of lines) w = Math.max(w, measure(l, fontFamily, fontSize));
  return { w, h: Math.max(1, lines.length) * lineHeight, lines };
}

// ---------- labels ----------
// The box (in the element's own, unrotated space) available to a container's text.
export function innerBox(el) {
  const { x, y, w, h } = el;
  if (el.type === 'ellipse') {
    const iw = w * 0.7071 - 8; const ih = h * 0.7071 - 8;
    return { x: x + (w - iw) / 2, y: y + (h - ih) / 2, w: Math.max(iw, 4), h: Math.max(ih, 4) };
  }
  if (el.type === 'diamond') return { x: x + w / 4 + 2, y: y + h / 4 + 2, w: Math.max(w / 2 - 4, 4), h: Math.max(h / 2 - 4, 4) };
  if (el.type === 'database') {
    const ry = databaseCap(el);
    return { x: x + 8, y: y + ry * 2, w: Math.max(w - 16, 4), h: Math.max(h - ry * 3, 4) };
  }
  const pad = el.type === 'sticky' ? 12 : 8;
  return { x: x + pad, y: y + pad, w: Math.max(w - pad * 2, 4), h: Math.max(h - pad * 2, 4) };
}

function layoutIn(box, text, el, wrap) {
  const fontSize = el.fontSize || 20; const fontFamily = el.fontFamily || 1;
  const size = textSize(text, { fontFamily, fontSize, maxWidth: wrap ? box.w : undefined });
  const align = el.textAlign || 'center'; const valign = el.verticalAlign || 'middle';
  const anchorX = align === 'left' ? box.x : align === 'right' ? box.x + box.w : box.x + box.w / 2;
  const left = align === 'left' ? box.x : align === 'right' ? box.x + box.w - size.w : box.x + (box.w - size.w) / 2;
  const top = valign === 'top' ? box.y : valign === 'bottom' ? box.y + box.h - size.h : box.y + (box.h - size.h) / 2;
  return { lines: size.lines, lineHeight: fontSize * 1.25, fontSize, fontFamily, anchor: align === 'left' ? 'start' : align === 'right' ? 'end' : 'middle', anchorX, top, box: { x: left, y: top, w: size.w, h: size.h } };
}

export function labelLayout(el) {
  if (!el.text) return null;
  return layoutIn(innerBox(el), el.text, el, true);
}

// Midpoint (by length) of a polyline, absolute points.
export function polylineMidpoint(pts) {
  if (!pts.length) return { x: 0, y: 0 };
  let total = 0;
  for (let i = 1; i < pts.length; i++) total += Math.hypot(pts[i].x - pts[i - 1].x, pts[i].y - pts[i - 1].y);
  let target = total / 2;
  for (let i = 1; i < pts.length; i++) {
    const l = Math.hypot(pts[i].x - pts[i - 1].x, pts[i].y - pts[i - 1].y);
    if (target <= l && l > 0) { const t = target / l; return { x: pts[i - 1].x + (pts[i].x - pts[i - 1].x) * t, y: pts[i - 1].y + (pts[i].y - pts[i - 1].y) * t }; }
    target -= l;
  }
  return pts[pts.length - 1];
}

// Layout of an arrow/line label: centred on the path midpoint, never wider than 220px (wraps).
export function linearLabelLayout(board, el) {
  if (!el.text) return null;
  const pts = resolvePoints(board, el);
  if (pts.length < 2) return null;
  const path = el.arrowType === 'round' && pts.length >= 3 ? curveSamples(pts, 8) : pts;
  const mid = polylineMidpoint(path);
  const size = textSize(el.text, { fontFamily: el.fontFamily || 1, fontSize: el.fontSize || 16, maxWidth: 220 });
  const box = { x: mid.x - size.w / 2, y: mid.y - size.h / 2, w: size.w, h: size.h };
  return { lines: size.lines, lineHeight: (el.fontSize || 16) * 1.25, fontSize: el.fontSize || 16, fontFamily: el.fontFamily || 1, anchor: 'middle', anchorX: mid.x, top: box.y, box };
}

function frameNameBox(el) {
  const s = textSize(el.name || '', { fontFamily: 2, fontSize: 14 });
  return { x: el.x, y: el.y - 24, w: Math.max(s.w + 8, 24), h: 22 };
}
export { frameNameBox };

// ---------- distance helpers ----------
function distToSegment(p, a, b) {
  const dx = b.x - a.x; const dy = b.y - a.y;
  const l2 = dx * dx + dy * dy;
  const t = l2 === 0 ? 0 : clamp(((p.x - a.x) * dx + (p.y - a.y) * dy) / l2, 0, 1);
  return Math.hypot(p.x - (a.x + dx * t), p.y - (a.y + dy * t));
}
function distToPolyline(p, pts, closed = false) {
  if (pts.length === 1) return Math.hypot(p.x - pts[0].x, p.y - pts[0].y);
  let best = Infinity;
  const n = closed ? pts.length : pts.length - 1;
  for (let i = 0; i < n; i++) best = Math.min(best, distToSegment(p, pts[i], pts[(i + 1) % pts.length]));
  return best;
}
function inPolygon(p, pts) {
  let inside = false;
  for (let i = 0, j = pts.length - 1; i < pts.length; j = i++) {
    const a = pts[i]; const b = pts[j];
    if ((a.y > p.y) !== (b.y > p.y) && p.x < ((b.x - a.x) * (p.y - a.y)) / (b.y - a.y) + a.x) inside = !inside;
  }
  return inside;
}
const inRect = (p, r, m = 0) => p.x >= r.x - m && p.x <= r.x + r.w + m && p.y >= r.y - m && p.y <= r.y + r.h + m;

function ellipsePoly(cx, cy, a, b, n = 40, from = 0, to = Math.PI * 2) {
  const out = [];
  for (let i = 0; i <= n; i++) { const t = from + ((to - from) * i) / n; out.push({ x: cx + a * Math.cos(t), y: cy + b * Math.sin(t) }); }
  return out;
}

// Silhouette of a shape in its own (unrotated) space as a polygon, plus extra open outline polylines (database top ellipse).
function outlineOf(el) {
  const { x, y, w, h } = el;
  switch (el.type) {
    case 'diamond': return { poly: [{ x: x + w / 2, y }, { x: x + w, y: y + h / 2 }, { x: x + w / 2, y: y + h }, { x, y: y + h / 2 }], extra: [] };
    case 'ellipse': return { poly: ellipsePoly(x + w / 2, y + h / 2, w / 2, h / 2).slice(0, -1), extra: [] };
    case 'database': {
      const a = w / 2; const ry = databaseCap(el); const cx = x + a;
      const poly = [...ellipsePoly(cx, y + ry, a, ry, 24, Math.PI, Math.PI * 2), ...ellipsePoly(cx, y + h - ry, a, ry, 24, 0, Math.PI)];
      return { poly, extra: [ellipsePoly(cx, y + ry, a, ry, 32, 0, Math.PI)] };
    }
    default: return { poly: [{ x, y }, { x: x + w, y }, { x: x + w, y: y + h }, { x, y: y + h }], extra: [] };
  }
}

// Distance from a point (in the element's local space) to the shape: 0 inside, otherwise distance to the outline.
function shapeDistanceLocal(el, p) {
  const { poly } = outlineOf(el);
  return inPolygon(p, poly) ? 0 : distToPolyline(p, poly, true);
}

function toLocal(el, point) {
  return el.angle ? rotatePoint(point, center(el), -el.angle) : point;
}

// ---------- hit testing ----------
export function hitElement(board, el, point, tol = 0) {
  const sw = (el.strokeWidth || 1) / 2;
  if (el.type === 'freedraw') {
    const abs = (el.points || []).map(([px, py]) => ({ x: el.x + px, y: el.y + py }));
    if (!abs.length) return false;
    return distToPolyline(point, abs) <= tol + sw + 2;
  }
  if (isLinear(el)) {
    const pts = resolvePoints(board, el);
    if (pts.length < 2) return false;
    const path = el.arrowType === 'round' && pts.length >= 3 ? curveSamples(pts, 10) : pts;
    if (distToPolyline(point, path) <= tol + sw + 1) return true;
    const lab = linearLabelLayout(board, el);
    return !!lab && inRect(point, lab.box, 4);
  }
  const p = toLocal(el, point);
  const box = { x: el.x, y: el.y, w: el.w, h: el.h };
  if (el.type === 'frame') {
    const { poly } = outlineOf(el);
    if (distToPolyline(p, poly, true) <= tol + sw + 1) return true;
    return !!el.name && inRect(p, frameNameBox(el), 0);
  }
  if (el.type === 'image' || el.type === 'text' || el.type === 'sticky') return inRect(p, box, tol);
  // rect, diamond, ellipse, database
  if (el.text) {
    const lay = labelLayout(el);
    if (lay && inRect(p, lay.box, 3)) return true;
  }
  const d = shapeDistanceLocal(el, p);
  if (isFilled(el)) return d <= tol;
  if (d === 0) {
    const { poly } = outlineOf(el);
    return distToPolyline(p, poly, true) <= tol + sw + 1 || outlineOf(el).extra.some((e) => distToPolyline(p, e) <= tol + sw + 1);
  }
  return d <= tol + sw + 1;
}

export function hitTestTop(board, point, tol = 0, { skipLocked = false, ids } = {}) {
  const only = ids ? new Set(ids) : null;
  for (let i = board.items.length - 1; i >= 0; i--) {
    const el = board.items[i];
    if (only && !only.has(el.id)) continue;
    if (skipLocked && el.locked) continue;
    if (hitElement(board, el, point, tol)) return el;
  }
  return null;
}

// ---------- marquee ----------
function segHitsRect(a, b, r) {
  if (inRect(a, r) || inRect(b, r)) return true;
  const c = [{ x: r.x, y: r.y }, { x: r.x + r.w, y: r.y }, { x: r.x + r.w, y: r.y + r.h }, { x: r.x, y: r.y + r.h }];
  const cross = (p, q, s) => (q.x - p.x) * (s.y - p.y) - (q.y - p.y) * (s.x - p.x);
  for (let i = 0; i < 4; i++) {
    const p = c[i]; const q = c[(i + 1) % 4];
    if (cross(a, b, p) * cross(a, b, q) <= 0 && cross(p, q, a) * cross(p, q, b) <= 0) return true;
  }
  return false;
}

export function elementsInBox(board, box, { contain = false } = {}) {
  const b = { x: Math.min(box.x, box.x + box.w), y: Math.min(box.y, box.y + box.h), w: Math.abs(box.w), h: Math.abs(box.h) };
  const out = [];
  for (const el of board.items) {
    const eb = elementBounds(board, el);
    if (contain) {
      if (eb.x >= b.x && eb.y >= b.y && eb.x + eb.w <= b.x + b.w && eb.y + eb.h <= b.y + b.h) out.push(el.id);
      continue;
    }
    if (eb.x > b.x + b.w || eb.x + eb.w < b.x || eb.y > b.y + b.h || eb.y + eb.h < b.y) continue;
    if (hasPoints(el)) {
      const pts = hasPoints(el) && el.type !== 'freedraw' ? resolvePoints(board, el) : el.points.map(([px, py]) => ({ x: el.x + px, y: el.y + py }));
      const path = el.arrowType === 'round' && pts.length >= 3 ? curveSamples(pts, 8) : pts;
      let hit = path.length === 1 ? inRect(path[0], b) : false;
      for (let i = 1; i < path.length && !hit; i++) hit = segHitsRect(path[i - 1], path[i], b);
      if (hit) out.push(el.id);
    } else out.push(el.id);
  }
  return out;
}

// ---------- handles ----------
export function transformHandles(box, angle = 0, zoom = 1, { rotation = true, linear = false } = {}) {
  const c = { x: box.x + box.w / 2, y: box.y + box.h / 2 };
  const l = box.x; const r = box.x + box.w; const t = box.y; const b = box.y + box.h;
  const raw = { nw: [l, t], ne: [r, t], se: [r, b], sw: [l, b] };
  if (!linear) Object.assign(raw, { n: [c.x, t], e: [r, c.y], s: [c.x, b], w: [l, c.y] });
  if (rotation && !linear) raw.rotate = [c.x, t - 24 / (zoom || 1)];
  const out = {};
  for (const [k, [x, y]] of Object.entries(raw)) out[k] = rotatePoint({ x, y }, c, angle);
  return out;
}

export function handleAt(handles, point, zoom = 1) {
  const rad = 8 / (zoom || 1);
  for (const name of ['rotate', 'nw', 'ne', 'se', 'sw', 'n', 'e', 's', 'w']) {
    const h = handles[name];
    if (h && Math.hypot(point.x - h.x, point.y - h.y) <= rad) return name;
  }
  return null;
}

// ---------- resizing ----------
// New [lo, hi] along one axis. `anchor` is the fixed edge (ignored when fromCenter), `p` the dragged position.
function axisSpan(anchor, p, c, fromCenter, min) {
  if (fromCenter) { const half = Math.max(Math.abs(p - c), min / 2); return [c - half, c + half, p >= c ? 1 : -1]; }
  let d = p - anchor;
  if (Math.abs(d) < min) d = d < 0 ? -min : min;
  return [Math.min(anchor, anchor + d), Math.max(anchor, anchor + d), d < 0 ? -1 : 1];
}

export function resizeBox(el, handle, point, { keepAspect = false, fromCenter = false } = {}) {
  const angle = el.angle || 0;
  const cen = { x: el.x + el.w / 2, y: el.y + el.h / 2 };
  const p = angle ? rotatePoint(point, cen, -angle) : point;
  const west = handle.includes('w'); const east = handle.includes('e');
  const north = handle.includes('n'); const south = handle.includes('s');
  const min = MIN_SIZE.w;
  let x0 = el.x; let x1 = el.x + el.w; let y0 = el.y; let y1 = el.y + el.h;
  let sx = 1; let sy = 1;
  if (west || east) [x0, x1, sx] = axisSpan(west ? el.x + el.w : el.x, p.x, cen.x, fromCenter, min);
  if (north || south) [y0, y1, sy] = axisSpan(north ? el.y + el.h : el.y, p.y, cen.y, fromCenter, min);
  if (keepAspect && el.w > 0 && el.h > 0) {
    const corner = (west || east) && (north || south);
    const ax = west ? el.x + el.w : el.x; const ay = north ? el.y + el.h : el.y;
    const span = (anchor, size, dir, c) => (fromCenter ? [c - size / 2, c + size / 2] : (dir > 0 ? [anchor, anchor + size] : [anchor - size, anchor]));
    if (corner) {
      const s = Math.max((x1 - x0) / el.w, (y1 - y0) / el.h);
      const nw = Math.max(el.w * s, min); const nh = Math.max(el.h * s, min);
      [x0, x1] = span(ax, nw, sx, cen.x);
      [y0, y1] = span(ay, nh, sy, cen.y);
    } else if (west || east) {
      const nh = Math.max(((x1 - x0) * el.h) / el.w, min);
      [y0, y1] = [cen.y - nh / 2, cen.y + nh / 2];
    } else {
      const nw = Math.max(((y1 - y0) * el.w) / el.h, min);
      [x0, x1] = [cen.x - nw / 2, cen.x + nw / 2];
    }
  }
  const w = x1 - x0; const h = y1 - y0;
  const nc = rotatePoint({ x: (x0 + x1) / 2, y: (y0 + y1) / 2 }, cen, angle);
  return { x: nc.x - w / 2, y: nc.y - h / 2, w, h };
}

const FONT_RANGE = LIMITS.fontSize;

function pointsPatch(el, scaleX, scaleY, nx, ny, ox, oy) {
  // Scale relative points about the element's old bounds origin (ox, oy), placing the new bounds origin at (nx, ny).
  const pts = el.points.map(([px, py]) => [(el.x + px - ox) * scaleX + nx, (el.y + py - oy) * scaleY + ny]);
  const tmp = { x: 0, y: 0, points: pts, w: 0, h: 0 };
  normalizePoints(tmp);
  return { x: tmp.x, y: tmp.y, points: tmp.points, w: tmp.w, h: tmp.h };
}

export function resizeSelection(board, ids, handle, point, { keepAspect = false, fromCenter = false } = {}) {
  const els = ids.map((id) => findItem(board, id)).filter(Boolean);
  const out = new Map();
  if (!els.length) return out;
  const single = els.length === 1 ? els[0] : null;
  const allImages = els.every((e) => e.type === 'image');
  const aspect = keepAspect || (allImages && handle.length === 2);
  let oldBox; let angle = 0;
  if (single && !hasPoints(single)) { oldBox = { x: single.x, y: single.y, w: single.w, h: single.h }; angle = single.angle || 0; } else oldBox = selectionBounds(board, els);
  const nb = resizeBox({ ...oldBox, angle }, handle, point, { keepAspect: aspect, fromCenter });
  const sx = oldBox.w > 0 ? nb.w / oldBox.w : 1;
  const sy = oldBox.h > 0 ? nb.h / oldBox.h : 1;
  for (const el of els) {
    if (hasPoints(el)) {
      const b = pointsBounds(el.points);
      const ox = el.x + b.x; const oy = el.y + b.y;
      const nx = nb.x + (ox - oldBox.x) * sx; const ny = nb.y + (oy - oldBox.y) * sy;
      out.set(el.id, pointsPatch(el, sx, sy, nx, ny, ox, oy));
      continue;
    }
    if (single) {
      const patch = { x: nb.x, y: nb.y, w: nb.w, h: nb.h };
      if (el.type === 'text') scaleText(el, patch, (sx + sy) / 2, handle);
      out.set(el.id, patch);
      continue;
    }
    const c = center(el);
    const ncx = nb.x + (c.x - oldBox.x) * sx; const ncy = nb.y + (c.y - oldBox.y) * sy;
    const w = Math.max(el.w * sx, 1); const h = Math.max(el.h * sy, 1);
    const patch = { x: ncx - w / 2, y: ncy - h / 2, w, h };
    if (el.type === 'text') scaleText(el, patch, (sx + sy) / 2, 'nw');
    out.set(el.id, patch);
  }
  return out;
}

function scaleText(el, patch, k, handle) {
  const fontSize = clamp(Math.round(el.fontSize * k * 10) / 10, FONT_RANGE[0], FONT_RANGE[1]);
  const s = textSize(el.text, { fontFamily: el.fontFamily, fontSize });
  const w = Math.max(s.w, 4); const h = s.h;
  const right = patch.x + patch.w; const bottom = patch.y + patch.h;
  if (handle.includes('w')) patch.x = right - w;
  if (handle.includes('n')) patch.y = bottom - h;
  patch.w = w; patch.h = h; patch.fontSize = fontSize;
}

export function rotateAngle(c, point, { snap = false } = {}) {
  let a = Math.atan2(point.y - c.y, point.x - c.x) + Math.PI / 2;
  if (snap) { const step = Math.PI / 12; a = Math.round(a / step) * step; }
  const tau = Math.PI * 2;
  a = ((a % tau) + tau) % tau;
  return Math.abs(a - tau) < 1e-9 ? 0 : a;
}

export const snapToGrid = (v, grid) => (grid > 0 ? Math.round(v / grid) * grid : v);

// ---------- snapping ----------
export function snapMove(board, movingIds, dx, dy, { threshold = 6, others } = {}) {
  const moving = new Set(movingIds);
  const mEls = movingIds.map((id) => findItem(board, id)).filter(Boolean);
  const mb = selectionBounds(board, mEls);
  if (!mb) return { dx, dy, guides: [] };
  let targets;
  if (others) targets = others.map((o) => (typeof o === 'object' ? o : findItem(board, o))).filter(Boolean);
  else targets = board.items.filter((e) => !moving.has(e.id));
  const tb = targets.filter((e) => !moving.has(e.id)).map((e) => elementBounds(board, e));
  const lines = (b) => ({ x: [b.x, b.x + b.w / 2, b.x + b.w], y: [b.y, b.y + b.h / 2, b.y + b.h] });
  const pick = (axis, d) => {
    const mine = lines({ x: mb.x + (axis === 'x' ? d : dx), y: mb.y + (axis === 'y' ? d : dy), w: mb.w, h: mb.h })[axis];
    let best = null;
    for (const t of tb) for (const tv of lines(t)[axis]) for (const mv of mine) {
      const diff = tv - mv;
      if (Math.abs(diff) <= threshold && (best === null || Math.abs(diff) < Math.abs(best))) best = diff;
    }
    return best === null ? d : d + best;
  };
  const ndx = pick('x', dx);
  const ndy = pick('y', dy);
  const moved = { x: mb.x + ndx, y: mb.y + ndy, w: mb.w, h: mb.h };
  const guides = [];
  const addGuide = (axis, pos, a0, a1) => {
    const g = guides.find((q) => q.axis === axis && Math.abs(q.pos - pos) < 0.5);
    if (g) { g.from = Math.min(g.from, a0); g.to = Math.max(g.to, a1); } else guides.push({ axis, pos, from: a0, to: a1 });
  };
  const mine = lines(moved);
  for (const t of tb) {
    const tl = lines(t);
    for (const mv of mine.x) for (const tv of tl.x) if (Math.abs(mv - tv) < 0.5) addGuide('x', tv, Math.min(moved.y, t.y), Math.max(moved.y + moved.h, t.y + t.h));
    for (const mv of mine.y) for (const tv of tl.y) if (Math.abs(mv - tv) < 0.5) addGuide('y', tv, Math.min(moved.x, t.x), Math.max(moved.x + moved.w, t.x + t.w));
  }
  return { dx: ndx, dy: ndy, guides };
}

// ---------- arrow binding ----------
export function nearestBindTarget(board, point, tol = 8, { excludeId } = {}) {
  let best = null; let bestD = Infinity;
  for (let i = board.items.length - 1; i >= 0; i--) {
    const el = board.items[i];
    if (!BINDABLE_TYPES.includes(el.type) || el.id === excludeId) continue;
    const d = shapeDistanceLocal(el, toLocal(el, point));
    if (d <= tol && d < bestD) { best = el; bestD = d; if (d === 0) break; }
  }
  return best;
}

// ---------- freedraw simplification (Douglas-Peucker, iterative) ----------
export function simplifyStroke(points, tolerance = 1) {
  const n = points.length;
  if (n <= 2) return points.slice();
  const keep = new Uint8Array(n);
  keep[0] = 1; keep[n - 1] = 1;
  const stack = [[0, n - 1]];
  while (stack.length) {
    const [a, b] = stack.pop();
    let maxD = 0; let idx = -1;
    const A = { x: points[a][0], y: points[a][1] }; const B = { x: points[b][0], y: points[b][1] };
    for (let i = a + 1; i < b; i++) {
      const d = distToSegment({ x: points[i][0], y: points[i][1] }, A, B);
      if (d > maxD) { maxD = d; idx = i; }
    }
    if (idx !== -1 && maxD > tolerance) { keep[idx] = 1; stack.push([a, idx], [idx, b]); }
  }
  return points.filter((_, i) => keep[i]);
}

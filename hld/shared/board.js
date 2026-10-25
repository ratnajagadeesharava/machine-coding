// Board model shared by the browser whiteboard and the server (which re-validates boards and describes them for the AI).
// Pure functions, no DOM. See docs/WHITEBOARD_API.md for the element reference.
import {
  ARROW_TYPES, ARROWHEAD_IDS, BINDABLE_TYPES, CONTAINER_TYPES, DATA_URL_RE, DEFAULT_LABELS, DEFAULT_SIZES, DEFAULT_STYLE, ELEMENT_TYPES,
  FILL_STYLES, FONT_FAMILIES, LIMITS, LINEAR_TYPES, NO_ROTATE_TYPES, POINT_TYPES, ROUGHNESS, ROUNDNESS, STROKE_STYLES, STROKE_WIDTHS,
  TEXT_ALIGNS, VERTICAL_ALIGNS,
} from './wbconst.js';

export * from './wbconst.js';
export const BOARD_VERSION = 2;
export const SHAPE_TYPES = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text', 'image', 'frame']; // box-like elements

// ---------- small helpers ----------
export const isShape = (e) => SHAPE_TYPES.includes(e.type);
export const isLinear = (e) => LINEAR_TYPES.includes(e.type);
export const isArrow = (e) => e.type === 'arrow';
export const hasPoints = (e) => POINT_TYPES.includes(e.type);
export const isContainer = (e) => CONTAINER_TYPES.includes(e.type);
export const canRotate = (e) => !NO_ROTATE_TYPES.includes(e.type);

export function emptyBoard() {
  return { version: BOARD_VERSION, items: [], nextId: 1 };
}

export function newId(board) {
  return board.nextId++;
}

export function findItem(board, id) {
  return board.items.find((i) => i.id === id) || null;
}

export const cloneBoard = (board) => JSON.parse(JSON.stringify(board));
export const randomSeed = () => Math.floor(Math.random() * 2 ** 31);

export function center(el) {
  return { x: el.x + el.w / 2, y: el.y + el.h / 2 };
}

export function rotatePoint(p, c, angle) {
  if (!angle) return { x: p.x, y: p.y };
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const dx = p.x - c.x;
  const dy = p.y - c.y;
  return { x: c.x + dx * cos - dy * sin, y: c.y + dx * sin + dy * cos };
}

// ---------- element factory ----------
// Creates a complete element with defaults for its type. `props` override; the caller assigns `id` (use newId(board)).
export function createElement(type, props = {}) {
  if (!ELEMENT_TYPES.includes(type)) throw new Error(`unknown element type: ${type}`);
  const [dw, dh] = DEFAULT_SIZES[type] || [0, 0];
  const el = {
    id: props.id ?? 0, type, x: 0, y: 0, w: dw, h: dh, angle: 0,
    strokeColor: DEFAULT_STYLE.strokeColor, backgroundColor: DEFAULT_STYLE.backgroundColor, fillStyle: DEFAULT_STYLE.fillStyle,
    strokeWidth: DEFAULT_STYLE.strokeWidth, strokeStyle: DEFAULT_STYLE.strokeStyle, roughness: DEFAULT_STYLE.roughness,
    opacity: DEFAULT_STYLE.opacity, roundness: DEFAULT_STYLE.roundness, seed: randomSeed(),
    groupIds: [], frameId: null, locked: false, link: null,
  };
  if (CONTAINER_TYPES.includes(type) || type === 'text') {
    Object.assign(el, {
      text: DEFAULT_LABELS[type] ?? '', fontFamily: DEFAULT_STYLE.fontFamily, fontSize: DEFAULT_STYLE.fontSize,
      textAlign: type === 'text' ? 'left' : DEFAULT_STYLE.textAlign, verticalAlign: DEFAULT_STYLE.verticalAlign,
    });
  }
  if (type === 'sticky') Object.assign(el, { backgroundColor: '#ffec99', fillStyle: 'solid', roundness: 'sharp', text: '', textAlign: 'left', verticalAlign: 'top' });
  if (type === 'text') Object.assign(el, { text: '', backgroundColor: 'transparent' });
  if (type === 'frame') Object.assign(el, { name: 'Frame', backgroundColor: 'transparent', strokeWidth: 1, roughness: 0, roundness: 'sharp' });
  if (type === 'image') Object.assign(el, { dataUrl: '', backgroundColor: 'transparent' });
  if (POINT_TYPES.includes(type)) Object.assign(el, { points: [[0, 0], [0, 0]], w: 0, h: 0 });
  if (LINEAR_TYPES.includes(type)) {
    Object.assign(el, {
      arrowType: 'sharp', startArrowhead: null, endArrowhead: type === 'arrow' ? 'arrow' : null, from: null, to: null, text: '',
      fontFamily: DEFAULT_STYLE.fontFamily, fontSize: 16,
    });
  }
  if (type === 'freedraw') el.points = [[0, 0]];
  return { ...el, ...props };
}

// ---------- geometry ----------
export function pointsBounds(points) {
  if (!points.length) return { x: 0, y: 0, w: 0, h: 0 };
  let minX = Infinity; let minY = Infinity; let maxX = -Infinity; let maxY = -Infinity;
  for (const [x, y] of points) {
    minX = Math.min(minX, x); minY = Math.min(minY, y); maxX = Math.max(maxX, x); maxY = Math.max(maxY, y);
  }
  return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
}

// Point where the line from the shape's centre towards (tx, ty) leaves the shape (angle-aware). `gap` pushes the point outwards.
export function edgePoint(shape, tx, ty, gap = 0) {
  const c = center(shape);
  const target = rotatePoint({ x: tx, y: ty }, c, -(shape.angle || 0)); // into the shape's own frame
  const dx = target.x - c.x;
  const dy = target.y - c.y;
  if (dx === 0 && dy === 0) return c;
  const hw = Math.max(shape.w / 2, 1);
  const hh = Math.max(shape.h / 2, 1);
  let t;
  if (shape.type === 'ellipse') t = 1 / Math.sqrt((dx / hw) ** 2 + (dy / hh) ** 2);
  else if (shape.type === 'diamond') t = 1 / (Math.abs(dx) / hw + Math.abs(dy) / hh);
  else t = Math.min(hw / Math.abs(dx || 1e-9), hh / Math.abs(dy || 1e-9));
  const len = Math.hypot(dx, dy);
  const k = t + (gap ? gap / len : 0);
  return rotatePoint({ x: c.x + dx * k, y: c.y + dy * k }, c, shape.angle || 0);
}

// Midpoint of one side ('left' | 'right' | 'top' | 'bottom') of a (possibly rotated) shape.
export function sideAnchor(shape, side) {
  const c = center(shape);
  const p = { left: { x: shape.x, y: c.y }, right: { x: shape.x + shape.w, y: c.y }, top: { x: c.x, y: shape.y }, bottom: { x: c.x, y: shape.y + shape.h } }[side];
  return rotatePoint(p, c, shape.angle || 0);
}

function pickSide(from, to) {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  return Math.abs(dx) >= Math.abs(dy) ? (dx >= 0 ? 'right' : 'left') : (dy >= 0 ? 'bottom' : 'top');
}
const opposite = { left: 'right', right: 'left', top: 'bottom', bottom: 'top' };
const horizontal = (side) => side === 'left' || side === 'right';

// Orthogonal route between two points. Sides say which way the route leaves the start and enters the end.
export function elbowRoute(start, end, startSide, endSide) {
  const hs = horizontal(startSide);
  const he = horizontal(endSide);
  if (hs && he) { const mx = (start.x + end.x) / 2; return [start, { x: mx, y: start.y }, { x: mx, y: end.y }, end]; }
  if (!hs && !he) { const my = (start.y + end.y) / 2; return [start, { x: start.x, y: my }, { x: end.x, y: my }, end]; }
  return hs ? [start, { x: end.x, y: start.y }, end] : [start, { x: start.x, y: end.y }, end];
}

// Absolute points of a point-based element (line, arrow, freedraw). Bound arrow ends follow their shapes; elbow arrows are re-routed.
export function resolvePoints(board, el) {
  const base = (el.points || []).map(([px, py]) => ({ x: el.x + px, y: el.y + py }));
  if (el.type !== 'arrow' || base.length < 2) return base;
  const from = el.from != null ? findItem(board, el.from) : null;
  const to = el.to != null ? findItem(board, el.to) : null;
  const a = from && BINDABLE_TYPES.includes(from.type) ? from : null;
  const b = to && BINDABLE_TYPES.includes(to.type) ? to : null;
  if (el.arrowType === 'elbow') {
    const s0 = base[0];
    const e0 = base[base.length - 1];
    const ca = a ? center(a) : s0;
    const cb = b ? center(b) : e0;
    const startSide = a ? pickSide(ca, cb) : pickSide(s0, e0);
    const endSide = b ? opposite[pickSide(cb, ca)] : opposite[pickSide(e0, s0)];
    const s = a ? sideAnchor(a, startSide) : s0;
    const e = b ? sideAnchor(b, endSide) : e0;
    return elbowRoute(s, e, startSide, endSide);
  }
  if (a) { const next = base[1]; base[0] = edgePoint(a, next.x, next.y); }
  if (b) { const prev = base[base.length - 2]; base[base.length - 1] = edgePoint(b, prev.x, prev.y); }
  return base;
}

// Axis-aligned world bounds of any element (rotation and bound arrow ends included).
export function elementBounds(board, el) {
  if (hasPoints(el)) {
    const pts = resolvePoints(board, el);
    const b = pointsBounds(pts.map((p) => [p.x, p.y]));
    return b;
  }
  if (!el.angle) return { x: el.x, y: el.y, w: el.w, h: el.h };
  const c = center(el);
  const corners = [[el.x, el.y], [el.x + el.w, el.y], [el.x + el.w, el.y + el.h], [el.x, el.y + el.h]].map(([x, y]) => {
    const p = rotatePoint({ x, y }, c, el.angle);
    return [p.x, p.y];
  });
  return pointsBounds(corners);
}

export function selectionBounds(board, elements) {
  if (!elements.length) return null;
  let minX = Infinity; let minY = Infinity; let maxX = -Infinity; let maxY = -Infinity;
  for (const e of elements) {
    const b = elementBounds(board, e);
    minX = Math.min(minX, b.x); minY = Math.min(minY, b.y); maxX = Math.max(maxX, b.x + b.w); maxY = Math.max(maxY, b.y + b.h);
  }
  return { x: minX, y: minY, w: maxX - minX, h: maxY - minY };
}

// After editing `points`, re-anchor the element so points[0] is [0,0] and refresh w/h.
export function normalizePoints(el) {
  if (!el.points || !el.points.length) return el;
  const [ox, oy] = el.points[0];
  if (ox !== 0 || oy !== 0) {
    el.x += ox; el.y += oy;
    el.points = el.points.map(([x, y]) => [x - ox, y - oy]);
  }
  const b = pointsBounds(el.points);
  el.w = b.w; el.h = b.h;
  return el;
}

// ---------- cloning (paste, duplicate, library insert) ----------
// Returns deep copies with fresh ids (taken from board.nextId), offset by (dx, dy). Bindings and groups are remapped;
// an arrow bound to something outside the copied set keeps its current absolute end instead of a dangling binding.
export function cloneElements(board, elements, { dx = 0, dy = 0, source = board } = {}) {
  const idMap = new Map();
  const groupMap = new Map();
  for (const e of elements) idMap.set(e.id, newId(board));
  const out = [];
  for (const e of elements) {
    const c = JSON.parse(JSON.stringify(e));
    c.id = idMap.get(e.id);
    c.seed = randomSeed();
    c.groupIds = (e.groupIds || []).map((g) => { if (!groupMap.has(g)) groupMap.set(g, newId(board)); return groupMap.get(g); });
    c.frameId = e.frameId != null && idMap.has(e.frameId) ? idMap.get(e.frameId) : null;
    if (e.type === 'arrow') {
      const abs = resolvePoints(source, e);
      const keepFrom = e.from != null && idMap.has(e.from);
      const keepTo = e.to != null && idMap.has(e.to);
      c.from = keepFrom ? idMap.get(e.from) : null;
      c.to = keepTo ? idMap.get(e.to) : null;
      if (e.arrowType !== 'elbow' && (!keepFrom || !keepTo)) {
        c.x = abs[0].x; c.y = abs[0].y;
        c.points = abs.map((p) => [p.x - abs[0].x, p.y - abs[0].y]);
      }
    }
    c.x += dx; c.y += dy;
    out.push(c);
  }
  return out;
}

// ---------- stats and description for the AI ----------
const COMPONENT_TYPES = ['rect', 'diamond', 'ellipse', 'database'];
const KIND = {
  rect: 'box',
  diamond: 'diamond (often a load balancer, router or decision)',
  ellipse: 'oval (often a client or external actor)',
  database: 'database / datastore',
};

export function labelOf(el) {
  const t = (el.text || el.name || '').trim();
  return t || `unlabelled ${el.type}`;
}

function arrowLabel(el) {
  return (el.text || '').trim();
}

function endpoints(board, a) {
  const f = a.from != null ? findItem(board, a.from) : null;
  const t = a.to != null ? findItem(board, a.to) : null;
  return { f: f && BINDABLE_TYPES.includes(f.type) ? f : null, t: t && BINDABLE_TYPES.includes(t.type) ? t : null };
}

export function boardStats(board) {
  const counts = { components: 0, connections: 0, arrows: 0, lines: 0, notes: 0, frames: 0, images: 0, sketches: 0 };
  for (const e of board.items) {
    if (COMPONENT_TYPES.includes(e.type)) counts.components += 1;
    else if (e.type === 'sticky' || e.type === 'text') counts.notes += 1;
    else if (e.type === 'frame') counts.frames += 1;
    else if (e.type === 'image') counts.images += 1;
    else if (e.type === 'freedraw') counts.sketches += 1;
    else if (e.type === 'line') counts.lines += 1;
    else if (e.type === 'arrow') {
      counts.arrows += 1;
      const { f, t } = endpoints(board, e);
      if (f && t) counts.connections += 1;
    }
  }
  return counts;
}

// Plain-text description of the board, written for the AI interviewer (and handy in tests).
export function describeBoard(board) {
  if (!board.items.length) return 'The whiteboard is empty.';
  const stats = boardStats(board);
  const components = board.items.filter((e) => COMPONENT_TYPES.includes(e.type)).sort((a, b) => a.x - b.x || a.y - b.y);
  const notes = board.items.filter((e) => e.type === 'sticky' || e.type === 'text').sort((a, b) => a.y - b.y || a.x - b.x);
  const frames = board.items.filter((e) => e.type === 'frame');
  const arrows = board.items.filter((e) => e.type === 'arrow');
  const lines = [`${stats.components} component(s), ${stats.connections} connection(s), ${stats.notes} note(s).`];

  if (components.length) {
    lines.push('Components (left to right, with rough x,y position):');
    components.forEach((c, i) => {
      const inFrame = c.frameId != null ? findItem(board, c.frameId) : null;
      lines.push(`  ${i + 1}. "${labelOf(c)}" - ${KIND[c.type]} @ (${Math.round(c.x)}, ${Math.round(c.y)})${inFrame ? ` inside frame "${labelOf(inFrame)}"` : ''}`);
    });
  }
  if (frames.length) {
    lines.push('Frames (labelled regions):', ...frames.map((f) => {
      const inside = board.items.filter((e) => e.frameId === f.id && COMPONENT_TYPES.includes(e.type)).map((e) => `"${labelOf(e)}"`);
      return `  "${labelOf(f)}"${inside.length ? ` contains ${inside.join(', ')}` : ' (empty)'}`;
    }));
  }

  const connected = new Set();
  const flows = [];
  const loose = [];
  for (const a of arrows) {
    const { f, t } = endpoints(board, a);
    const label = arrowLabel(a);
    const suffix = label ? ` [label: "${label}"]` : '';
    const fromHead = !!a.startArrowhead;
    const toHead = !!a.endArrowhead;
    const link = fromHead && toHead ? '<->' : fromHead && !toHead ? '<-' : toHead ? '->' : '--';
    if (f && t) {
      flows.push(`  "${labelOf(f)}" ${link} "${labelOf(t)}"${suffix}`);
      connected.add(f.id); connected.add(t.id);
    } else {
      if (f) connected.add(f.id);
      if (t) connected.add(t.id);
      loose.push(`  ${f ? `"${labelOf(f)}"` : '(nothing)'} ${link} ${t ? `"${labelOf(t)}"` : '(nothing)'}${suffix}`);
    }
  }
  if (flows.length) lines.push('Connections (arrow direction):', ...flows);
  if (loose.length) lines.push('Arrows with a loose end (not attached to a component on both sides):', ...loose);
  const isolated = components.filter((c) => !connected.has(c.id));
  if (isolated.length && components.length > 1) lines.push('Components with no arrows attached: ' + isolated.map((c) => `"${labelOf(c)}"`).join(', '));
  const plainLines = stats.lines;
  if (plainLines) lines.push(`${plainLines} plain line(s) without arrow binding.`);
  if (notes.length) lines.push('Notes and free text:', ...notes.map((n) => `  - ${n.type === 'sticky' ? '[sticky] ' : ''}"${(n.text || '').trim() || '(empty)'}"`));
  const extras = [];
  if (stats.sketches) extras.push(`${stats.sketches} freehand sketch(es)`);
  if (stats.images) extras.push(`${stats.images} image(s)`);
  if (extras.length) lines.push(`Also on the board (contents not visible to you): ${extras.join(', ')}.`);
  return lines.join('\n');
}

// Stable fingerprint of what the board says (kinds, labels, connections) - positions are bucketed so a tiny drag is not a change.
export function boardSignature(board) {
  const parts = board.items.map((i) => {
    if (i.type === 'arrow') return `a:${i.from ?? ''}>${i.to ?? ''}:${i.startArrowhead || ''}:${i.endArrowhead || ''}:${(i.text || '').trim()}`;
    if (i.type === 'line') return `l:${Math.round(i.x / 40)}:${Math.round(i.y / 40)}`;
    if (i.type === 'freedraw') return `f:${Math.round(i.x / 40)}:${Math.round(i.y / 40)}:${(i.points || []).length >> 3}`;
    if (i.type === 'image') return `i:${Math.round(i.x / 40)}:${Math.round(i.y / 40)}`;
    return `s:${i.type}:${(i.text || i.name || '').trim()}:${Math.round(i.x / 40)}:${Math.round(i.y / 40)}`;
  });
  parts.sort();
  let h = 5381;
  const s = parts.join('|');
  for (let k = 0; k < s.length; k++) h = ((h << 5) + h + s.charCodeAt(k)) | 0;
  return `${board.items.length}:${(h >>> 0).toString(36)}`;
}

// A light copy for sending to the server/AI: drops image data and thins out freehand strokes.
export function toAiBoard(board) {
  return {
    version: BOARD_VERSION, nextId: board.nextId,
    items: board.items.map((e) => {
      if (e.type === 'image') return { ...e, dataUrl: '' };
      if (e.type === 'freedraw') return { ...e, points: e.points.length > 2 ? [e.points[0], e.points[e.points.length - 1]] : e.points };
      return e;
    }),
  };
}

// ---------- sanitising untrusted boards (localStorage, files, requests) ----------
const clamp = (v, lo, hi, d) => (Number.isFinite(v) ? Math.max(lo, Math.min(hi, v)) : d);
const str = (v, max) => (typeof v === 'string' ? v.slice(0, max) : '');
const oneOf = (v, list, d) => (list.includes(v) ? v : d);
const COLOR_RE = /^(transparent|#[0-9a-fA-F]{3,8}|rgba?\([0-9 ,.%]+\)|[a-zA-Z]{3,20})$/;
const color = (v, d) => (typeof v === 'string' && COLOR_RE.test(v) ? v : d);
const STROKE_WIDTH_VALUES = Object.values(STROKE_WIDTHS);
const ROUGHNESS_VALUES = Object.values(ROUGHNESS);

function sanitizeCommon(raw, id, type) {
  const d = DEFAULT_STYLE;
  return {
    id, type,
    x: clamp(raw.x, -LIMITS.coord, LIMITS.coord, 0), y: clamp(raw.y, -LIMITS.coord, LIMITS.coord, 0),
    w: clamp(raw.w ?? raw.width, 0, LIMITS.size, 0), h: clamp(raw.h ?? raw.height, 0, LIMITS.size, 0),
    angle: NO_ROTATE_TYPES.includes(type) ? 0 : clamp(raw.angle, -1000, 1000, 0),
    strokeColor: color(raw.strokeColor, d.strokeColor), backgroundColor: color(raw.backgroundColor, d.backgroundColor),
    fillStyle: oneOf(raw.fillStyle, FILL_STYLES, d.fillStyle), strokeWidth: STROKE_WIDTH_VALUES.includes(raw.strokeWidth) ? raw.strokeWidth : d.strokeWidth,
    strokeStyle: oneOf(raw.strokeStyle, STROKE_STYLES, d.strokeStyle), roughness: ROUGHNESS_VALUES.includes(raw.roughness) ? raw.roughness : d.roughness,
    opacity: clamp(raw.opacity, LIMITS.opacity[0], LIMITS.opacity[1], 100), roundness: oneOf(raw.roundness, ROUNDNESS, d.roundness),
    seed: Number.isInteger(raw.seed) ? raw.seed : (id * 2654435761) >>> 0,
    groupIds: (Array.isArray(raw.groupIds) ? raw.groupIds : []).filter(Number.isInteger).slice(0, LIMITS.groupDepth),
    frameId: Number.isInteger(raw.frameId) ? raw.frameId : null,
    locked: raw.locked === true,
    link: typeof raw.link === 'string' && /^https?:\/\//i.test(raw.link) ? raw.link.slice(0, LIMITS.link) : null,
  };
}

function sanitizeText(raw, el, max) {
  el.text = str(raw.text, max);
  el.fontFamily = FONT_FAMILIES[raw.fontFamily] ? raw.fontFamily : DEFAULT_STYLE.fontFamily;
  el.fontSize = clamp(raw.fontSize, LIMITS.fontSize[0], LIMITS.fontSize[1], DEFAULT_STYLE.fontSize);
  el.textAlign = oneOf(raw.textAlign, TEXT_ALIGNS, el.type === 'text' ? 'left' : DEFAULT_STYLE.textAlign);
  el.verticalAlign = oneOf(raw.verticalAlign, VERTICAL_ALIGNS, DEFAULT_STYLE.verticalAlign);
}

function sanitizePoints(raw, minCount) {
  const pts = (Array.isArray(raw.points) ? raw.points : []).slice(0, LIMITS.points)
    .filter((p) => Array.isArray(p) && Number.isFinite(p[0]) && Number.isFinite(p[1]))
    .map(([x, y]) => [clamp(x, -LIMITS.size, LIMITS.size, 0), clamp(y, -LIMITS.size, LIMITS.size, 0)]);
  return pts.length >= minCount ? pts : null;
}

// Accepts version 1 boards (rect/ellipse/database/text + straight arrows with x1,y1,x2,y2) and version 2 boards. Never throws.
export function sanitizeBoard(input) {
  const out = emptyBoard();
  if (!input || typeof input !== 'object' || !Array.isArray(input.items)) return out;
  const ids = new Set();
  for (const item of input.items.slice(0, LIMITS.items)) {
    if (!item || typeof item !== 'object' || !ELEMENT_TYPES.includes(item.type) || !Number.isInteger(item.id) || ids.has(item.id)) continue;
    let raw = item;
    const type = raw.type;
    const el = sanitizeCommon(raw, raw.id, type);
    if (POINT_TYPES.includes(type)) {
      let pts = sanitizePoints(raw, type === 'freedraw' ? 1 : 2);
      if (!pts && type === 'arrow' && Number.isFinite(raw.x1)) { // version 1 arrow
        el.x = clamp(raw.x1, -LIMITS.coord, LIMITS.coord, 0); el.y = clamp(raw.y1, -LIMITS.coord, LIMITS.coord, 0);
        pts = [[0, 0], [clamp(raw.x2, -LIMITS.coord, LIMITS.coord, 0) - el.x, clamp(raw.y2, -LIMITS.coord, LIMITS.coord, 0) - el.y]];
        raw = { ...raw, endArrowhead: 'arrow', startArrowhead: null };
      }
      if (!pts) continue;
      el.points = pts;
      normalizePoints(el);
      if (LINEAR_TYPES.includes(type)) {
        el.arrowType = oneOf(raw.arrowType, type === 'arrow' ? ARROW_TYPES : ['sharp', 'round'], 'sharp');
        el.startArrowhead = ARROWHEAD_IDS.includes(raw.startArrowhead) ? raw.startArrowhead : null;
        el.endArrowhead = ARROWHEAD_IDS.includes(raw.endArrowhead) ? raw.endArrowhead : (type === 'arrow' && raw.endArrowhead === undefined ? 'arrow' : null);
        el.from = type === 'arrow' && Number.isInteger(raw.from) ? raw.from : null;
        el.to = type === 'arrow' && Number.isInteger(raw.to) ? raw.to : null;
        sanitizeText(raw, el, LIMITS.label);
        el.fontSize = clamp(raw.fontSize, LIMITS.fontSize[0], LIMITS.fontSize[1], 16);
      }
    } else {
      el.w = Math.max(type === 'text' ? 4 : 12, el.w);
      el.h = Math.max(type === 'text' ? 4 : 12, el.h);
      if (CONTAINER_TYPES.includes(type)) sanitizeText(raw, el, LIMITS.label);
      if (type === 'text') sanitizeText(raw, el, LIMITS.text);
      if (type === 'frame') el.name = str(raw.name ?? raw.text, LIMITS.frameName);
      if (type === 'image') {
        el.dataUrl = typeof raw.dataUrl === 'string' && raw.dataUrl.length <= LIMITS.dataUrl && (DATA_URL_RE.test(raw.dataUrl) || raw.dataUrl === '') ? raw.dataUrl : '';
      }
    }
    ids.add(raw.id);
    out.items.push(el);
  }
  const byId = new Map(out.items.map((e) => [e.id, e]));
  for (const e of out.items) {
    if (e.frameId != null && byId.get(e.frameId)?.type !== 'frame') e.frameId = null;
    if (e.type === 'arrow') {
      if (e.from != null && !BINDABLE_TYPES.includes(byId.get(e.from)?.type)) e.from = null;
      if (e.to != null && !BINDABLE_TYPES.includes(byId.get(e.to)?.type)) e.to = null;
    }
  }
  const maxId = Math.max(0, ...out.items.map((e) => e.id), ...out.items.flatMap((e) => e.groupIds));
  out.nextId = Math.max(Number.isInteger(input.nextId) ? input.nextId : 1, maxId + 1, 1);
  return out;
}

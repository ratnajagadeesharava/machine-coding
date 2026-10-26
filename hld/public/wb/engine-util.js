// Pure helpers for the whiteboard engine (no DOM): clipboard payloads, drag boxes, angle/point constraints, per-type creation style.
// Kept DOM-free so they can be unit-tested under node --test.
import { DEFAULT_STYLE, normalizePoints } from '../../shared/board.js';

export const CLIP_MARKER = 'hld-whiteboard:';
export const HISTORY_LIMIT = 200;
export const HISTORY_BYTES = 120 * 1024 * 1024;

export const SVG_NS = 'http://www.w3.org/2000/svg';
export const svgEl = (tag, attrs = {}, parent) => {
  const n = document.createElementNS(SVG_NS, tag);
  for (const k of Object.keys(attrs)) if (attrs[k] !== null && attrs[k] !== undefined) n.setAttribute(k, attrs[k]);
  if (parent) parent.appendChild(n);
  return n;
};

export const fin = (v, d = 0) => (Number.isFinite(v) ? v : d);
export const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

// ---------- clipboard ----------
export function encodeClip(elements) {
  return CLIP_MARKER + JSON.stringify({ type: 'hld-whiteboard/clipboard', version: 2, elements });
}

// The system clipboard gets readable text (so pasting into chat, notes or a terminal shows words, not JSON) and the
// board data hidden in an HTML attribute that only this whiteboard looks for.
const utf8ToB64 = (str) => btoa(unescape(encodeURIComponent(str)));
const b64ToUtf8 = (b64) => decodeURIComponent(escape(atob(b64)));
export function clipPlain(elements) {
  const words = elements.map((e) => e.text || e.name || '').filter(Boolean).join('\n');
  return words || `Whiteboard selection (${elements.length} element${elements.length === 1 ? '' : 's'})`;
}
const escHtml = (t) => String(t).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
export function clipHtml(elements) {
  const payload = utf8ToB64(JSON.stringify({ type: 'hld-whiteboard/clipboard', version: 2, elements }));
  return `<meta charset="utf-8"><span data-hld-clip="${payload}">${escHtml(clipPlain(elements)).replace(/\n/g, '<br>')}</span>`;
}
export function clipFromHtml(html) {
  const m = typeof html === 'string' ? /data-hld-clip="([A-Za-z0-9+/=]+)"/.exec(html) : null;
  if (!m) return null;
  try {
    const o = JSON.parse(b64ToUtf8(m[1]));
    const els = Array.isArray(o?.elements) ? o.elements.filter((e) => e && typeof e === 'object' && typeof e.type === 'string') : [];
    return els.length ? els : null;
  } catch { return null; }
}

// -> { kind: 'hld', elements } | { kind: 'excalidraw', json } | { kind: 'text', text } | null
export function decodeClip(text) {
  if (typeof text !== 'string' || !text.trim()) return null;
  if (text.startsWith(CLIP_MARKER)) {
    try {
      const o = JSON.parse(text.slice(CLIP_MARKER.length));
      const els = Array.isArray(o?.elements) ? o.elements.filter((e) => e && typeof e === 'object' && typeof e.type === 'string') : [];
      if (els.length) return { kind: 'hld', elements: els };
    } catch { /* plain text that merely starts with the marker */ }
    return { kind: 'text', text };
  }
  const t = text.trim();
  if (t[0] === '{' && t.includes('excalidraw')) {
    try {
      const o = JSON.parse(t);
      if (o && o.type === 'excalidraw/clipboard' && Array.isArray(o.elements)) return { kind: 'excalidraw', json: o };
    } catch { /* not json */ }
  }
  return { kind: 'text', text };
}

// ---------- shallow element copy (strings are shared, arrays that ops mutate are copied) ----------
export function cloneEl(e) {
  const c = { ...e };
  if (e.points) c.points = e.points.map((p) => p.slice());
  if (e.groupIds) c.groupIds = e.groupIds.slice();
  return c;
}

// ---------- geometry of gestures ----------
// Box dragged from a to b. square: equal sides, centre: a is the centre.
export function dragBox(a, b, { square = false, centre = false } = {}) {
  let dx = b.x - a.x;
  let dy = b.y - a.y;
  if (square) {
    const m = Math.max(Math.abs(dx), Math.abs(dy));
    dx = Math.sign(dx || 1) * m;
    dy = Math.sign(dy || 1) * m;
  }
  if (centre) return { x: a.x - Math.abs(dx), y: a.y - Math.abs(dy), w: Math.abs(dx) * 2, h: Math.abs(dy) * 2 };
  return { x: Math.min(a.x, a.x + dx), y: Math.min(a.y, a.y + dy), w: Math.abs(dx), h: Math.abs(dy) };
}

// Constrain b relative to a to the nearest multiple of 45 degrees (keeps length).
export function snapAngle45(a, b) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len = Math.hypot(dx, dy);
  if (!len) return { x: b.x, y: b.y };
  const step = Math.PI / 4;
  const ang = Math.round(Math.atan2(dy, dx) / step) * step;
  return { x: a.x + Math.cos(ang) * len, y: a.y + Math.sin(ang) * len };
}

export const snapRotation = (angle, step = Math.PI / 12) => Math.round(angle / step) * step;
export const normAngle = (a) => { const t = Math.PI * 2; return ((a % t) + t) % t; };

// Absolute points -> element x/y/points (points[0] = [0,0]); refreshes w/h.
export function setAbsPoints(el, abs) {
  const first = abs[0];
  el.x = first.x;
  el.y = first.y;
  el.points = abs.map((p) => [p.x - first.x, p.y - first.y]);
  normalizePoints(el);
  return el;
}

export function distToSegment(p, a, b) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const l2 = dx * dx + dy * dy;
  let t = l2 ? ((p.x - a.x) * dx + (p.y - a.y) * dy) / l2 : 0;
  t = clamp(t, 0, 1);
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

// Largest size with the same aspect that fits in maxW x maxH (never upscales).
export function fitSize(w, h, maxW, maxH) {
  const k = Math.min(1, maxW / Math.max(w, 1), maxH / Math.max(h, 1));
  return { w: Math.max(1, Math.round(w * k)), h: Math.max(1, Math.round(h * k)) };
}

// ---------- creation style ----------
const COMMON = ['strokeColor', 'backgroundColor', 'fillStyle', 'strokeWidth', 'strokeStyle', 'roughness', 'opacity'];
const FONT = ['fontFamily', 'fontSize', 'textAlign', 'verticalAlign'];
const pick = (src, keys) => Object.fromEntries(keys.filter((k) => src[k] !== undefined).map((k) => [k, src[k]]));

// Props for createElement(type, ...) from the current item style.
export function creationProps(type, style, { textAlignSet = false } = {}) {
  const s = { ...DEFAULT_STYLE, ...style };
  switch (type) {
    case 'rect': case 'diamond': case 'ellipse': case 'database':
      return { ...pick(s, COMMON), ...pick(s, FONT), roundness: s.roundness, text: '' };
    case 'sticky': {
      const p = { ...pick(s, ['strokeColor', 'strokeWidth', 'strokeStyle', 'roughness', 'opacity']), ...pick(s, ['fontFamily', 'fontSize']), text: '' };
      if (s.backgroundColor && s.backgroundColor !== 'transparent') p.backgroundColor = s.backgroundColor;
      return p;
    }
    case 'text': {
      const p = { ...pick(s, ['strokeColor', 'opacity', 'fontFamily', 'fontSize']) };
      if (textAlignSet) p.textAlign = s.textAlign;
      return p;
    }
    case 'frame': return {};
    case 'image': return { opacity: s.opacity };
    case 'freedraw': return pick(s, ['strokeColor', 'backgroundColor', 'strokeWidth', 'strokeStyle', 'opacity', 'fillStyle']);
    case 'line': return { ...pick(s, COMMON), arrowType: s.arrowType === 'round' ? 'round' : 'sharp', startArrowhead: null, endArrowhead: null, roundness: s.roundness };
    case 'arrow': return { ...pick(s, COMMON), arrowType: s.arrowType, startArrowhead: s.startArrowhead, endArrowhead: s.endArrowhead, roundness: s.roundness };
    default: return {};
  }
}

// Allowed keys of the "current item style".
export const STYLE_KEYS = Object.keys(DEFAULT_STYLE);

// Import / export: our .hld.json scene files, Excalidraw scenes (.excalidraw) and libraries (.hld lib json, .excalidrawlib), plus browser file helpers.
// Everything that parses is untrusted input: results always go through sanitizeBoard and nothing is ever evaluated.
// The data functions are DOM-free (Node-testable); the file helpers feature-detect `document`.
import {
  ARROWHEAD_IDS, BINDABLE_TYPES, CONTAINER_TYPES, DATA_URL_RE, LIMITS, cloneElements, createElement, isLinear, pointsBounds,
  resolvePoints, sanitizeBoard, selectionBounds,
} from '../../shared/board.js';

export const SCENE_TYPE = 'hld-whiteboard';
export const LIBRARY_TYPE = 'hld-library';
export const SCENE_SOURCE = 'hld-room';
export const MAX_FILE_BYTES = 25 * 1024 * 1024;
export const MAX_LIBRARY_ITEMS = 500;

const isObj = (v) => v && typeof v === 'object' && !Array.isArray(v);
const num = (v, d = 0) => (Number.isFinite(v) ? v : d);
const COLOR_RE = /^(transparent|#[0-9a-fA-F]{3,8}|rgba?\([0-9 ,.%]+\)|[a-zA-Z]{3,20})$/;
const cssColor = (v) => (typeof v === 'string' && COLOR_RE.test(v) ? v : null);

// ---------------------------------------------------------------- scene files (.hld.json)
function cleanAppState(a) {
  const out = {};
  if (!isObj(a)) return out;
  if (a.theme === 'light' || a.theme === 'dark') out.theme = a.theme;
  const bg = a.background ?? a.viewBackgroundColor;
  if (cssColor(bg)) out.background = bg;
  if (typeof a.grid === 'boolean') out.grid = a.grid;
  else if (typeof a.gridModeEnabled === 'boolean') out.grid = a.gridModeEnabled;
  else if (Number.isFinite(a.gridSize) && a.gridSize > 0) out.grid = true;
  return out;
}

export function serializeScene(board, { appState } = {}) {
  const b = sanitizeBoard(board);
  return JSON.stringify({
    type: SCENE_TYPE, version: 2, source: SCENE_SOURCE, exportedAt: new Date().toISOString(),
    items: b.items, nextId: b.nextId, appState: cleanAppState(appState),
  }, null, 2);
}

const looksLikeExcalidrawElement = (e) => isObj(e) && typeof e.id === 'string' && typeof e.type === 'string';

export function parseScene(text) {
  if (typeof text !== 'string') throw new Error('Expected the text of a whiteboard file.');
  if (text.length > MAX_FILE_BYTES) throw new Error('This file is too large (limit 25 MB).');
  let data;
  try { data = JSON.parse(text.replace(/^﻿/, '')); } catch { throw new Error('This file is not valid JSON.'); }
  if (Array.isArray(data)) {
    if (data.length === 0 || data.every(looksLikeExcalidrawElement)) return { board: importExcalidraw(data), appState: {} };
    throw new Error('This JSON array does not look like whiteboard or Excalidraw elements.');
  }
  if (!isObj(data)) throw new Error('This file is not a whiteboard scene.');
  if (data.type === 'excalidraw-library' || data.type === LIBRARY_TYPE) throw new Error('This is a library file, not a scene. Import it from the library panel.');
  if (data.type === 'excalidraw' || data.type === 'excalidraw/clipboard' || (Array.isArray(data.elements) && !Array.isArray(data.items))) {
    if (!Array.isArray(data.elements)) throw new Error('This Excalidraw file has no elements.');
    return { board: importExcalidraw(data), appState: cleanAppState(data.appState) };
  }
  if (Array.isArray(data.items)) {
    if (data.type !== undefined && data.type !== SCENE_TYPE) throw new Error(`Unknown file type "${String(data.type).slice(0, 40)}".`);
    return { board: sanitizeBoard({ items: data.items, nextId: data.nextId }), appState: cleanAppState(data.appState) };
  }
  throw new Error('Unrecognised file: expected a .hld.json or .excalidraw scene.');
}

// ---------------------------------------------------------------- Excalidraw mappings
const EX_TYPE = {
  rectangle: 'rect', diamond: 'diamond', ellipse: 'ellipse', arrow: 'arrow', line: 'line', freedraw: 'freedraw', text: 'text', image: 'image',
  frame: 'frame', magicframe: 'frame', stickynote: 'sticky',
};
const EX_ARROWHEAD = {
  arrow: 'arrow', bar: 'bar', dot: 'circle', circle: 'circle', circle_outline: 'circle_outline', triangle: 'triangle',
  triangle_outline: 'triangle_outline', diamond: 'diamond', diamond_outline: 'diamond_outline', crowfoot_one: 'crowfoot_one',
  crowfoot_many: 'crowfoot_many', crowfoot_one_or_many: 'crowfoot_one_or_many',
  cardinality_one: 'crowfoot_one', cardinality_exactly_one: 'crowfoot_one', cardinality_zero_or_one: 'crowfoot_one',
  cardinality_many: 'crowfoot_many', cardinality_zero_or_many: 'crowfoot_many', cardinality_one_or_many: 'crowfoot_one_or_many',
};
const EX_CONTAINER_TARGETS = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'arrow'];
const FONT_IN = { 1: 1, 5: 1, 2: 2, 6: 2, 3: 3, 8: 3 }; // Virgil/Excalifont, Helvetica/Nunito, Cascadia/Comic Shanns; others -> hand-drawn
const FONT_OUT = { 1: 5, 2: 6, 3: 3 };
const snapWidth = (w) => (!Number.isFinite(w) ? 2 : w <= 1.5 ? 1 : w <= 3 ? 2 : 4);
const snapRough = (r) => (!Number.isFinite(r) ? 1 : Math.max(0, Math.min(2, Math.round(r))));
const LINE_HEIGHT = 1.25;
const GLYPH = { 1: 0.55, 2: 0.55, 3: 0.6 };

// ---------------------------------------------------------------- Excalidraw import
// Returns { board, skipped } where `skipped` counts live elements of unsupported types (embeddables, iframes...).
export function importExcalidrawDetailed(json) {
  let elements = [];
  let files = {};
  if (Array.isArray(json)) elements = json;
  else if (isObj(json)) { elements = Array.isArray(json.elements) ? json.elements : []; files = isObj(json.files) ? json.files : {}; }
  elements = elements.slice(0, LIMITS.items * 3).filter((e) => isObj(e) && !e.isDeleted);

  let skipped = 0;
  const known = new Map(); // excalidraw id -> element
  const order = [];
  for (const e of elements) {
    if (!EX_TYPE[e.type]) { if (e.type !== 'selection') skipped += 1; continue; }
    order.push(e);
    if (typeof e.id === 'string') known.set(e.id, e);
  }
  const ourType = (e) => {
    const t = EX_TYPE[e.type];
    const hint = e.customData?.hld?.type;
    if (t === 'rect' && (hint === 'sticky' || hint === 'database')) return hint;
    return t;
  };

  // text elements bound to a container become that container's label
  const labelFor = new Map(); // container id -> text element
  const labelIds = new Set();
  for (const e of order) {
    if (e.type !== 'text' || typeof e.containerId !== 'string') continue;
    const c = known.get(e.containerId);
    if (c && EX_CONTAINER_TARGETS.includes(ourType(c)) && !labelFor.has(e.containerId)) { labelFor.set(e.containerId, e); labelIds.add(e); }
    else if (!c && !elements.some((x) => x.id === e.containerId)) { /* container gone: keep as free text */ } else if (!c) { labelIds.add(e); } // container is unsupported (skipped) -> drop
  }

  let next = 1;
  const idMap = new Map();
  const groupMap = new Map();
  for (const e of order) if (!labelIds.has(e) && typeof e.id === 'string') idMap.set(e.id, next++);
  const groupId = (g) => { if (!groupMap.has(g)) groupMap.set(g, next++); return groupMap.get(g); };

  const items = [];
  for (const e of order) {
    if (labelIds.has(e) || !idMap.has(e.id)) continue;
    const type = ourType(e);
    const id = idMap.get(e.id);
    const w = Math.abs(num(e.width));
    const h = Math.abs(num(e.height));
    const props = {
      id, x: num(e.x), y: num(e.y), w, h, angle: num(e.angle),
      strokeColor: cssColor(e.strokeColor) || '#1e1e1e', backgroundColor: cssColor(e.backgroundColor) || 'transparent',
      fillStyle: ['hachure', 'cross-hatch', 'solid', 'zigzag'].includes(e.fillStyle) ? e.fillStyle : 'solid',
      strokeWidth: snapWidth(e.strokeWidth), strokeStyle: ['solid', 'dashed', 'dotted'].includes(e.strokeStyle) ? e.strokeStyle : 'solid',
      roughness: snapRough(e.roughness), opacity: Math.max(0, Math.min(100, num(e.opacity, 100))),
      roundness: e.roundness == null ? 'sharp' : 'round',
      groupIds: (Array.isArray(e.groupIds) ? e.groupIds : []).filter((g) => typeof g === 'string').slice(0, LIMITS.groupDepth).map(groupId),
      locked: e.locked === true, link: typeof e.link === 'string' ? e.link : null,
      frameId: null,
    };
    if (Number.isInteger(e.seed)) props.seed = e.seed;
    if (['ellipse', 'freedraw', 'text', 'image'].includes(type)) props.roundness = 'round';
    const label = labelFor.get(e.id);

    if (type === 'text') {
      Object.assign(props, {
        text: String(e.text ?? '').slice(0, LIMITS.text), fontFamily: FONT_IN[e.fontFamily] || 1, fontSize: num(e.fontSize, 20),
        textAlign: ['left', 'center', 'right'].includes(e.textAlign) ? e.textAlign : 'left',
        verticalAlign: ['top', 'middle', 'bottom'].includes(e.verticalAlign) ? e.verticalAlign : 'top', backgroundColor: 'transparent',
      });
    } else if (type === 'image') {
      const f = typeof e.fileId === 'string' ? files[e.fileId] : null;
      const url = isObj(f) ? f.dataURL : '';
      props.dataUrl = typeof url === 'string' && url.length <= LIMITS.dataUrl && DATA_URL_RE.test(url) ? url : '';
      props.backgroundColor = 'transparent';
    } else if (type === 'frame') {
      props.name = String(e.name ?? '').slice(0, LIMITS.frameName);
      props.roundness = 'sharp';
    } else if (type === 'freedraw' || type === 'line' || type === 'arrow') {
      const pts = (Array.isArray(e.points) ? e.points : []).filter((p) => Array.isArray(p) && Number.isFinite(p[0]) && Number.isFinite(p[1]))
        .slice(0, LIMITS.points).map((p) => [p[0], p[1]]);
      if (pts.length < (type === 'freedraw' ? 1 : 2)) { skipped += 1; continue; }
      props.points = pts;
      props.angle = 0;
      if (type !== 'freedraw') {
        props.arrowType = type === 'arrow' && e.elbowed ? 'elbow' : e.roundness != null ? 'round' : 'sharp';
        props.startArrowhead = EX_ARROWHEAD[e.startArrowhead] || null;
        props.endArrowhead = EX_ARROWHEAD[e.endArrowhead] || null;
        if (type === 'line') { props.startArrowhead = null; props.endArrowhead = null; }
        if (type === 'arrow') {
          props.from = idMap.get(e.startBinding?.elementId) ?? null;
          props.to = idMap.get(e.endBinding?.elementId) ?? null;
        }
        props.roundness = 'round';
      } else props.backgroundColor = 'transparent';
    }
    if (label) {
      props.text = String(label.originalText ?? label.text ?? '').slice(0, LIMITS.label);
      props.fontFamily = FONT_IN[label.fontFamily] || 1;
      props.fontSize = num(label.fontSize, type === 'arrow' ? 16 : 20);
      props.textAlign = ['left', 'center', 'right'].includes(label.textAlign) ? label.textAlign : 'center';
      props.verticalAlign = ['top', 'middle', 'bottom'].includes(label.verticalAlign) ? label.verticalAlign : 'middle';
    } else if (type === 'sticky' || CONTAINER_TYPES.includes(type)) {
      props.text = '';
    }
    if (type === 'sticky') { props.fillStyle = 'solid'; if (props.backgroundColor === 'transparent') props.backgroundColor = '#ffec99'; }
    const el = createElement(type, props);
    items.push({ el, ex: e });
  }
  for (const { el, ex } of items) {
    if (typeof ex.frameId === 'string' && idMap.has(ex.frameId) && el.type !== 'frame') el.frameId = idMap.get(ex.frameId);
  }
  const board = sanitizeBoard({ items: items.map((i) => i.el), nextId: next });
  return { board, skipped };
}

export function importExcalidraw(json) {
  return importExcalidrawDetailed(json).board;
}

// ---------------------------------------------------------------- Excalidraw export
const B62 = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
// Fractional index keys in ascending order: a0..az, b00..bzz, c000... (the integer-part format of the fractional-indexing scheme)
function fractionalKey(n) {
  if (n < 62) return 'a' + B62[n];
  n -= 62;
  if (n < 62 * 62) return 'b' + B62[Math.floor(n / 62)] + B62[n % 62];
  n -= 62 * 62;
  return 'c' + B62[Math.floor(n / 3844) % 62] + B62[Math.floor(n / 62) % 62] + B62[n % 62];
}

function wrapLines(text, fontFamily, fontSize, maxW) {
  const k = (GLYPH[fontFamily] || 0.55) * fontSize;
  const out = [];
  for (const para of String(text).split('\n')) {
    if (!maxW || maxW < k * 2) { out.push(para); continue; }
    let line = '';
    for (const word of para.split(/(?<= )/)) {
      if ((line + word).trimEnd().length * k <= maxW) { line += word; continue; }
      if (line) out.push(line.trimEnd());
      line = word;
      while (line.length * k > maxW && line.length > 1) { const n = Math.max(1, Math.floor(maxW / k)); out.push(line.slice(0, n)); line = line.slice(n); }
    }
    out.push(line.trimEnd());
  }
  return out;
}
function measure(lines, fontFamily, fontSize) {
  const k = (GLYPH[fontFamily] || 0.55) * fontSize;
  return { w: Math.max(1, Math.max(...lines.map((l) => l.length)) * k), h: lines.length * fontSize * LINE_HEIGHT };
}

const rand = () => Math.floor(Math.random() * 2 ** 31);
const ROUND_ADAPTIVE = 3;
const ROUND_PROPORTIONAL = 2;

function fixedPointFor(shape, p) {
  const w = Math.max(shape.w, 1);
  const h = Math.max(shape.h, 1);
  const cx = shape.x + shape.w / 2;
  const cy = shape.y + shape.h / 2;
  const cos = Math.cos(-(shape.angle || 0));
  const sin = Math.sin(-(shape.angle || 0));
  const dx = p.x - cx;
  const dy = p.y - cy;
  const lx = cx + dx * cos - dy * sin;
  const ly = cy + dx * sin + dy * cos;
  const r = (v) => Math.round(Math.max(0, Math.min(1, v)) * 1000) / 1000;
  return [r((lx - shape.x) / w), r((ly - shape.y) / h)];
}

export function exportExcalidraw(boardIn, { appState } = {}) {
  const board = sanitizeBoard(boardIn);
  const byId = new Map(board.items.map((e) => [e.id, e]));
  const now = Date.now();
  const gid = (g) => `g${g}`;
  const eid = (id) => `e${id}`;
  const out = [];
  const exById = new Map();
  const files = {};
  const fileIds = new Map();
  const bound = new Map(); // our shape id -> boundElements[]
  const addBound = (id, entry) => { if (!bound.has(id)) bound.set(id, []); bound.get(id).push(entry); };

  const baseOf = (el, id, type, extra = {}) => ({
    id, type, x: el.x, y: el.y, width: el.w, height: el.h, angle: el.angle || 0,
    strokeColor: el.strokeColor, backgroundColor: el.backgroundColor, fillStyle: el.fillStyle, strokeWidth: el.strokeWidth,
    strokeStyle: el.strokeStyle, roughness: el.roughness, opacity: el.opacity,
    groupIds: (el.groupIds || []).map(gid), frameId: el.frameId != null && byId.get(el.frameId)?.type === 'frame' ? eid(el.frameId) : null,
    index: null, roundness: null, seed: el.seed, version: 1, versionNonce: rand(), isDeleted: false, boundElements: null,
    updated: now, link: el.link || null, locked: !!el.locked, ...extra,
  });
  const textElement = (id, cx, cy, text, o) => {
    const lines = o.lines || String(text).split('\n');
    const m = measure(lines, o.fontFamily, o.fontSize);
    const w = o.w ?? m.w;
    const h = o.h ?? m.h;
    return {
      id, type: 'text', x: cx - w / 2, y: cy - h / 2, width: w, height: h, angle: o.angle || 0,
      strokeColor: o.strokeColor, backgroundColor: 'transparent', fillStyle: 'solid', strokeWidth: 2, strokeStyle: 'solid', roughness: o.roughness ?? 1,
      opacity: o.opacity ?? 100, groupIds: o.groupIds || [], frameId: o.frameId ?? null, index: null, roundness: null, seed: o.seed ?? rand(),
      version: 1, versionNonce: rand(), isDeleted: false, boundElements: null, updated: now, link: null, locked: !!o.locked,
      text: lines.join('\n'), fontSize: o.fontSize, fontFamily: FONT_OUT[o.fontFamily] || 5, textAlign: o.textAlign, verticalAlign: o.verticalAlign,
      containerId: o.containerId ?? null, originalText: String(text), autoResize: true, lineHeight: LINE_HEIGHT,
      baseline: Math.round(o.fontSize * 0.9),
    };
  };
  const rotateAbout = (px, py, cx, cy, a) => {
    if (!a) return [px, py];
    const c = Math.cos(a); const s = Math.sin(a);
    return [cx + (px - cx) * c - (py - cy) * s, cy + (px - cx) * s + (py - cy) * c];
  };

  // arrows first need to know which shapes they are bound to, so collect bindings up front
  for (const el of board.items) {
    if (el.type !== 'arrow') continue;
    if (el.from != null && byId.has(el.from)) addBound(el.from, { type: 'arrow', id: eid(el.id) });
    if (el.to != null && byId.has(el.to) && el.to !== el.from) addBound(el.to, { type: 'arrow', id: eid(el.id) });
    else if (el.to != null && el.to === el.from) { /* self loop: one entry is enough */ }
  }

  for (const el of board.items) {
    const id = eid(el.id);
    const label = (el.text || '').trim() ? el.text : '';
    const labelId = `${id}-t`;
    let ex;
    let labelEl = null;
    if (el.type === 'rect' || el.type === 'diamond' || el.type === 'ellipse' || el.type === 'database' || el.type === 'sticky') {
      const type = el.type === 'diamond' ? 'diamond' : el.type === 'ellipse' ? 'ellipse' : 'rectangle';
      const round = el.roundness === 'round' && el.type !== 'ellipse' && el.type !== 'sticky';
      ex = baseOf(el, id, type, {
        roundness: round ? { type: type === 'rectangle' ? ROUND_ADAPTIVE : ROUND_PROPORTIONAL } : null,
        ...(el.type === 'database' || el.type === 'sticky' ? { customData: { hld: { type: el.type } } } : {}),
      });
      if (label) {
        const pad = 5;
        const maxW = el.type === 'diamond' ? el.w / 2 : el.type === 'ellipse' ? el.w * 0.7 : Math.max(10, el.w - pad * 2);
        const lines = wrapLines(label, el.fontFamily, el.fontSize, maxW);
        const m = measure(lines, el.fontFamily, el.fontSize);
        const cxs = el.x + el.w / 2; const cys = el.y + el.h / 2;
        const ax = el.textAlign === 'left' ? el.x + pad + m.w / 2 : el.textAlign === 'right' ? el.x + el.w - pad - m.w / 2 : cxs;
        const ay = el.verticalAlign === 'top' ? el.y + pad + m.h / 2 : el.verticalAlign === 'bottom' ? el.y + el.h - pad - m.h / 2 : cys;
        const [tx, ty] = rotateAbout(ax, ay, cxs, cys, el.angle);
        labelEl = textElement(labelId, tx, ty, label, {
          lines, fontFamily: el.fontFamily, fontSize: el.fontSize, textAlign: el.textAlign, verticalAlign: el.verticalAlign, strokeColor: el.strokeColor,
          opacity: el.opacity, angle: el.angle, groupIds: ex.groupIds, frameId: ex.frameId, containerId: id, seed: el.seed, locked: el.locked,
        });
        addBound(el.id, { type: 'text', id: labelId });
      }
    } else if (el.type === 'text') {
      const lines = String(el.text).split('\n');
      const m = measure(lines, el.fontFamily, el.fontSize);
      ex = textElement(id, el.x + (el.w || m.w) / 2, el.y + (el.h || m.h) / 2, el.text, {
        lines, w: el.w || m.w, h: el.h || m.h, fontFamily: el.fontFamily, fontSize: el.fontSize, textAlign: el.textAlign, verticalAlign: el.verticalAlign,
        strokeColor: el.strokeColor, opacity: el.opacity, angle: el.angle, groupIds: (el.groupIds || []).map(gid),
        frameId: el.frameId != null && byId.get(el.frameId)?.type === 'frame' ? eid(el.frameId) : null, seed: el.seed, locked: el.locked,
      });
      ex.link = el.link || null;
      ex.roughness = el.roughness;
    } else if (el.type === 'frame') {
      ex = baseOf(el, id, 'frame', { name: el.name || null });
    } else if (el.type === 'image') {
      if (!el.dataUrl) continue;
      if (!fileIds.has(el.dataUrl)) {
        const fid = `f${fileIds.size + 1}${(el.seed >>> 0).toString(36)}`;
        fileIds.set(el.dataUrl, fid);
        const mime = /^data:([^;]+);/.exec(el.dataUrl)?.[1] || 'image/png';
        files[fid] = { mimeType: mime, id: fid, dataURL: el.dataUrl, created: now, lastRetrieved: now };
      }
      ex = baseOf(el, id, 'image', { status: 'saved', fileId: fileIds.get(el.dataUrl), scale: [1, 1], crop: null });
    } else if (el.type === 'freedraw') {
      ex = baseOf(el, id, 'freedraw', {
        points: el.points.map((p) => [p[0], p[1]]), pressures: [], simulatePressure: true, lastCommittedPoint: null, backgroundColor: 'transparent',
      });
      ex.angle = 0;
    } else if (isLinear(el)) {
      const abs = resolvePoints(board, el);
      const b = pointsBounds(abs.map((p) => [p.x, p.y]));
      const head = (h) => (ARROWHEAD_IDS.includes(h) ? h : null);
      const binding = (shapeId, p) => {
        const s = shapeId != null ? byId.get(shapeId) : null;
        return s && BINDABLE_TYPES.includes(s.type) ? { elementId: eid(s.id), fixedPoint: fixedPointFor(s, p), mode: 'orbit' } : null;
      };
      ex = baseOf(el, id, el.type, {
        x: abs[0].x, y: abs[0].y, width: b.w, height: b.h, angle: 0, backgroundColor: 'transparent', fillStyle: 'solid',
        points: abs.map((p) => [p.x - abs[0].x, p.y - abs[0].y]), lastCommittedPoint: null,
        startBinding: el.type === 'arrow' ? binding(el.from, abs[0]) : null, endBinding: el.type === 'arrow' ? binding(el.to, abs[abs.length - 1]) : null,
        startArrowhead: head(el.startArrowhead), endArrowhead: head(el.endArrowhead),
        roundness: el.arrowType === 'round' ? { type: ROUND_PROPORTIONAL } : null,
      });
      if (el.type === 'arrow') {
        ex.elbowed = el.arrowType === 'elbow';
        if (ex.elbowed) Object.assign(ex, { fixedSegments: null, startIsSpecial: null, endIsSpecial: null });
      } else ex.polygon = false;
      if (label && el.type === 'arrow') {
        // label sits at the midpoint of the path's length
        let total = 0;
        for (let i = 1; i < abs.length; i++) total += Math.hypot(abs[i].x - abs[i - 1].x, abs[i].y - abs[i - 1].y);
        let acc = 0; let mx = abs[0].x; let my = abs[0].y;
        for (let i = 1; i < abs.length; i++) {
          const seg = Math.hypot(abs[i].x - abs[i - 1].x, abs[i].y - abs[i - 1].y);
          if (acc + seg >= total / 2 && seg > 0) { const t = (total / 2 - acc) / seg; mx = abs[i - 1].x + (abs[i].x - abs[i - 1].x) * t; my = abs[i - 1].y + (abs[i].y - abs[i - 1].y) * t; break; }
          acc += seg;
        }
        labelEl = textElement(labelId, mx, my, label, {
          fontFamily: el.fontFamily, fontSize: el.fontSize, textAlign: 'center', verticalAlign: 'middle', strokeColor: el.strokeColor, opacity: el.opacity,
          groupIds: ex.groupIds, frameId: ex.frameId, containerId: id, seed: el.seed, locked: el.locked,
        });
        addBound(el.id, { type: 'text', id: labelId });
      }
    } else continue;
    exById.set(el.id, ex);
    out.push(ex);
    if (labelEl) out.push(labelEl);
  }
  // write boundElements now that all links are known
  for (const [id, list] of bound) {
    const ex = exById.get(id);
    if (ex) ex.boundElements = list;
  }
  // an arrow whose bound shape was not exported (e.g. an empty image) loses that binding
  const exportedIds = new Set(out.map((e) => e.id));
  for (const e of out) {
    if (e.type !== 'arrow') continue;
    if (e.startBinding && !exportedIds.has(e.startBinding.elementId)) e.startBinding = null;
    if (e.endBinding && !exportedIds.has(e.endBinding.elementId)) e.endBinding = null;
  }
  out.forEach((e, i) => { e.index = fractionalKey(i); });
  const a = cleanAppState(appState);
  return {
    type: 'excalidraw', version: 2, source: SCENE_SOURCE, elements: out,
    appState: { gridSize: a.grid ? 20 : null, gridModeEnabled: !!a.grid, viewBackgroundColor: a.background || '#ffffff', ...(a.theme ? { theme: a.theme } : {}) },
    files,
  };
}

// ---------------------------------------------------------------- libraries
function normalizeItemElements(elements) {
  const b = sanitizeBoard({ items: elements, nextId: 1 });
  const bounds = selectionBounds(b, b.items);
  if (bounds) {
    const dx = -bounds.x; const dy = -bounds.y;
    for (const e of b.items) { e.x += dx; e.y += dy; }
  }
  return b;
}
const cleanText = (v, max, d) => (typeof v === 'string' && v.trim() ? v.trim().slice(0, max) : d);

function cleanLibraryItem(raw, i, fromExcalidraw) {
  const rawEls = Array.isArray(raw) ? raw : isObj(raw) ? raw.elements : null;
  if (!Array.isArray(rawEls) || !rawEls.length) return null;
  const elements = fromExcalidraw ? importExcalidraw(rawEls.slice(0, LIMITS.items)).items : rawEls.slice(0, LIMITS.items);
  const board = normalizeItemElements(elements);
  if (!board.items.length) return null;
  const meta = isObj(raw) ? raw : {};
  return {
    id: cleanText(meta.id, 80, `item-${i + 1}`), name: cleanText(meta.name, 80, `Item ${i + 1}`),
    category: cleanText(meta.category, 60, fromExcalidraw ? 'Imported' : 'My library'), elements: board.items,
  };
}

export function serializeLibrary(items) {
  const clean = (Array.isArray(items) ? items : []).slice(0, MAX_LIBRARY_ITEMS).map((it, i) => cleanLibraryItem(it, i, false)).filter(Boolean);
  return JSON.stringify({ type: LIBRARY_TYPE, version: 1, source: SCENE_SOURCE, items: clean }, null, 2);
}

// Our library as an .excalidrawlib (v2) so it can be loaded on excalidraw.com.
export function serializeExcalidrawLibrary(items) {
  const clean = (Array.isArray(items) ? items : []).slice(0, MAX_LIBRARY_ITEMS).map((it, i) => cleanLibraryItem(it, i, false)).filter(Boolean);
  return JSON.stringify({
    type: 'excalidraw-library', version: 2, source: SCENE_SOURCE,
    libraryItems: clean.map((it) => ({
      id: it.id, status: 'published', created: Date.now(), name: it.name,
      elements: exportExcalidraw({ items: it.elements, nextId: it.elements.length + 1000 }).elements,
    })),
  }, null, 2);
}

export function parseLibrary(text) {
  if (typeof text !== 'string') throw new Error('Expected the text of a library file.');
  if (text.length > MAX_FILE_BYTES) throw new Error('This file is too large (limit 25 MB).');
  let data;
  try { data = JSON.parse(text.replace(/^﻿/, '')); } catch { throw new Error('This file is not valid JSON.'); }
  let rawItems;
  let fromEx = false;
  if (isObj(data) && data.type === LIBRARY_TYPE && Array.isArray(data.items)) rawItems = data.items;
  else if (isObj(data) && Array.isArray(data.libraryItems)) { rawItems = data.libraryItems; fromEx = true; } // excalidrawlib v2
  else if (isObj(data) && Array.isArray(data.library)) { rawItems = data.library; fromEx = true; } // excalidrawlib v1
  else if (isObj(data) && data.type === 'excalidraw' && Array.isArray(data.elements)) { rawItems = [{ name: 'Scene', elements: data.elements }]; fromEx = true; }
  else throw new Error('Unrecognised library file: expected a .excalidrawlib or a whiteboard library.');
  return rawItems.slice(0, MAX_LIBRARY_ITEMS).map((it, i) => cleanLibraryItem(it, i, fromEx)).filter(Boolean);
}

// ---------------------------------------------------------------- library items <-> board elements
let itemCounter = 0;
export function libraryItemFromElements(board, elements, name = 'Untitled', category = 'My library') {
  const scratch = { version: 2, items: [], nextId: 1 };
  const clones = cloneElements(scratch, JSON.parse(JSON.stringify(elements)), { source: board });
  scratch.items = clones;
  const b = selectionBounds(scratch, clones);
  if (b) for (const e of clones) { e.x -= b.x; e.y -= b.y; }
  itemCounter += 1;
  return {
    id: `my-${Date.now().toString(36)}-${itemCounter}-${Math.floor(Math.random() * 1e6).toString(36)}`,
    name: cleanText(name, 80, 'Untitled'), category: cleanText(category, 60, 'My library'), elements: clones,
  };
}

export function instantiateLibraryItem(board, item, { x = 0, y = 0 } = {}) {
  const src = { version: 2, items: item.elements, nextId: 1 };
  const scratch = { version: 2, items: [], nextId: Math.max(1, board?.nextId || 1) }; // do not burn the real board's ids; insertElements clones again
  const b = selectionBounds(src, item.elements) || { x: 0, y: 0 };
  return cloneElements(scratch, item.elements, { dx: x - b.x, dy: y - b.y, source: src });
}

// ---------------------------------------------------------------- browser helpers
const hasDom = () => typeof document !== 'undefined' && typeof document.createElement === 'function';

export function downloadBlob(filename, blob) {
  if (!hasDom()) throw new Error('downloads need a browser');
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.rel = 'noopener';
  a.style.display = 'none';
  document.body.appendChild(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
}

export function downloadText(filename, text, mime = 'application/json') {
  downloadBlob(filename, new Blob([text], { type: `${mime};charset=utf-8` }));
}

// Opens the native file picker. Resolves with the File, or null when cancelled.
export function pickFile(accept = '') {
  if (!hasDom()) return Promise.resolve(null);
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    if (accept) input.accept = accept;
    input.style.display = 'none';
    let done = false;
    const finish = (f) => { if (done) return; done = true; input.remove(); resolve(f); };
    input.addEventListener('change', () => finish(input.files && input.files[0] ? input.files[0] : null));
    input.addEventListener('cancel', () => finish(null));
    document.body.appendChild(input);
    input.click();
  });
}

export function readFileText(file) {
  if (!file) return Promise.reject(new Error('No file selected.'));
  if (file.size > MAX_FILE_BYTES) return Promise.reject(new Error('This file is too large (limit 25 MB).'));
  if (typeof file.text === 'function') return file.text();
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(String(r.result));
    r.onerror = () => reject(new Error('Could not read the file.'));
    r.readAsText(file);
  });
}

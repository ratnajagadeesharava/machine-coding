// Pure board operations for the whiteboard (style, z-order, groups, align, flip, duplicate, delete, frames, find).
// DOM-free. Every function MUTATES the board it is given (the engine passes a working copy and does the history) and returns what changed.
// Conventions:
//  - `ids` may be an Array or a Set; unknown / non-integer ids are ignored, duplicates collapse. Results are always in z-order.
//  - LOCKED elements cannot be selected in the editor, so operations that act on "the selection as a whole" (reorder, group, ungroup, align,
//    distribute, flip, duplicate) skip locked elements that were passed in explicitly. Style changes, lock/unlock, delete and move act on
//    exactly the ids they are given (the caller decides). Group mates / frame children pulled in by expansion are not filtered again.
import {
  ARROWHEAD_IDS, ARROW_TYPES, BINDABLE_TYPES, ELEMENT_TYPES, FILL_STYLES, FONT_FAMILIES, LIMITS, ROUGHNESS, ROUNDNESS, STROKE_STYLES, STROKE_WIDTHS,
  TEXT_ALIGNS, VERTICAL_ALIGNS, center, cloneElements, elementBounds, findItem, newId, normalizePoints, resolvePoints, rotatePoint, selectionBounds,
} from '../../shared/board.js';

export const MIXED = Symbol.for('wb.mixed');

// ---------- style property table ----------
const BOX = ['rect', 'diamond', 'ellipse', 'database', 'sticky'];
export const STYLE_PROPS = {
  strokeColor: [...BOX, 'text', 'frame', 'freedraw', 'line', 'arrow'],
  backgroundColor: [...BOX, 'freedraw'],
  fillStyle: [...BOX, 'freedraw'],
  strokeWidth: [...BOX, 'freedraw', 'line', 'arrow'],
  strokeStyle: [...BOX, 'freedraw', 'line', 'arrow'],
  roughness: [...BOX, 'freedraw', 'line', 'arrow'],
  opacity: [...ELEMENT_TYPES],
  roundness: ['rect', 'diamond', 'sticky'],
  fontFamily: [...BOX, 'text', 'arrow'],
  fontSize: [...BOX, 'text', 'arrow'],
  textAlign: [...BOX, 'text'],
  verticalAlign: [...BOX, 'text'],
  arrowType: ['arrow', 'line'],
  startArrowhead: ['arrow'],
  endArrowhead: ['arrow'],
};
const PROP_NAMES = Object.keys(STYLE_PROPS);
const TOOL_TYPES = Object.fromEntries(ELEMENT_TYPES.map((t) => [t, [t]]));
const COLOR_RE = /^(transparent|#[0-9a-fA-F]{3,8}|rgba?\([0-9 ,.%]+\)|[a-zA-Z]{3,20})$/;
const STROKE_WIDTH_VALUES = Object.values(STROKE_WIDTHS);
const ROUGHNESS_VALUES = Object.values(ROUGHNESS);

// Normalises/validates one style value for one element. Returns { ok, value }.
function checkValue(prop, v, el) {
  const bad = { ok: false };
  const good = (value) => ({ ok: true, value });
  switch (prop) {
    case 'strokeColor': case 'backgroundColor': return typeof v === 'string' && COLOR_RE.test(v) ? good(v) : bad;
    case 'fillStyle': return FILL_STYLES.includes(v) ? good(v) : bad;
    case 'strokeWidth': return STROKE_WIDTH_VALUES.includes(v) ? good(v) : bad;
    case 'strokeStyle': return STROKE_STYLES.includes(v) ? good(v) : bad;
    case 'roughness': return ROUGHNESS_VALUES.includes(v) ? good(v) : bad;
    case 'opacity': return typeof v === 'number' && Number.isFinite(v) ? good(Math.max(LIMITS.opacity[0], Math.min(LIMITS.opacity[1], Math.round(v)))) : bad;
    case 'roundness': return ROUNDNESS.includes(v) ? good(v) : bad;
    case 'fontFamily': { const n = typeof v === 'string' && /^\d+$/.test(v) ? Number(v) : v; return FONT_FAMILIES[n] ? good(n) : bad; }
    case 'fontSize': return typeof v === 'number' && Number.isFinite(v) ? good(Math.max(LIMITS.fontSize[0], Math.min(LIMITS.fontSize[1], Math.round(v)))) : bad;
    case 'textAlign': return TEXT_ALIGNS.includes(v) ? good(v) : bad;
    case 'verticalAlign': return VERTICAL_ALIGNS.includes(v) ? good(v) : bad;
    case 'arrowType': return (el.type === 'line' ? ['sharp', 'round'] : ARROW_TYPES).includes(v) ? good(v) : bad;
    case 'startArrowhead': case 'endArrowhead': return v === null || ARROWHEAD_IDS.includes(v) ? good(v) : bad;
    default: return bad;
  }
}

export const styleApplies = (el, prop) => !!STYLE_PROPS[prop] && STYLE_PROPS[prop].includes(el.type);

// ---------- small helpers ----------
function idList(ids) {
  if (ids == null) return [];
  const arr = Array.isArray(ids) ? ids : (typeof ids === 'object' && Symbol.iterator in ids && typeof ids !== 'string' ? [...ids] : []);
  return arr.filter(Number.isInteger);
}
const itemsOf = (board) => (board && Array.isArray(board.items) ? board.items : []);
// elements for ids, in z-order, unknown ids dropped
function byIds(board, ids) {
  const set = new Set(idList(ids));
  return itemsOf(board).filter((e) => set.has(e.id));
}
const unlocked = (els) => els.filter((e) => !e.locked);
const idsOf = (els) => els.map((e) => e.id);
const finite = (n) => Number.isFinite(n);
const setItems = (board, arr) => { board.items.splice(0, board.items.length, ...arr); };
const sameOrder = (a, b) => a.length === b.length && a.every((e, i) => e === b[i]);

// Local text size estimate so ops.js has no dependency on geometry.js. The engine may re-measure afterwards.
const CHAR_W = { 1: 0.56, 2: 0.55, 3: 0.6 };
const LINE_H = 1.25;
export function approxTextSize(text, { fontFamily = 1, fontSize = 20, maxWidth } = {}) {
  const base = (CHAR_W[fontFamily] || 0.56) * fontSize;
  const charW = (ch) => {
    if (fontFamily === 3) return base;
    if ('iljtf.,:;!|\'` '.includes(ch)) return base * 0.55;
    if ('mwMW@'.includes(ch)) return base * 1.45;
    if (ch >= 'A' && ch <= 'Z') return base * 1.1;
    return base;
  };
  const width = (s) => { let w = 0; for (const ch of s) w += charW(ch); return w; };
  let lines = String(text ?? '').split('\n');
  if (maxWidth && maxWidth > 0) {
    const wrapped = [];
    for (const line of lines) {
      let cur = '';
      for (const word of line.split(/(?<= )/)) {
        if (cur && width(cur + word) > maxWidth) { wrapped.push(cur.replace(/ +$/, '')); cur = ''; }
        if (!cur && width(word) > maxWidth) { // break an over-long word
          let piece = '';
          for (const ch of word) { if (piece && width(piece + ch) > maxWidth) { wrapped.push(piece); piece = ''; } piece += ch; }
          cur = piece;
        } else cur += word;
      }
      wrapped.push(cur.replace(/ +$/, ''));
    }
    lines = wrapped;
  }
  const w = lines.reduce((m, l) => Math.max(m, width(l)), 0);
  return { w: Math.ceil(w), h: Math.ceil(lines.length * fontSize * LINE_H), lines };
}

// Absolute (stored, unresolved) points of a point-based element, and writing them back.
const storedAbs = (el) => (el.points || []).map(([px, py]) => [el.x + px, el.y + py]);
function setAbs(el, abs) {
  const [ox, oy] = abs[0];
  el.x = ox; el.y = oy;
  el.points = abs.map(([x, y]) => [x - ox, y - oy]);
  normalizePoints(el);
}

// Keeps "frame children are above their frame" true: a child that sits below its frame is moved to just above that frame.
function fixFrameOrder(board) {
  const items = board.items;
  const pos = new Map(items.map((e, i) => [e.id, i]));
  const deferred = new Map();
  const out = [];
  const emit = (e) => {
    out.push(e);
    const kids = deferred.get(e.id);
    if (kids) { deferred.delete(e.id); kids.forEach(emit); }
  };
  for (const e of items) {
    const fp = e.frameId != null ? pos.get(e.frameId) : undefined;
    if (fp !== undefined && fp > pos.get(e.id)) {
      if (!deferred.has(e.frameId)) deferred.set(e.frameId, []);
      deferred.get(e.frameId).push(e);
    } else emit(e);
  }
  for (const kids of deferred.values()) kids.forEach((k) => out.push(k)); // frame ids that form cycles
  if (sameOrder(items, out)) return false;
  setItems(board, out);
  return true;
}

// ---------- style ----------
export function applyStyle(board, ids, patch) {
  const changed = [];
  if (!patch || typeof patch !== 'object') return changed;
  for (const el of byIds(board, ids)) {
    let touched = false;
    let resize = false;
    for (const prop of PROP_NAMES) {
      if (!Object.prototype.hasOwnProperty.call(patch, prop) || !styleApplies(el, prop)) continue;
      const r = checkValue(prop, patch[prop], el);
      if (!r.ok || el[prop] === r.value) continue;
      el[prop] = r.value;
      touched = true;
      if (prop === 'fontSize' || prop === 'fontFamily') resize = true;
      if (prop === 'arrowType' && r.value === 'elbow' && el.type === 'arrow' && el.points.length > 2) {
        el.points = [el.points[0], el.points[el.points.length - 1]]; // an elbow route ignores intermediate points
        normalizePoints(el);
      }
    }
    if (resize && el.type === 'text') fitText(el);
    if (touched) changed.push(el.id);
  }
  return changed;
}

// Re-sizes a free text element to its text, keeping the edge that its alignment anchors.
function fitText(el) {
  const s = approxTextSize(el.text, el);
  const w = Math.max(4, s.w);
  const h = Math.max(4, s.h);
  if (el.textAlign === 'center') el.x += (el.w - w) / 2;
  else if (el.textAlign === 'right') el.x += el.w - w;
  el.w = w; el.h = h;
}

export function styleSummary(board, ids, fallbackStyle = {}, tool = null) {
  const els = byIds(board, ids);
  const values = {};
  const applies = {};
  const fb = (prop) => (fallbackStyle && fallbackStyle[prop] !== undefined ? fallbackStyle[prop] : undefined);
  for (const prop of PROP_NAMES) {
    if (!els.length) {
      const types = tool && TOOL_TYPES[tool];
      applies[prop] = types ? types.some((t) => STYLE_PROPS[prop].includes(t)) : true;
      values[prop] = fb(prop);
      continue;
    }
    const subset = els.filter((e) => styleApplies(e, prop));
    applies[prop] = subset.length > 0;
    if (!subset.length) { values[prop] = fb(prop); continue; }
    const first = subset[0][prop];
    values[prop] = subset.every((e) => e[prop] === first) ? first : MIXED;
  }
  return { values, applies };
}

export function copyStyle(el) {
  const out = {};
  if (!el) return out;
  for (const prop of PROP_NAMES) if (styleApplies(el, prop) && el[prop] !== undefined) out[prop] = el[prop];
  return out;
}
export const pasteStyle = (board, ids, style) => applyStyle(board, ids, style);

// ---------- selection ----------
export function expandSelection(board, ids) {
  const items = itemsOf(board);
  const sel = new Set(byIds(board, ids).map((e) => e.id));
  let grew = true;
  while (grew) {
    grew = false;
    const groups = new Set();
    const frames = new Set();
    for (const e of items) {
      if (!sel.has(e.id)) continue;
      if (e.groupIds && e.groupIds.length) groups.add(e.groupIds[e.groupIds.length - 1]);
      if (e.type === 'frame') frames.add(e.id);
    }
    for (const e of items) {
      if (sel.has(e.id)) continue;
      if ((e.groupIds || []).some((g) => groups.has(g)) || (e.frameId != null && frames.has(e.frameId))) { sel.add(e.id); grew = true; }
    }
  }
  return items.filter((e) => sel.has(e.id)).map((e) => e.id);
}

// ---------- z-order ----------
export function reorder(board, ids, dir) {
  if (!['front', 'forward', 'backward', 'back'].includes(dir)) return false;
  const picked = unlocked(byIds(board, ids));
  if (!picked.length) return false;
  const sel = new Set(expandSelection(board, idsOf(picked)));
  const before = board.items.slice();
  let arr = before.slice();
  if (dir === 'front') arr = [...arr.filter((e) => !sel.has(e.id)), ...arr.filter((e) => sel.has(e.id))];
  else if (dir === 'back') arr = [...arr.filter((e) => sel.has(e.id)), ...arr.filter((e) => !sel.has(e.id))];
  else if (dir === 'forward') {
    for (let i = arr.length - 2; i >= 0; i--) if (sel.has(arr[i].id) && !sel.has(arr[i + 1].id)) [arr[i], arr[i + 1]] = [arr[i + 1], arr[i]];
  } else {
    for (let i = 1; i < arr.length; i++) if (sel.has(arr[i].id) && !sel.has(arr[i - 1].id)) [arr[i], arr[i - 1]] = [arr[i - 1], arr[i]];
  }
  setItems(board, arr);
  fixFrameOrder(board);
  return !sameOrder(before, board.items);
}

// ---------- groups ----------
export function group(board, ids) {
  const picked = unlocked(byIds(board, ids));
  const els = byIds(board, expandSelection(board, idsOf(picked))).filter((e) => e.type !== 'frame');
  if (els.length < 2) return null;
  const outer = els.map((e) => (e.groupIds.length ? e.groupIds[e.groupIds.length - 1] : null));
  if (outer[0] !== null && outer.every((g) => g === outer[0])) return null; // already exactly one group
  if (els.some((e) => e.groupIds.length >= LIMITS.groupDepth)) return null;
  const gid = newId(board);
  for (const e of els) e.groupIds.push(gid);
  return gid;
}

export function ungroup(board, ids) {
  const picked = unlocked(byIds(board, ids));
  const dissolve = new Set(picked.filter((e) => e.groupIds.length).map((e) => e.groupIds[e.groupIds.length - 1]));
  if (!dissolve.size) return [];
  const changed = [];
  for (const e of board.items) {
    if (!e.groupIds.some((g) => dissolve.has(g))) continue;
    e.groupIds = e.groupIds.filter((g) => !dissolve.has(g));
    changed.push(e.id);
  }
  return changed;
}

function pruneSingletonGroups(board) {
  const count = new Map();
  for (const e of board.items) for (const g of e.groupIds || []) count.set(g, (count.get(g) || 0) + 1);
  for (const e of board.items) if ((e.groupIds || []).some((g) => count.get(g) < 2)) e.groupIds = e.groupIds.filter((g) => count.get(g) >= 2);
}

// ---------- lock ----------
export function setLocked(board, ids, locked) {
  const changed = [];
  const flag = !!locked;
  for (const e of byIds(board, expandSelection(board, ids))) if (e.locked !== flag) { e.locked = flag; changed.push(e.id); }
  return changed;
}
export function unlockAll(board) {
  const changed = [];
  for (const e of itemsOf(board)) if (e.locked) { e.locked = false; changed.push(e.id); }
  return changed;
}

// ---------- moving ----------
// Does this end of the arrow stay attached to a shape that is NOT moving along with it?
function pinned(board, set, arrow, key) {
  const id = arrow[key];
  if (id == null || set.has(id)) return false;
  const target = findItem(board, id);
  return !!target && BINDABLE_TYPES.includes(target.type);
}

// Moves exactly the elements in `set` (Set of ids). Arrows keep ends that are bound to shapes outside the set.
function moveSet(board, set, dx, dy) {
  if (!finite(dx) || !finite(dy) || (dx === 0 && dy === 0)) return;
  for (const el of board.items) {
    if (!set.has(el.id)) continue;
    if (el.type === 'arrow' && el.points && el.points.length >= 2) {
      const ps = pinned(board, set, el, 'from');
      const pe = pinned(board, set, el, 'to');
      if (ps || pe) {
        const abs = storedAbs(el);
        abs.forEach((p, i) => {
          if ((i === 0 && ps) || (i === abs.length - 1 && pe)) return;
          p[0] += dx; p[1] += dy;
        });
        setAbs(el, abs);
        continue;
      }
    }
    el.x += dx; el.y += dy;
  }
}

export function moveElements(board, ids, dx, dy) {
  const els = byIds(board, ids);
  const set = new Set(idsOf(els));
  const frames = new Set(els.filter((e) => e.type === 'frame').map((e) => e.id));
  for (const e of board.items) if (e.frameId != null && frames.has(e.frameId)) set.add(e.id);
  moveSet(board, set, dx, dy);
}

// ---------- align / distribute ----------
// A "unit" is a group (by outermost group id) or a single element; children of a selected frame belong to the frame's unit.
function buildUnits(board, els) {
  const set = new Set(idsOf(els));
  const byId = new Map(els.map((e) => [e.id, e]));
  const keyOf = (e) => {
    if (e.type !== 'frame' && e.frameId != null && set.has(e.frameId) && byId.get(e.frameId)?.type === 'frame') return keyOf(byId.get(e.frameId));
    return e.groupIds && e.groupIds.length ? `g${e.groupIds[e.groupIds.length - 1]}` : `e${e.id}`;
  };
  const units = new Map();
  for (const e of els) {
    const k = keyOf(e);
    if (!units.has(k)) units.set(k, []);
    units.get(k).push(e);
  }
  return [...units.values()].map((members) => ({ members, ids: new Set(idsOf(members)), box: selectionBounds(board, members) }));
}

function shiftUnit(board, unit, dx, dy, changed) {
  if (!finite(dx) || !finite(dy) || (Math.abs(dx) < 1e-9 && Math.abs(dy) < 1e-9)) return;
  moveSet(board, unit.ids, dx, dy);
  for (const id of unit.ids) changed.add(id);
}

const selectionElements = (board, ids) => byIds(board, expandSelection(board, idsOf(unlocked(byIds(board, ids)))));

export function align(board, ids, dir) {
  if (!['left', 'right', 'top', 'bottom', 'hcenter', 'vcenter'].includes(dir)) return [];
  const els = selectionElements(board, ids);
  const units = buildUnits(board, els);
  if (units.length < 2) return [];
  const b = selectionBounds(board, els);
  const changed = new Set();
  for (const u of units) {
    const ub = u.box;
    let dx = 0; let dy = 0;
    if (dir === 'left') dx = b.x - ub.x;
    else if (dir === 'right') dx = b.x + b.w - (ub.x + ub.w);
    else if (dir === 'hcenter') dx = b.x + b.w / 2 - (ub.x + ub.w / 2);
    else if (dir === 'top') dy = b.y - ub.y;
    else if (dir === 'bottom') dy = b.y + b.h - (ub.y + ub.h);
    else dy = b.y + b.h / 2 - (ub.y + ub.h / 2);
    shiftUnit(board, u, dx, dy, changed);
  }
  return board.items.filter((e) => changed.has(e.id)).map((e) => e.id);
}

export function distribute(board, ids, axis) {
  if (axis !== 'h' && axis !== 'v') return [];
  const units = buildUnits(board, selectionElements(board, ids));
  if (units.length < 3) return [];
  const h = axis === 'h';
  const lo = (u) => (h ? u.box.x : u.box.y);
  const size = (u) => (h ? u.box.w : u.box.h);
  units.sort((a, b) => lo(a) + size(a) / 2 - (lo(b) + size(b) / 2));
  const first = units[0];
  const last = units[units.length - 1];
  const span = lo(last) + size(last) - lo(first);
  const total = units.reduce((s, u) => s + size(u), 0);
  const gap = (span - total) / (units.length - 1);
  const changed = new Set();
  let cursor = lo(first) + size(first) + gap;
  for (let i = 1; i < units.length - 1; i++) {
    const u = units[i];
    const d = cursor - lo(u);
    shiftUnit(board, u, h ? d : 0, h ? 0 : d, changed);
    cursor += size(u) + gap;
  }
  return board.items.filter((e) => changed.has(e.id)).map((e) => e.id);
}

// ---------- flip ----------
export function flip(board, ids, axis) {
  if (axis !== 'h' && axis !== 'v') return [];
  const els = selectionElements(board, ids);
  if (!els.length) return [];
  const b = selectionBounds(board, els);
  if (!b || ![b.x, b.y, b.w, b.h].every(finite)) return [];
  const h = axis === 'h';
  const c = h ? b.x + b.w / 2 : b.y + b.h / 2;
  const set = new Set(idsOf(els));
  const mirror = (v) => 2 * c - v;
  // resolve arrow geometry first: shapes are about to move
  const resolved = new Map();
  for (const e of els) if (e.type === 'arrow' || e.type === 'line' || e.type === 'freedraw') {
    let pts = e.type === 'arrow' ? resolvePoints(board, e).map((p) => [p.x, p.y]) : storedAbs(e);
    if (e.type === 'arrow' && e.arrowType === 'elbow' && pts.length > 2) pts = [pts[0], pts[pts.length - 1]];
    resolved.set(e.id, pts);
  }
  for (const e of els) {
    if (resolved.has(e.id)) {
      let pts = resolved.get(e.id).map(([x, y]) => (h ? [mirror(x), y] : [x, mirror(y)]));
      if (e.type === 'arrow' || e.type === 'line') {
        if (h) { // the start stays the left-hand end: reverse the path, swap heads and bindings
          pts = pts.reverse();
          [e.startArrowhead, e.endArrowhead] = [e.endArrowhead, e.startArrowhead];
          [e.from, e.to] = [e.to, e.from];
        }
        if (e.type === 'arrow') {
          if (e.from != null && !set.has(e.from)) e.from = null; // its shape did not flip with it: keep the drawn end instead
          if (e.to != null && !set.has(e.to)) e.to = null;
        }
      }
      setAbs(e, pts);
    } else {
      const cc = center(e);
      if (h) e.x = mirror(cc.x) - e.w / 2; else e.y = mirror(cc.y) - e.h / 2;
      if (e.angle) e.angle = -e.angle;
    }
  }
  return idsOf(els);
}

// ---------- duplicate / delete ----------
export function duplicate(board, ids, { dx = 20, dy = 20 } = {}) {
  const picked = unlocked(byIds(board, ids));
  const els = byIds(board, expandSelection(board, idsOf(picked)));
  if (!els.length) return [];
  const clones = cloneElements(board, els, { dx: finite(dx) ? dx : 0, dy: finite(dy) ? dy : 0 });
  const dupFrames = new Set(els.filter((e) => e.type === 'frame').map((e) => e.id));
  // a copy of a frame child stays in that frame (when the frame itself was not copied)
  els.forEach((e, i) => {
    if (e.frameId != null && !dupFrames.has(e.frameId) && findItem(board, e.frameId)) clones[i].frameId = e.frameId;
  });
  const set = new Set(idsOf(els));
  let top = 0;
  board.items.forEach((e, i) => { if (set.has(e.id)) top = i; });
  board.items.splice(top + 1, 0, ...clones);
  fixFrameOrder(board);
  return idsOf(clones);
}

export function deleteElements(board, ids) {
  const els = byIds(board, ids);
  const gone = new Set(idsOf(els));
  for (const e of els) if (e.type === 'frame') for (const k of board.items) if (k.frameId === e.id) gone.add(k.id);
  if (!gone.size) return [];
  for (const a of board.items) { // arrows lose their binding but keep where they were drawn
    if (gone.has(a.id) || a.type !== 'arrow' || !((a.from != null && gone.has(a.from)) || (a.to != null && gone.has(a.to)))) continue;
    let pts = resolvePoints(board, a).map((p) => [p.x, p.y]);
    if (a.arrowType === 'elbow' && pts.length > 2) pts = [pts[0], pts[pts.length - 1]];
    if (pts.length >= 2) setAbs(a, pts);
    if (a.from != null && gone.has(a.from)) a.from = null;
    if (a.to != null && gone.has(a.to)) a.to = null;
  }
  const removed = board.items.filter((e) => gone.has(e.id)).map((e) => e.id);
  setItems(board, board.items.filter((e) => !gone.has(e.id)));
  for (const e of board.items) if (e.frameId != null && gone.has(e.frameId)) e.frameId = null;
  pruneSingletonGroups(board);
  return removed;
}

// ---------- frames ----------
export function assignFrames(board, ids) {
  const changed = [];
  const frames = itemsOf(board).map((e, i) => ({ e, i })).filter((x) => x.e.type === 'frame');
  for (const el of byIds(board, ids)) {
    if (el.type === 'frame' || el.locked) continue;
    const b = elementBounds(board, el);
    const cx = b.x + b.w / 2;
    const cy = b.y + b.h / 2;
    let target = null;
    for (const { e: f } of frames) { // later in z-order = topmost
      if (f.id !== el.id && cx >= f.x && cx <= f.x + f.w && cy >= f.y && cy <= f.y + f.h) target = f;
    }
    const next = target ? target.id : null;
    if (el.frameId !== next) { el.frameId = next; changed.push(el.id); }
  }
  fixFrameOrder(board);
  return changed;
}

// ---------- arrow binding ----------
function hitsShape(el, p, tol) {
  const c = center(el);
  const q = rotatePoint(p, c, -(el.angle || 0));
  const dx = q.x - c.x;
  const dy = q.y - c.y;
  const hw = el.w / 2 + tol;
  const hh = el.h / 2 + tol;
  if (hw <= 0 || hh <= 0) return false;
  if (el.type === 'ellipse') return (dx / hw) ** 2 + (dy / hh) ** 2 <= 1;
  if (el.type === 'diamond') return Math.abs(dx) / hw + Math.abs(dy) / hh <= 1;
  return Math.abs(dx) <= hw && Math.abs(dy) <= hh;
}

function shapeAt(board, p, tol, excludeId) {
  const items = board.items;
  for (let i = items.length - 1; i >= 0; i--) {
    const e = items[i];
    if (e.id !== excludeId && BINDABLE_TYPES.includes(e.type) && hitsShape(e, p, tol)) return e;
  }
  return null;
}

export function bindArrow(board, arrowId, { start, end, tol = 6 } = {}) {
  const a = Number.isInteger(arrowId) ? findItem(board, arrowId) : null;
  if (!a || a.type !== 'arrow' || !a.points || a.points.length < 2) return;
  const t = finite(tol) ? Math.max(0, tol) : 6;
  const abs = storedAbs(a);
  const valid = (p) => p && finite(p.x) && finite(p.y);
  const resolveEnd = (given, idx) => { // -> { id|null, moved point|null }
    if (given === null) return { id: null };
    const p = valid(given) ? { x: given.x, y: given.y } : { x: abs[idx][0], y: abs[idx][1] };
    const shape = shapeAt(board, p, t, a.id);
    return { id: shape ? shape.id : null, p: valid(given) ? p : null };
  };
  const s = resolveEnd(start, 0);
  const e = resolveEnd(end, abs.length - 1);
  a.from = s.id;
  a.to = e.id;
  if (s.p) abs[0] = [s.p.x, s.p.y];
  if (e.p) abs[abs.length - 1] = [e.p.x, e.p.y];
  if (s.p || e.p) setAbs(a, abs);
}

// ---------- find ----------
export function findText(board, query, { caseSensitive = false } = {}) {
  if (typeof query !== 'string' || !query) return [];
  const norm = (s) => (caseSensitive ? s : s.toLowerCase());
  const q = norm(query);
  const out = [];
  for (const e of itemsOf(board)) {
    const hay = e.type === 'frame' ? e.name : e.text;
    if (typeof hay === 'string' && norm(hay).includes(q)) out.push(e.id);
  }
  return out;
}

// Board model shared by the browser whiteboard and the server (which re-validates and describes it for the AI).
//
// A board is { items: Item[], nextId: number }. Items are either shapes or arrows:
//   shape: { id, type: 'rect' | 'ellipse' | 'database' | 'text', x, y, w, h, text }
//   arrow: { id, type: 'arrow', from: id|null, to: id|null, x1, y1, x2, y2, text }
// A bound arrow end follows its shape; an unbound end uses the stored coordinates.

export const SHAPE_TYPES = ['rect', 'ellipse', 'database', 'text'];
export const ITEM_TYPES = [...SHAPE_TYPES, 'arrow'];
export const DEFAULT_LABELS = { rect: 'Service', ellipse: 'Client', database: 'Database', text: 'Note' };
export const DEFAULT_SIZES = {
  rect: [140, 80],
  ellipse: [130, 80],
  database: [110, 100],
  text: [120, 36],
};
export const LIMITS = { items: 300, text: 120, coord: 100000 };

export function emptyBoard() {
  return { items: [], nextId: 1 };
}

export const isShape = (item) => SHAPE_TYPES.includes(item.type);
export const isArrow = (item) => item.type === 'arrow';

export function findItem(board, id) {
  return board.items.find((i) => i.id === id) || null;
}

export function center(shape) {
  return { x: shape.x + shape.w / 2, y: shape.y + shape.h / 2 };
}

// Point where the line from the shape's centre towards (tx, ty) leaves the shape.
export function edgePoint(shape, tx, ty) {
  const c = center(shape);
  const dx = tx - c.x;
  const dy = ty - c.y;
  if (dx === 0 && dy === 0) return c;
  const hw = Math.max(shape.w / 2, 1);
  const hh = Math.max(shape.h / 2, 1);
  let t;
  if (shape.type === 'ellipse') {
    t = 1 / Math.sqrt((dx / hw) ** 2 + (dy / hh) ** 2);
  } else {
    t = Math.min(hw / Math.abs(dx || 1e-9), hh / Math.abs(dy || 1e-9));
  }
  return { x: c.x + dx * t, y: c.y + dy * t };
}

// Resolve an arrow's two end points, following bound shapes.
export function arrowEnds(board, arrow) {
  const from = arrow.from != null ? findItem(board, arrow.from) : null;
  const to = arrow.to != null ? findItem(board, arrow.to) : null;
  const a = from && isShape(from) ? from : null;
  const b = to && isShape(to) ? to : null;
  let p1 = { x: arrow.x1, y: arrow.y1 };
  let p2 = { x: arrow.x2, y: arrow.y2 };
  if (a && b) {
    const ca = center(a);
    const cb = center(b);
    p1 = edgePoint(a, cb.x, cb.y);
    p2 = edgePoint(b, ca.x, ca.y);
  } else if (a) {
    p1 = edgePoint(a, p2.x, p2.y);
  } else if (b) {
    p2 = edgePoint(b, p1.x, p1.y);
  }
  return { x1: p1.x, y1: p1.y, x2: p2.x, y2: p2.y, fromShape: a, toShape: b };
}

export function labelOf(item) {
  const t = (item.text || '').trim();
  if (t) return t;
  return `unlabelled ${item.type}`;
}

const KIND = {
  rect: 'box',
  ellipse: 'oval (often a client or external actor)',
  database: 'database / datastore',
  text: 'note',
};

export function boardStats(board) {
  const shapes = board.items.filter(isShape);
  const components = shapes.filter((s) => s.type !== 'text');
  const arrows = board.items.filter(isArrow);
  const connections = arrows.filter((a) => {
    const f = a.from != null ? findItem(board, a.from) : null;
    const t = a.to != null ? findItem(board, a.to) : null;
    return f && t && isShape(f) && isShape(t);
  });
  return { components: components.length, connections: connections.length, arrows: arrows.length, notes: shapes.length - components.length };
}

// Plain-text description of the board, written for the AI interviewer (and handy in tests).
export function describeBoard(board) {
  const shapes = board.items.filter(isShape);
  const components = shapes.filter((s) => s.type !== 'text').sort((a, b) => a.x - b.x || a.y - b.y);
  const notes = shapes.filter((s) => s.type === 'text').sort((a, b) => a.y - b.y || a.x - b.x);
  const arrows = board.items.filter(isArrow);
  if (!board.items.length) return 'The whiteboard is empty.';

  const lines = [];
  const stats = boardStats(board);
  lines.push(`${stats.components} component(s), ${stats.connections} connection(s), ${stats.notes} note(s).`);

  if (components.length) {
    lines.push('Components (left to right, with rough x,y position):');
    components.forEach((c, i) => {
      lines.push(`  ${i + 1}. "${labelOf(c)}" - ${KIND[c.type]} @ (${Math.round(c.x)}, ${Math.round(c.y)})`);
    });
  }

  const connected = new Set();
  const flows = [];
  const dangling = [];
  for (const a of arrows) {
    const f = a.from != null ? findItem(board, a.from) : null;
    const t = a.to != null ? findItem(board, a.to) : null;
    const fl = f && isShape(f) ? `"${labelOf(f)}"` : null;
    const tl = t && isShape(t) ? `"${labelOf(t)}"` : null;
    const label = (a.text || '').trim();
    const suffix = label ? ` [label: "${label}"]` : '';
    if (fl && tl) {
      flows.push(`  ${fl} -> ${tl}${suffix}`);
      connected.add(f.id);
      connected.add(t.id);
    } else {
      if (f && isShape(f)) connected.add(f.id);
      if (t && isShape(t)) connected.add(t.id);
      dangling.push(`  ${fl || '(nothing)'} -> ${tl || '(nothing)'}${suffix}`);
    }
  }
  if (flows.length) lines.push('Connections (arrow direction):', ...flows);
  if (dangling.length) lines.push('Arrows with a loose end (not attached to a component on both sides):', ...dangling);

  const isolated = components.filter((c) => !connected.has(c.id));
  if (isolated.length && components.length > 1) {
    lines.push('Components with no arrows attached: ' + isolated.map((c) => `"${labelOf(c)}"`).join(', '));
  }
  if (notes.length) {
    lines.push('Free-text notes:', ...notes.map((n) => `  - "${(n.text || '').trim() || '(empty)'}"`));
  }
  return lines.join('\n');
}

// Stable fingerprint of what the board says (labels, kinds, connections) - positions are bucketed so a tiny drag is not a change.
export function boardSignature(board) {
  const parts = board.items.map((i) => {
    if (isArrow(i)) return `a:${i.from ?? ''}>${i.to ?? ''}:${(i.text || '').trim()}`;
    return `s:${i.type}:${(i.text || '').trim()}:${Math.round(i.x / 40)}:${Math.round(i.y / 40)}`;
  });
  parts.sort();
  let h = 5381;
  const s = parts.join('|');
  for (let k = 0; k < s.length; k++) h = ((h << 5) + h + s.charCodeAt(k)) | 0;
  return `${board.items.length}:${(h >>> 0).toString(36)}`;
}

const num = (v, d = 0) => (Number.isFinite(v) ? Math.max(-LIMITS.coord, Math.min(LIMITS.coord, v)) : d);
const str = (v) => (typeof v === 'string' ? v.slice(0, LIMITS.text) : '');

// Validate and normalise an untrusted board (from localStorage or a request body). Never throws.
export function sanitizeBoard(input) {
  const out = emptyBoard();
  if (!input || typeof input !== 'object' || !Array.isArray(input.items)) return out;
  const ids = new Set();
  for (const raw of input.items.slice(0, LIMITS.items)) {
    if (!raw || typeof raw !== 'object' || !ITEM_TYPES.includes(raw.type)) continue;
    const id = Number.isInteger(raw.id) ? raw.id : null;
    if (id == null || ids.has(id)) continue;
    ids.add(id);
    if (raw.type === 'arrow') {
      out.items.push({
        id, type: 'arrow',
        from: Number.isInteger(raw.from) ? raw.from : null,
        to: Number.isInteger(raw.to) ? raw.to : null,
        x1: num(raw.x1), y1: num(raw.y1), x2: num(raw.x2), y2: num(raw.y2),
        text: str(raw.text),
      });
    } else {
      out.items.push({
        id, type: raw.type,
        x: num(raw.x), y: num(raw.y),
        w: Math.max(20, Math.min(2000, num(raw.w, 100))),
        h: Math.max(16, Math.min(2000, num(raw.h, 60))),
        text: str(raw.text),
      });
    }
  }
  // drop dangling bindings
  const valid = new Set(out.items.filter(isShape).map((s) => s.id));
  for (const a of out.items.filter(isArrow)) {
    if (a.from != null && !valid.has(a.from)) a.from = null;
    if (a.to != null && !valid.has(a.to)) a.to = null;
  }
  out.nextId = Math.max(Number.isInteger(input.nextId) ? input.nextId : 1, ...out.items.map((i) => i.id + 1), 1);
  return out;
}

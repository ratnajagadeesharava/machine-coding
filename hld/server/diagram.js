// Turns the "expected architecture" Aria describes (named boxes and labelled links) into a real whiteboard board.
// The model never supplies coordinates: boxes are laid out left to right by how far they are from the entry point, which keeps the picture tidy.
import { createElement, edgePoint, center, emptyBoard, normalizePoints, sanitizeBoard } from '../shared/board.js';

export const DIAGRAM_KINDS = ['client', 'service', 'database', 'cache', 'queue', 'balancer', 'external', 'storage'];
export const DIAGRAM_LIMITS = { nodes: 14, edges: 24, label: 26, edgeLabel: 22, caption: 300 };

const LOOK = {
  client: { type: 'ellipse', bg: '#e9ecef', w: 140, h: 70 },
  service: { type: 'rect', bg: '#a5d8ff', w: 160, h: 70 },
  database: { type: 'database', bg: '#b2f2bb', w: 120, h: 110 },
  cache: { type: 'rect', bg: '#ffec99', w: 150, h: 70 },
  queue: { type: 'rect', bg: '#ffd8a8', w: 160, h: 70 },
  balancer: { type: 'diamond', bg: '#e5dbff', w: 150, h: 100 },
  external: { type: 'rect', bg: '#ffffff', w: 150, h: 70 },
  storage: { type: 'database', bg: '#99e9f2', w: 120, h: 110 },
};

const clean = (v, n) => (typeof v === 'string' ? v.replace(/\s+/g, ' ').trim().slice(0, n) : '');

// -> { board, caption } or null when the description cannot make a useful picture (fewer than two boxes).
export function buildExpectedDiagram(raw) {
  if (!raw || typeof raw !== 'object') return null;
  const seen = new Set();
  const nodes = [];
  for (const n of Array.isArray(raw.nodes) ? raw.nodes : []) {
    const id = clean(n?.id, 40);
    const label = clean(n?.label, DIAGRAM_LIMITS.label);
    if (!id || !label || seen.has(id) || nodes.length >= DIAGRAM_LIMITS.nodes) continue;
    seen.add(id);
    nodes.push({ id, label, kind: DIAGRAM_KINDS.includes(n.kind) ? n.kind : 'service' });
  }
  if (nodes.length < 2) return null;
  const edges = [];
  const edgeKey = new Set();
  for (const e of Array.isArray(raw.edges) ? raw.edges : []) {
    const from = clean(e?.from, 40); const to = clean(e?.to, 40);
    if (!seen.has(from) || !seen.has(to) || from === to || edgeKey.has(`${from}>${to}`) || edges.length >= DIAGRAM_LIMITS.edges) continue;
    edgeKey.add(`${from}>${to}`);
    edges.push({ from, to, label: clean(e.label, DIAGRAM_LIMITS.edgeLabel) });
  }

  // depth = shortest number of hops from an entry point (a box with nothing pointing at it; clients first)
  const incoming = new Map(nodes.map((n) => [n.id, 0]));
  for (const e of edges) incoming.set(e.to, incoming.get(e.to) + 1);
  const depth = new Map();
  let queue = nodes.filter((n) => incoming.get(n.id) === 0 || n.kind === 'client').map((n) => n.id);
  if (!queue.length) queue = [nodes[0].id];
  for (const id of queue) depth.set(id, 0);
  while (queue.length) {
    const next = [];
    for (const id of queue) {
      for (const e of edges) {
        if (e.from === id && !depth.has(e.to)) { depth.set(e.to, depth.get(id) + 1); next.push(e.to); }
      }
    }
    queue = next;
  }
  let maxDepth = Math.max(0, ...depth.values());
  for (const n of nodes) if (!depth.has(n.id)) depth.set(n.id, ++maxDepth); // unreachable boxes go to the far right

  const columns = new Map();
  for (const n of nodes) {
    const d = depth.get(n.id);
    if (!columns.has(d)) columns.set(d, []);
    columns.get(d).push(n);
  }
  const board = emptyBoard();
  const byNode = new Map();
  const COL = 270; const ROW = 150;
  const tallest = Math.max(...[...columns.values()].map((c) => c.length));
  for (const [d, col] of [...columns.entries()].sort((a, b) => a[0] - b[0])) {
    col.forEach((n, i) => {
      const look = LOOK[n.kind];
      const el = createElement(look.type, {
        id: board.nextId++, x: d * COL, y: (i + (tallest - col.length) / 2) * ROW + (ROW - look.h) / 2, w: look.w, h: look.h,
        text: n.label, backgroundColor: look.bg, fillStyle: 'solid', fontSize: 16, roughness: 1, seed: 1000 + board.nextId * 7919,
      });
      board.items.push(el);
      byNode.set(n.id, el);
    });
  }
  for (const e of edges) {
    const f = byNode.get(e.from); const t = byNode.get(e.to);
    const s = edgePoint(f, center(t).x, center(t).y);
    const end = edgePoint(t, center(f).x, center(f).y);
    const a = createElement('arrow', {
      id: board.nextId++, from: f.id, to: t.id, text: e.label, fontSize: 14, roughness: 1, seed: 2000 + board.nextId * 104729,
      x: s.x, y: s.y, points: [[0, 0], [end.x - s.x, end.y - s.y]],
    });
    normalizePoints(a);
    board.items.push(a);
  }
  return { board: sanitizeBoard(board), caption: clean(raw.caption, DIAGRAM_LIMITS.caption) };
}

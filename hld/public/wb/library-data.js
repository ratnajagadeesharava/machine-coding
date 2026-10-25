// Built-in "System design" stencils. Each item is { id, name, category, elements } with v2 elements normalised to origin (0,0)
// and ids local to the item. Built programmatically so shapes, labels and bound arrows stay consistent.
import { center, createElement, edgePoint, normalizePoints, resolvePoints, selectionBounds } from '../../shared/board.js';

const BG = {
  blue: '#a5d8ff', green: '#b2f2bb', yellow: '#ffec99', red: '#ffc9c9', violet: '#e5dbff', orange: '#ffd8a8', cyan: '#99e9f2', gray: '#e9ecef', none: 'transparent',
};

class Builder {
  constructor() { this.items = []; this.nextId = 1; }

  add(type, props) {
    const el = createElement(type, { id: this.nextId++, roughness: 1, seed: 1000 + this.nextId * 7919, ...props });
    this.items.push(el);
    return el;
  }

  // x,y,w,h box shape with optional label; `bg` is a BG key
  shape(type, x, y, w, h, text, bg = 'none', extra = {}) {
    return this.add(type, { x, y, w, h, text: text || '', backgroundColor: BG[bg] || bg, fontSize: 20, ...extra });
  }

  text(x, y, text, extra = {}) {
    const fontSize = extra.fontSize || 16;
    const lines = String(text).split('\n');
    return this.add('text', { x, y, w: Math.max(...lines.map((l) => l.length)) * fontSize * 0.55, h: lines.length * fontSize * 1.25, text, fontSize, ...extra });
  }

  arrow(from, to, extra = {}) {
    const { label, ...rest } = extra;
    return this.add('arrow', { from: from.id, to: to.id, text: label || '', arrowType: 'sharp', ...rest });
  }

  group(...els) {
    const g = this.nextId++;
    for (const e of els) e.groupIds.push(g);
  }

  // Compute arrow geometry from the final shape positions, then normalise to the origin.
  finish() {
    const board = { version: 2, items: this.items, nextId: this.nextId };
    for (const a of this.items) {
      if (a.type !== 'arrow') continue;
      const f = this.items.find((e) => e.id === a.from);
      const t = this.items.find((e) => e.id === a.to);
      const s = edgePoint(f, center(t).x, center(t).y);
      const e = edgePoint(t, center(f).x, center(f).y);
      a.x = s.x; a.y = s.y;
      a.points = [[0, 0], [e.x - s.x, e.y - s.y]];
      if (a.arrowType === 'elbow') { const r = resolvePoints(board, a); a.x = r[0].x; a.y = r[0].y; a.points = r.map((p) => [p.x - r[0].x, p.y - r[0].y]); }
      normalizePoints(a);
    }
    const b = selectionBounds(board, this.items);
    for (const e of this.items) { e.x = Math.round((e.x - b.x) * 10) / 10; e.y = Math.round((e.y - b.y) * 10) / 10; }
    return this.items;
  }
}

const slug = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
const stencil = (category, name, fn) => {
  const b = new Builder();
  fn(b);
  return { id: `builtin-${slug(name)}`, name, category, elements: b.finish() };
};
const DASH = { strokeStyle: 'dashed' };
const SMALL = { fontSize: 16 };

export const BUILTIN_LIBRARY = [
  // ---- Clients
  stencil('Clients', 'Web client', (b) => {
    const body = b.shape('rect', 0, 0, 180, 120, 'Web client', 'blue');
    const bar = b.shape('rect', 0, 0, 180, 26, '', 'gray', { roundness: 'sharp' });
    b.shape('ellipse', 10, 8, 10, 10, '', 'red', { strokeWidth: 1 });
    b.shape('ellipse', 26, 8, 10, 10, '', 'yellow', { strokeWidth: 1 });
    b.shape('ellipse', 42, 8, 10, 10, '', 'green', { strokeWidth: 1 });
    void body; void bar;
  }),
  stencil('Clients', 'Mobile app', (b) => {
    b.shape('rect', 0, 0, 100, 170, 'Mobile\napp', 'blue', { fontSize: 18 });
    b.shape('rect', 36, 10, 28, 6, '', 'gray', { strokeWidth: 1, roundness: 'round' });
    b.shape('ellipse', 44, 148, 12, 12, '', 'none', { strokeWidth: 1 });
  }),
  stencil('Clients', 'User / actor', (b) => { b.shape('ellipse', 0, 0, 140, 80, 'User', 'yellow'); }),
  stencil('Clients', 'Desktop app', (b) => {
    b.shape('rect', 0, 0, 180, 110, 'Desktop app', 'blue');
    b.shape('rect', 70, 118, 40, 14, '', 'gray', { strokeWidth: 1 });
    b.shape('rect', 45, 132, 90, 8, '', 'gray', { strokeWidth: 1 });
  }),

  // ---- Compute
  stencil('Compute', 'Service', (b) => { b.shape('rect', 0, 0, 160, 80, 'Service', 'green'); }),
  stencil('Compute', 'Microservice', (b) => {
    const s = b.shape('rect', 0, 0, 160, 80, 'Microservice', 'green', { fontSize: 18 });
    const m = b.shape('ellipse', -10, -10, 26, 26, '\u00b5', 'yellow', { fontSize: 16, strokeWidth: 1 });
    b.group(s, m);
  }),
  stencil('Compute', 'Worker', (b) => { b.shape('rect', 0, 0, 160, 80, 'Worker', 'violet', DASH); }),
  stencil('Compute', 'Serverless function', (b) => { b.shape('rect', 0, 0, 170, 80, 'Function\n(serverless)', 'orange', { fontSize: 18 }); }),
  stencil('Compute', 'Cron / scheduler', (b) => { b.shape('ellipse', 0, 0, 160, 90, 'Cron\nscheduler', 'yellow', { fontSize: 18 }); }),

  // ---- Storage
  stencil('Storage', 'Relational DB', (b) => { b.shape('database', 0, 0, 120, 120, 'SQL DB', 'blue', { fontSize: 18 }); }),
  stencil('Storage', 'Document DB', (b) => { b.shape('database', 0, 0, 120, 120, 'Document\nDB', 'green', { fontSize: 16 }); }),
  stencil('Storage', 'Key-value store', (b) => { b.shape('database', 0, 0, 120, 120, 'Key-value\nstore', 'yellow', { fontSize: 16 }); }),
  stencil('Storage', 'Object storage', (b) => {
    b.shape('rect', 0, 0, 190, 100, 'Object storage', 'orange', { verticalAlign: 'top', fontSize: 18 });
    b.shape('rect', 16, 56, 46, 30, '', 'yellow', { strokeWidth: 1 });
    b.shape('rect', 72, 56, 46, 30, '', 'yellow', { strokeWidth: 1 });
    b.shape('rect', 128, 56, 46, 30, '', 'yellow', { strokeWidth: 1 });
  }),
  stencil('Storage', 'Data warehouse', (b) => { b.shape('database', 0, 0, 130, 130, 'Data\nwarehouse', 'violet', { fontSize: 16 }); }),
  stencil('Storage', 'Search index', (b) => { b.shape('rect', 0, 0, 160, 80, 'Search index', 'cyan', { fontSize: 18 }); }),

  // ---- Messaging
  stencil('Messaging', 'Message queue', (b) => {
    b.shape('rect', 0, 0, 230, 90, 'Message queue', 'yellow', { verticalAlign: 'bottom', fontSize: 18 });
    for (let i = 0; i < 4; i++) b.shape('rect', 14 + i * 54, 14, 44, 34, '', 'orange', { strokeWidth: 1 });
  }),
  stencil('Messaging', 'Pub/Sub topic', (b) => {
    const pub = b.shape('rect', 0, 55, 130, 60, 'Publisher', 'green', { fontSize: 18 });
    const topic = b.shape('ellipse', 200, 45, 140, 80, 'Topic', 'yellow');
    const s1 = b.shape('rect', 410, 0, 140, 60, 'Subscriber A', 'blue', { fontSize: 16 });
    const s2 = b.shape('rect', 410, 110, 140, 60, 'Subscriber B', 'blue', { fontSize: 16 });
    b.arrow(pub, topic); b.arrow(topic, s1); b.arrow(topic, s2);
  }),
  stencil('Messaging', 'Event stream (Kafka)', (b) => {
    b.shape('rect', 0, 0, 260, 140, 'Event stream (Kafka)', 'orange', { verticalAlign: 'top', fontSize: 18 });
    ['Partition 0', 'Partition 1', 'Partition 2'].forEach((p, i) => b.shape('rect', 16, 44 + i * 30, 228, 26, p, 'yellow', { fontSize: 14, strokeWidth: 1 }));
  }),
  stencil('Messaging', 'Notification service', (b) => {
    const n = b.shape('rect', 0, 55, 170, 70, 'Notification\nservice', 'red', { fontSize: 18 });
    const e = b.shape('ellipse', 260, 0, 110, 52, 'Email', 'none', SMALL);
    const s = b.shape('ellipse', 260, 64, 110, 52, 'SMS', 'none', SMALL);
    const p = b.shape('ellipse', 260, 128, 110, 52, 'Push', 'none', SMALL);
    b.arrow(n, e); b.arrow(n, s); b.arrow(n, p);
  }),

  // ---- Networking
  stencil('Networking', 'Load balancer', (b) => { b.shape('diamond', 0, 0, 170, 110, 'Load\nbalancer', 'violet', { fontSize: 16 }); }),
  stencil('Networking', 'API gateway', (b) => { b.shape('rect', 0, 0, 170, 80, 'API gateway', 'violet', { fontSize: 18 }); }),
  stencil('Networking', 'CDN', (b) => { b.shape('ellipse', 0, 0, 160, 90, 'CDN', 'cyan'); }),
  stencil('Networking', 'DNS', (b) => { b.shape('ellipse', 0, 0, 140, 80, 'DNS', 'yellow'); }),
  stencil('Networking', 'Reverse proxy', (b) => { b.shape('rect', 0, 0, 170, 80, 'Reverse proxy', 'violet', { fontSize: 18 }); }),
  stencil('Networking', 'Rate limiter', (b) => { b.shape('rect', 0, 0, 160, 80, 'Rate limiter', 'orange', { fontSize: 18 }); }),
  stencil('Networking', 'Firewall', (b) => { b.shape('rect', 0, 0, 160, 80, 'Firewall', 'red', { fillStyle: 'hachure' }); }),

  // ---- Caching
  stencil('Caching', 'Cache (Redis)', (b) => { b.shape('database', 0, 0, 120, 120, 'Cache\n(Redis)', 'red', { fontSize: 16 }); }),

  // ---- Observability
  stencil('Observability', 'Metrics + dashboard', (b) => {
    const m = b.shape('rect', 0, 0, 150, 70, 'Metrics', 'green', { fontSize: 18 });
    const d = b.shape('rect', 240, 0, 150, 70, 'Dashboard', 'blue', { fontSize: 18 });
    b.arrow(m, d);
  }),
  stencil('Observability', 'Logging pipeline', (b) => {
    const a = b.shape('rect', 0, 0, 130, 70, 'App logs', 'green', { fontSize: 16 });
    const l = b.shape('rect', 210, 0, 150, 70, 'Log\naggregator', 'violet', { fontSize: 16 });
    const s = b.shape('database', 440, -20, 110, 110, 'Log store', 'blue', { fontSize: 16 });
    b.arrow(a, l); b.arrow(l, s);
  }),
  stencil('Observability', 'Alerting', (b) => {
    const m = b.shape('rect', 0, 25, 130, 70, 'Metrics', 'green', { fontSize: 16 });
    const t = b.shape('diamond', 200, 0, 160, 120, 'Over\nlimit?', 'yellow', { fontSize: 16 });
    const p = b.shape('ellipse', 430, 25, 140, 70, 'Page on-call', 'red', { fontSize: 16 });
    b.arrow(m, t); b.arrow(t, p, { label: 'yes' });
  }),

  // ---- Patterns
  stencil('Patterns', 'Client > LB > services > DB', (b) => {
    const c = b.shape('ellipse', 0, 125, 120, 70, 'Client', 'yellow', { fontSize: 18 });
    const lb = b.shape('diamond', 190, 100, 150, 120, 'Load\nbalancer', 'violet', { fontSize: 16 });
    const svc = [0, 1, 2].map((i) => b.shape('rect', 430, i * 100, 130, 70, `Service ${i + 1}`, 'green', { fontSize: 16 }));
    const db = b.shape('database', 650, 105, 110, 110, 'Database', 'blue', { fontSize: 16 });
    b.arrow(c, lb);
    svc.forEach((s) => { b.arrow(lb, s); b.arrow(s, db); });
  }),
  stencil('Patterns', 'Cache-aside', (b) => {
    const s = b.shape('rect', 130, 0, 160, 80, 'Service', 'green');
    const c = b.shape('database', 0, 190, 110, 110, 'Cache', 'red', { fontSize: 18 });
    const d = b.shape('database', 310, 190, 110, 110, 'Database', 'blue', { fontSize: 16 });
    b.arrow(s, c, { label: '1. read', startArrowhead: 'arrow' });
    b.arrow(s, d, { label: '2. on miss' });
  }),
  stencil('Patterns', 'Read replicas', (b) => {
    const p = b.shape('database', 130, 0, 120, 120, 'Primary', 'blue', { fontSize: 18 });
    const r1 = b.shape('database', 0, 230, 120, 120, 'Replica 1', 'green', { fontSize: 16 });
    const r2 = b.shape('database', 260, 230, 120, 120, 'Replica 2', 'green', { fontSize: 16 });
    b.arrow(p, r1, { label: 'replicate', strokeStyle: 'dashed' });
    b.arrow(p, r2, { label: 'replicate', strokeStyle: 'dashed' });
  }),
  stencil('Patterns', 'Queue-worker pipeline', (b) => {
    const pr = b.shape('rect', 0, 55, 140, 70, 'Producer', 'green', { fontSize: 18 });
    const q = b.shape('rect', 230, 55, 140, 70, 'Queue', 'yellow', { fontSize: 18 });
    const w1 = b.shape('rect', 460, 0, 140, 60, 'Worker 1', 'violet', { fontSize: 16 });
    const w2 = b.shape('rect', 460, 120, 140, 60, 'Worker 2', 'violet', { fontSize: 16 });
    b.arrow(pr, q); b.arrow(q, w1); b.arrow(q, w2);
  }),
  stencil('Patterns', 'CDN in front of origin', (b) => {
    const u = b.shape('ellipse', 0, 0, 120, 70, 'Users', 'yellow', { fontSize: 18 });
    const c = b.shape('ellipse', 200, 0, 140, 70, 'CDN', 'cyan', { fontSize: 18 });
    const o = b.shape('rect', 420, 0, 150, 70, 'Origin server', 'green', { fontSize: 16 });
    b.arrow(u, c); b.arrow(c, o, { label: 'miss' });
  }),
  stencil('Patterns', 'Sharded database', (b) => {
    const r = b.shape('rect', 0, 105, 150, 70, 'Shard router', 'violet', { fontSize: 16 });
    [0, 1, 2].forEach((i) => {
      const s = b.shape('database', 290, i * 125, 110, 110, `Shard ${i + 1}`, 'blue', { fontSize: 16 });
      b.arrow(r, s, { label: `id % 3 = ${i}` });
    });
  }),
];

export const LIBRARY_CATEGORIES = [...new Set(BUILTIN_LIBRARY.map((i) => i.category))];

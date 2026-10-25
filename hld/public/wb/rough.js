// Seeded hand-drawn ("sketchy") path generation. DOM-free: everything returns SVG path data strings.
// Same element + seed => identical output. roughness 0 = exact geometry, 1 = slight wobble + overshooting double stroke, 2 = strong wobble.

// ---------- numbers and PRNG ----------
export const num = (v) => {
  const r = Math.round(v * 100) / 100;
  if (!Number.isFinite(r)) return 0;
  return Object.is(r, -0) ? 0 : r;
};

// mulberry32: tiny, fast, good enough for jitter.
export function makeRng(seed) {
  let a = (Number.isFinite(seed) ? Math.floor(seed) : 1) >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));
const P = (x, y) => ({ x, y });
const fmt = (p) => `${num(p.x)} ${num(p.y)}`;
const toPoint = (p) => (Array.isArray(p) ? P(p[0], p[1]) : P(p.x, p.y));
const unit = (dx, dy) => { const l = Math.hypot(dx, dy) || 1; return P(dx / l, dy / l); };

// ---------- primitives ----------
// Line segment from the current point `a` to `b`; returns { cmd, end }. With roughness the line bows and the end wobbles.
function segment(rng, a, b, r) {
  if (!r) return { cmd: `L${fmt(b)}`, end: b };
  const len = Math.hypot(b.x - a.x, b.y - a.y);
  const base = Math.min(1.8, 0.35 + len * 0.012) * boost(r);
  const end = P(b.x + (rng() - 0.5) * base * 1.2, b.y + (rng() - 0.5) * base * 1.2);
  const n = unit(-(b.y - a.y), b.x - a.x);
  const bow1 = (rng() - 0.5) * 2 * base;
  const bow2 = (rng() - 0.5) * 2 * base;
  const c1 = P(a.x + (end.x - a.x) * 0.33 + n.x * bow1, a.y + (end.y - a.y) * 0.33 + n.y * bow1);
  const c2 = P(a.x + (end.x - a.x) * 0.67 + n.x * bow2, a.y + (end.y - a.y) * 0.67 + n.y * bow2);
  return { cmd: `C${fmt(c1)} ${fmt(c2)} ${fmt(end)}`, end };
}

const boost = (r) => r * (0.7 + 0.3 * r); // roughness 1 -> 1x, 2 -> 2.6x
const wobblePt = (rng, p, r, amt = 0.8) => (r ? P(p.x + (rng() - 0.5) * amt * boost(r) * 1.5, p.y + (rng() - 0.5) * amt * boost(r) * 1.5) : p);

// Catmull-Rom spline through points -> cubic bezier commands (no leading M). `closed` wraps around.
function splineCommands(pts, closed) {
  const n = pts.length;
  const at = (i) => (closed ? pts[((i % n) + n) % n] : pts[clamp(i, 0, n - 1)]);
  const segs = closed ? n : n - 1;
  let out = '';
  for (let i = 0; i < segs; i++) {
    const p0 = at(i - 1); const p1 = at(i); const p2 = at(i + 1); const p3 = at(i + 2);
    const c1 = P(p1.x + (p2.x - p0.x) / 6, p1.y + (p2.y - p0.y) / 6);
    const c2 = P(p2.x - (p3.x - p1.x) / 6, p2.y - (p3.y - p1.y) / 6);
    out += `C${fmt(c1)} ${fmt(c2)} ${fmt(p2)}`;
  }
  return out;
}

// Points on the smooth curve through `points` (used for hit testing and bounds). Returns [{x,y}].
export function curveSamples(points, perSegment = 12) {
  const pts = points.map(toPoint);
  if (pts.length < 3) return pts;
  const n = pts.length;
  const out = [pts[0]];
  for (let i = 0; i < n - 1; i++) {
    const p0 = pts[Math.max(0, i - 1)]; const p1 = pts[i]; const p2 = pts[i + 1]; const p3 = pts[Math.min(n - 1, i + 2)];
    const c1 = P(p1.x + (p2.x - p0.x) / 6, p1.y + (p2.y - p0.y) / 6);
    const c2 = P(p2.x - (p3.x - p1.x) / 6, p2.y - (p3.y - p1.y) / 6);
    for (let k = 1; k <= perSegment; k++) {
      const t = k / perSegment; const u = 1 - t;
      out.push(P(
        u * u * u * p1.x + 3 * u * u * t * c1.x + 3 * u * t * t * c2.x + t * t * t * p2.x,
        u * u * u * p1.y + 3 * u * u * t * c1.y + 3 * u * t * t * c2.y + t * t * t * p2.y,
      ));
    }
  }
  return out;
}

export function cornerRadius(el) {
  if (el.roundness !== 'round') return 0;
  const shorter = Math.min(el.w, el.h);
  return clamp(shorter * 0.25, 0, el.type === 'diamond' ? 26 : 32);
}

// Polyline / polygon with optionally rounded corners. `closed` draws all corners and returns to the start.
function polyPath(pts, radius, closed, rng, r) {
  const n = pts.length;
  if (n < 2) return '';
  const j = pts.map((p) => wobblePt(rng, p, r, 0.7));
  const corner = (i) => {
    const cur = j[i];
    const prev = j[(i - 1 + n) % n];
    const next = j[(i + 1) % n];
    const dIn = unit(cur.x - prev.x, cur.y - prev.y);
    const dOut = unit(next.x - cur.x, next.y - cur.y);
    const rr = Math.min(radius, Math.hypot(cur.x - prev.x, cur.y - prev.y) / 2, Math.hypot(next.x - cur.x, next.y - cur.y) / 2);
    return { cur, entry: P(cur.x - dIn.x * rr, cur.y - dIn.y * rr), exit: P(cur.x + dOut.x * rr, cur.y + dOut.y * rr), rr };
  };
  const cs = j.map((_, i) => ((closed || (i > 0 && i < n - 1)) && radius > 0 ? corner(i) : { cur: j[i], entry: j[i], exit: j[i], rr: 0 }));
  let d = `M${fmt(cs[0].exit)}`;
  let at = cs[0].exit;
  const steps = closed ? n : n - 1;
  for (let k = 1; k <= steps; k++) {
    const c = cs[k % n];
    const s = segment(rng, at, c.entry, r);
    d += s.cmd; at = s.end;
    if (c.rr > 0 && (closed || k < n - 1)) {
      const exit = k === steps && closed ? cs[0].exit : c.exit;
      d += `Q${fmt(c.cur)} ${fmt(exit)}`; at = exit;
    }
  }
  if (closed && !r) d += 'Z';
  return d;
}

function ellipseSketch(rng, cx, cy, a, b, r, lap) {
  const per = 2 * Math.PI * Math.sqrt((a * a + b * b) / 2);
  const n = clamp(Math.round(per / 22), 12, 40);
  const amp = boost(r) * clamp(0.5 + (a + b) * 0.01, 0.6, 2.4);
  const start = rng() * Math.PI * 2;
  const over = lap === 0 ? 0.05 : 0.11; // radians-fraction of the circle to overshoot
  const count = n + Math.max(1, Math.round(n * over));
  const pts = [];
  for (let i = 0; i <= count; i++) {
    const t = start + (i / n) * Math.PI * 2;
    pts.push(P(cx + (a + (rng() - 0.5) * 2 * amp) * Math.cos(t), cy + (b + (rng() - 0.5) * 2 * amp) * Math.sin(t)));
  }
  return `M${fmt(pts[0])}${splineCommands(pts, false)}`;
}

const ellipseExact = (cx, cy, a, b) => `M${num(cx - a)} ${num(cy)}A${num(a)} ${num(b)} 0 0 1 ${num(cx + a)} ${num(cy)}A${num(a)} ${num(b)} 0 0 1 ${num(cx - a)} ${num(cy)}Z`;

const passes = (r) => (r >= 1 ? 2 : 1);

// ---------- shapes ----------
function polygonOf(el) {
  const { x, y, w, h } = el;
  if (el.type === 'diamond') return [P(x + w / 2, y), P(x + w, y + h / 2), P(x + w / 2, y + h), P(x, y + h / 2)];
  return [P(x, y), P(x + w, y), P(x + w, y + h), P(x, y + h)];
}

export function databaseCap(el) {
  return Math.max(4, Math.min(el.h * 0.15, el.w * 0.25));
}

function databaseSilhouette(el) {
  const { x, y, w, h } = el; const a = w / 2; const ry = databaseCap(el);
  return `M${num(x)} ${num(y + ry)}A${num(a)} ${num(ry)} 0 0 1 ${num(x + w)} ${num(y + ry)}L${num(x + w)} ${num(y + h - ry)}A${num(a)} ${num(ry)} 0 0 1 ${num(x)} ${num(y + h - ry)}Z`;
}

function databaseStroke(el, rng, r) {
  const { x, y, w, h } = el; const a = w / 2; const ry = databaseCap(el); const cx = x + a;
  if (!r) {
    return `${ellipseExact(cx, y + ry, a, ry)}M${num(x)} ${num(y + ry)}L${num(x)} ${num(y + h - ry)}A${num(a)} ${num(ry)} 0 0 0 ${num(x + w)} ${num(y + h - ry)}L${num(x + w)} ${num(y + ry)}`;
  }
  let d = '';
  for (let pass = 0; pass < passes(r); pass++) {
    d += ellipseSketch(rng, cx, y + ry, a, ry, r, pass);
    const wob = r * 0.9;
    const l = segment(rng, P(x, y + ry), P(x, y + h - ry), r);
    d += `M${fmt(wobblePt(rng, P(x, y + ry), r))}${l.cmd}`;
    const bot = [];
    const N = 10;
    for (let i = 0; i <= N; i++) {
      const t = Math.PI - (i / N) * Math.PI; // left -> bottom -> right
      bot.push(P(cx + (a + (rng() - 0.5) * wob) * Math.cos(t), y + h - ry + (ry + (rng() - 0.5) * wob) * Math.sin(t)));
    }
    d += `M${fmt(bot[0])}${splineCommands(bot, false)}`;
    const rr = segment(rng, P(x + w, y + h - ry), P(x + w, y + ry), r);
    d += `M${fmt(wobblePt(rng, P(x + w, y + h - ry), r))}${rr.cmd}`;
  }
  return d;
}

export function shapePath(el) {
  const r = el.roughness || 0;
  const rng = makeRng(el.seed);
  const hasFill = !!el.backgroundColor && el.backgroundColor !== 'transparent' && el.type !== 'frame';
  const fillKind = hasFill ? (el.fillStyle || 'solid') : null;
  const { x, y, w, h } = el;
  let fill; let stroke;
  if (el.type === 'ellipse') {
    fill = ellipseExact(x + w / 2, y + h / 2, w / 2, h / 2);
    if (!r) stroke = fill;
    else {
      stroke = '';
      for (let p = 0; p < passes(r); p++) stroke += ellipseSketch(rng, x + w / 2, y + h / 2, w / 2, h / 2, r, p);
    }
  } else if (el.type === 'database') {
    fill = databaseSilhouette(el);
    stroke = databaseStroke(el, rng, r);
  } else {
    const radius = (el.type === 'rect' || el.type === 'diamond') ? cornerRadius(el) : 0;
    const poly = polygonOf(el);
    fill = polyPath(poly, radius, true, rng, 0);
    if (!r) stroke = fill;
    else {
      stroke = '';
      for (let p = 0; p < passes(r); p++) stroke += polyPath(poly, radius, true, rng, r * (p ? 0.8 : 1));
    }
  }
  return { fill: hasFill ? fill : null, stroke, fillKind, silhouette: fill };
}

// ---------- fill patterns ----------
// Clip the infinite line through `p` with direction `d` to the box; returns [a, b] or null.
function clipLine(p, d, box) {
  let t0 = -Infinity; let t1 = Infinity;
  const test = (pv, dv, lo, hi) => {
    if (Math.abs(dv) < 1e-9) return pv >= lo && pv <= hi;
    let a = (lo - pv) / dv; let b = (hi - pv) / dv;
    if (a > b) [a, b] = [b, a];
    t0 = Math.max(t0, a); t1 = Math.min(t1, b);
    return t0 <= t1;
  };
  if (!test(p.x, d.x, box.x, box.x + box.w) || !test(p.y, d.y, box.y, box.y + box.h)) return null;
  return [P(p.x + d.x * t0, p.y + d.y * t0), P(p.x + d.x * t1, p.y + d.y * t1)];
}

function hatchLines(box, angleDeg, gap, rng, r) {
  const th = (angleDeg * Math.PI) / 180;
  const d = P(Math.cos(th), Math.sin(th));
  const n = P(-d.y, d.x);
  const c = P(box.x + box.w / 2, box.y + box.h / 2);
  const half = (Math.abs(n.x) * box.w + Math.abs(n.y) * box.h) / 2;
  const lines = [];
  const first = Math.ceil(-half / gap) * gap + gap / 2 * 0; // keep lines symmetric around the centre
  for (let t = first; t <= half; t += gap) {
    const seg = clipLine(P(c.x + n.x * t, c.y + n.y * t), d, box);
    if (!seg) continue;
    if (Math.hypot(seg[1].x - seg[0].x, seg[1].y - seg[0].y) < 0.5) continue;
    lines.push(seg);
  }
  return lines.map(([a, b]) => {
    if (!r) return [a, b];
    const e = (v) => (rng() - 0.5) * r * 1.6 + v;
    return [P(e(a.x), e(a.y)), P(e(b.x), e(b.y))];
  });
}

export function fillPattern(el) {
  const kind = el.fillStyle || 'solid';
  const sp = shapePath({ ...el, backgroundColor: el.backgroundColor && el.backgroundColor !== 'transparent' ? el.backgroundColor : '#000' });
  if (kind === 'solid') return { kind: 'solid', d: sp.silhouette };
  const box = { x: el.x, y: el.y, w: Math.max(el.w, 1), h: Math.max(el.h, 1) };
  const r = el.roughness || 0;
  const rng = makeRng((el.seed >>> 0) ^ 0x9e3779b9);
  const gap = Math.max((el.strokeWidth || 2) * 4, 6);
  const line = (a, b) => `M${fmt(a)}${segment(rng, a, b, r * 0.6).cmd}`;
  if (kind === 'zigzag') {
    const lines = hatchLines(box, -41, gap, rng, r);
    let d = '';
    lines.forEach(([a, b], i) => {
      const [s, e] = i % 2 ? [b, a] : [a, b];
      d += i === 0 ? `M${fmt(s)}L${fmt(e)}` : `L${fmt(s)}L${fmt(e)}`;
    });
    return { kind, d };
  }
  let d = hatchLines(box, -41, gap, rng, r).map(([a, b]) => line(a, b)).join('');
  if (kind === 'cross-hatch') d += hatchLines(box, 41, gap, rng, r).map(([a, b]) => line(a, b)).join('');
  return { kind: kind === 'cross-hatch' ? 'cross-hatch' : 'hachure', d };
}

// ---------- lines and arrows ----------
function orthogonalize(pts) {
  const out = [pts[0]];
  for (let i = 1; i < pts.length; i++) {
    const a = out[out.length - 1]; const b = pts[i];
    if (Math.abs(a.x - b.x) > 0.5 && Math.abs(a.y - b.y) > 0.5) out.push(P(b.x, a.y));
    out.push(b);
  }
  return out;
}

export function linearPath(points, { arrowType = 'sharp', roughness = 0, seed = 1 } = {}) {
  let pts = (points || []).map(toPoint);
  if (pts.length < 2) return '';
  const r = roughness || 0;
  const rng = makeRng(seed);
  let d = '';
  const np = passes(r);
  if (arrowType === 'round' && pts.length >= 3) {
    for (let p = 0; p < np; p++) {
      const j = pts.map((q, i) => (i === 0 || i === pts.length - 1 ? wobblePt(rng, q, r, 0.5) : wobblePt(rng, q, r, 1)));
      d += `M${fmt(j[0])}${splineCommands(j, false)}`;
    }
    return d;
  }
  if (arrowType === 'elbow') {
    pts = orthogonalize(pts);
    for (let p = 0; p < np; p++) d += polyPath(pts, 10, false, rng, r * 0.6);
    return d;
  }
  for (let p = 0; p < np; p++) d += polyPath(pts, 0, false, rng, r);
  return d;
}

// ---------- freedraw ----------
export function freedrawPath(points, strokeWidth = 2) {
  let raw = (points || []).map(toPoint);
  if (!raw.length) return '';
  const maxW = strokeWidth * 1.6 + 0.8; // full width in the middle of a stroke
  const circle = (c, rad) => `M${num(c.x - rad)} ${num(c.y)}A${num(rad)} ${num(rad)} 0 1 0 ${num(c.x + rad)} ${num(c.y)}A${num(rad)} ${num(rad)} 0 1 0 ${num(c.x - rad)} ${num(c.y)}Z`;
  // drop near-duplicates
  const pts = [raw[0]];
  for (let i = 1; i < raw.length; i++) {
    const last = pts[pts.length - 1];
    if (Math.hypot(raw[i].x - last.x, raw[i].y - last.y) >= 0.6) pts.push(raw[i]);
  }
  if (pts.length === 1) return circle(pts[0], maxW / 2 * 0.9);
  // light smoothing (moving average) keeping the end points
  const sm = pts.map((p, i) => {
    if (i === 0 || i === pts.length - 1) return p;
    const a = pts[i - 1]; const b = pts[i + 1];
    return P((a.x + 2 * p.x + b.x) / 4, (a.y + 2 * p.y + b.y) / 4);
  });
  const n = sm.length;
  const dist = [0];
  for (let i = 1; i < n; i++) dist.push(dist[i - 1] + Math.hypot(sm[i].x - sm[i - 1].x, sm[i].y - sm[i - 1].y));
  const total = dist[n - 1];
  const taper = Math.min(total / 2.5, 14 + strokeWidth * 2);
  const half = sm.map((_, i) => {
    const e = Math.min(dist[i], total - dist[i]);
    const k = taper > 0 ? Math.min(1, e / taper) : 1;
    const seg = i === 0 ? dist[1] - dist[0] : dist[i] - dist[i - 1];
    const speed = clamp(seg / 12, 0, 1); // longer steps = faster = slightly thinner
    return (maxW / 2) * (0.3 + 0.7 * Math.sin((k * Math.PI) / 2)) * (1 - 0.22 * speed);
  });
  // smooth the widths
  for (let pass = 0; pass < 2; pass++) for (let i = 1; i < n - 1; i++) half[i] = (half[i - 1] + 2 * half[i] + half[i + 1]) / 4;
  const left = []; const right = [];
  for (let i = 0; i < n; i++) {
    const a = sm[Math.max(0, i - 1)]; const b = sm[Math.min(n - 1, i + 1)];
    const t = unit(b.x - a.x, b.y - a.y);
    const nx = -t.y; const ny = t.x;
    left.push(P(sm[i].x + nx * half[i], sm[i].y + ny * half[i]));
    right.push(P(sm[i].x - nx * half[i], sm[i].y - ny * half[i]));
  }
  const smoothTo = (arr) => {
    let s = '';
    for (let i = 1; i < arr.length - 1; i++) {
      const mid = P((arr[i].x + arr[i + 1].x) / 2, (arr[i].y + arr[i + 1].y) / 2);
      s += `Q${fmt(arr[i])} ${fmt(mid)}`;
    }
    return s + `L${fmt(arr[arr.length - 1])}`;
  };
  const rr = right.slice().reverse();
  const endR = Math.max(half[n - 1], 0.5); const startR = Math.max(half[0], 0.5);
  return `M${fmt(left[0])}${smoothTo(left)}A${num(endR)} ${num(endR)} 0 0 1 ${fmt(rr[0])}${smoothTo(rr)}A${num(startR)} ${num(startR)} 0 0 1 ${fmt(left[0])}Z`;
}

// ---------- arrowheads ----------
// `direction` = radians of the way the head points (towards the tip). Returns { d, fill, inset }: `inset` is how far the line body
// should stop short of the tip for closed heads so the line does not show through them.
export function arrowheadPath(kind, tip, direction, strokeWidth = 2) {
  const t = toPoint(tip);
  const L = 10 + 3 * strokeWidth;
  const ux = Math.cos(direction); const uy = Math.sin(direction);
  const at = (back, side) => P(t.x - ux * back + -uy * side, t.y - uy * back + ux * side);
  const poly = (...ps) => `M${ps.map(fmt).join('L')}`;
  switch (kind) {
    case 'arrow': return { d: `${poly(at(L, -L * 0.45), t, at(L, L * 0.45))}`, fill: false, inset: 0 };
    case 'triangle': return { d: `${poly(t, at(L, -L * 0.42), at(L, L * 0.42))}Z`, fill: true, inset: L * 0.9 };
    case 'triangle_outline': return { d: `${poly(t, at(L, -L * 0.42), at(L, L * 0.42))}Z`, fill: false, inset: L * 0.9 };
    case 'circle': case 'circle_outline': {
      const rad = L * 0.32; const c = at(rad, 0);
      return { d: `M${num(c.x - rad)} ${num(c.y)}A${num(rad)} ${num(rad)} 0 1 0 ${num(c.x + rad)} ${num(c.y)}A${num(rad)} ${num(rad)} 0 1 0 ${num(c.x - rad)} ${num(c.y)}Z`, fill: kind === 'circle', inset: rad * 2 };
    }
    case 'diamond': case 'diamond_outline':
      return { d: `${poly(t, at(L * 0.55, -L * 0.34), at(L * 1.1, 0), at(L * 0.55, L * 0.34))}Z`, fill: kind === 'diamond', inset: L * 1.1 };
    case 'bar': return { d: poly(at(0, -L * 0.45), at(0, L * 0.45)), fill: false, inset: 0 };
    case 'crowfoot_one': return { d: `${poly(at(L * 0.35, -L * 0.45), at(L * 0.35, L * 0.45))}${poly(at(L * 0.7, -L * 0.45), at(L * 0.7, L * 0.45))}`, fill: false, inset: 0 };
    case 'crowfoot_many': return { d: `${poly(at(L, 0), at(0, -L * 0.5))}${poly(at(L, 0), t)}${poly(at(L, 0), at(0, L * 0.5))}`, fill: false, inset: 0 };
    case 'crowfoot_one_or_many':
      return { d: `${poly(at(L, 0), at(0, -L * 0.5))}${poly(at(L, 0), t)}${poly(at(L, 0), at(0, L * 0.5))}${poly(at(L + 5, -L * 0.45), at(L + 5, L * 0.45))}`, fill: false, inset: 0 };
    default: return { d: '', fill: false, inset: 0 };
  }
}

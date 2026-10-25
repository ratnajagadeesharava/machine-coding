import test from 'node:test';
import assert from 'node:assert/strict';
import { createElement, ARROWHEADS, FILL_STYLES } from '../shared/board.js';
import { arrowheadPath, fillPattern, freedrawPath, linearPath, makeRng, shapePath, curveSamples } from '../public/wb/rough.js';

const SHAPES = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'frame'];
const mk = (type, props = {}) => createElement(type, { id: 1, x: 10, y: 20, w: 140, h: 90, seed: 42, backgroundColor: '#a5d8ff', ...props });
const validPath = (d) => /^[MLCQAZ0-9\s.,eE+-]+$/.test(d) && !/NaN|undefined|Infinity/.test(d) && /^M/.test(d);

test('prng is deterministic and seed dependent', () => {
  const a = makeRng(7); const b = makeRng(7); const c = makeRng(8);
  const sa = [a(), a(), a()]; assert.deepEqual(sa, [b(), b(), b()]);
  assert.notDeepEqual(sa, [c(), c(), c()]);
  assert.ok(sa.every((v) => v >= 0 && v < 1));
});

test('shapePath is deterministic per seed and differs between seeds and roughness', () => {
  for (const t of SHAPES) for (const r of [0, 1, 2]) {
    const el = mk(t, { roughness: r });
    const a = shapePath(el); const b = shapePath({ ...el });
    assert.deepEqual(a, b, `${t} r${r} deterministic`);
    assert.ok(validPath(a.stroke), `${t} r${r} stroke valid: ${a.stroke.slice(0, 60)}`);
    if (t !== 'frame') assert.ok(validPath(a.fill));
    if (r > 0) assert.notEqual(shapePath({ ...el, seed: 43 }).stroke, a.stroke, `${t} seed matters`);
  }
  const clean = shapePath(mk('rect', { roughness: 0, seed: 1 })).stroke;
  assert.equal(clean, shapePath(mk('rect', { roughness: 0, seed: 99 })).stroke, 'roughness 0 ignores the seed');
  assert.notEqual(shapePath(mk('rect', { roughness: 1 })).stroke, shapePath(mk('rect', { roughness: 2 })).stroke);
});

test('fill is null for transparent and for frames; fillKind follows fillStyle', () => {
  assert.equal(shapePath(mk('rect', { backgroundColor: 'transparent' })).fill, null);
  assert.equal(shapePath(mk('frame')).fill, null);
  for (const f of FILL_STYLES) assert.equal(shapePath(mk('ellipse', { fillStyle: f })).fillKind, f);
});

test('rounded corners only for round rect/diamond; sticky is always sharp', () => {
  const sharp = shapePath(mk('rect', { roundness: 'sharp', roughness: 0 })).stroke;
  const round = shapePath(mk('rect', { roundness: 'round', roughness: 0 })).stroke;
  assert.ok(!sharp.includes('Q') && round.includes('Q'));
  assert.ok(!shapePath(mk('sticky', { roundness: 'round', roughness: 0 })).stroke.includes('Q'));
  assert.ok(shapePath(mk('diamond', { roundness: 'round', roughness: 0 })).stroke.includes('Q'));
});

test('ellipse uses arcs when clean; database has cap ellipse, sides and bottom arc', () => {
  assert.match(shapePath(mk('ellipse', { roughness: 0 })).stroke, /A/);
  const db = shapePath(mk('database', { roughness: 0, w: 100, h: 120 })).stroke;
  assert.equal((db.match(/A/g) || []).length, 3); // two halves of the top ellipse + the bottom arc
  assert.ok(db.includes('L'));
});

test('fill patterns: solid, hachure, cross-hatch (more lines), zigzag (one continuous path)', () => {
  const el = mk('rect', { roughness: 0, strokeWidth: 2 });
  const solid = fillPattern({ ...el, fillStyle: 'solid' });
  assert.equal(solid.kind, 'solid'); assert.ok(solid.d.length > 0);
  const h = fillPattern({ ...el, fillStyle: 'hachure' });
  const x = fillPattern({ ...el, fillStyle: 'cross-hatch' });
  const z = fillPattern({ ...el, fillStyle: 'zigzag' });
  assert.equal(h.kind, 'hachure'); assert.equal(x.kind, 'cross-hatch'); assert.equal(z.kind, 'zigzag');
  const count = (d) => (d.match(/M/g) || []).length;
  assert.ok(count(h.d) > 5);
  assert.ok(count(x.d) > count(h.d));
  assert.equal(count(z.d), 1);
  for (const p of [h, x, z]) assert.ok(validPath(p.d));
});

test('hachure gap follows stroke width and lines run at about -41 degrees inside the box', () => {
  const base = mk('rect', { roughness: 0, fillStyle: 'hachure' });
  const nThin = (fillPattern({ ...base, strokeWidth: 1 }).d.match(/M/g) || []).length;
  const nBold = (fillPattern({ ...base, strokeWidth: 4 }).d.match(/M/g) || []).length;
  assert.ok(nThin > nBold, 'gap max(sw*4, 6): 1 -> 6, 4 -> 16');
  const d = fillPattern({ ...base, strokeWidth: 2 }).d;
  const nums = [...d.matchAll(/M(-?[\d.]+) (-?[\d.]+)L(-?[\d.]+) (-?[\d.]+)/g)].map((m) => m.slice(1).map(Number));
  assert.ok(nums.length > 3);
  for (const [x1, y1, x2, y2] of nums) {
    assert.ok(Math.min(x1, x2) >= 10 - 0.01 && Math.max(x1, x2) <= 150.01 && Math.min(y1, y2) >= 20 - 0.01 && Math.max(y1, y2) <= 110.01);
    const ang = (Math.atan2(y2 - y1, x2 - x1) * 180) / Math.PI;
    const norm = ((ang % 180) + 180) % 180; // direction ignoring orientation
    assert.ok(Math.abs(norm - 139) < 0.5, `angle ${ang}`);
  }
});

test('linearPath: sharp polyline, round curve, elbow with corner radius; deterministic', () => {
  const pts = [[0, 0], [60, 10], [100, 80]];
  const sharp = linearPath(pts, { arrowType: 'sharp', roughness: 0, seed: 1 });
  assert.equal(sharp, 'M0 0L60 10L100 80');
  const round = linearPath(pts, { arrowType: 'round', roughness: 0, seed: 1 });
  assert.ok(round.includes('C') && !round.includes('L'));
  const elbow = linearPath([{ x: 0, y: 0 }, { x: 50, y: 0 }, { x: 50, y: 60 }], { arrowType: 'elbow', roughness: 0, seed: 1 });
  assert.ok(elbow.includes('Q') && elbow.includes('L'));
  const twoPoint = linearPath([[0, 0], [80, 40]], { arrowType: 'elbow', roughness: 0, seed: 1 });
  assert.ok(twoPoint.includes('Q'), 'a diagonal elbow is made orthogonal');
  for (const r of [0, 1, 2]) for (const arrowType of ['sharp', 'round', 'elbow']) {
    const a = linearPath(pts, { arrowType, roughness: r, seed: 5 });
    assert.equal(a, linearPath(pts, { arrowType, roughness: r, seed: 5 }));
    assert.ok(validPath(a));
  }
  assert.notEqual(linearPath(pts, { arrowType: 'sharp', roughness: 1, seed: 5 }), linearPath(pts, { arrowType: 'sharp', roughness: 1, seed: 6 }));
  assert.equal(linearPath([[0, 0]], {}), '');
  // artist roughness draws a second, overshooting stroke
  assert.equal((linearPath(pts, { arrowType: 'sharp', roughness: 1, seed: 5 }).match(/M/g) || []).length, 2);
});

test('curveSamples passes through the points', () => {
  const s = curveSamples([[0, 0], [50, 50], [100, 0]], 8);
  assert.deepEqual(s[0], { x: 0, y: 0 });
  assert.ok(s.some((p) => Math.abs(p.x - 50) < 1e-6 && Math.abs(p.y - 50) < 1e-6));
});

test('freedrawPath: closed, filled outline for 1, 2 and many points; thinner at the ends', () => {
  assert.ok(freedrawPath([[0, 0]], 2).endsWith('Z'));
  assert.ok(validPath(freedrawPath([[0, 0], [30, 0]], 2)));
  const pts = Array.from({ length: 50 }, (_, i) => [i * 4, Math.sin(i / 5) * 15]);
  const d = freedrawPath(pts, 2);
  assert.ok(validPath(d) && d.endsWith('Z'));
  assert.equal(d, freedrawPath(pts, 2));
  assert.equal(freedrawPath([], 2), '');
  // the end cap radius (A command) is smaller than the full half-width
  const arcs = [...d.matchAll(/A([\d.]+) /g)].map((m) => Number(m[1]));
  assert.ok(arcs.length === 2 && arcs.every((r) => r < 2 * 1.6 / 2 + 0.5));
});

test('every arrowhead kind returns path data scaled by stroke width and oriented along direction', () => {
  const kinds = ARROWHEADS.map((a) => a.id).filter(Boolean);
  assert.equal(kinds.length, 11);
  const bbox = (d) => { const n = [...d.matchAll(/(-?[\d.]+) (-?[\d.]+)/g)].map((m) => [Number(m[1]), Number(m[2])]); return { w: Math.max(...n.map((p) => p[0])) - Math.min(...n.map((p) => p[0])), h: Math.max(...n.map((p) => p[1])) - Math.min(...n.map((p) => p[1])) }; };
  for (const k of kinds) {
    const h = arrowheadPath(k, { x: 100, y: 50 }, 0, 2);
    assert.ok(h.d.length > 0 && typeof h.fill === 'boolean', k);
    assert.ok(validPath(h.d), k);
    const big = arrowheadPath(k, { x: 100, y: 50 }, 0, 4);
    if (k.startsWith('circle')) assert.notEqual(big.d, h.d); else assert.ok(bbox(big.d).h > bbox(h.d).h, `${k} scales with stroke width`);
    const rotated = arrowheadPath(k, { x: 100, y: 50 }, Math.PI / 2, 2);
    assert.notEqual(rotated.d, h.d, `${k} follows direction`);
    // the head sits behind the tip, never in front of it
    const xs = [...h.d.matchAll(/(-?[\d.]+) (-?[\d.]+)/g)].map((m) => Number(m[1]));
    assert.ok(Math.max(...xs) <= 100.01, `${k} not past tip`);
  }
  assert.equal(arrowheadPath('triangle', { x: 0, y: 0 }, 0, 2).fill, true);
  assert.equal(arrowheadPath('triangle_outline', { x: 0, y: 0 }, 0, 2).fill, false);
  assert.equal(arrowheadPath('circle', { x: 0, y: 0 }, 0, 2).fill, true);
  assert.equal(arrowheadPath('arrow', { x: 0, y: 0 }, 0, 2).fill, false);
  assert.equal(arrowheadPath('nonsense', { x: 0, y: 0 }, 0, 2).d, '');
});

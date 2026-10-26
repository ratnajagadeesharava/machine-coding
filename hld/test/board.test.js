import assert from 'node:assert/strict';
import test from 'node:test';
import {
  boardSignature, boardStats, cloneElements, createElement, describeBoard, edgePoint, elementBounds, elbowRoute, emptyBoard, LIMITS,
  newId, normalizePoints, resolvePoints, rotatePoint, sanitizeBoard, selectionBounds, sideAnchor, toAiBoard,
} from '../shared/board.js';

function add(board, type, props = {}) {
  const el = createElement(type, { id: newId(board), ...props });
  board.items.push(el);
  return el;
}
const arrowBetween = (board, a, b, extra = {}) => add(board, 'arrow', { x: a.x + a.w, y: a.y + a.h / 2, points: [[0, 0], [b.x - (a.x + a.w), 0]], from: a.id, to: b.id, ...extra });

function sample() {
  const b = emptyBoard();
  const client = add(b, 'ellipse', { x: 0, y: 100, text: 'Client' });
  const api = add(b, 'rect', { x: 300, y: 100, text: 'API Server' });
  const db = add(b, 'database', { x: 600, y: 100, text: 'Links DB' });
  add(b, 'rect', { x: 300, y: 300, text: 'Cache' });
  add(b, 'text', { x: 20, y: 400, text: 'reads 100x writes' });
  arrowBetween(b, client, api, { text: 'HTTPS' });
  arrowBetween(b, api, db);
  add(b, 'arrow', { x: 400, y: 150, points: [[0, 0], [0, 100]], from: api.id, text: 'to nowhere' });
  return b;
}

test('empty board is described as empty, with zeroed stats', () => {
  assert.equal(describeBoard(emptyBoard()), 'The whiteboard is empty.');
  assert.deepEqual(boardStats(emptyBoard()), { components: 0, connections: 0, arrows: 0, lines: 0, notes: 0, frames: 0, images: 0, sketches: 0 });
  assert.equal(emptyBoard().version, 2);
});

test('createElement gives each type sensible defaults and respects overrides', () => {
  for (const type of ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text', 'image', 'frame', 'freedraw', 'line', 'arrow']) {
    const el = createElement(type, { id: 1 });
    assert.equal(el.type, type);
    assert.equal(el.angle, 0);
    assert.deepEqual(el.groupIds, []);
    assert.equal(el.locked, false);
    assert.ok(Number.isInteger(el.seed));
  }
  assert.equal(createElement('sticky', { id: 1 }).backgroundColor, '#ffec99');
  assert.equal(createElement('arrow', { id: 1 }).endArrowhead, 'arrow');
  assert.equal(createElement('line', { id: 1 }).endArrowhead, null);
  assert.equal(createElement('rect', { id: 1, strokeColor: '#ff0000' }).strokeColor, '#ff0000');
  assert.throws(() => createElement('banana'), /unknown element type/);
});

test('describeBoard lists components, connections, loose arrows, isolated components and notes', () => {
  const text = describeBoard(sample());
  assert.match(text, /4 component\(s\), 2 connection\(s\), 1 note\(s\)\./);
  assert.match(text, /"Client" - oval/);
  assert.match(text, /"Links DB" - database/);
  assert.match(text, /"Client" -> "API Server" \[label: "HTTPS"\]/);
  assert.match(text, /"API Server" -> "Links DB"/);
  assert.match(text, /loose end[^\n]*\n\s+"API Server" -> \(nothing\) \[label: "to nowhere"\]/);
  assert.match(text, /Components with no arrows attached: "Cache"/);
  assert.match(text, /"reads 100x writes"/);
});

test('describeBoard understands arrowheads, diamonds, stickies, frames, lines, sketches and images', () => {
  const b = emptyBoard();
  const frame = add(b, 'frame', { x: 0, y: 0, w: 700, h: 400, name: 'Backend' });
  const lb = add(b, 'diamond', { x: 50, y: 50, text: 'LB', frameId: frame.id });
  const svc = add(b, 'rect', { x: 300, y: 50, text: 'Service', frameId: frame.id });
  const q = add(b, 'rect', { x: 300, y: 250, text: 'Queue' });
  arrowBetween(b, lb, svc, { startArrowhead: 'arrow' }); // <->
  add(b, 'arrow', { x: 380, y: 140, points: [[0, 0], [0, 110]], from: svc.id, to: q.id, endArrowhead: null }); // plain link
  add(b, 'arrow', { x: 380, y: 140, points: [[0, 0], [0, 110]], from: q.id, to: svc.id, startArrowhead: 'arrow', endArrowhead: null }); // <-
  add(b, 'sticky', { x: 500, y: 300, text: 'Remember to shard' });
  add(b, 'line', { x: 0, y: 500, points: [[0, 0], [100, 0]] });
  add(b, 'freedraw', { x: 0, y: 600, points: [[0, 0], [5, 5], [9, 3]] });
  add(b, 'image', { x: 0, y: 700, dataUrl: 'data:image/png;base64,AAAA' });
  const text = describeBoard(b);
  assert.match(text, /"LB" - diamond[^\n]*inside frame "Backend"/);
  assert.match(text, /Frames \(labelled regions\):\n\s+"Backend" contains "LB", "Service"/);
  assert.match(text, /"LB" <-> "Service"/);
  assert.match(text, /"Service" -- "Queue"/);
  assert.match(text, /"Queue" <- "Service"/);
  assert.match(text, /\[sticky\] "Remember to shard"/);
  assert.match(text, /1 plain line\(s\)/);
  assert.match(text, /1 freehand sketch\(es\), 1 image\(s\)/);
});

test('boardStats counts only fully attached arrows as connections', () => {
  assert.deepEqual(boardStats(sample()), { components: 4, connections: 2, arrows: 3, lines: 0, notes: 1, frames: 0, images: 0, sketches: 0 });
});

test('edgePoint leaves rectangles, ellipses and diamonds on their boundary, even rotated', () => {
  const r = createElement('rect', { id: 1, x: 0, y: 0, w: 100, h: 60 });
  assert.deepEqual(edgePoint(r, 300, 30), { x: 100, y: 30 });
  assert.deepEqual(edgePoint(r, 50, -200), { x: 50, y: 0 });
  assert.deepEqual(edgePoint(r, 50, 30), { x: 50, y: 30 }, 'target at the centre returns the centre');
  const e = createElement('ellipse', { id: 2, x: 0, y: 0, w: 100, h: 60 });
  const pe = edgePoint(e, 300, 30);
  assert.ok(Math.abs(pe.x - 100) < 1e-9 && Math.abs(pe.y - 30) < 1e-9);
  const d = createElement('diamond', { id: 3, x: 0, y: 0, w: 100, h: 60 });
  const pd = edgePoint(d, 300, 30);
  assert.ok(Math.abs(pd.x - 100) < 1e-9 && Math.abs(pd.y - 30) < 1e-9, 'right vertex');
  const pd2 = edgePoint(d, 150, 30 - 90); // up-right diagonal hits the slanted edge, not the bounding box
  assert.ok(pd2.x < 100 && pd2.y > 0);
  const rot = createElement('rect', { id: 4, x: 0, y: 0, w: 100, h: 20, angle: Math.PI / 2 });
  const pr = edgePoint(rot, 50, 500); // rotated 90deg: the long side now faces up/down
  assert.ok(Math.abs(pr.x - 50) < 1e-6 && Math.abs(pr.y - 60) < 1e-6, JSON.stringify(pr));
  const gap = edgePoint(r, 300, 30, 10);
  assert.ok(Math.abs(gap.x - 110) < 1e-9);
});

test('resolvePoints follows bound shapes, keeps free ends, and re-routes elbow arrows', () => {
  const b = sample();
  const [client, api, , , , a1, , loose] = b.items;
  const p = resolvePoints(b, a1);
  assert.ok(p[0].x > 125 && p[0].x <= 140, `starts on the right edge of the ellipse: ${p[0].x}`);
  assert.ok(Math.abs(p[1].x - 300) < 1e-6, `ends on the left edge of API: ${p[1].x}`);
  api.x += 100; // moving a bound shape moves the arrow end without touching the arrow
  assert.ok(Math.abs(resolvePoints(b, a1)[1].x - 400) < 1e-6);
  const free = resolvePoints(b, loose);
  assert.equal(free[1].y, 250);
  assert.ok(client);
  const e = add(b, 'arrow', { x: 0, y: 0, points: [[0, 0], [10, 10]], arrowType: 'elbow', from: client.id, to: api.id });
  const route = resolvePoints(b, e);
  assert.ok(route.length >= 3, 'elbow routes have corners');
  for (let i = 1; i < route.length; i++) assert.ok(route[i].x === route[i - 1].x || route[i].y === route[i - 1].y, 'segments are axis-aligned');
});

test('elbowRoute produces orthogonal paths for every side combination', () => {
  const s = { x: 0, y: 0 };
  const e = { x: 100, y: 60 };
  for (const [a, c] of [['right', 'left'], ['bottom', 'top'], ['right', 'top'], ['bottom', 'left']]) {
    const r = elbowRoute(s, e, a, c);
    assert.deepEqual(r[0], s);
    assert.deepEqual(r[r.length - 1], e);
    for (let i = 1; i < r.length; i++) assert.ok(r[i].x === r[i - 1].x || r[i].y === r[i - 1].y);
  }
  assert.deepEqual(sideAnchor(createElement('rect', { id: 1, x: 0, y: 0, w: 100, h: 50 }), 'bottom'), { x: 50, y: 50 });
});

test('elementBounds accounts for rotation and linear points; selectionBounds unions them', () => {
  const r = createElement('rect', { id: 1, x: 0, y: 0, w: 100, h: 20, angle: Math.PI / 2 });
  const b = elementBounds(emptyBoard(), r);
  assert.ok(Math.abs(b.w - 20) < 1e-6 && Math.abs(b.h - 100) < 1e-6);
  const line = createElement('line', { id: 2, x: 10, y: 10, points: [[0, 0], [50, 20], [-5, 40]] });
  assert.deepEqual(elementBounds(emptyBoard(), line), { x: 5, y: 10, w: 55, h: 40 });
  const u = selectionBounds(emptyBoard(), [createElement('rect', { id: 3, x: 0, y: 0, w: 10, h: 10 }), createElement('rect', { id: 4, x: 100, y: 50, w: 10, h: 10 })]);
  assert.deepEqual(u, { x: 0, y: 0, w: 110, h: 60 });
  assert.equal(selectionBounds(emptyBoard(), []), null);
});

test('normalizePoints re-anchors points so the first is [0,0] and refreshes w/h', () => {
  const el = createElement('line', { id: 1, x: 100, y: 100, points: [[10, 5], [60, 25], [30, -15]] });
  normalizePoints(el);
  assert.equal(el.x, 110);
  assert.equal(el.y, 105);
  assert.deepEqual(el.points[0], [0, 0]);
  assert.deepEqual(el.points[1], [50, 20]);
  assert.equal(el.w, 50);
  assert.equal(el.h, 40);
});

test('cloneElements assigns fresh ids and remaps bindings, groups and frames', () => {
  const b = emptyBoard();
  const f = add(b, 'frame', { x: 0, y: 0, w: 400, h: 300 });
  const a = add(b, 'rect', { x: 10, y: 10, groupIds: [77], frameId: f.id });
  const c = add(b, 'rect', { x: 200, y: 10, groupIds: [77], frameId: f.id });
  const outside = add(b, 'rect', { x: 600, y: 10 });
  const ar = arrowBetween(b, a, c);
  const toOutside = arrowBetween(b, a, outside);
  b.nextId = 100;
  const copies = cloneElements(b, [f, a, c, ar, toOutside], { dx: 30, dy: 40 });
  const ids = copies.map((e) => e.id);
  assert.equal(new Set(ids).size, ids.length);
  assert.ok(ids.every((id) => id >= 100));
  const [cf, ca, cc, car, cto] = copies;
  assert.equal(ca.frameId, cf.id);
  assert.deepEqual(ca.groupIds, cc.groupIds);
  assert.notDeepEqual(ca.groupIds, [77]);
  assert.equal(car.from, ca.id);
  assert.equal(car.to, cc.id);
  assert.equal(cto.from, ca.id);
  assert.equal(cto.to, null, 'binding to an element outside the copy is dropped');
  assert.equal(ca.x, a.x + 30);
  assert.equal(ca.y, a.y + 40);
  const standalone = cloneElements(b, [ar], { dx: 5, dy: 5 })[0];
  assert.equal(standalone.from, null);
  assert.equal(standalone.to, null);
  assert.ok(Math.abs(standalone.x - (resolvePoints(b, ar)[0].x + 5)) < 1e-6, 'free ends keep their drawn position');
});

test('sanitizeBoard tolerates garbage and normalises values', () => {
  assert.deepEqual(sanitizeBoard(null), emptyBoard());
  assert.deepEqual(sanitizeBoard('x'), emptyBoard());
  assert.deepEqual(sanitizeBoard({ items: 'no' }), emptyBoard());
  const b = sanitizeBoard({
    nextId: 'x',
    items: [
      { id: 1, type: 'rect', x: NaN, y: 5, w: -3, h: 1e9, text: 'a'.repeat(5000), strokeColor: 'url(javascript:alert(1))', fillStyle: 'weird', strokeWidth: 3, opacity: 900, angle: 'q' },
      { id: 1, type: 'rect', x: 0, y: 0, w: 10, h: 10 },
      { id: 2, type: 'banana', x: 0, y: 0 },
      { id: 3, type: 'arrow', from: 1, to: 99, points: [[0, 0], ['a', 5], [10, 10]], endArrowhead: 'laser' },
      { id: 4, type: 'image', x: 0, y: 0, w: 50, h: 50, dataUrl: 'javascript:alert(1)' },
      { id: 5, type: 'image', x: 0, y: 0, w: 50, h: 50, dataUrl: 'data:image/png;base64,iVBORw0KGgo=' },
      { id: 6, type: 'text', x: 0, y: 0, text: '<script>', link: 'javascript:x', frameId: 999, groupIds: [1, 'a', 3] },
      null, 5,
    ],
  });
  assert.deepEqual(b.items.map((e) => e.id), [1, 3, 4, 5, 6]);
  const [r, a, imgBad, imgOk, t] = b.items;
  assert.equal(r.x, 0);
  assert.equal(r.w, 12, 'minimum size');
  assert.equal(r.h, LIMITS.size);
  assert.equal(r.text.length, LIMITS.label);
  assert.equal(r.strokeColor, '#1e1e1e', 'unsafe colour replaced');
  assert.equal(r.fillStyle, 'solid');
  assert.equal(r.strokeWidth, 2);
  assert.equal(r.opacity, 100);
  assert.equal(r.angle, 0);
  assert.equal(a.from, 1);
  assert.equal(a.to, null, 'binding to a missing shape is dropped');
  assert.equal(a.points.length, 2, 'invalid points are dropped');
  assert.equal(a.endArrowhead, null, 'unknown arrowhead dropped');
  assert.equal(imgBad.dataUrl, '');
  assert.ok(imgOk.dataUrl.startsWith('data:image/png'));
  assert.equal(t.link, null);
  assert.equal(t.frameId, null);
  assert.deepEqual(t.groupIds, [1, 3]);
  assert.equal(t.text, '<script>', 'text is data; escaping happens at render time');
  assert.ok(b.nextId > 6);
});

test('sanitizeBoard upgrades version 1 boards (straight arrows with x1..y2)', () => {
  const b = sanitizeBoard({
    nextId: 3,
    items: [{ id: 1, type: 'rect', x: 0, y: 0, w: 100, h: 60, text: 'A' }, { id: 2, type: 'arrow', from: 1, to: null, x1: 100, y1: 30, x2: 200, y2: 50, text: 'x' }],
  });
  assert.equal(b.version, 2);
  const a = b.items[1];
  assert.equal(a.x, 100);
  assert.equal(a.y, 30);
  assert.deepEqual(a.points, [[0, 0], [100, 20]]);
  assert.equal(a.endArrowhead, 'arrow');
  assert.equal(a.startArrowhead, null);
  assert.equal(a.from, 1);
  assert.equal(b.items[0].fillStyle, 'solid');
});

test('sanitizeBoard drops frames that are not frames, caps the item count and keeps nextId ahead of every id', () => {
  const items = Array.from({ length: LIMITS.items + 50 }, (_, i) => ({ id: i + 1, type: 'rect', x: i, y: i, w: 20, h: 20 }));
  assert.equal(sanitizeBoard({ items }).items.length, LIMITS.items);
  const b = sanitizeBoard({ items: [{ id: 1, type: 'rect', x: 0, y: 0, w: 20, h: 20 }, { id: 2, type: 'rect', x: 0, y: 0, w: 20, h: 20, frameId: 1, groupIds: [40] }], nextId: 2 });
  assert.equal(b.items[1].frameId, null, 'a non-frame cannot be a frame parent');
  assert.equal(b.nextId, 41, 'group ids also reserve ids');
});

test('boardSignature ignores small drags but notices real edits', () => {
  const base = boardSignature(sample());
  const nudged = sample();
  nudged.items[1].x += 3;
  assert.equal(boardSignature(nudged), base);
  const renamed = sample();
  renamed.items[1].text = 'Gateway';
  assert.notEqual(boardSignature(renamed), base);
  const moved = sample();
  moved.items[1].x += 400;
  assert.notEqual(boardSignature(moved), base);
  const fewer = sample();
  fewer.items.pop();
  assert.notEqual(boardSignature(fewer), base);
  const flipped = sample();
  flipped.items[5].startArrowhead = 'arrow';
  assert.notEqual(boardSignature(flipped), base, 'arrowheads change the meaning of a connection');
});

test('toAiBoard strips image data and thins freehand strokes without touching the original', () => {
  const b = emptyBoard();
  add(b, 'image', { dataUrl: 'data:image/png;base64,AAAA'.padEnd(5000, 'A') });
  add(b, 'freedraw', { points: Array.from({ length: 200 }, (_, i) => [i, i]) });
  const light = toAiBoard(b);
  assert.equal(light.items[0].dataUrl, '');
  assert.equal(light.items[1].points.length, 2);
  assert.ok(b.items[0].dataUrl.length > 4000);
  assert.equal(b.items[1].points.length, 200);
  assert.ok(JSON.stringify(light).length < 2000);
});

test('rotatePoint rotates about a centre', () => {
  const p = rotatePoint({ x: 10, y: 0 }, { x: 0, y: 0 }, Math.PI / 2);
  assert.ok(Math.abs(p.x) < 1e-9 && Math.abs(p.y - 10) < 1e-9);
  assert.deepEqual(rotatePoint({ x: 3, y: 4 }, { x: 0, y: 0 }, 0), { x: 3, y: 4 });
});

test('new text defaults to the normal sans font, not hand-drawn', async () => {
  const { DEFAULT_STYLE, FONT_FAMILIES, createElement } = await import('../shared/board.js');
  assert.equal(DEFAULT_STYLE.fontFamily, 2);
  assert.equal(FONT_FAMILIES[DEFAULT_STYLE.fontFamily].name, 'Normal');
  assert.equal(createElement('text', { id: 1 }).fontFamily, 2);
});

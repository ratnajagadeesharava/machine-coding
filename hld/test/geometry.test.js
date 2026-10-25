import test from 'node:test';
import assert from 'node:assert/strict';
import { emptyBoard, newId, createElement, elementBounds, rotatePoint } from '../shared/board.js';
import {
  elementsInBox, handleAt, hitElement, hitTestTop, nearestBindTarget, resizeBox, resizeSelection, rotateAngle, simplifyStroke, snapMove, snapToGrid,
  textSize, transformHandles, labelLayout,
} from '../public/wb/geometry.js';

function setup(...defs) {
  const b = emptyBoard();
  const out = defs.map(([type, props]) => { const el = createElement(type, { id: newId(b), seed: 1, ...props }); b.items.push(el); return el; });
  return [b, ...out];
}
const near = (a, b, eps = 1e-6) => assert.ok(Math.abs(a - b) < eps, `${a} !~ ${b}`);

test('hollow shapes hit on outline and label only; filled shapes hit inside', () => {
  const [b, hollow, filled] = setup(
    ['rect', { x: 0, y: 0, w: 100, h: 60, text: '' }],
    ['rect', { x: 200, y: 0, w: 100, h: 60, backgroundColor: '#ffc9c9', text: '' }],
  );
  assert.equal(hitElement(b, hollow, { x: 50, y: 30 }, 4), false);
  assert.equal(hitElement(b, hollow, { x: 50, y: 1 }, 4), true);
  assert.equal(hitElement(b, hollow, { x: 100, y: 30 }, 4), true);
  assert.equal(hitElement(b, hollow, { x: 50, y: -10 }, 4), false);
  assert.equal(hitElement(b, filled, { x: 250, y: 30 }, 4), true);
  assert.equal(hitElement(b, filled, { x: 250, y: 70 }, 4), false);
  hollow.text = 'Hello';
  assert.equal(hitElement(b, hollow, { x: 50, y: 30 }, 4), true, 'label area counts');
});

test('ellipse, diamond and database hit tests', () => {
  const [b, e, d, db] = setup(
    ['ellipse', { x: 0, y: 0, w: 100, h: 60, backgroundColor: '#fff' }],
    ['diamond', { x: 200, y: 0, w: 100, h: 100, backgroundColor: 'transparent', text: '' }],
    ['database', { x: 400, y: 0, w: 100, h: 120, backgroundColor: '#fff' }],
  );
  assert.ok(hitElement(b, e, { x: 50, y: 30 }, 0));
  assert.ok(!hitElement(b, e, { x: 3, y: 3 }, 0), 'bbox corner is outside the ellipse');
  assert.ok(hitElement(b, d, { x: 250, y: 2 }, 3));
  assert.ok(!hitElement(b, d, { x: 250, y: 50 }, 3), 'hollow centre');
  assert.ok(!hitElement(b, d, { x: 203, y: 3 }, 3), 'diamond corner area is empty');
  assert.ok(hitElement(b, db, { x: 450, y: 60 }, 0));
  assert.ok(hitElement(b, db, { x: 450, y: 8 }, 0), 'inside the top cap');
  assert.ok(!hitElement(b, db, { x: 401, y: 1 }, 0));
});

test('hit tests are rotation aware', () => {
  const [b, r] = setup(['rect', { x: 0, y: 0, w: 200, h: 20, backgroundColor: '#fff', angle: Math.PI / 2 }]);
  // centre (100,10); rotated 90deg -> occupies x 90..110, y -90..110
  assert.ok(hitElement(b, r, { x: 100, y: 100 }, 0));
  assert.ok(!hitElement(b, r, { x: 190, y: 10 }, 0));
  const [b2, h] = setup(['rect', { x: 0, y: 0, w: 200, h: 20, angle: Math.PI / 2 }]);
  assert.ok(hitElement(b2, h, { x: 90, y: 50 }, 3), 'rotated outline');
  assert.ok(!hitElement(b2, h, { x: 100, y: 50 }, 3), 'rotated hollow interior');
});

test('lines, arrows, freedraw: distance to the path', () => {
  const [b, ln, ar, fd] = setup(
    ['line', { x: 0, y: 0, points: [[0, 0], [100, 0], [100, 100]], w: 100, h: 100 }],
    ['arrow', { x: 300, y: 0, points: [[0, 0], [100, 0]], w: 100, h: 0, text: 'label' }],
    ['freedraw', { x: 0, y: 300, points: [[0, 0], [20, 10], [40, 0]], w: 40, h: 10 }],
  );
  assert.ok(hitElement(b, ln, { x: 50, y: 3 }, 4));
  assert.ok(hitElement(b, ln, { x: 98, y: 50 }, 4));
  assert.ok(!hitElement(b, ln, { x: 50, y: 50 }, 4), 'inside the corner is not a hit');
  assert.ok(hitElement(b, ar, { x: 350, y: 8 }, 2), 'arrow label counts');
  assert.ok(!hitElement(b, ar, { x: 320, y: 40 }, 2));
  assert.ok(hitElement(b, fd, { x: 20, y: 312 }, 4));
  assert.ok(!hitElement(b, fd, { x: 20, y: 330 }, 4));
});

test('frames hit on outline and name only; images and sticky by box', () => {
  const [b, f, img, st] = setup(
    ['frame', { x: 0, y: 0, w: 300, h: 200, name: 'Backend' }],
    ['image', { x: 400, y: 0, w: 50, h: 50 }],
    ['sticky', { x: 500, y: 0, w: 100, h: 100 }],
  );
  assert.ok(!hitElement(b, f, { x: 150, y: 100 }, 4));
  assert.ok(hitElement(b, f, { x: 150, y: 2 }, 4));
  assert.ok(hitElement(b, f, { x: 20, y: -10 }, 4), 'name label');
  assert.ok(hitElement(b, img, { x: 425, y: 25 }, 0));
  assert.ok(hitElement(b, st, { x: 550, y: 50 }, 0));
});

test('hitTestTop returns the topmost, honours skipLocked and ids', () => {
  const [b, a, c] = setup(
    ['rect', { x: 0, y: 0, w: 100, h: 100, backgroundColor: '#fff' }],
    ['rect', { x: 20, y: 20, w: 100, h: 100, backgroundColor: '#fff' }],
  );
  assert.equal(hitTestTop(b, { x: 50, y: 50 }, 2).id, c.id);
  c.locked = true;
  assert.equal(hitTestTop(b, { x: 50, y: 50 }, 2, { skipLocked: true }).id, a.id);
  assert.equal(hitTestTop(b, { x: 50, y: 50 }, 2, { ids: [c.id] }).id, c.id);
  assert.equal(hitTestTop(b, { x: 500, y: 50 }, 2), null);
});

test('elementsInBox: contain vs intersect, rotated bounds, lines', () => {
  const [b, a, c, ln] = setup(
    ['rect', { x: 0, y: 0, w: 50, h: 50 }],
    ['rect', { x: 40, y: 40, w: 100, h: 100 }],
    ['line', { x: 0, y: 200, points: [[0, 0], [100, 100]], w: 100, h: 100 }],
  );
  const box = { x: -10, y: -10, w: 70, h: 70 };
  assert.deepEqual(elementsInBox(b, box, { contain: true }), [a.id]);
  assert.deepEqual(elementsInBox(b, box, { contain: false }), [a.id, c.id]);
  // a box that overlaps only the line's bounding box corner, not the line
  assert.deepEqual(elementsInBox(b, { x: 80, y: 205, w: 20, h: 20 }, { contain: false }), []);
  assert.deepEqual(elementsInBox(b, { x: 45, y: 245, w: 20, h: 20 }, { contain: false }), [ln.id]);
});

test('bounds with rotation', () => {
  const [b, r] = setup(['rect', { x: 0, y: 0, w: 100, h: 20, angle: Math.PI / 2 }]);
  const bb = elementBounds(b, r);
  near(bb.w, 20); near(bb.h, 100); near(bb.x, 40); near(bb.y, -40);
});

test('transformHandles + handleAt (rotated)', () => {
  const box = { x: 0, y: 0, w: 100, h: 50 };
  const h0 = transformHandles(box, 0, 1);
  assert.deepEqual(h0.nw, { x: 0, y: 0 }); assert.deepEqual(h0.se, { x: 100, y: 50 });
  assert.deepEqual(h0.rotate, { x: 50, y: -24 });
  const h90 = transformHandles(box, Math.PI / 2, 1);
  near(h90.nw.x, 75); near(h90.nw.y, -25);
  assert.equal(handleAt(h90, { x: 75, y: -25 }, 1), 'nw');
  assert.equal(handleAt(h0, { x: 50, y: -22 }, 1), 'rotate');
  assert.equal(handleAt(h0, { x: 50, y: 25 }, 1), null);
  assert.equal(handleAt(h0, { x: 6, y: 6 }, 2), null, 'radius shrinks with zoom');
  assert.ok(!('rotate' in transformHandles(box, 0, 1, { rotation: false })));
  assert.deepEqual(Object.keys(transformHandles(box, 0, 1, { linear: true })).sort(), ['ne', 'nw', 'se', 'sw']);
});

test('resizeBox: unrotated, aspect, from centre, minimum size', () => {
  const el = { x: 0, y: 0, w: 100, h: 50, angle: 0 };
  assert.deepEqual(resizeBox(el, 'se', { x: 200, y: 100 }, {}), { x: 0, y: 0, w: 200, h: 100 });
  assert.deepEqual(resizeBox(el, 'nw', { x: -20, y: -10 }, {}), { x: -20, y: -10, w: 120, h: 60 });
  assert.deepEqual(resizeBox(el, 'e', { x: 150, y: 999 }, {}), { x: 0, y: 0, w: 150, h: 50 });
  const asp = resizeBox(el, 'se', { x: 200, y: 60 }, { keepAspect: true });
  near(asp.w / asp.h, 2); near(asp.w, 200); near(asp.x, 0);
  const cen = resizeBox(el, 'se', { x: 150, y: 75 }, { fromCenter: true });
  assert.deepEqual(cen, { x: -50, y: -25, w: 200, h: 100 });
  const tiny = resizeBox(el, 'se', { x: 2, y: 2 }, {});
  assert.ok(tiny.w >= 12 && tiny.h >= 12);
});

test('resizeBox honours the angle: the opposite corner stays fixed in the world', () => {
  const angle = Math.PI / 6;
  const el = { x: 0, y: 0, w: 100, h: 50, angle };
  const c = { x: 50, y: 25 };
  const nwWorld = rotatePoint({ x: 0, y: 0 }, c, angle);
  const target = rotatePoint({ x: 150, y: 80 }, c, angle); // drag SE handle to local (150,80)
  const r = resizeBox(el, 'se', target, {});
  near(r.w, 150); near(r.h, 80);
  const nc = { x: r.x + r.w / 2, y: r.y + r.h / 2 };
  const nwAfter = rotatePoint({ x: r.x, y: r.y }, nc, angle);
  near(nwAfter.x, nwWorld.x, 1e-6); near(nwAfter.y, nwWorld.y, 1e-6);
});

test('resizeSelection: single element, multi selection about the opposite handle, text font, points', () => {
  const [b, a, c, t, ln] = setup(
    ['rect', { x: 0, y: 0, w: 100, h: 100 }],
    ['rect', { x: 100, y: 100, w: 100, h: 100 }],
    ['text', { x: 300, y: 0, w: 100, h: 25, text: 'Hello', fontSize: 20, fontFamily: 2 }],
    ['line', { x: 0, y: 400, points: [[0, 0], [100, 50]], w: 100, h: 50 }],
  );
  const single = resizeSelection(b, [a.id], 'se', { x: 200, y: 50 }, {});
  assert.deepEqual(single.get(a.id), { x: 0, y: 0, w: 200, h: 50 });
  // multi: selection box (0,0)-(200,200) dragged SE to (400,200): sx=2, sy=1
  const multi = resizeSelection(b, [a.id, c.id], 'se', { x: 400, y: 200 }, {});
  assert.deepEqual(multi.get(a.id), { x: 0, y: 0, w: 200, h: 100 });
  assert.deepEqual(multi.get(c.id), { x: 200, y: 100, w: 200, h: 100 });
  // from the top-left handle the bottom-right stays put
  const nwDrag = resizeSelection(b, [a.id, c.id], 'nw', { x: -200, y: -200 }, { keepAspect: true });
  const pa = nwDrag.get(a.id); const pc = nwDrag.get(c.id);
  near(pc.x + pc.w, 200); near(pc.y + pc.h, 200);
  near(pa.w, 200); near(pa.h, 200);
  // text scales its font size
  const tx = resizeSelection(b, [t.id], 'se', { x: 500, y: 50 }, { keepAspect: true });
  const tp = tx.get(t.id);
  near(tp.fontSize, 40); near(tp.h, 50); assert.ok(tp.w > 70);
  // points
  const lp = resizeSelection(b, [ln.id], 'se', { x: 200, y: 500 }, {}).get(ln.id);
  assert.deepEqual(lp.points, [[0, 0], [200, 100]]);
  assert.equal(lp.x, 0); assert.equal(lp.y, 400);
  // multi with a line: positions scale too
  const mixed = resizeSelection(b, [a.id, ln.id], 'se', { x: 200, y: 900 }, {});
  const lm = mixed.get(ln.id); const am = mixed.get(a.id);
  near(am.h, 100 * (900 / 450));
  near(lm.y, 400 * (900 / 450));
});

test('resizeSelection keeps image aspect on corner drags', () => {
  const [b, img] = setup(['image', { x: 0, y: 0, w: 200, h: 100 }]);
  const p = resizeSelection(b, [img.id], 'se', { x: 400, y: 120 }, {}).get(img.id);
  near(p.w / p.h, 2);
});

test('rotateAngle with and without snap', () => {
  near(rotateAngle({ x: 0, y: 0 }, { x: 0, y: -10 }, {}), 0);
  near(rotateAngle({ x: 0, y: 0 }, { x: 10, y: 0 }, {}), Math.PI / 2);
  near(rotateAngle({ x: 0, y: 0 }, { x: 0, y: 10 }, {}), Math.PI);
  near(rotateAngle({ x: 0, y: 0 }, { x: 10, y: -9 }, { snap: true }), Math.PI / 4);
  const a = rotateAngle({ x: 0, y: 0 }, { x: -10, y: 0 }, {});
  assert.ok(a >= 0 && a < Math.PI * 2);
  assert.equal(snapToGrid(33, 20), 40); assert.equal(snapToGrid(29, 20), 20); assert.equal(snapToGrid(-11, 20), -20);
});

test('snapMove snaps edges and centres and returns guides', () => {
  const [b, m, o] = setup(
    ['rect', { x: 0, y: 0, w: 100, h: 50 }],
    ['rect', { x: 300, y: 200, w: 60, h: 80 }],
  );
  // moving rect's left edge would land at 297 -> snaps to 300; centre y 25+dy
  let r = snapMove(b, [m.id], 297, 0, { threshold: 6 });
  assert.equal(r.dx, 300);
  assert.equal(r.dy, 0);
  const g = r.guides.find((q) => q.axis === 'x' && q.pos === 300);
  assert.ok(g && g.from <= 0 && g.to >= 280);
  // centre to centre vertically: moving centre y 25+dy vs other centre 240 -> dy 213 snaps to 215
  r = snapMove(b, [m.id], 0, 213, { threshold: 6 });
  assert.equal(r.dy, 215);
  assert.ok(r.guides.some((q) => q.axis === 'y' && q.pos === 240));
  // right edge to left edge: right = 100+dx -> 300
  r = snapMove(b, [m.id], 203, 500, { threshold: 6 });
  assert.equal(r.dx, 200);
  // nothing near
  r = snapMove(b, [m.id], 150, 500, { threshold: 6 });
  assert.deepEqual(r, { dx: 150, dy: 500, guides: [] });
  // explicit others
  r = snapMove(b, [m.id], 297, 0, { threshold: 6, others: [] });
  assert.equal(r.dx, 297);
  // moving elements never snap to themselves
  r = snapMove(b, [m.id, o.id], 3, 3, { threshold: 6 });
  assert.equal(r.dx, 3);
});

test('nearestBindTarget picks the shape near the point, not the excluded one', () => {
  const [b, a, c] = setup(
    ['rect', { x: 0, y: 0, w: 100, h: 100 }],
    ['ellipse', { x: 200, y: 0, w: 100, h: 100 }],
  );
  assert.equal(nearestBindTarget(b, { x: 104, y: 50 }, 8, {}).id, a.id);
  assert.equal(nearestBindTarget(b, { x: 150, y: 50 }, 8, {}), null);
  assert.equal(nearestBindTarget(b, { x: 250, y: 50 }, 8, {}).id, c.id);
  assert.equal(nearestBindTarget(b, { x: 250, y: 50 }, 8, { excludeId: c.id }), null);
});

test('textSize: line height, wrapping by words, long words, newlines, families', () => {
  const one = textSize('Hello world', { fontFamily: 2, fontSize: 20 });
  assert.equal(one.lines.length, 1);
  near(one.h, 25);
  assert.ok(one.w > 80 && one.w < 160);
  const multi = textSize('a\nb\n\nc', { fontFamily: 1, fontSize: 16 });
  assert.deepEqual(multi.lines, ['a', 'b', '', 'c']); near(multi.h, 80);
  const wrapped = textSize('the quick brown fox jumps over the lazy dog', { fontFamily: 2, fontSize: 20, maxWidth: 120 });
  assert.ok(wrapped.lines.length > 2);
  assert.equal(wrapped.lines.join(' '), 'the quick brown fox jumps over the lazy dog');
  assert.ok(wrapped.w <= 120 + 0.001);
  const long = textSize('Supercalifragilisticexpialidocious', { fontFamily: 2, fontSize: 20, maxWidth: 100 });
  assert.ok(long.lines.length > 1 && long.w <= 100 + 0.001);
  assert.equal(long.lines.join(''), 'Supercalifragilisticexpialidocious');
  const code = textSize('abcdefghij', { fontFamily: 3, fontSize: 10 });
  near(code.w, 60);
  assert.ok(textSize('Hello', { fontFamily: 1, fontSize: 20 }).w !== textSize('Hello', { fontFamily: 3, fontSize: 20 }).w);
  assert.ok(textSize('iiii', { fontFamily: 2, fontSize: 20 }).w < textSize('WWWW', { fontFamily: 2, fontSize: 20 }).w);
  assert.deepEqual(textSize('', { fontFamily: 1, fontSize: 20 }).lines, ['']);
});

test('labelLayout aligns inside the shape', () => {
  const el = createElement('rect', { x: 0, y: 0, w: 200, h: 100, text: 'Hi', fontSize: 20, textAlign: 'center', verticalAlign: 'middle' });
  const mid = labelLayout(el);
  near(mid.anchorX, 100); assert.equal(mid.anchor, 'middle');
  near(mid.top + mid.box.h / 2, 50);
  const tl = labelLayout({ ...el, textAlign: 'left', verticalAlign: 'top' });
  near(tl.anchorX, 8); near(tl.top, 8); assert.equal(tl.anchor, 'start');
  const br = labelLayout({ ...el, textAlign: 'right', verticalAlign: 'bottom' });
  near(br.anchorX, 192); near(br.top + br.box.h, 92);
  const ell = labelLayout({ ...el, type: 'ellipse' });
  assert.ok(ell.box.w <= 200 * 0.7072);
  assert.equal(labelLayout({ ...el, text: '' }), null);
});

test('simplifyStroke removes collinear points and keeps corners', () => {
  const line = Array.from({ length: 20 }, (_, i) => [i, i * 0.001]);
  assert.equal(simplifyStroke(line, 1).length, 2);
  const corner = [[0, 0], [10, 0], [20, 0], [20, 10], [20, 20]];
  assert.deepEqual(simplifyStroke(corner, 0.5), [[0, 0], [20, 0], [20, 20]]);
  assert.deepEqual(simplifyStroke([[1, 1]], 1), [[1, 1]]);
});

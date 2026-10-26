import test from 'node:test';
import assert from 'node:assert/strict';
import { emptyBoard, createElement, newId, findItem, sanitizeBoard, resolvePoints, elementBounds, selectionBounds, BINDABLE_TYPES } from '../shared/board.js';
import {
  MIXED, STYLE_PROPS, applyStyle, styleSummary, expandSelection, reorder, group, ungroup, setLocked, unlockAll, align, distribute, flip,
  duplicate, deleteElements, moveElements, assignFrames, bindArrow, findText, copyStyle, pasteStyle, approxTextSize,
} from '../public/wb/ops.js';

function add(board, type, props = {}) {
  const el = createElement(type, { ...props, id: newId(board) });
  if ((type === 'arrow' || type === 'line') && props.points) {
    const [ox, oy] = el.points[0];
    el.x = props.x ?? ox; el.y = props.y ?? oy;
    el.points = el.points.map(([x, y]) => [x - ox, y - oy]);
    const xs = el.points.map((p) => p[0]); const ys = el.points.map((p) => p[1]);
    el.w = Math.max(...xs) - Math.min(...xs); el.h = Math.max(...ys) - Math.min(...ys);
  }
  board.items.push(el);
  return el;
}
const ids = (b) => b.items.map((e) => e.id);
const rect = (b, x, y, w = 100, h = 50, extra = {}) => add(b, 'rect', { x, y, w, h, ...extra });

// ---------- MIXED / STYLE_PROPS ----------
test('MIXED is a shared symbol and STYLE_PROPS lists applicable types', () => {
  assert.equal(MIXED, Symbol.for('wb.mixed'));
  assert.ok(STYLE_PROPS.strokeColor.includes('arrow') && !STYLE_PROPS.strokeColor.includes('image'));
  assert.ok(!STYLE_PROPS.backgroundColor.includes('text') && !STYLE_PROPS.backgroundColor.includes('line') && STYLE_PROPS.backgroundColor.includes('freedraw'));
  assert.ok(!STYLE_PROPS.roundness.includes('ellipse') && !STYLE_PROPS.roundness.includes('database'));
  assert.deepEqual(STYLE_PROPS.startArrowhead, ['arrow']);
  assert.ok(STYLE_PROPS.arrowType.includes('line'));
});

// ---------- applyStyle ----------
test('applyStyle skips non-applicable props and returns only changed ids', () => {
  const b = emptyBoard();
  const r = rect(b, 0, 0); const t = add(b, 'text', { text: 'hi' }); const img = add(b, 'image');
  const changed = applyStyle(b, [r.id, t.id, img.id], { backgroundColor: '#ffc9c9', strokeColor: '#e03131', opacity: 40 });
  assert.deepEqual(changed, [r.id, t.id, img.id]);
  assert.equal(r.backgroundColor, '#ffc9c9');
  assert.equal(t.backgroundColor, 'transparent');
  assert.equal(t.strokeColor, '#e03131');
  assert.equal(img.strokeColor, '#1e1e1e');
  assert.equal(img.opacity, 40);
  assert.deepEqual(applyStyle(b, [r.id], { backgroundColor: '#ffc9c9' }), []); // no change
});

test('applyStyle rejects invalid values and hostile input', () => {
  const b = emptyBoard();
  const r = rect(b, 0, 0);
  const before = JSON.stringify(r);
  const bad = { strokeColor: 'url(javascript:1)', fillStyle: 'wavy', strokeWidth: 3, strokeStyle: 'x', roughness: 9, opacity: NaN, roundness: 'pointy', fontFamily: 99, fontSize: Infinity, textAlign: 'justify', verticalAlign: 'x', backgroundColor: 5 };
  assert.deepEqual(applyStyle(b, [r.id], bad), []);
  assert.equal(JSON.stringify(r), before);
  assert.deepEqual(applyStyle(b, [r.id], null), []);
  assert.deepEqual(applyStyle(b, [999, NaN, 'x', null], { opacity: 10 }), []);
  assert.deepEqual(applyStyle(b, undefined, { opacity: 10 }), []);
  assert.deepEqual(applyStyle(b, [r.id], { __proto__: { opacity: 5 }, constructor: 1 }), []);
});

test('applyStyle clamps opacity/fontSize, accepts Set ids and string font family', () => {
  const b = emptyBoard();
  const r = rect(b, 0, 0);
  applyStyle(b, new Set([r.id]), { opacity: 250, fontSize: 1000, fontFamily: '3' });
  assert.equal(r.opacity, 100);
  assert.equal(r.fontSize, 200);
  assert.equal(r.fontFamily, 3);
  applyStyle(b, [r.id], { opacity: -5, fontSize: 2 });
  assert.equal(r.opacity, 0);
  assert.equal(r.fontSize, 8);
});

test('applyStyle respects element-specific props (roundness, arrowheads, arrowType on lines)', () => {
  const b = emptyBoard();
  const e = add(b, 'ellipse'); const r = rect(b, 0, 0); const ar = add(b, 'arrow', { points: [[0, 0], [100, 0]] });
  const ln = add(b, 'line', { points: [[0, 0], [50, 0]] });
  applyStyle(b, [e.id, r.id], { roundness: 'sharp' });
  assert.equal(e.roundness, 'round');
  assert.equal(r.roundness, 'sharp');
  applyStyle(b, [ar.id, ln.id, r.id], { endArrowhead: 'triangle', startArrowhead: 'bar' });
  assert.equal(ar.endArrowhead, 'triangle'); assert.equal(ar.startArrowhead, 'bar');
  assert.equal(ln.endArrowhead, null);
  assert.ok(!('endArrowhead' in r));
  applyStyle(b, [ar.id], { endArrowhead: null });
  assert.equal(ar.endArrowhead, null);
  applyStyle(b, [ar.id], { endArrowhead: 'nope' });
  assert.equal(ar.endArrowhead, null);
  applyStyle(b, [ln.id], { arrowType: 'elbow' });
  assert.equal(ln.arrowType, 'sharp');
  applyStyle(b, [ln.id], { arrowType: 'round' });
  assert.equal(ln.arrowType, 'round');
});

test('applyStyle: elbow resets intermediate points, bindings stay', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0); const d = rect(b, 300, 200);
  const ar = add(b, 'arrow', { points: [[100, 25], [150, 60], [220, 90], [300, 225]], from: s.id, to: d.id });
  const first = [ar.x, ar.y];
  assert.deepEqual(applyStyle(b, [ar.id], { arrowType: 'elbow' }), [ar.id]);
  assert.equal(ar.points.length, 2);
  assert.deepEqual([ar.x, ar.y], first);
  assert.deepEqual(ar.points[1], [200, 200]);
  assert.equal(ar.from, s.id);
  assert.equal(ar.to, d.id);
  applyStyle(b, [ar.id], { arrowType: 'elbow' }); // already elbow
  assert.equal(ar.points.length, 2);
});

test('applyStyle: text element is re-measured after font changes and keeps its alignment anchor', () => {
  const b = emptyBoard();
  const left = add(b, 'text', { x: 100, y: 100, text: 'Hello world', fontSize: 20, textAlign: 'left', w: 100, h: 25 });
  const mid = add(b, 'text', { x: 100, y: 100, text: 'Hello world', fontSize: 20, textAlign: 'center', w: 100, h: 25 });
  const right = add(b, 'text', { x: 100, y: 100, text: 'Hello world', fontSize: 20, textAlign: 'right', w: 100, h: 25 });
  applyStyle(b, ids(b), { fontSize: 36 });
  const s = approxTextSize('Hello world', { fontFamily: 2, fontSize: 36 }); // 2 = the default (Normal) font
  for (const t of [left, mid, right]) { assert.equal(t.fontSize, 36); assert.equal(t.w, s.w); assert.equal(t.h, s.h); }
  assert.equal(left.x, 100);
  assert.equal(mid.x + mid.w / 2, 150);
  assert.equal(right.x + right.w, 200);
  const before = left.w;
  applyStyle(b, [left.id], { fontFamily: 3 });
  assert.notEqual(left.w, before);
  const c = rect(b, 0, 0, 100, 50, { text: 'label' });
  applyStyle(b, [c.id], { fontSize: 36 });
  assert.equal(c.w, 100); // containers keep their size
});

test('approxTextSize handles lines, wrapping, empty text and hostile input', () => {
  const one = approxTextSize('abc', { fontFamily: 2, fontSize: 20 });
  const two = approxTextSize('abc\nabc', { fontFamily: 2, fontSize: 20 });
  assert.equal(two.lines.length, 2);
  assert.equal(two.h, one.h * 2);
  assert.equal(two.w, one.w);
  const wrapped = approxTextSize('one two three four five six', { fontFamily: 2, fontSize: 20, maxWidth: 80 });
  assert.ok(wrapped.lines.length > 2 && wrapped.w <= 90);
  const long = approxTextSize('x'.repeat(100), { fontSize: 20, maxWidth: 50 });
  assert.ok(long.lines.length > 5);
  assert.equal(approxTextSize('').lines.length, 1);
  assert.ok(approxTextSize(undefined).h > 0);
  assert.ok(approxTextSize(12345).w > 0);
  assert.ok(approxTextSize('abc', { fontFamily: 3, fontSize: 20 }).w > approxTextSize('abc', { fontFamily: 2, fontSize: 20 }).w - 1);
});

// ---------- styleSummary ----------
test('styleSummary: same, MIXED and applies', () => {
  const b = emptyBoard();
  const r1 = rect(b, 0, 0, 10, 10, { strokeColor: '#e03131' }); const r2 = rect(b, 0, 0, 10, 10, { strokeColor: '#e03131' });
  const t = add(b, 'text', { strokeColor: '#e03131' });
  let s = styleSummary(b, [r1.id, r2.id], {});
  assert.equal(s.values.strokeColor, '#e03131');
  assert.equal(s.values.fillStyle, 'solid');
  assert.equal(s.applies.arrowType, false);
  assert.equal(s.applies.roundness, true);
  s = styleSummary(b, [r1.id, r2.id, t.id], {});
  assert.equal(s.values.strokeColor, '#e03131');
  assert.equal(s.applies.fontSize, true);
  assert.equal(s.applies.backgroundColor, true);
  r2.strokeColor = '#2f9e44';
  r2.fontSize = 28;
  s = styleSummary(b, [r1.id, r2.id, t.id], {});
  assert.equal(s.values.strokeColor, MIXED);
  assert.equal(s.values.fontSize, MIXED);
  // text does not take part in backgroundColor
  t.backgroundColor = '#000000';
  s = styleSummary(b, [r1.id, t.id], {});
  assert.notEqual(s.values.backgroundColor, MIXED);
});

test('styleSummary: arrowheads mixed, image only opacity/others', () => {
  const b = emptyBoard();
  const a1 = add(b, 'arrow', { points: [[0, 0], [5, 5]] }); const a2 = add(b, 'arrow', { points: [[0, 0], [5, 5]], endArrowhead: null });
  const img = add(b, 'image');
  assert.equal(styleSummary(b, [a1.id, a2.id], {}).values.endArrowhead, MIXED);
  assert.equal(styleSummary(b, [a1.id], {}).values.endArrowhead, 'arrow');
  const s = styleSummary(b, [img.id], {});
  assert.equal(s.applies.strokeColor, false);
  assert.equal(s.applies.opacity, true);
  assert.equal(s.values.strokeColor, undefined);
});

test('styleSummary: empty selection uses fallback and tool restriction', () => {
  const b = emptyBoard();
  const fb = { strokeColor: '#1971c2', fontSize: 28 };
  const s = styleSummary(b, [], fb);
  assert.equal(s.values.strokeColor, '#1971c2');
  assert.ok(Object.values(s.applies).every(Boolean));
  const arrow = styleSummary(b, [], fb, 'arrow');
  assert.equal(arrow.applies.endArrowhead, true);
  assert.equal(arrow.applies.backgroundColor, false);
  const rc = styleSummary(b, [], fb, 'rect');
  assert.equal(rc.applies.endArrowhead, false);
  assert.equal(rc.applies.roundness, true);
  assert.equal(styleSummary(b, [], fb, 'ellipse').applies.roundness, false);
  assert.ok(Object.values(styleSummary(b, [], fb, 'select').applies).every(Boolean));
  assert.ok(Object.values(styleSummary(b, [12345], undefined).applies).every(Boolean)); // unknown ids == empty
});

// ---------- copy / paste style ----------
test('copyStyle / pasteStyle', () => {
  const b = emptyBoard();
  const r = rect(b, 0, 0, 10, 10, { strokeColor: '#e03131', backgroundColor: '#b2f2bb', roughness: 2, text: 'x' });
  const t = add(b, 'text', { text: 'y' });
  const st = copyStyle(r);
  assert.equal(st.strokeColor, '#e03131');
  assert.equal(st.roughness, 2);
  assert.ok(!('arrowType' in st) && !('startArrowhead' in st));
  assert.ok(!('w' in st) && !('text' in st));
  assert.ok(!('backgroundColor' in copyStyle(t)));
  assert.deepEqual(copyStyle(null), {});
  const changed = pasteStyle(b, [t.id], st);
  assert.deepEqual(changed, [t.id]);
  assert.equal(t.strokeColor, '#e03131');
  assert.equal(t.backgroundColor, 'transparent');
  assert.equal(t.text, 'y');
  assert.deepEqual(pasteStyle(b, [t.id], { w: 5, text: 'zzz', id: 9 }), []);
});

// ---------- expandSelection ----------
test('expandSelection: groups (outermost) and frames', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0); const c = rect(b, 0, 0); const d = rect(b, 0, 0); const e = rect(b, 0, 0);
  a.groupIds = [100]; c.groupIds = [100, 200]; // c is in an outer group too
  d.groupIds = [200]; // sharing outer group 200 with c
  assert.deepEqual(expandSelection(b, [a.id]), [a.id, c.id, d.id].sort((x, y) => ids(b).indexOf(x) - ids(b).indexOf(y)));
  assert.deepEqual(expandSelection(b, [e.id]), [e.id]);
  assert.deepEqual(expandSelection(b, [999, 'x']), []);
  assert.deepEqual(expandSelection(b, []), []);
  const f = add(b, 'frame', { x: 0, y: 0, w: 500, h: 500 });
  const k1 = rect(b, 10, 10, 10, 10, { frameId: f.id }); const k2 = rect(b, 20, 20, 10, 10, { frameId: f.id, groupIds: [300] }); const mate = rect(b, 900, 900, 10, 10, { groupIds: [300] });
  assert.deepEqual(expandSelection(b, [f.id]).sort(), [f.id, k1.id, k2.id, mate.id].sort());
  assert.deepEqual(expandSelection(b, [k1.id]), [k1.id]);
});

// ---------- reorder ----------
test('reorder front/back keep relative order', () => {
  const b = emptyBoard();
  const [a, c, d, e, f] = [0, 1, 2, 3, 4].map((i) => rect(b, i, 0));
  assert.equal(reorder(b, [a.id, d.id], 'front'), true);
  assert.deepEqual(ids(b), [c.id, e.id, f.id, a.id, d.id]);
  assert.equal(reorder(b, [a.id, d.id], 'front'), false);
  assert.equal(reorder(b, [f.id, e.id], 'back'), true);
  assert.deepEqual(ids(b), [e.id, f.id, c.id, a.id, d.id]);
  assert.equal(reorder(b, [e.id], 'back'), false);
});

test('reorder forward/backward step past one neighbour; runs move together', () => {
  const b = emptyBoard();
  const [a, c, d, e, f] = [0, 1, 2, 3, 4].map((i) => rect(b, i, 0));
  assert.equal(reorder(b, [a.id, c.id], 'forward'), true);
  assert.deepEqual(ids(b), [d.id, a.id, c.id, e.id, f.id]);
  assert.equal(reorder(b, [f.id], 'forward'), false);
  assert.equal(reorder(b, [e.id, f.id], 'forward'), false);
  assert.equal(reorder(b, [a.id, f.id], 'backward'), true);
  assert.deepEqual(ids(b), [a.id, d.id, c.id, f.id, e.id]);
  assert.equal(reorder(b, [a.id], 'backward'), false);
});

test('reorder: groups move together, invalid input, locked skipped', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 10, 10, { groupIds: [50] }); const x = rect(b, 0, 0); const c = rect(b, 0, 0, 10, 10, { groupIds: [50] }); const y = rect(b, 0, 0);
  assert.equal(reorder(b, [a.id], 'front'), true);
  assert.deepEqual(ids(b), [x.id, y.id, a.id, c.id]);
  assert.equal(reorder(b, [999], 'front'), false);
  assert.equal(reorder(b, [], 'front'), false);
  assert.equal(reorder(b, [a.id], 'sideways'), false);
  x.locked = true;
  assert.equal(reorder(b, [x.id], 'front'), false); // locked elements are ignored like in the editor
  assert.deepEqual(ids(b), [x.id, y.id, a.id, c.id]);
});

test('reorder: frame children stay above their frame', () => {
  const b = emptyBoard();
  const lone = rect(b, 0, 0);
  const f = add(b, 'frame', { x: 0, y: 0, w: 500, h: 500 });
  const k = rect(b, 10, 10, 20, 20, { frameId: f.id });
  const k2 = rect(b, 10, 10, 20, 20, { frameId: f.id });
  assert.equal(reorder(b, [k.id], 'back'), false); // cannot go below its frame
  assert.deepEqual(ids(b), [lone.id, f.id, k.id, k2.id]);
  assert.equal(reorder(b, [k.id], 'backward'), false);
  reorder(b, [f.id], 'back'); // frame moves its children along
  assert.deepEqual(ids(b), [f.id, k.id, k2.id, lone.id]);
  reorder(b, [f.id], 'front');
  assert.deepEqual(ids(b), [lone.id, f.id, k.id, k2.id]);
  reorder(b, [lone.id], 'front');
  assert.deepEqual(ids(b), [f.id, k.id, k2.id, lone.id]);
});

// ---------- group / ungroup ----------
test('group needs >= 2, assigns new outermost id, nests', () => {
  const b = emptyBoard();
  const [a, c, d, e] = [0, 1, 2, 3].map((i) => rect(b, i * 10, 0));
  assert.equal(group(b, [a.id]), null);
  assert.equal(group(b, []), null);
  assert.equal(group(b, [a.id, 999]), null);
  const before = b.nextId;
  const g1 = group(b, [a.id, c.id]);
  assert.equal(g1, before);
  assert.deepEqual(a.groupIds, [g1]);
  assert.equal(group(b, [a.id, c.id]), null); // already exactly this group
  const g2 = group(b, [a.id, d.id]); // a is in g1 -> c is pulled in
  assert.ok(g2 > g1);
  assert.deepEqual(a.groupIds, [g1, g2]);
  assert.deepEqual(c.groupIds, [g1, g2]);
  assert.deepEqual(d.groupIds, [g2]);
  assert.deepEqual(e.groupIds, []);
  assert.deepEqual(ungroup(b, [d.id]).sort(), [a.id, c.id, d.id].sort());
  assert.deepEqual(a.groupIds, [g1]);
  assert.deepEqual(d.groupIds, []);
  assert.deepEqual(ungroup(b, [e.id]), []);
  assert.deepEqual(ungroup(b, [a.id]).sort(), [a.id, c.id].sort());
  assert.deepEqual(c.groupIds, []);
});

test('group ignores locked elements and frames; depth limit', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0); const c = rect(b, 0, 0, 10, 10, { locked: true }); const f = add(b, 'frame');
  assert.equal(group(b, [a.id, c.id]), null);
  assert.equal(group(b, [a.id, f.id]), null);
  const d = rect(b, 0, 0, 10, 10, { groupIds: [1, 2, 3, 4, 5, 6, 7, 8] });
  const e = rect(b, 0, 0);
  assert.equal(group(b, [d.id, e.id]), null);
});

test('group: children of a selected frame can be grouped', () => {
  const b = emptyBoard();
  const f = add(b, 'frame', { w: 400, h: 400 });
  const k1 = rect(b, 5, 5, 10, 10, { frameId: f.id }); const k2 = rect(b, 50, 5, 10, 10, { frameId: f.id });
  const g = group(b, [f.id]);
  assert.ok(g);
  assert.deepEqual(f.groupIds, []);
  assert.deepEqual(k1.groupIds, [g]);
  assert.deepEqual(k2.groupIds, [g]);
});

// ---------- lock ----------
test('setLocked / unlockAll', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 10, 10, { groupIds: [9] }); const c = rect(b, 0, 0, 10, 10, { groupIds: [9] }); const d = rect(b, 0, 0);
  assert.deepEqual(setLocked(b, [a.id], true), [a.id, c.id]);
  assert.deepEqual(setLocked(b, [a.id], true), []);
  assert.ok(c.locked && !d.locked);
  d.locked = true;
  assert.deepEqual(unlockAll(b), [a.id, c.id, d.id]);
  assert.deepEqual(unlockAll(b), []);
  assert.deepEqual(setLocked(b, [404], true), []);
  assert.deepEqual(setLocked(b, [d.id], 1), [d.id]);
  assert.equal(d.locked, true);
});

// ---------- align ----------
test('align by selection bounds, all six directions', () => {
  const mk = () => { const b = emptyBoard(); const a = rect(b, 0, 0, 40, 20); const c = rect(b, 100, 50, 60, 30); const d = rect(b, 30, 200, 20, 100); return { b, a, c, d }; };
  let { b, a, c, d } = mk();
  assert.deepEqual(align(b, [a.id, c.id, d.id], 'left'), [c.id, d.id]);
  assert.deepEqual([a.x, c.x, d.x], [0, 0, 0]);
  ({ b, a, c, d } = mk()); align(b, ids(b), 'right');
  assert.deepEqual([a.x + a.w, c.x + c.w, d.x + d.w], [160, 160, 160]);
  ({ b, a, c, d } = mk()); align(b, ids(b), 'hcenter');
  assert.deepEqual([a.x + 20, c.x + 30, d.x + 10], [80, 80, 80]);
  ({ b, a, c, d } = mk()); align(b, ids(b), 'top');
  assert.deepEqual([a.y, c.y, d.y], [0, 0, 0]);
  ({ b, a, c, d } = mk()); align(b, ids(b), 'bottom');
  assert.deepEqual([a.y + 20, c.y + 30, d.y + 100], [300, 300, 300]);
  ({ b, a, c, d } = mk()); align(b, ids(b), 'vcenter');
  assert.deepEqual([a.y + 10, c.y + 15, d.y + 50], [150, 150, 150]);
});

test('align: edge cases (single, invalid dir, locked, NaN)', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0); const c = rect(b, 100, 100); const l = rect(b, 500, 500, 100, 50, { locked: true });
  assert.deepEqual(align(b, [a.id], 'left'), []);
  assert.deepEqual(align(b, [a.id, c.id], 'middle'), []);
  assert.deepEqual(align(b, [a.id, c.id, l.id], 'right'), [a.id]); // locked l is ignored: bounds are a..c only
  assert.equal(l.x, 500);
  assert.deepEqual(align(b, [a.id, 12345], 'left'), []);
  assert.deepEqual(align(b, [], 'left'), []);
  const n = rect(b, NaN, 0);
  assert.doesNotThrow(() => align(b, [a.id, n.id], 'left'));
});

test('align treats groups as units and frames carry children', () => {
  const b = emptyBoard();
  const g1 = rect(b, 0, 0, 20, 20, { groupIds: [7] }); const g2 = rect(b, 100, 0, 20, 20, { groupIds: [7] });
  const o = rect(b, 300, 300, 20, 20);
  align(b, [g1.id, g2.id, o.id], 'left');
  assert.deepEqual([g1.x, g2.x, o.x], [0, 100, 0]); // group keeps its internal layout
  const b2 = emptyBoard();
  const f = add(b2, 'frame', { x: 100, y: 0, w: 200, h: 200 }); const k = rect(b2, 120, 20, 20, 20, { frameId: f.id }); const o2 = rect(b2, 0, 500, 50, 50);
  align(b2, [f.id, o2.id], 'left');
  assert.equal(f.x, 0); assert.equal(k.x, 20); // child moved once, with its frame
});

test('align moves free arrow ends of selected arrows, bound ends follow', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 200, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 225]], from: s.id, to: d.id });
  const free = add(b, 'arrow', { points: [[500, 0], [600, 40]] });
  align(b, [d.id, free.id], 'left');
  assert.equal(d.x, 300);
  assert.equal(free.x, 300);
  const pts = resolvePoints(b, ar);
  assert.ok(Math.abs(pts[1].x - 300) < 1e-6 || pts[1].x >= 300 - 1e-6); // still ends on d
});

// ---------- distribute ----------
test('distribute h/v gives equal gaps', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 20, 20); const c = rect(b, 30, 0, 40, 20); const d = rect(b, 500, 0, 20, 20); const e = rect(b, 70, 0, 60, 20);
  const changed = distribute(b, [a.id, c.id, d.id, e.id], 'h');
  const sorted = [a, c, e, d].sort((p, q) => p.x - q.x);
  const gaps = [1, 2, 3].map((i) => sorted[i].x - (sorted[i - 1].x + sorted[i - 1].w));
  assert.ok(gaps.every((g) => Math.abs(g - gaps[0]) < 1e-9), JSON.stringify(gaps));
  assert.ok(changed.length >= 1 && !changed.includes(a.id) && !changed.includes(d.id));
  assert.equal(a.x, 0); assert.equal(d.x, 500);
  const b2 = emptyBoard();
  const [p, q, r] = [0, 40, 400].map((y) => rect(b2, 0, y, 10, 20));
  distribute(b2, ids(b2), 'v');
  assert.equal(q.y, 200);
  assert.equal(p.y, 0); assert.equal(r.y, 400);
});

test('distribute edge cases: <3, invalid axis, locked, groups as units', () => {
  const b = emptyBoard();
  const [a, c, d] = [0, 10, 100].map((x) => rect(b, x, 0, 10, 10));
  assert.deepEqual(distribute(b, [a.id, c.id], 'h'), []);
  assert.deepEqual(distribute(b, ids(b), 'x'), []);
  d.locked = true;
  assert.deepEqual(distribute(b, ids(b), 'h'), []); // only 2 usable
  d.locked = false;
  assert.deepEqual(distribute(b, ids(b), 'h'), [c.id]);
  assert.equal(c.x, 50);
  const b2 = emptyBoard();
  const g1 = rect(b2, 0, 0, 10, 10, { groupIds: [5] }); const g2 = rect(b2, 20, 0, 10, 10, { groupIds: [5] }); const m = rect(b2, 30, 0, 10, 10); const z = rect(b2, 300, 0, 10, 10);
  // units: group (0..30), m (30..40), z (300..310) -> m moves to the centre gap
  distribute(b2, ids(b2), 'h');
  assert.equal(g1.x, 0); assert.equal(g2.x, 20);
  assert.equal(m.x, 30 + ((310 - 0 - (30 + 10 + 10)) / 2));
});

// ---------- flip ----------
test('flip h mirrors positions about the selection centre', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 20, 20); const c = rect(b, 80, 40, 20, 20);
  assert.deepEqual(flip(b, [a.id, c.id], 'h'), [a.id, c.id]);
  assert.equal(a.x, 80); assert.equal(c.x, 0);
  assert.equal(a.y, 0); assert.equal(c.y, 40);
  flip(b, [a.id, c.id], 'h');
  assert.deepEqual([a.x, c.x], [0, 80]);
  flip(b, [a.id, c.id], 'v');
  assert.equal(a.y, 40); assert.equal(c.y, 0);
  const rot = rect(b, 0, 0, 40, 20, { angle: 0.5 });
  const cx = rot.x + 20;
  flip(b, [rot.id], 'h'); // a single element flips about its own centre
  assert.equal(rot.angle, -0.5);
  assert.ok(Math.abs(rot.x + 20 - cx) < 1e-9);
});

test('flip mirrors lines, freedraw and arrows (heads and bindings swap)', () => {
  const b = emptyBoard();
  const ln = add(b, 'line', { x: 0, y: 0, points: [[0, 0], [100, 40]] });
  const fd = add(b, 'freedraw', { x: 10, y: 10, points: [[0, 0], [30, 10], [60, 0]], w: 60, h: 10 });
  flip(b, [ln.id, fd.id], 'h');
  const lb = elementBounds(b, ln);
  assert.deepEqual([lb.x, lb.w], [0, 100]);
  assert.deepEqual([ln.x, ln.y], [0, 40]); // reversed path: starts at the (new) left end
  assert.deepEqual(ln.points, [[0, 0], [100, -40]]);
  assert.deepEqual(fd.points.map((p) => p[0]), [0, -30, -60].map((v) => v)); // freedraw keeps its point order, mirrored
  const fb = elementBounds(b, fd);
  assert.ok(Math.abs(fb.w - 60) < 1e-9);
  // arrow between two shapes, flipped together
  const b2 = emptyBoard();
  const s = rect(b2, 0, 0, 100, 50); const d = rect(b2, 300, 0, 100, 50);
  const ar = add(b2, 'arrow', { points: [[100, 25], [300, 25]], from: s.id, to: d.id, startArrowhead: 'bar', endArrowhead: 'triangle' });
  flip(b2, [s.id, d.id, ar.id], 'h');
  assert.equal(s.x, 300); assert.equal(d.x, 0);
  assert.equal(ar.from, d.id); assert.equal(ar.to, s.id);
  assert.equal(ar.startArrowhead, 'triangle'); assert.equal(ar.endArrowhead, 'bar');
  const pts = resolvePoints(b2, ar);
  assert.ok(pts[0].x < pts[1].x); // starts at the (now left) shape d... which is the head side
  // semantic preserved: head 'triangle' is still at the shape that was `to`
  assert.equal(ar.startArrowhead === 'triangle' && ar.from === d.id, true);
  // vertical flip keeps order
  flip(b2, [s.id, d.id, ar.id], 'v');
  assert.equal(ar.from, d.id);
});

test('flip an arrow whose shapes are not selected unbinds and keeps geometry finite', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 25]], from: s.id, to: d.id });
  flip(b, [ar.id], 'h');
  assert.equal(ar.from, null); assert.equal(ar.to, null);
  assert.ok(ar.points.flat().every(Number.isFinite));
  assert.deepEqual(flip(b, [], 'h'), []);
  assert.deepEqual(flip(b, [ar.id], 'z'), []);
  assert.deepEqual(flip(b, [404], 'h'), []);
});

test('flip frame carries children; locked ignored', () => {
  const b = emptyBoard();
  const f = add(b, 'frame', { x: 0, y: 0, w: 200, h: 100 }); const k = rect(b, 10, 10, 20, 20, { frameId: f.id });
  const l = rect(b, 900, 900, 20, 20, { locked: true });
  flip(b, [f.id, l.id], 'h');
  assert.equal(f.x, 0); assert.equal(k.x, 170); assert.equal(l.x, 900);
});

// ---------- duplicate ----------
test('duplicate offsets, keeps clones right after the originals, remaps groups/bindings', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 40, 40, { groupIds: [77] }); const c = rect(b, 100, 0, 40, 40, { groupIds: [77] });
  const top = rect(b, 500, 500);
  b.nextId = 100;
  const ar = add(b, 'arrow', { points: [[40, 20], [100, 20]], from: a.id, to: c.id });
  const out = duplicate(b, [a.id, c.id, ar.id], { dx: 7, dy: 9 });
  assert.equal(out.length, 3);
  const [a2, c2, ar2] = out.map((id) => findItem(b, id));
  assert.equal(a2.x, 7); assert.equal(a2.y, 9);
  assert.equal(a2.groupIds.length, 1);
  assert.notEqual(a2.groupIds[0], 77);
  assert.equal(a2.groupIds[0], c2.groupIds[0]);
  assert.equal(ar2.from, a2.id); assert.equal(ar2.to, c2.id);
  // z-order: originals, then clones, then the later `top`? clones go right after the last original (the arrow)
  assert.deepEqual(ids(b), [a.id, c.id, top.id, ar.id, ...out]);
  assert.equal(new Set(ids(b)).size, b.items.length);
});

test('duplicate defaults, hostile input, frame children, arrow bound outside', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0);
  const [id2] = duplicate(b, [a.id]);
  assert.equal(findItem(b, id2).x, 20);
  assert.deepEqual(duplicate(b, [404]), []);
  assert.deepEqual(duplicate(b, null), []);
  assert.doesNotThrow(() => duplicate(b, [a.id], { dx: NaN, dy: 'x' }));
  const b2 = emptyBoard();
  const f = add(b2, 'frame', { x: 0, y: 0, w: 300, h: 300 }); const k = rect(b2, 10, 10, 20, 20, { frameId: f.id });
  const out = duplicate(b2, [f.id], { dx: 400, dy: 0 });
  const [f2, k2] = out.map((id) => findItem(b2, id));
  assert.equal(f2.type, 'frame');
  assert.equal(k2.frameId, f2.id);
  assert.equal(k.frameId, f.id);
  assert.ok(b2.items.indexOf(k2) > b2.items.indexOf(f2));
  // child duplicated alone stays in its frame
  const [k3] = duplicate(b2, [k.id], { dx: 5, dy: 5 });
  assert.equal(findItem(b2, k3).frameId, f.id);
  // arrow with a shape outside the copy keeps its drawn end and no binding
  const b3 = emptyBoard();
  const s = rect(b3, 0, 0, 100, 50); const ar = add(b3, 'arrow', { points: [[100, 25], [250, 25]], from: s.id });
  const [ar2] = duplicate(b3, [ar.id], { dx: 0, dy: 50 });
  assert.equal(findItem(b3, ar2).from, null);
  assert.equal(findItem(b3, ar2).y, 75);
});

test('duplicate skips locked elements', () => {
  const b = emptyBoard();
  const l = rect(b, 0, 0, 10, 10, { locked: true });
  assert.deepEqual(duplicate(b, [l.id]), []);
  assert.equal(b.items.length, 1);
});

// ---------- deleteElements ----------
test('deleteElements unbinds arrows but keeps their drawn ends', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 25]], from: s.id, to: d.id });
  const before = resolvePoints(b, ar);
  assert.deepEqual(deleteElements(b, [s.id]), [s.id]);
  assert.equal(ar.from, null); assert.equal(ar.to, d.id);
  const after = resolvePoints(b, ar);
  assert.ok(Math.abs(after[0].x - before[0].x) < 1e-6 && Math.abs(after[0].y - before[0].y) < 1e-6);
  deleteElements(b, [d.id]);
  assert.equal(ar.to, null);
  const end = resolvePoints(b, ar)[1];
  assert.ok(Math.abs(end.x - 300) < 1e-6);
  assert.equal(b.items.length, 1);
});

test('deleteElements: frames take children, unknown ids, groups pruned, deleted arrow not touched', () => {
  const b = emptyBoard();
  const f = add(b, 'frame', { w: 300, h: 300 }); const k = rect(b, 10, 10, 20, 20, { frameId: f.id }); const out = rect(b, 900, 900);
  const a = rect(b, 0, 0, 10, 10, { groupIds: [5] }); const c = rect(b, 0, 0, 10, 10, { groupIds: [5] });
  assert.deepEqual(deleteElements(b, [f.id, 999, 'x']), [f.id, k.id]);
  assert.deepEqual(ids(b), [out.id, a.id, c.id]);
  assert.deepEqual(deleteElements(b, [a.id]), [a.id]);
  assert.deepEqual(c.groupIds, []);
  assert.deepEqual(deleteElements(b, []), []);
  assert.deepEqual(deleteElements(b, undefined), []);
  const s = rect(b, 0, 0); const ar = add(b, 'arrow', { points: [[0, 0], [5, 5]], from: s.id });
  deleteElements(b, [s.id, ar.id]);
  assert.equal(findItem(b, ar.id), null);
});

test('deleteElements: elbow arrow collapses to its two drawn ends', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 200, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 225]], from: s.id, to: d.id, arrowType: 'elbow' });
  deleteElements(b, [s.id]);
  assert.equal(ar.points.length, 2);
  assert.ok(ar.points.flat().every(Number.isFinite));
});

// ---------- moveElements ----------
test('moveElements: shapes, frames with children, bound arrows follow implicitly', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 25]], from: s.id, to: d.id });
  const snapshot = JSON.stringify(ar);
  moveElements(b, [s.id], 0, 100);
  assert.equal(s.y, 100);
  assert.equal(JSON.stringify(ar), snapshot); // arrow not edited
  const pts = resolvePoints(b, ar);
  assert.ok(pts[0].y > 50);
  const f = add(b, 'frame', { x: 0, y: 500, w: 200, h: 200 }); const k = rect(b, 10, 510, 20, 20, { frameId: f.id });
  moveElements(b, [f.id], 5, 7);
  assert.deepEqual([f.x, f.y, k.x, k.y], [5, 507, 15, 517]);
  moveElements(b, [k.id], 1, 1);
  assert.deepEqual([f.x, k.x], [5, 16]);
});

test('moveElements: arrows with bound ends outside the set only move free ends', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [200, 25]], from: s.id });
  moveElements(b, [ar.id], 0, 40);
  assert.equal(ar.from, s.id);
  assert.deepEqual([ar.x, ar.y], [100, 25]);
  assert.deepEqual(ar.points[1], [100, 40]);
  assert.equal(ar.h, 40);
  // end bound outside, start free
  const b2 = emptyBoard();
  const d = rect(b2, 300, 0, 100, 50);
  const a2 = add(b2, 'arrow', { points: [[100, 25], [150, 60], [300, 25]], to: d.id });
  moveElements(b2, [a2.id], 10, 10);
  assert.deepEqual([a2.x, a2.y], [110, 35]);
  assert.deepEqual(a2.points[2], [190, -10]); // the bound end kept its absolute stored position (300, 25)
  assert.equal(a2.to, d.id);
});

test('moveElements: arrow and both shapes together simply translate; NaN ignored', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50); const d = rect(b, 300, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[100, 25], [300, 25]], from: s.id, to: d.id });
  moveElements(b, ids(b), 10, 20);
  assert.deepEqual([s.x, d.x, ar.x, ar.y], [10, 310, 110, 45]);
  assert.deepEqual(ar.points, [[0, 0], [200, 0]]);
  moveElements(b, ids(b), NaN, 5);
  moveElements(b, [999], 5, 5);
  assert.equal(s.x, 10);
});

// ---------- assignFrames ----------
test('assignFrames sets topmost containing frame by centre and clears', () => {
  const b = emptyBoard();
  const f1 = add(b, 'frame', { x: 0, y: 0, w: 400, h: 400 }); const f2 = add(b, 'frame', { x: 100, y: 100, w: 100, h: 100 });
  const inBoth = rect(b, 120, 120, 20, 20); const inOne = rect(b, 10, 10, 20, 20); const outside = rect(b, 1000, 1000, 20, 20, { frameId: f1.id });
  const straddle = rect(b, 390, 390, 60, 60); // centre 420 -> outside f1
  const changed = assignFrames(b, ids(b));
  assert.equal(inBoth.frameId, f2.id);
  assert.equal(inOne.frameId, f1.id);
  assert.equal(outside.frameId, null);
  assert.equal(straddle.frameId, null);
  assert.equal(f1.frameId, null); assert.equal(f2.frameId, null);
  assert.deepEqual(changed, [inBoth.id, inOne.id, outside.id]);
  assert.deepEqual(assignFrames(b, ids(b)), []);
});

test('assignFrames leaves locked elements, fixes z-order, handles arrows and unknown ids', () => {
  const b = emptyBoard();
  const k = rect(b, 10, 10, 20, 20);
  const lk = rect(b, 10, 10, 20, 20, { locked: true });
  const f = add(b, 'frame', { x: 0, y: 0, w: 100, h: 100 });
  const ar = add(b, 'arrow', { points: [[10, 10], [40, 40]] });
  assignFrames(b, [k.id, lk.id, ar.id, 999]);
  assert.equal(k.frameId, f.id); assert.equal(lk.frameId, null); assert.equal(ar.frameId, f.id);
  assert.ok(b.items.indexOf(k) > b.items.indexOf(f)); // child moved above its frame
  assert.equal(b.items.indexOf(lk), 0);
});

// ---------- bindArrow ----------
test('bindArrow binds to topmost bindable shape under each end', () => {
  const b = emptyBoard();
  const fr = add(b, 'frame', { x: 0, y: 0, w: 500, h: 500 });
  const under = rect(b, 100, 100, 100, 100); const over = rect(b, 120, 120, 100, 100);
  const el = add(b, 'ellipse', { x: 300, y: 300, w: 100, h: 100 });
  const ar = add(b, 'arrow', { points: [[150, 150], [350, 350]] });
  bindArrow(b, ar.id, { tol: 4 });
  assert.equal(ar.from, over.id); // topmost
  assert.equal(ar.to, el.id);
  bindArrow(b, ar.id, { start: null, tol: 4 });
  assert.equal(ar.from, null);
  assert.equal(ar.to, el.id);
  bindArrow(b, ar.id, { end: null });
  assert.equal(ar.to, null);
  assert.ok(![under.id, fr.id].includes(ar.from));
  // ellipse corner is outside the curve
  const ar2 = add(b, 'arrow', { points: [[10, 10], [301, 301]] });
  bindArrow(b, ar2.id, { tol: 0 });
  assert.equal(ar2.to, null);
  bindArrow(b, ar2.id, { tol: 20 });
  assert.equal(ar2.to, el.id);
});

test('bindArrow: explicit points, tolerance, unbind, hostile input', () => {
  const b = emptyBoard();
  const s = rect(b, 0, 0, 100, 50);
  const ar = add(b, 'arrow', { points: [[500, 500], [600, 600]] });
  bindArrow(b, ar.id, { start: { x: 101, y: 25 }, tol: 3 });
  assert.equal(ar.from, s.id);
  assert.deepEqual([ar.x, ar.y], [101, 25]); // the end moved to the given point
  assert.equal(ar.to, null);
  bindArrow(b, ar.id, { start: { x: 140, y: 25 }, tol: 3 });
  assert.equal(ar.from, null);
  assert.doesNotThrow(() => bindArrow(b, 999, {}));
  assert.doesNotThrow(() => bindArrow(b, s.id, {}));
  assert.doesNotThrow(() => bindArrow(b, ar.id, { start: { x: NaN, y: 1 }, tol: NaN }));
  assert.doesNotThrow(() => bindArrow(b, ar.id));
  const rotated = rect(b, 1000, 0, 200, 20, { angle: Math.PI / 2 }); // rotated: now vertical around centre (1100, 10)
  bindArrow(b, ar.id, { end: { x: 1100, y: 90 }, tol: 2 });
  assert.equal(ar.to, rotated.id);
  // arrows and lines are never targets
  const other = add(b, 'arrow', { points: [[0, 0], [10, 10]] });
  bindArrow(b, ar.id, { end: { x: 5, y: 5 } });
  assert.equal(ar.to === other.id, false);
  for (const id of [ar.from, ar.to]) if (id != null) assert.ok(BINDABLE_TYPES.includes(findItem(b, id).type));
});

// ---------- findText ----------
test('findText searches text, frame names and arrow labels in z-order', () => {
  const b = emptyBoard();
  const a = rect(b, 0, 0, 10, 10, { text: 'API Gateway' }); const f = add(b, 'frame', { name: 'gateway tier' }); const ar = add(b, 'arrow', { points: [[0, 0], [5, 5]], text: 'to GATEWAY' });
  const t = add(b, 'text', { text: 'unrelated' }); const e = rect(b, 0, 0, 10, 10, { text: '' });
  assert.deepEqual(findText(b, 'gateway'), [a.id, f.id, ar.id]);
  assert.deepEqual(findText(b, 'GATEWAY', { caseSensitive: true }), [ar.id]);
  assert.deepEqual(findText(b, 'nope'), []);
  assert.deepEqual(findText(b, ''), []);
  assert.deepEqual(findText(b, null), []);
  assert.deepEqual(findText(b, 'UNRELATED'), [t.id]);
  assert.ok(!findText(b, 'e').includes(e.id));
  assert.deepEqual(findText(emptyBoard(), 'x'), []);
});

// ---------- property-style randomized test ----------
function rng(seed) {
  let s = seed >>> 0;
  return () => { s = (s + 0x6d2b79f5) >>> 0; let t = s; t = Math.imul(t ^ (t >>> 15), t | 1); t ^= t + Math.imul(t ^ (t >>> 7), t | 61); return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
}

function checkInvariants(b, label) {
  const seen = new Set();
  const byId = new Map();
  for (const e of b.items) { assert.ok(!seen.has(e.id), `${label}: duplicate id ${e.id}`); seen.add(e.id); byId.set(e.id, e); }
  const pos = new Map(b.items.map((e, i) => [e.id, i]));
  for (const e of b.items) {
    if (e.frameId != null) {
      assert.equal(byId.get(e.frameId)?.type, 'frame', `${label}: dangling frameId on ${e.id}`);
      assert.ok(pos.get(e.frameId) < pos.get(e.id), `${label}: child ${e.id} below its frame`);
    }
    if (e.type === 'arrow') for (const k of ['from', 'to']) if (e[k] != null) assert.ok(BINDABLE_TYPES.includes(byId.get(e[k])?.type), `${label}: dangling ${k} on ${e.id}`);
    for (const g of e.groupIds) { assert.ok(Number.isInteger(g)); assert.ok(g < b.nextId, `${label}: group id beyond nextId`); }
    assert.equal(new Set(e.groupIds).size, e.groupIds.length);
    for (const k of ['x', 'y', 'w', 'h', 'angle']) assert.ok(Number.isFinite(e[k]), `${label}: ${k} not finite on ${e.id}`);
    if (e.points) { assert.deepEqual(e.points[0], [0, 0]); assert.ok(e.points.flat().every(Number.isFinite)); }
    assert.ok(e.id < b.nextId);
  }
  const clean = sanitizeBoard(b);
  assert.deepEqual(clean.items.map((e) => e.id), b.items.map((e) => e.id), `${label}: sanitizeBoard dropped something`);
  // group membership: every group id has >= 1 element (singletons are allowed except after deletes, which prune)
}

function randomBoard(r) {
  const b = emptyBoard();
  const n = 4 + Math.floor(r() * 10);
  const frames = [];
  for (let i = 0; i < 2; i++) if (r() < 0.6) frames.push(add(b, 'frame', { x: r() * 300, y: r() * 300, w: 150 + r() * 200, h: 150 + r() * 200 }));
  const types = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text'];
  for (let i = 0; i < n; i++) {
    const el = add(b, types[Math.floor(r() * types.length)], { x: r() * 600, y: r() * 600, w: 20 + r() * 100, h: 20 + r() * 80, angle: r() < 0.2 ? r() : 0, text: r() < 0.5 ? `n${i}` : '' });
    if (r() < 0.3 && frames.length) el.frameId = frames[Math.floor(r() * frames.length)].id;
  }
  const shapes = b.items.filter((e) => BINDABLE_TYPES.includes(e.type));
  const m = Math.floor(r() * 5);
  for (let i = 0; i < m; i++) {
    const s = shapes[Math.floor(r() * shapes.length)]; const d = shapes[Math.floor(r() * shapes.length)];
    add(b, r() < 0.8 ? 'arrow' : 'line', { points: [[s.x + s.w, s.y + s.h / 2], [r() < 0.5 ? d.x : r() * 600, d.y + d.h / 2], ...(r() < 0.3 ? [[300, 300]] : [])], from: r() < 0.7 ? s.id : null, to: r() < 0.7 ? d.id : null, arrowType: r() < 0.3 ? 'elbow' : 'sharp' });
  }
  for (const l of b.items.filter((e) => e.type === 'line')) { l.from = null; l.to = null; l.arrowType = 'sharp'; }
  if (r() < 0.5) add(b, 'freedraw', { x: r() * 500, y: r() * 500, points: [[0, 0], [10, 5], [20, 0]], w: 20, h: 5 });
  // frames first keeps the frame-below-children invariant
  b.items.sort((p, q) => (p.type === 'frame' ? 0 : 1) - (q.type === 'frame' ? 0 : 1));
  return b;
}

test('randomized operation sequences keep the board valid (seeded)', () => {
  for (let seed = 1; seed <= 60; seed++) {
    const r = rng(seed);
    const b = randomBoard(r);
    checkInvariants(b, `seed ${seed} initial`);
    for (let step = 0; step < 40; step++) {
      const all = ids(b);
      const pick = () => { const k = Math.floor(r() * 5); return Array.from({ length: k }, () => (r() < 0.05 ? 99999 + step : all[Math.floor(r() * all.length)])); };
      const sel = pick();
      const op = Math.floor(r() * 18);
      const beforeIds = new Set(ids(b));
      const label = `seed ${seed} step ${step} op ${op}`;
      switch (op) {
        case 0: applyStyle(b, sel, { strokeColor: '#e03131', fontSize: 8 + r() * 50, fontFamily: 1 + Math.floor(r() * 3), arrowType: ['sharp', 'round', 'elbow'][Math.floor(r() * 3)], opacity: r() * 120 - 10 }); break;
        case 1: reorder(b, sel, ['front', 'forward', 'backward', 'back'][Math.floor(r() * 4)]); break;
        case 2: group(b, sel); break;
        case 3: ungroup(b, sel); break;
        case 4: align(b, sel, ['left', 'right', 'top', 'bottom', 'hcenter', 'vcenter'][Math.floor(r() * 6)]); break;
        case 5: distribute(b, sel, r() < 0.5 ? 'h' : 'v'); break;
        case 6: flip(b, sel, r() < 0.5 ? 'h' : 'v'); break;
        case 7: duplicate(b, sel, { dx: r() * 50, dy: r() * 50 }); break;
        case 8: if (r() < 0.5) deleteElements(b, sel.slice(0, 1)); break;
        case 9: moveElements(b, sel, r() * 100 - 50, r() * 100 - 50); break;
        case 10: assignFrames(b, sel); break;
        case 11: { const a = b.items.find((e) => e.type === 'arrow'); if (a) bindArrow(b, a.id, { tol: r() * 20, start: r() < 0.2 ? null : undefined }); break; }
        case 12: setLocked(b, sel, r() < 0.5); break;
        case 13: unlockAll(b); break;
        case 14: pasteStyle(b, sel, copyStyle(b.items[Math.floor(r() * b.items.length)])); break;
        case 15: expandSelection(b, sel); break;
        case 16: findText(b, 'n'); styleSummary(b, sel, {}, null); break;
        default: assignFrames(b, all); break;
      }
      checkInvariants(b, label);
      const afterIds = new Set(ids(b));
      if ([1, 2, 3, 4, 5, 6, 9, 10, 12, 13, 14, 15, 16].includes(op)) assert.deepEqual([...afterIds].sort(), [...beforeIds].sort(), `${label}: element set changed`);
      if (op === 7) for (const id of beforeIds) assert.ok(afterIds.has(id), `${label}: duplicate removed an element`);
      if (op === 8) for (const id of afterIds) assert.ok(beforeIds.has(id), `${label}: delete added an element`);
    }
  }
});

test('selectionBounds sanity after ops (align keeps the overall box)', () => {
  const r = rng(42);
  for (let k = 0; k < 20; k++) {
    const b = emptyBoard();
    const els = Array.from({ length: 4 }, () => rect(b, r() * 500, r() * 500, 10 + r() * 80, 10 + r() * 80));
    const before = selectionBounds(b, els);
    align(b, ids(b), 'left');
    const after = selectionBounds(b, els);
    assert.ok(Math.abs(after.x - before.x) < 1e-9);
    assert.ok(els.every((e) => Math.abs(e.x - before.x) < 1e-9));
  }
});

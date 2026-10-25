import assert from 'node:assert/strict';
import test from 'node:test';
import {
  arrowEnds, boardSignature, boardStats, describeBoard, edgePoint, emptyBoard, LIMITS, sanitizeBoard,
} from '../shared/board.js';

const shape = (id, type, x, y, text, w = 100, h = 60) => ({ id, type, x, y, w, h, text });
const arrow = (id, from, to, text = '', extra = {}) => ({ id, type: 'arrow', from, to, x1: 0, y1: 0, x2: 10, y2: 10, text, ...extra });

function sample() {
  return {
    nextId: 10,
    items: [
      shape(1, 'ellipse', 0, 100, 'Client'),
      shape(2, 'rect', 300, 100, 'API Server'),
      shape(3, 'database', 600, 100, 'Links DB'),
      shape(4, 'rect', 300, 300, 'Cache'),
      shape(5, 'text', 20, 400, 'reads 100x writes', 150, 30),
      arrow(6, 1, 2, 'HTTPS'),
      arrow(7, 2, 3),
      arrow(8, 2, null, 'to nowhere'),
    ],
  };
}

test('empty board is described as empty', () => {
  assert.equal(describeBoard(emptyBoard()), 'The whiteboard is empty.');
  assert.deepEqual(boardStats(emptyBoard()), { components: 0, connections: 0, arrows: 0, notes: 0 });
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

test('components are listed left to right and unlabelled ones are called out', () => {
  const board = { nextId: 5, items: [shape(1, 'rect', 500, 0, ''), shape(2, 'rect', 10, 0, 'First')] };
  const text = describeBoard(board);
  assert.ok(text.indexOf('"First"') < text.indexOf('"unlabelled rect"'));
});

test('boardStats counts only fully attached arrows as connections', () => {
  assert.deepEqual(boardStats(sample()), { components: 4, connections: 2, arrows: 3, notes: 1 });
});

test('edgePoint leaves rectangles and ellipses on their boundary', () => {
  const r = shape(1, 'rect', 0, 0, '', 100, 60);
  assert.deepEqual(edgePoint(r, 300, 30), { x: 100, y: 30 });
  assert.deepEqual(edgePoint(r, 50, -200), { x: 50, y: 0 });
  const e = shape(2, 'ellipse', 0, 0, '', 100, 60);
  const p = edgePoint(e, 300, 30);
  assert.ok(Math.abs(p.x - 100) < 1e-9 && Math.abs(p.y - 30) < 1e-9);
  assert.deepEqual(edgePoint(r, 50, 30), { x: 50, y: 30 }, 'target at the centre returns the centre');
});

test('arrowEnds follows bound shapes and keeps free ends', () => {
  const board = sample();
  const e = arrowEnds(board, board.items[5]); // Client(0..100) -> API Server(300..400)
  assert.ok(e.x1 > 90 && e.x1 <= 100, `starts on the right edge of Client: ${e.x1}`);
  assert.ok(e.x2 >= 300 && e.x2 < 310, `ends on the left edge of API Server: ${e.x2}`);
  const loose = arrowEnds(board, board.items[7]);
  assert.equal(loose.x2, 10);
  assert.equal(loose.y2, 10);
  assert.equal(loose.toShape, null);
});

test('sanitizeBoard tolerates garbage and normalises numbers', () => {
  assert.deepEqual(sanitizeBoard(null), emptyBoard());
  assert.deepEqual(sanitizeBoard('x'), emptyBoard());
  assert.deepEqual(sanitizeBoard({ items: 'no' }), emptyBoard());
  const b = sanitizeBoard({
    nextId: 'x',
    items: [
      { id: 1, type: 'rect', x: NaN, y: 5, w: -3, h: 1e9, text: 'a'.repeat(500) },
      { id: 1, type: 'rect', x: 0, y: 0, w: 10, h: 10 },               // duplicate id
      { id: 2, type: 'banana', x: 0, y: 0 },                           // unknown type
      { id: 3, type: 'arrow', from: 1, to: 99, x1: 1, y1: 2, x2: 'z', y2: 4 },
      null, 5,
    ],
  });
  assert.equal(b.items.length, 2);
  const [r, a] = b.items;
  assert.equal(r.x, 0);
  assert.equal(r.w, 20);
  assert.equal(r.h, 2000);
  assert.equal(r.text.length, LIMITS.text);
  assert.equal(a.from, 1);
  assert.equal(a.to, null, 'binding to a missing shape is dropped');
  assert.equal(a.x2, 0);
  assert.equal(b.nextId, 4);
});

test('sanitizeBoard caps the number of items', () => {
  const items = Array.from({ length: LIMITS.items + 50 }, (_, i) => shape(i + 1, 'rect', i, i, 'x'));
  assert.equal(sanitizeBoard({ items }).items.length, LIMITS.items);
});

test('boardSignature ignores small drags but notices real edits', () => {
  const a = sample();
  const base = boardSignature(a);
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
});

import test from 'node:test';
import assert from 'node:assert/strict';
import {
  serializeScene, parseScene, exportExcalidraw, importExcalidraw, importExcalidrawDetailed, serializeLibrary, parseLibrary,
  serializeExcalidrawLibrary, libraryItemFromElements, instantiateLibraryItem, downloadText, pickFile,
} from '../public/wb/interop.js';
import { createElement, emptyBoard, newId, sanitizeBoard, resolvePoints, findItem, selectionBounds } from '../shared/board.js';

const PNG = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';

function sampleBoard() {
  const b = emptyBoard();
  const add = (type, props) => { const e = createElement(type, { id: newId(b), ...props }); b.items.push(e); return e; };
  const frame = add('frame', { x: -40, y: -40, w: 700, h: 400, name: 'System' });
  const g = newId(b);
  const a = add('rect', { x: 0, y: 0, w: 160, h: 80, text: 'API\nGateway', backgroundColor: '#a5d8ff', strokeColor: '#1971c2', fillStyle: 'hachure', strokeWidth: 4, roughness: 2, opacity: 70, groupIds: [g], frameId: frame.id, link: 'https://example.com/a' });
  const d = add('database', { x: 300, y: -15, w: 110, h: 110, text: 'DB', backgroundColor: '#b2f2bb', groupIds: [g], frameId: frame.id });
  const lb = add('diamond', { x: 0, y: 200, w: 140, h: 100, text: 'LB', strokeStyle: 'dashed', roundness: 'sharp', angle: 0.3 });
  const el = add('ellipse', { x: 500, y: 200, w: 120, h: 80, text: 'User', locked: true, fontFamily: 3, fontSize: 16 });
  const st = add('sticky', { x: 500, y: 20, w: 120, h: 120, text: 'note' });
  const ar = add('arrow', { x: 160, y: 40, points: [[0, 0], [140, 0]], w: 140, h: 0, from: a.id, to: d.id, text: 'writes', endArrowhead: 'crowfoot_many', startArrowhead: 'bar', arrowType: 'round', strokeStyle: 'dotted' });
  const el2 = add('arrow', { x: 70, y: 200, points: [[0, 0], [0, -80]], w: 0, h: 80, from: lb.id, to: a.id, arrowType: 'elbow', endArrowhead: 'triangle' });
  add('line', { x: 0, y: 330, points: [[0, 0], [100, 20], [200, 0]], arrowType: 'round', w: 200, h: 20 });
  add('freedraw', { x: 300, y: 300, points: [[0, 0], [10, 5], [20, 2], [30, 20]], strokeColor: '#e03131' });
  add('text', { x: 250, y: 330, w: 120, h: 50, text: 'hello\nworld', fontSize: 20, fontFamily: 2, textAlign: 'center' });
  add('image', { x: 400, y: 300, w: 60, h: 40, dataUrl: PNG });
  void el; void st; void ar; void el2;
  return b;
}
const norm = (b) => sanitizeBoard(b);

test('scene round trip and appState whitelist', () => {
  const board = sampleBoard();
  const text = serializeScene(board, { appState: { theme: 'dark', background: '#f8f9fa', grid: true, evil: 'x', zoom: 3 } });
  const j = JSON.parse(text);
  assert.equal(j.type, 'hld-whiteboard'); assert.equal(j.version, 2); assert.deepEqual(Object.keys(j.appState).sort(), ['background', 'grid', 'theme']);
  const r = parseScene(text);
  assert.deepEqual(r.board, norm(board));
  assert.deepEqual(r.appState, { theme: 'dark', background: '#f8f9fa', grid: true });
});

test('parseScene accepts bare boards and arrays; rejects junk', () => {
  const board = sampleBoard();
  assert.equal(parseScene(JSON.stringify({ items: board.items, nextId: board.nextId })).board.items.length, board.items.length);
  assert.equal(parseScene(JSON.stringify([{ id: 'a', type: 'rectangle', x: 1, y: 2, width: 30, height: 40 }])).board.items.length, 1);
  assert.throws(() => parseScene('not json'), /valid JSON/);
  assert.throws(() => parseScene('42'), /not a whiteboard/);
  assert.throws(() => parseScene('{"foo":1}'), /Unrecognised/);
  assert.throws(() => parseScene('[1,2]'), /array/);
  assert.throws(() => parseScene('x'.repeat(26 * 1024 * 1024)), /too large/);
  const hostile = parseScene(JSON.stringify({ type: 'hld-whiteboard', items: [{ id: 1, type: 'rect', x: 'a', strokeColor: 'url(javascript:1)', text: 5, link: 'javascript:alert(1)' }, { id: 2, type: 'image', dataUrl: 'javascript:1' }], appState: { theme: 'neon', background: '<x>' } }));
  assert.equal(hostile.board.items[0].link, null); assert.equal(hostile.board.items[1].dataUrl, ''); assert.deepEqual(hostile.appState, {});
});

test('exportExcalidraw produces a loadable Excalidraw v2 document', () => {
  const ex = exportExcalidraw(sampleBoard(), { appState: { grid: true, background: '#fff' } });
  assert.equal(ex.type, 'excalidraw'); assert.equal(ex.version, 2);
  assert.equal(ex.appState.gridSize, 20); assert.equal(ex.appState.viewBackgroundColor, '#fff');
  const ids = new Set();
  let lastIndex = '';
  for (const e of ex.elements) {
    assert.equal(typeof e.id, 'string'); assert.ok(!ids.has(e.id)); ids.add(e.id);
    for (const k of ['x', 'y', 'width', 'height', 'angle', 'seed', 'version', 'versionNonce', 'updated']) assert.ok(Number.isFinite(e[k]), `${e.type}.${k}`);
    for (const k of ['strokeColor', 'backgroundColor', 'fillStyle', 'strokeStyle', 'index']) assert.equal(typeof e[k], 'string');
    assert.equal(e.isDeleted, false); assert.ok(Array.isArray(e.groupIds)); assert.ok('boundElements' in e); assert.ok('link' in e); assert.ok('locked' in e); assert.ok('roundness' in e);
    assert.ok(e.index > lastIndex, 'fractional index ascending'); lastIndex = e.index;
  }
  // bound text
  const rect = ex.elements.find((e) => e.type === 'rectangle' && !e.customData);
  const label = ex.elements.find((e) => e.containerId === rect.id);
  assert.ok(label && label.type === 'text'); assert.equal(label.originalText, 'API\nGateway');
  assert.ok(rect.boundElements.some((b) => b.type === 'text' && b.id === label.id));
  assert.ok(rect.boundElements.some((b) => b.type === 'arrow'));
  const arrow = ex.elements.find((e) => e.type === 'arrow' && e.endArrowhead === 'crowfoot_many');
  assert.equal(arrow.startBinding.elementId, rect.id); assert.ok(arrow.startBinding.fixedPoint.every((n) => n >= 0 && n <= 1));
  assert.ok(ex.elements.some((e) => e.containerId === arrow.id)); assert.equal(arrow.roundness.type, 2);
  assert.equal(ex.elements.find((e) => e.type === 'arrow' && e.elbowed).roundness, null);
  const img = ex.elements.find((e) => e.type === 'image');
  assert.equal(ex.files[img.fileId].dataURL, PNG); assert.equal(ex.files[img.fileId].mimeType, 'image/png'); assert.ok(ex.files[img.fileId].created);
  assert.deepEqual(rect.groupIds.length, 1);
  assert.ok(ex.elements.find((e) => e.type === 'frame').name === 'System');
  JSON.parse(JSON.stringify(ex));
});

function summary(b) {
  return b.items.map((e) => ({
    type: e.type, text: e.text ?? e.name ?? '', stroke: e.strokeColor, bg: e.backgroundColor, fill: e.fillStyle, sw: e.strokeWidth, ss: e.strokeStyle, r: e.roughness, o: e.opacity,
    rd: e.roundness, locked: e.locked, link: e.link, ff: e.fontFamily, fs: e.fontSize, ta: e.textAlign, va: e.verticalAlign,
    ah: [e.arrowType, e.startArrowhead, e.endArrowhead], gl: e.groupIds.length, hasFrame: e.frameId != null, from: e.from != null, to: e.to != null,
    bind: [e.from != null ? b.items.findIndex((x) => x.id === e.from) : -1, e.to != null ? b.items.findIndex((x) => x.id === e.to) : -1],
    frameIdx: e.frameId != null ? b.items.findIndex((x) => x.id === e.frameId) : -1,
  }));
}

test('our board -> Excalidraw -> our board keeps everything', () => {
  const orig = norm(sampleBoard());
  const back = importExcalidraw(JSON.parse(JSON.stringify(exportExcalidraw(orig))));
  assert.equal(back.items.length, orig.items.length);
  const a = summary(orig); const b = summary(back);
  a.forEach((x, i) => { if (x.type !== 'text' && x.type !== 'freedraw' && x.type !== 'line') { /* fonts for non-text shapes carried via label */ } assert.deepEqual(b[i], x, `element ${i} (${x.type})`); });
  orig.items.forEach((o, i) => {
    const n = back.items[i];
    if (o.type === 'arrow' || o.type === 'line' || o.type === 'freedraw') {
      const p1 = resolvePoints(orig, o); const p2 = resolvePoints(back, n);
      assert.equal(p1.length, p2.length);
      p1.forEach((p, k) => { assert.ok(Math.abs(p.x - p2[k].x) <= 1 && Math.abs(p.y - p2[k].y) <= 1, `${o.type}/${o.arrowType} point ${k} ${JSON.stringify(p)} vs ${JSON.stringify(p2[k])}`); });
    } else {
      for (const k of ['x', 'y', 'w', 'h']) assert.ok(Math.abs(o[k] - n[k]) <= 1, `${o.type}.${k}: ${o[k]} vs ${n[k]}`);
      assert.ok(Math.abs(o.angle - n.angle) < 1e-6);
    }
    assert.equal(n.seed, o.seed);
    if (o.type === 'image') assert.equal(n.dataUrl, PNG);
  });
});

// A hand-written fixture shaped like a real Excalidraw file
const el = (o) => ({
  angle: 0, strokeColor: '#1e1e1e', backgroundColor: 'transparent', fillStyle: 'solid', strokeWidth: 2, strokeStyle: 'solid', roughness: 1, opacity: 100,
  groupIds: [], frameId: null, index: 'a0', roundness: null, seed: 1234, version: 5, versionNonce: 99, isDeleted: false, boundElements: null, updated: 1700000000000,
  link: null, locked: false, ...o,
});
const fixture = () => ({
  type: 'excalidraw', version: 2, source: 'https://excalidraw.com',
  elements: [
    el({ id: 'frame1', type: 'frame', x: -20, y: -60, width: 600, height: 380, name: 'Backend', strokeColor: '#bbb', strokeWidth: 1, roughness: 0 }),
    el({ id: 'box1', type: 'rectangle', x: 0, y: 0, width: 160, height: 80, backgroundColor: '#a5d8ff', fillStyle: 'hachure', roundness: { type: 3 }, frameId: 'frame1', groupIds: ['grpA'], boundElements: [{ type: 'text', id: 'txt1' }, { type: 'arrow', id: 'arr1' }] }),
    el({ id: 'txt1', type: 'text', x: 30, y: 28, width: 100, height: 25, text: 'Web\nServer', originalText: 'Web Server', fontSize: 20, fontFamily: 5, textAlign: 'center', verticalAlign: 'middle', containerId: 'box1', lineHeight: 1.25, autoResize: true, frameId: 'frame1', groupIds: ['grpA'] }),
    el({ id: 'box2', type: 'ellipse', x: 320, y: 0, width: 120, height: 80, backgroundColor: '#ffc9c9', strokeWidth: 1, roundness: { type: 2 }, boundElements: [{ type: 'arrow', id: 'arr1' }] }),
    el({ id: 'arr1', type: 'arrow', x: 160, y: 40, width: 160, height: 0, points: [[0, 0], [160, 0]], startBinding: { elementId: 'box1', focus: 0, gap: 1 }, endBinding: { elementId: 'box2', focus: 0, gap: 1 }, startArrowhead: 'dot', endArrowhead: 'cardinality_many', lastCommittedPoint: null, elbowed: false, boundElements: [{ type: 'text', id: 'txt2' }], roundness: { type: 2 } }),
    el({ id: 'txt2', type: 'text', x: 220, y: 15, width: 40, height: 25, text: 'reads', originalText: 'reads', fontSize: 16, fontFamily: 2, textAlign: 'center', verticalAlign: 'middle', containerId: 'arr1', lineHeight: 1.25 }),
    el({ id: 'free1', type: 'freedraw', x: 10, y: 150, width: 40, height: 20, points: [[0, 0], [10, 8], [25, 20], [40, 5]], pressures: [0.5, 0.6, 0.7, 0.5], simulatePressure: false, lastCommittedPoint: [40, 5], strokeColor: '#e03131', strokeWidth: 1.8 }),
    el({ id: 'img1', type: 'image', x: 300, y: 150, width: 64, height: 64, fileId: 'file1', status: 'saved', scale: [1, 1] }),
    el({ id: 'gone', type: 'rectangle', x: 0, y: 0, width: 5, height: 5, isDeleted: true }),
    el({ id: 'emb', type: 'embeddable', x: 0, y: 300, width: 100, height: 100 }),
    el({ id: 'loose', type: 'text', x: 100, y: 250, width: 90, height: 25, text: 'free text', originalText: 'free text', fontSize: 28, fontFamily: 1, textAlign: 'left', verticalAlign: 'top', containerId: null, lineHeight: 1.25 }),
    el({ id: 'dia', type: 'diamond', x: 400, y: 250, width: 100, height: 100, strokeStyle: 'dashed' }),
    el({ id: 'elb', type: 'arrow', x: 0, y: 200, width: 50, height: 30, points: [[0, 0], [50, 0], [50, 30]], startBinding: null, endBinding: null, startArrowhead: null, endArrowhead: 'triangle_outline', elbowed: true, roundness: null }),
  ],
  appState: { viewBackgroundColor: '#f8f9fa', gridSize: 20, theme: 'dark' },
  files: { file1: { mimeType: 'image/png', id: 'file1', dataURL: PNG, created: 1 } },
});

test('imports a realistic Excalidraw fixture', () => {
  const { board, skipped } = importExcalidrawDetailed(fixture());
  assert.equal(skipped, 1);
  const by = (t) => board.items.filter((e) => e.type === t);
  assert.equal(board.items.length, 9); // frame, rect, ellipse, arrow, freedraw, image, text, diamond, elbow arrow
  const rect = by('rect')[0];
  assert.equal(rect.text, 'Web Server'); assert.equal(rect.fontFamily, 1); assert.equal(rect.fillStyle, 'hachure'); assert.equal(rect.roundness, 'round');
  assert.equal(rect.frameId, by('frame')[0].id); assert.equal(by('frame')[0].name, 'Backend'); assert.equal(rect.groupIds.length, 1);
  const arrow = by('arrow')[0];
  assert.equal(arrow.from, rect.id); assert.equal(arrow.to, by('ellipse')[0].id); assert.equal(arrow.text, 'reads'); assert.equal(arrow.fontFamily, 2);
  assert.equal(arrow.startArrowhead, 'circle'); assert.equal(arrow.endArrowhead, 'crowfoot_many'); assert.equal(arrow.arrowType, 'round');
  assert.equal(by('arrow')[1].arrowType, 'elbow'); assert.equal(by('arrow')[1].endArrowhead, 'triangle_outline');
  assert.equal(by('freedraw')[0].points.length, 4); assert.equal(by('freedraw')[0].strokeWidth, 2);
  assert.equal(by('image')[0].dataUrl, PNG);
  assert.equal(by('text')[0].text, 'free text'); assert.equal(by('diamond')[0].strokeStyle, 'dashed');
  assert.equal(importExcalidraw(fixture()).items.length, 9);
});

test('Excalidraw -> ours -> Excalidraw -> ours is stable', () => {
  const one = importExcalidraw(fixture());
  const two = importExcalidraw(JSON.parse(JSON.stringify(exportExcalidraw(one))));
  assert.deepEqual(summary(two), summary(one));
  one.items.forEach((e, i) => { for (const k of ['x', 'y', 'w', 'h']) assert.ok(Math.abs(e[k] - two.items[i][k]) <= 1, `${e.type}.${k}`); });
  const ex = exportExcalidraw(one);
  assert.equal(ex.elements.filter((e) => e.type === 'text').length, 3);
});

test('import maps font, widths, arrowheads and ignores hostile data', () => {
  const mk = (o) => el({ type: 'rectangle', x: 0, y: 0, width: 10, height: 10, ...o });
  const b = importExcalidraw({ elements: [
    mk({ id: 'a', strokeWidth: 3.5, roughness: 7, fillStyle: 'weird', strokeColor: 'url(x)', link: 'javascript:alert(1)', groupIds: ['g', 'g2'], opacity: 500 }),
    el({ id: 't', type: 'text', x: 0, y: 0, width: 5, height: 5, text: 'c', fontSize: 20, fontFamily: 3, textAlign: 'left', verticalAlign: 'top', containerId: null }),
    el({ id: 't2', type: 'text', x: 0, y: 0, width: 5, height: 5, text: 'x', fontSize: 20, fontFamily: 9, textAlign: 'left', verticalAlign: 'top', containerId: 'missing' }),
    el({ id: 'im', type: 'image', x: 0, y: 0, width: 5, height: 5, fileId: 'f' }),
    { id: 'junk', type: 'rectangle', x: 'NaN', width: null },
    null, 5,
  ], files: { f: { dataURL: 'data:text/html;base64,AAAA' } } });
  const [a, t, t2, im] = b.items;
  assert.equal(a.strokeWidth, 4); assert.equal(a.roughness, 2); assert.equal(a.fillStyle, 'solid'); assert.equal(a.strokeColor, '#1e1e1e'); assert.equal(a.link, null); assert.equal(a.opacity, 100);
  assert.equal(t.fontFamily, 3); assert.equal(t2.fontFamily, 1); assert.equal(im.dataUrl, '');
  assert.equal(a.groupIds.length, 2);
  assert.deepEqual(importExcalidraw(null).items, []);
});

test('arrowhead mapping covers cardinality and unknown names', () => {
  const arr = (h) => importExcalidraw([el({ id: 'x', type: 'arrow', x: 0, y: 0, width: 10, height: 0, points: [[0, 0], [10, 0]], startArrowhead: h, endArrowhead: 'nonsense' })]).items[0];
  assert.equal(arr('cardinality_one').startArrowhead, 'crowfoot_one');
  assert.equal(arr('cardinality_zero_or_many').startArrowhead, 'crowfoot_many');
  assert.equal(arr('cardinality_one_or_many').startArrowhead, 'crowfoot_one_or_many');
  assert.equal(arr('dot').startArrowhead, 'circle');
  assert.equal(arr('nope').endArrowhead, null);
});

test('libraries: our format, excalidrawlib v1/v2, normalisation and sanitising', () => {
  const board = sampleBoard();
  const item = libraryItemFromElements(board, board.items.filter((e) => ['rect', 'database', 'arrow'].includes(e.type) && e.type !== 'database' || e.type === 'database'), 'Pair');
  assert.equal(Math.min(...item.elements.map((e) => e.x)), 0);
  const text = serializeLibrary([item, { id: 'x', name: 'Empty', elements: [] }]);
  const back = parseLibrary(text);
  assert.equal(back.length, 1); assert.equal(back[0].name, 'Pair'); assert.equal(back[0].category, 'My library');
  assert.equal(back[0].elements.length, item.elements.length);
  const v2 = parseLibrary(JSON.stringify({ type: 'excalidraw-library', version: 2, libraryItems: [{ id: 'i', status: 'published', created: 1, name: 'N', elements: fixture().elements }] }));
  assert.equal(v2[0].name, 'N'); assert.equal(v2[0].elements.length, 9); assert.ok(Math.min(...v2[0].elements.map((e) => e.x)) <= 0.001);
  const v1 = parseLibrary(JSON.stringify({ type: 'excalidrawlib', version: 1, library: [fixture().elements.slice(1, 3), []] }));
  assert.equal(v1.length, 1); assert.equal(v1[0].elements[0].text, 'Web Server');
  const many = JSON.stringify({ type: 'hld-library', items: Array.from({ length: 600 }, (_, i) => ({ id: `i${i}`, name: 'n', elements: [{ id: 1, type: 'rect', x: 5, y: 5, w: 10, h: 10 }] })) });
  assert.equal(parseLibrary(many).length, 500);
  assert.throws(() => parseLibrary('{"a":1}'), /Unrecognised/);
  assert.throws(() => parseLibrary('zzz'), /valid JSON/);
  const roundTrip = parseLibrary(serializeExcalidrawLibrary(back));
  assert.equal(roundTrip[0].elements.length, back[0].elements.length);
});

test('instantiateLibraryItem translates to the target and leaves the board untouched', () => {
  const board = emptyBoard(); board.nextId = 40;
  const item = libraryItemFromElements(sampleBoard(), sampleBoard().items.slice(1, 4), 'x');
  const els = instantiateLibraryItem(board, item, { x: 100, y: 200 });
  assert.equal(board.nextId, 40); assert.equal(els.length, item.elements.length);
  assert.ok(Math.abs(selectionBounds({ items: els }, els).x - 100) < 1e-6 && Math.abs(selectionBounds({ items: els }, els).y - 200) < 1e-6);
  assert.notDeepEqual(els.map((e) => e.id), item.elements.map((e) => e.id));
  const b2 = sanitizeBoard({ items: els, nextId: 100 });
  assert.equal(b2.items.length, els.length);
  void findItem;
});

test('browser helpers do not break Node', async () => {
  assert.equal(await pickFile('.json'), null);
  assert.throws(() => downloadText('a.json', '{}'), /browser/);
});

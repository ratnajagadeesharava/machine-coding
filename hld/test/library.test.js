import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { BUILTIN_LIBRARY, LIBRARY_CATEGORIES } from '../public/wb/library-data.js';
import { sanitizeBoard, resolvePoints, BINDABLE_TYPES } from '../shared/board.js';
import { serializeLibrary, parseLibrary, exportExcalidraw, importExcalidraw } from '../public/wb/interop.js';

const board = (it) => ({ version: 2, items: it.elements, nextId: 1000 });

test('built-in library has the expected size, categories and unique names', () => {
  assert.ok(BUILTIN_LIBRARY.length >= 30);
  for (const c of ['Clients', 'Compute', 'Storage', 'Messaging', 'Networking', 'Caching', 'Observability', 'Patterns']) assert.ok(LIBRARY_CATEGORIES.includes(c), c);
  const names = BUILTIN_LIBRARY.map((i) => i.name);
  assert.equal(new Set(names).size, names.length);
  assert.equal(new Set(BUILTIN_LIBRARY.map((i) => i.id)).size, names.length);
});

test('every stencil is sane: sanitize keeps it, normalised at the origin, bindings point at members', () => {
  for (const it of BUILTIN_LIBRARY) {
    const s = sanitizeBoard(board(it));
    assert.equal(s.items.length, it.elements.length, it.name);
    assert.deepEqual(s.items.map((e) => e.type), it.elements.map((e) => e.type), it.name);
    const ids = new Set(it.elements.map((e) => e.id));
    assert.equal(ids.size, it.elements.length, `${it.name} unique ids`);
    assert.ok(Math.min(...it.elements.map((e) => e.x)) === 0 || Math.min(...it.elements.map((e) => e.y)) === 0, `${it.name} touches origin`);
    for (const e of it.elements) {
      assert.ok(e.x >= -0.01 && e.y >= -0.01, `${it.name} no negative coords`);
      if (e.type === 'arrow') {
        for (const k of ['from', 'to']) { assert.ok(ids.has(e[k]), `${it.name} ${k}`); assert.ok(BINDABLE_TYPES.includes(it.elements.find((x) => x.id === e[k]).type)); }
        const pts = resolvePoints(board(it), e);
        assert.ok(pts.length >= 2 && pts.every((p) => Number.isFinite(p.x)));
      }
    }
  }
});

test('stencils survive library files and Excalidraw export/import', () => {
  const back = parseLibrary(serializeLibrary(BUILTIN_LIBRARY));
  assert.equal(back.length, BUILTIN_LIBRARY.length);
  for (const it of BUILTIN_LIBRARY) {
    const r = importExcalidraw(exportExcalidraw(board(it)));
    assert.equal(r.items.length, it.elements.length, it.name);
    assert.deepEqual(r.items.map((e) => e.text ?? '').filter(Boolean), it.elements.map((e) => e.text ?? '').filter(Boolean), it.name);
  }
});

test('every stencil renders to well-formed SVG markup (when render.js exists)', async (t) => {
  if (!fs.existsSync(new URL('../public/wb/render.js', import.meta.url))) return t.skip('render.js not present');
  const { boardToSvgString } = await import('../public/wb/render.js');
  for (const it of BUILTIN_LIBRARY) {
    const svg = boardToSvgString(board(it), { background: '#fff' });
    assert.match(svg, /^<svg[\s\S]*<\/svg>\s*$/, it.name);
    assert.ok(!/NaN|undefined/.test(svg), `${it.name} has NaN/undefined`);
    const opens = (svg.match(/<(g|clipPath|defs)\b/g) || []).length;
    const closes = (svg.match(/<\/(g|clipPath|defs)>/g) || []).length;
    assert.equal(opens, closes, `${it.name} balanced tags`);
  }
});

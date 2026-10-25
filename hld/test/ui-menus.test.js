// DOM-free parts of the whiteboard menus (agent D): fuzzy ranking, command list coverage, scene embedding, library helpers.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { TOOLS } from '../shared/wbconst.js';
import { fuzzyScore, rankCommands, buildCommands, ENGINE_ACTIONS, CATEGORIES } from '../public/wb/ui-palette.js';
import { embedSceneInSvg, extractSceneFromSvg, utf8ToBase64, base64ToUtf8 } from '../public/wb/ui-export.js';
import { groupByCategory, filterItems } from '../public/wb/ui-library.js';
import { canvasEntries, elementEntries } from '../public/wb/ui-context.js';
import { fmtShortcut, safeName } from '../public/wb/menu-icons.js';
import { BUILTIN_LIBRARY } from '../public/wb/library-data.js';

test('fuzzyScore: substring beats subsequence, word starts rank higher, no match is -1', () => {
  assert.equal(fuzzyScore('zzz', 'Toggle grid'), -1);
  assert.ok(fuzzyScore('grid', 'Toggle grid') > fuzzyScore('tgd', 'Toggle grid'));
  assert.ok(fuzzyScore('tgd', 'Toggle grid') > 0);
  assert.ok(fuzzyScore('rect', 'Rectangle') > fuzzyScore('rect', 'Correct me'));
  assert.ok(fuzzyScore('align left', 'Align left') > 0);
  assert.equal(fuzzyScore('', 'anything'), 0);
  assert.ok(fuzzyScore('GRID', 'toggle grid') > 0, 'case-insensitive');
});

const fakeWb = () => new Proxy({ mode: {}, selection: new Set(), board: { items: [] } }, { get: (t, k) => (k in t ? t[k] : () => {}) });

test('buildCommands covers every tool, every engine action, all categories, unique ids', () => {
  const cmds = buildCommands(fakeWb());
  const ids = cmds.map((c) => c.id);
  assert.equal(new Set(ids).size, ids.length, 'unique ids');
  for (const t of TOOLS) assert.ok(ids.includes(`tool:${t.id}`), `tool ${t.id}`);
  assert.deepEqual([...new Set(cmds.map((c) => c.category))].sort(), [...CATEGORIES].sort());
  for (const c of cmds) assert.equal(typeof c.run, 'function', c.id);
});

test('ENGINE_ACTIONS lists every action method documented in section 5 of the API doc', () => {
  const doc = fs.readFileSync(new URL('../docs/WHITEBOARD_API.md', import.meta.url), 'utf8');
  const sec = doc.slice(doc.indexOf('**Actions**'), doc.indexOf('**Export:**'));
  const names = [...sec.matchAll(/`?\b([a-z][A-Za-z]+)\(/g)].map((m) => m[1]);
  // not commands: scene/state plumbing and view-less helpers
  const ignore = new Set(['canUndo', 'canRedo', 'setZoom', 'pan', 'setMode', 'setStyle']);
  const missing = [...new Set(names)].filter((n) => !ignore.has(n) && !ENGINE_ACTIONS.includes(n));
  assert.deepEqual(missing, []);
});

test('rankCommands: filter, ranking, recents first when the query is empty', () => {
  const cmds = buildCommands(fakeWb());
  const r = rankCommands(cmds, 'rect');
  assert.equal(r[0].cmd.id, 'tool:rect');
  assert.ok(r.every((x) => x.score > 0));
  assert.equal(rankCommands(cmds, 'qqqqzzzz').length, 0);
  const empty = rankCommands(cmds, '', ['grid', 'zen']);
  assert.deepEqual(empty.slice(0, 2).map((x) => x.cmd.id), ['grid', 'zen']);
  assert.equal(empty[0].group, 'Recent');
  assert.equal(empty.length, cmds.length);
  assert.equal(empty[2].group, 'Tools');
  // recent boost lifts a command in a tie-ish query
  const a = rankCommands(cmds, 'zoom').map((x) => x.cmd.id);
  const b = rankCommands(cmds, 'zoom', ['zoom-out']).map((x) => x.cmd.id);
  assert.ok(b.indexOf('zoom-out') <= a.indexOf('zoom-out'));
});

test('scene embedding in SVG round-trips unicode and is invisible to the renderer', () => {
  const scene = JSON.stringify({ a: 'Zażółć gęślą jaźń ✓ \u{1F600}', n: 'x'.repeat(100000) });
  assert.equal(base64ToUtf8(utf8ToBase64(scene)), scene);
  const svg = '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect/></svg>';
  const out = embedSceneInSvg(svg, scene);
  assert.match(out, /^<svg [^>]*><metadata id="hld-scene">/);
  assert.equal(extractSceneFromSvg(out), scene);
  assert.equal(extractSceneFromSvg(svg), null);
  assert.equal(extractSceneFromSvg('<metadata id="hld-scene">***</metadata>'), null);
});

test('library helpers group by category and filter case-insensitively', () => {
  const groups = groupByCategory(BUILTIN_LIBRARY);
  assert.ok(groups.length >= 5);
  assert.equal(groups.reduce((n, g) => n + g.items.length, 0), BUILTIN_LIBRARY.length);
  assert.ok(filterItems(BUILTIN_LIBRARY, 'CACHE').length >= 1);
  assert.equal(filterItems(BUILTIN_LIBRARY, '').length, BUILTIN_LIBRARY.length);
  assert.equal(filterItems(BUILTIN_LIBRARY, 'qqqqq').length, 0);
});

test('context menu entries: disabled states depend on the selection', () => {
  const mk = (n, extra = {}) => {
    const items = Array.from({ length: n }, (_, i) => ({ id: i + 1, groupIds: [], locked: false, ...extra }));
    return { selection: new Set(items.map((i) => i.id)), board: { items }, mode: {} };
  };
  const find = (entries, id) => entries.find((e) => e.id === id);
  let e = elementEntries(mk(1));
  assert.ok(find(e, 'group').disabled); assert.ok(find(e, 'align').disabled); assert.ok(find(e, 'ungroup').disabled);
  assert.ok(!find(e, 'link').disabled);
  e = elementEntries(mk(2));
  assert.ok(!find(e, 'group').disabled); assert.ok(!find(e, 'align').disabled);
  assert.ok(find(find(e, 'align').submenu, 'distribute-h').disabled);
  e = elementEntries(mk(3));
  assert.ok(!find(find(e, 'align').submenu, 'distribute-h').disabled);
  assert.equal(find(e, 'lock').label, 'Lock');
  e = elementEntries(mk(2, { locked: true, groupIds: [9] }));
  assert.equal(find(e, 'lock').label, 'Unlock'); assert.ok(!find(e, 'ungroup').disabled);
  assert.equal(find(e, 'layers').submenu.length, 4);
  const c = canvasEntries({ mode: { grid: true, zen: false } });
  assert.equal(c.find((x) => x.id === 'grid').check, true);
  assert.equal(c.find((x) => x.id === 'zen').check, false);
});

test('shortcut formatting and file names', () => {
  assert.equal(fmtShortcut('Mod+Shift+E', false), 'Ctrl+Shift+E');
  assert.equal(fmtShortcut('Mod+Shift+E', true), '⇧⌘E');
  assert.equal(fmtShortcut('', false), '');
  assert.equal(safeName('a/b:c*d?.json'), 'a b c d .json');
  assert.equal(safeName('   '), 'whiteboard');
});

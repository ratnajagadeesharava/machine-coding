// End-to-end test of the whiteboard chrome (toolbar, properties panel, zoom bar, help dialog) on top of the real engine.
// Run: node test/ui-chrome.e2e.mjs        (HLD_CHROME=/path/to/chrome if Chromium is not found; HLD_SHOTS=dir to keep screenshots)
import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(here, '..');
const MIME = { '.js': 'text/javascript', '.css': 'text/css', '.html': 'text/html', '.json': 'application/json', '.svg': 'image/svg+xml' };

function findChrome() {
  if (process.env.HLD_CHROME) return process.env.HLD_CHROME;
  const base = process.env.PLAYWRIGHT_BROWSERS_PATH || '/opt/pw-browsers';
  try {
    for (const d of fs.readdirSync(base).filter((n) => n.startsWith('chromium-')).sort().reverse()) {
      const p = path.join(base, d, 'chrome-linux', 'chrome');
      if (fs.existsSync(p)) return p;
    }
  } catch { /* fall through */ }
  return undefined;
}

// A scratch page: real engine + the four chrome modules (what mount.js assembles, minus menus/library).
const PAGE = `<!doctype html><html><head><meta charset="utf-8"><title>chrome test</title>
<link rel="stylesheet" href="/styles.css"><link rel="stylesheet" href="/wb/engine.css"><link rel="stylesheet" href="/wb/chrome.css">
<style>html,body{margin:0;height:100%}#host{position:absolute;inset:0}</style></head><body><div id="host"></div>
<script type="module">
import { Whiteboard } from '/wb/engine.js';
import { mountToolbar } from '/wb/ui-toolbar.js';
import { mountProperties } from '/wb/ui-properties.js';
import { mountZoomBar } from '/wb/ui-zoom.js';
import { mountHelp } from '/wb/ui-help.js';
const q = new URLSearchParams(location.search);
const host = document.getElementById('host');
const wb = new Whiteboard(host, { theme: q.get('theme') || 'light' });
window.wb = wb; window.libClicks = 0; window.requests = [];
wb.on('request', (e) => window.requests.push(e.name));
window.mods = { toolbar: mountToolbar(wb, host, { library: true, onLibrary() { window.libClicks++; } }), properties: mountProperties(wb, host), zoom: mountZoomBar(wb, host), help: mountHelp(wb, host) };
window.ready = true;
</script></body></html>`;

const server = http.createServer((req, res) => {
  const u = decodeURIComponent(req.url.split('?')[0]);
  if (u === '/chrome-test.html') { res.writeHead(200, { 'content-type': 'text/html' }); res.end(PAGE); return; }
  const f = u.startsWith('/shared/') ? path.join(ROOT, u) : path.join(ROOT, 'public', u);
  if (!f.startsWith(ROOT)) { res.writeHead(403); res.end(); return; }
  fs.readFile(f, (e, d) => {
    if (e) { res.writeHead(404); res.end('not found'); return; }
    res.writeHead(200, { 'content-type': MIME[path.extname(f)] || 'application/octet-stream' });
    res.end(d);
  });
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const base = `http://127.0.0.1:${server.address().port}`;
const shots = process.env.HLD_SHOTS || path.join(process.env.TMPDIR || '/tmp', 'wbui-shots');
fs.mkdirSync(shots, { recursive: true });

const browser = await chromium.launch({ executablePath: findChrome(), args: ['--no-sandbox'] });
const errors = [];
let passed = 0;
async function open(query = '', viewport = { width: 1440, height: 900 }) {
  const page = await browser.newPage({ viewport, deviceScaleFactor: 1 });
  page.on('pageerror', (e) => errors.push(e.message));
  page.on('console', (m) => { if (m.type() === 'error' && !/Failed to load resource/.test(m.text())) errors.push(m.text()); });
  await page.goto(`${base}/chrome-test.html${query}`);
  await page.waitForFunction('window.ready === true');
  return page;
}
async function test(name, fn, opts) {
  const page = await open(opts && opts.query, opts && opts.viewport);
  try { await fn(page); passed++; console.log('ok  -', name); } catch (e) { console.log('FAIL -', name); throw e; } finally { await page.close(); }
}
const board = (page) => page.evaluate(() => window.wb.getBoard());
const sel = (page) => page.evaluate(() => [...window.wb.selection]);
const el = async (page, i = 0) => (await board(page)).items[i];
const attr = (page, selector, name) => page.locator(selector).first().getAttribute(name);
const pressed = async (page, selector) => (await attr(page, selector, 'aria-pressed')) === 'true';
const props = (page) => page.locator('.wbui-props');
async function insert(page, specs, select = true) {
  return page.evaluate(({ specs, select }) => {
    const { createElement } = window.__shared;
    return window.wb.insertElements(specs.map((s, i) => createElement(s.type, { id: 9000 + i, ...(s.props || {}) })), { select });
  }, { specs, select });
}
// helper to build elements in the page without importing in every call
async function prep(page) {
  await page.evaluate(async () => { window.__shared = await import('/shared/board.js'); });
}
const rectSpec = (x, y, extra = {}) => ({ type: 'rect', props: { x, y, w: 140, h: 90, ...extra } });

// ------------------------------------------------------------------ toolbar
await test('toolbar: every tool button sets the tool; titles carry key hints', async (page) => {
  const main = ['hand', 'select', 'rect', 'diamond', 'ellipse', 'arrow', 'line', 'freedraw', 'text', 'eraser'];
  for (const id of main) {
    await page.click(`.wbui-toolbar button[data-tool="${id}"]`);
    assert.equal(await page.evaluate(() => window.wb.tool), id, id);
    assert.equal(await pressed(page, `.wbui-toolbar button[data-tool="${id}"]`), true, `${id} pressed`);
  }
  assert.match(await attr(page, 'button[data-tool="rect"]', 'title'), /^Rectangle - R or 2$/);
  assert.match(await attr(page, 'button[data-tool="hand"]', 'title'), /^Hand \(panning tool\) - H$/);
  await page.click('.wbui-toolbar button[data-tool="select"]');
  await page.click('.wbui-toolbar button[data-tool="image"]');
  assert.ok((await page.evaluate(() => window.requests)).includes('image'), 'image tool asks the host for a file');
  await page.click('[data-action="more-tools"]');
  for (const id of ['database', 'sticky', 'frame', 'laser', 'bucket', 'eyedropper']) {
    await page.click('[data-action="more-tools"]', { trial: true }).catch(() => {});
    if (!(await page.locator(`.wbui-popover button[data-tool="${id}"]`).count())) await page.click('[data-action="more-tools"]');
    await page.click(`.wbui-popover button[data-tool="${id}"]`);
    assert.equal(await page.evaluate(() => window.wb.tool), id, id);
    assert.equal(await page.locator('.wbui-popover').count(), 0, 'popover closes after a pick');
    assert.ok(await page.locator('[data-action="more-tools"]').evaluate((b) => b.classList.contains('is-active')), 'more button shows active hidden tool');
  }
});

await test('toolbar: keys switch tools and the highlight follows', async (page) => {
  const keys = { h: 'hand', v: 'select', r: 'rect', d: 'diamond', o: 'ellipse', a: 'arrow', l: 'line', p: 'freedraw', t: 'text', e: 'eraser', 2: 'rect', 4: 'ellipse' };
  await page.mouse.move(700, 500);
  for (const [k, id] of Object.entries(keys)) {
    await page.keyboard.press(k);
    assert.equal(await page.evaluate(() => window.wb.tool), id, `key ${k}`);
    assert.equal(await pressed(page, `.wbui-toolbar button[data-tool="${id}"]`), true, `highlight ${id}`);
    await page.keyboard.press('Escape');
  }
  await page.keyboard.press('f');
  assert.equal(await page.locator('[data-action="more-tools"]').evaluate((b) => b.classList.contains('is-active')), true);
});

await test('toolbar: tool lock toggle and library button', async (page) => {
  assert.equal(await pressed(page, '[data-action="tool-lock"]'), false);
  await page.click('[data-action="tool-lock"]');
  assert.equal(await page.evaluate(() => window.wb.toolLocked), true);
  assert.equal(await pressed(page, '[data-action="tool-lock"]'), true);
  await page.click('button[data-tool="rect"]');
  assert.equal(await page.locator('button[data-tool="rect"]').evaluate((b) => b.classList.contains('is-locked')), true, 'locked ring on active tool');
  await page.keyboard.press('q');
  assert.equal(await page.evaluate(() => window.wb.toolLocked), false);
  assert.equal(await pressed(page, '[data-action="tool-lock"]'), false);
  await page.click('[data-action="library"]');
  assert.equal(await page.evaluate(() => window.libClicks), 1);
});

await test('toolbar: roving tabindex and arrow-key navigation', async (page) => {
  const tab = () => page.evaluate(() => [...document.querySelectorAll('.wbui-toolbar button')].filter((b) => b.tabIndex === 0).map((b) => b.dataset.tool || b.dataset.action));
  assert.deepEqual(await tab(), ['select'], 'exactly one tab stop: the active tool');
  await page.focus('button[data-tool="select"]');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.tool), 'rect');
  assert.deepEqual(await tab(), ['rect']);
  await page.keyboard.press('End');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'library');
  await page.keyboard.press('Home');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'tool-lock');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.tool), 'select');
  await page.keyboard.press('Enter');
  assert.equal(await page.evaluate(() => window.wb.tool), 'select');
  await page.keyboard.press('ArrowRight');
  await page.keyboard.press('Space');
  assert.equal(await page.evaluate(() => window.wb.tool), 'rect', 'space activates the focused tool');
  // "more tools" menu is keyboard operable and Esc returns focus
  await page.focus('[data-action="more-tools"]');
  await page.keyboard.press('Enter');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.tool), 'database');
  await page.keyboard.press('ArrowRight');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.tool), 'sticky');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('.wbui-popover').count(), 0);
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'more-tools');
});

await test('toolbar: compact on narrow hosts (icons only, scrollable)', async (page) => {
  assert.equal(await page.locator('.wbui-toolbar-root.is-compact').count(), 1);
  assert.equal(await page.locator('.wbui-kbd').first().isVisible(), false, 'key hints hidden');
  const box = await page.locator('.wbui-toolbar').boundingBox();
  assert.ok(box.x >= 0 && box.x + box.width <= 500, 'toolbar stays inside the host');
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'no page-level horizontal scroll');
}, { viewport: { width: 500, height: 700 } });

// ------------------------------------------------------------------ properties
await test('properties: hidden for select/hand, shown for drawing tools and selections', async (page) => {
  assert.equal(await props(page).isVisible(), false);
  await page.click('button[data-tool="rect"]');
  assert.equal(await props(page).isVisible(), true, 'rect tool shows tool defaults');
  assert.equal(await page.locator('.wbui-row[data-row="textAlign"]').isVisible(), true);
  await page.click('button[data-tool="hand"]');
  assert.equal(await props(page).isVisible(), false);
  await page.click('button[data-tool="arrow"]');
  assert.equal(await props(page).isVisible(), true);
  assert.equal(await page.locator('.wbui-row[data-row="arrowheads"]').isVisible(), true);
  assert.equal(await page.locator('.wbui-row[data-row="fillStyle"]').isVisible(), false, 'arrows have no fill row');
  for (const t of ['eraser', 'laser', 'eyedropper']) {
    await page.evaluate((t) => window.wb.setTool(t), t);
    assert.equal(await props(page).isVisible(), false, t);
  }
});

await test('properties: draw a rectangle with the mouse, then restyle it through every control', async (page) => {
  await page.click('button[data-tool="rect"]');
  await page.mouse.move(500, 250);
  await page.mouse.down();
  await page.mouse.move(700, 360, { steps: 6 });
  await page.mouse.up();
  assert.equal((await board(page)).items.length, 1, 'rect drawn');
  assert.equal((await sel(page)).length, 1);
  assert.equal(await props(page).isVisible(), true, 'panel visible for the new selection');
  const click = (prop, value) => page.click(`.wbui-props [data-prop="${prop}"][data-value="${value}"]`);
  const field = async (f) => (await el(page))[f];

  await click('strokeColor', '#e03131'); assert.equal(await field('strokeColor'), '#e03131');
  assert.equal(await pressed(page, '.wbui-props [data-prop="strokeColor"][data-value="#e03131"]'), true);
  assert.equal(await pressed(page, '.wbui-props [data-prop="strokeColor"][data-value="#1e1e1e"]'), false);
  await click('backgroundColor', '#b2f2bb'); assert.equal(await field('backgroundColor'), '#b2f2bb');
  await click('backgroundColor', 'transparent'); assert.equal(await field('backgroundColor'), 'transparent');
  await click('backgroundColor', '#a5d8ff');
  for (const v of ['hachure', 'cross-hatch', 'zigzag', 'solid']) { await click('fillStyle', v); assert.equal(await field('fillStyle'), v); }
  for (const v of [1, 4, 2]) { await click('strokeWidth', v); assert.equal(await field('strokeWidth'), v); }
  for (const v of ['dashed', 'dotted', 'solid']) { await click('strokeStyle', v); assert.equal(await field('strokeStyle'), v); }
  for (const v of [0, 2, 1]) { await click('roughness', v); assert.equal(await field('roughness'), v); }
  for (const v of ['sharp', 'round']) { await click('roundness', v); assert.equal(await field('roundness'), v); }
  for (const v of [3, 2, 1]) { await click('fontFamily', v); assert.equal(await field('fontFamily'), v); }
  await click('fontSize', 36); assert.equal(await field('fontSize'), 36);
  await click('fontSize', 16); assert.equal(await field('fontSize'), 16);
  await page.fill('[data-prop-num="fontSize"]', '44'); await page.press('[data-prop-num="fontSize"]', 'Enter'); await page.locator('[data-prop-num="fontSize"]').blur();
  assert.equal(await field('fontSize'), 44, 'custom size');
  assert.equal(await pressed(page, '.wbui-props [data-prop="fontSize"][data-value="36"]'), false, 'no preset pressed for a custom size');
  for (const v of ['left', 'right', 'center']) { await click('textAlign', v); assert.equal(await field('textAlign'), v); }
  for (const v of ['top', 'bottom', 'middle']) { await click('verticalAlign', v); assert.equal(await field('verticalAlign'), v); }
  await page.locator('[data-prop-range="opacity"]').fill('40');
  assert.equal(await field('opacity'), 40);
  assert.equal((await page.locator('.wbui-opval').textContent()).trim(), '40');
  // panel reflects the element, also after undo
  await page.evaluate(() => window.wb.undo());
  assert.equal(await page.locator('[data-prop-range="opacity"]').inputValue(), '100');
});

await test('properties: extended palette, hex input validation, recent colours', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(100, 100)]);
  await page.click('[data-prop-chip="strokeColor"]');
  assert.equal(await page.locator('.wbui-pop-palette').isVisible(), true);
  await page.click('.wbui-pop-palette [data-prop="strokeColor"][data-value="#7950f2"]');
  assert.equal((await el(page)).strokeColor, '#7950f2');
  assert.equal(await pressed(page, '.wbui-pop-palette [data-value="#7950f2"]'), true);
  const hex = page.locator('[data-prop-hex="strokeColor"]');
  await hex.fill('zzz');
  assert.equal(await hex.getAttribute('aria-invalid'), 'true');
  assert.equal((await el(page)).strokeColor, '#7950f2', 'invalid input is not applied');
  await hex.fill('#0af');
  assert.equal(await hex.getAttribute('aria-invalid'), null);
  assert.equal((await el(page)).strokeColor, '#00aaff', '3-digit hex expands');
  await hex.fill('12ab9c'); await hex.press('Enter');
  assert.equal((await el(page)).strokeColor, '#12ab9c');
  assert.ok((await page.locator('.wbui-pal-recent [data-value]').count()) >= 1, 'recent colours listed');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('.wbui-pop-palette').count(), 0);
  assert.equal(await page.evaluate(() => document.activeElement.dataset.propChip), 'strokeColor', 'focus restored to the chip');
  // chip shows the custom colour
  assert.equal(await page.locator('[data-prop-chip="strokeColor"]').evaluate((c) => c.classList.contains('is-custom')), true);
  // background palette offers transparent
  await page.click('[data-prop-chip="backgroundColor"]');
  assert.equal(await page.locator('.wbui-pop-palette [data-value="transparent"]').count(), 1);
  await page.click('.wbui-pop-palette [data-value="#ffc9c9"]');
  assert.equal((await el(page)).backgroundColor, '#ffc9c9');
  await page.mouse.click(900, 600); // outside click closes
  assert.equal(await page.locator('.wbui-pop-palette').count(), 0);
});

await test('properties: sticky notes use the sticky palette', async (page) => {
  await prep(page);
  await insert(page, [{ type: 'sticky', props: { x: 100, y: 100 } }]);
  assert.equal(await page.locator('.wbui-props [data-prop="backgroundColor"][data-value="#e5dbff"]').count(), 1);
  await page.click('.wbui-props [data-prop="backgroundColor"][data-value="#e5dbff"]');
  assert.equal((await el(page)).backgroundColor, '#e5dbff');
});

await test('properties: arrowheads, arrow type, arrow label rows', async (page) => {
  await prep(page);
  await insert(page, [{ type: 'arrow', props: { x: 100, y: 100, points: [[0, 0], [200, 0]], w: 200, h: 0 } }]);
  assert.equal(await page.locator('.wbui-row[data-row="fillStyle"]').isVisible(), false);
  await page.click('[data-prop-picker="endArrowhead"]');
  assert.equal(await page.locator('.wbui-headopt').count(), 12, 'all 12 arrowheads offered');
  assert.equal(await pressed(page, '.wbui-headgrid [data-value="arrow"]'), true, 'current end head pressed');
  await page.click('.wbui-headgrid [data-value="crowfoot_many"]');
  assert.equal((await el(page)).endArrowhead, 'crowfoot_many');
  assert.equal(await page.locator('.wbui-popover').count(), 0, 'picker closes on pick');
  await page.click('[data-prop-picker="startArrowhead"]');
  assert.equal(await pressed(page, '.wbui-headgrid [data-value="null"]'), true, 'no head = "null" pressed');
  await page.click('.wbui-headgrid [data-value="diamond_outline"]');
  assert.equal((await el(page)).startArrowhead, 'diamond_outline');
  await page.click('[data-prop-picker="startArrowhead"]');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('.wbui-popover').count(), 0);
  await page.click('.wbui-props [data-prop="arrowType"][data-value="elbow"]');
  assert.equal((await el(page)).arrowType, 'elbow');
  await page.click('.wbui-props [data-prop="arrowType"][data-value="round"]');
  assert.equal((await el(page)).arrowType, 'round');
});

await test('properties: mixed selections show no pressed state and apply to all', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(100, 100, { strokeColor: '#e03131', strokeWidth: 1 }), rectSpec(300, 100, { strokeColor: '#2f9e44', strokeWidth: 1 })], false);
  await page.evaluate(() => window.wb.selectAll());
  assert.equal((await sel(page)).length, 2);
  const anyPressed = (prop) => page.evaluate((p) => [...document.querySelectorAll(`.wbui-props [data-prop="${p}"]`)].some((b) => b.getAttribute('aria-pressed') === 'true'), prop);
  assert.equal(await anyPressed('strokeColor'), false, 'mixed colour: nothing pressed');
  assert.equal(await page.locator('[data-prop-chip="strokeColor"]').evaluate((c) => c.classList.contains('is-mixed')), true);
  assert.equal(await anyPressed('strokeWidth'), true, 'same width: pressed');
  await page.evaluate(() => { const b = window.wb.board.items[0]; void b; });
  await page.click('.wbui-props [data-prop="strokeColor"][data-value="#1971c2"]');
  const items = (await board(page)).items;
  assert.deepEqual(items.map((i) => i.strokeColor), ['#1971c2', '#1971c2']);
  assert.equal(await anyPressed('strokeColor'), true);
});

await test('properties: layers, align, distribute, group, lock, flip, duplicate, delete, link, style copy', async (page) => {
  await prep(page);
  const ids = await insert(page, [rectSpec(100, 100, { w: 80 }), rectSpec(300, 220, { w: 100 }), rectSpec(620, 150, { w: 60 })], false);
  await page.evaluate(() => window.wb.selectAll());
  const order = async () => (await board(page)).items.map((i) => i.id);
  const act = (a) => page.click(`.wbui-props [data-action="${a}"]`);
  assert.equal(await page.locator('[data-action="align-left"]').isVisible(), true);
  assert.equal(await page.locator('[data-action="distribute-h"]').isVisible(), true);

  await page.evaluate((id) => window.wb.select([id]), ids[0]);
  assert.equal(await page.locator('[data-action="align-left"]').isVisible(), false, 'align needs 2+');
  assert.equal(await page.locator('[data-action="group"]').isVisible(), false);
  await act('bring-to-front'); assert.deepEqual(await order(), [ids[1], ids[2], ids[0]]);
  await act('send-backward'); assert.deepEqual(await order(), [ids[1], ids[0], ids[2]]);
  await act('send-to-back'); assert.deepEqual(await order(), [ids[0], ids[1], ids[2]]);
  await act('bring-forward'); assert.deepEqual(await order(), [ids[1], ids[0], ids[2]]);

  await page.evaluate(() => window.wb.selectAll());
  await act('align-left');
  assert.deepEqual([...new Set((await board(page)).items.map((i) => i.x))].length, 1, 'aligned left');
  await act('align-top');
  assert.equal(new Set((await board(page)).items.map((i) => i.y)).size, 1);
  // distribute: re-spread first
  await page.evaluate(([a, b, c]) => { void [a, b, c]; }, ids);
  await page.evaluate(() => window.wb.setBoard(window.wb.getBoard(), { resetHistory: false }));
  const spread = await insert(page, [rectSpec(0, 400, { w: 50 }), rectSpec(100, 400, { w: 50 }), rectSpec(500, 400, { w: 50 })], false);
  await page.evaluate((s) => window.wb.select(s), spread);
  await act('distribute-h');
  const xs = (await board(page)).items.filter((i) => spread.includes(i.id)).map((i) => i.x).sort((a, b) => a - b);
  assert.ok(Math.abs((xs[1] - xs[0]) - (xs[2] - xs[1])) < 1.5, 'even gaps after distribute: ' + xs);

  // group / ungroup
  await page.evaluate((s) => window.wb.select(s.slice(0, 2)), spread);
  await act('group');
  const g = (await board(page)).items.filter((i) => spread.slice(0, 2).includes(i.id));
  assert.ok(g.every((i) => i.groupIds.length === 1 && i.groupIds[0] === g[0].groupIds[0]), 'grouped');
  assert.equal(await page.locator('[data-action="ungroup"]').isVisible(), true);
  await act('ungroup');
  assert.ok((await board(page)).items.filter((i) => spread.includes(i.id)).every((i) => i.groupIds.length === 0), 'ungrouped');

  // lock
  await page.evaluate((id) => window.wb.select([id]), ids[2]);
  await act('toggle-lock');
  assert.equal((await board(page)).items.find((i) => i.id === ids[2]).locked, true);
  assert.equal((await sel(page)).length, 0, 'locking deselects (engine behaviour) and the panel follows');
  assert.equal(await props(page).isVisible(), false);
  await page.evaluate(() => window.wb.unlockAll());

  // flip
  await page.evaluate((id) => window.wb.select([id]), ids[2]);
  await insert(page, [{ type: 'arrow', props: { x: 100, y: 600, points: [[0, 0], [100, 40]], w: 100, h: 40, startArrowhead: null, endArrowhead: 'triangle' } }]);
  await act('flip-h');
  const arrow = (await board(page)).items.find((i) => i.type === 'arrow');
  assert.equal(arrow.startArrowhead, 'triangle', 'flip mirrors arrowheads');
  assert.equal(arrow.endArrowhead, null);
  await act('flip-v');

  // duplicate + delete
  const before = (await board(page)).items.length;
  await act('duplicate');
  assert.equal((await board(page)).items.length, before + 1);
  await act('delete');
  assert.equal((await board(page)).items.length, before);
  assert.equal((await sel(page)).length, 0);
  assert.equal(await props(page).isVisible(), false, 'panel hides when nothing is selected under the select tool');
});

await test('properties: link dialog accepts only http(s), can clear', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(100, 100)]);
  await page.click('.wbui-props [data-action="link"]');
  const dlg = page.locator('.wbui-dialog[open]');
  assert.equal(await dlg.count(), 1);
  await page.fill('[data-link-input]', 'javascript:alert(1)');
  await page.click('[data-action="link-save"]');
  assert.equal(await dlg.count(), 1, 'dialog stays open on invalid input');
  assert.match(await dlg.locator('.wbui-hex-err').textContent(), /http/);
  assert.equal((await el(page)).link, null);
  await page.fill('[data-link-input]', 'https://example.com/a');
  await page.click('[data-action="link-save"]');
  assert.equal(await page.locator('.wbui-dialog[open]').count(), 0);
  assert.equal((await el(page)).link, 'https://example.com/a');
  assert.equal(await page.locator('[data-action="link"]').evaluate((b) => b.classList.contains('is-on')), true);
  await page.click('[data-action="link"]');
  assert.equal(await page.inputValue('[data-link-input]'), 'https://example.com/a');
  await page.click('[data-action="link-remove"]');
  assert.equal((await el(page)).link, null);
  await page.click('[data-action="link"]');
  await page.fill('[data-link-input]', 'example.org');
  await page.keyboard.press('Enter');
  assert.equal((await el(page)).link, 'https://example.org/', 'bare domains get https');
  await page.click('[data-action="link"]');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('.wbui-dialog[open]').count(), 0, 'Esc closes the link dialog');
});

await test('properties: collapses behind a Properties button on narrow hosts', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(50, 150)]);
  assert.equal(await page.locator('.wbui-props-toggle').isVisible(), true);
  assert.equal(await props(page).isVisible(), false, 'collapsed by default');
  await page.click('.wbui-props-toggle');
  assert.equal(await props(page).isVisible(), true);
  assert.equal(await attr(page, '.wbui-props-toggle', 'aria-expanded'), 'true');
  const box = await props(page).boundingBox();
  assert.ok(box.x >= 0 && box.x + box.width <= 700 && box.y + box.height <= 700, 'panel fits the viewport');
  await page.click('.wbui-props [data-prop="strokeColor"][data-value="#e03131"]');
  assert.equal((await el(page)).strokeColor, '#e03131');
  await page.click('.wbui-props-toggle');
  assert.equal(await props(page).isVisible(), false);
}, { viewport: { width: 700, height: 520 } });

await test('properties: scrolls internally when the host is short', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(400, 100)]);
  const m = await props(page).evaluate((p) => ({ sh: p.scrollHeight, ch: p.clientHeight, bottom: p.getBoundingClientRect().bottom, vh: innerHeight }));
  assert.ok(m.sh > m.ch, 'content taller than the panel');
  assert.ok(m.bottom <= m.vh - 50, 'panel leaves room for the zoom bar');
}, { viewport: { width: 1000, height: 480 } });

// ------------------------------------------------------------------ zoom bar
await test('zoom bar: zoom buttons, percent reset, fit, undo/redo, stats', async (page) => {
  await prep(page);
  const zoom = () => page.evaluate(() => window.wb.view.zoom);
  assert.equal((await page.textContent('[data-action="zoom-reset"]')).trim(), '100%');
  await page.click('[data-action="zoom-in"]');
  assert.ok((await zoom()) > 1);
  assert.match((await page.textContent('[data-action="zoom-reset"]')).trim(), /^1\d\d%$/);
  await page.click('[data-action="zoom-in"]');
  await page.click('[data-action="zoom-out"]');
  await page.click('[data-action="zoom-out"]');
  await page.click('[data-action="zoom-out"]');
  assert.ok((await zoom()) < 1);
  await page.click('[data-action="zoom-reset"]');
  assert.equal(await zoom(), 1);
  assert.equal((await page.textContent('[data-action="zoom-reset"]')).trim(), '100%');
  await insert(page, [rectSpec(2000, 1500, { w: 600, h: 400 })]);
  await page.click('[data-action="zoom-fit"]');
  const c = await page.evaluate(() => window.wb.worldToClient(2300, 1700));
  assert.ok(c.x > 0 && c.x < 1440 && c.y > 0 && c.y < 900, 'fit brings far content into view: ' + JSON.stringify(c));

  // undo / redo
  assert.equal(await page.locator('[data-action="undo"]').isEnabled(), true, 'undo available after an edit');
  assert.equal(await page.locator('[data-action="redo"]').isDisabled(), true);
  await page.click('[data-action="undo"]');
  assert.equal((await board(page)).items.length, 0);
  assert.equal(await page.locator('[data-action="undo"]').isDisabled(), true);
  assert.equal(await page.locator('[data-action="redo"]').isEnabled(), true);
  await page.click('[data-action="redo"]');
  assert.equal((await board(page)).items.length, 1);
  assert.match(await attr(page, '[data-action="undo"]', 'title'), /Undo \((Ctrl|Cmd)\+Z\)/);

  // stats
  assert.equal(await page.locator('.wbui-stats').isVisible(), false);
  await page.click('[data-action="stats"]');
  assert.equal(await page.locator('.wbui-stats').isVisible(), true);
  assert.equal((await page.textContent('.wbui-stats [data-stat="elements"] .wbui-stat-v')).trim(), '1');
  await page.evaluate(() => window.wb.selectAll());
  assert.equal((await page.textContent('.wbui-stats [data-stat="width"] .wbui-stat-v')).trim(), '600');
  assert.equal((await page.textContent('.wbui-stats [data-stat="height"] .wbui-stat-v')).trim(), '400');
  assert.ok(await page.locator('.wbui-stats [data-stat="angle"]').count(), 'angle shown for a single element');
  await page.click('[data-action="stats"]');
  assert.equal(await page.locator('.wbui-stats').isVisible(), false);
});

// ------------------------------------------------------------------ help
await test('help: ? opens the shortcuts dialog, search filters, Esc closes', async (page) => {
  await page.mouse.move(700, 500);
  await page.keyboard.press('?');
  const dlg = page.locator('dialog.wbui-help[open]');
  await dlg.waitFor();
  const sections = await dlg.locator('.wbui-help-h').allTextContents();
  assert.deepEqual(sections, ['Tools', 'View', 'Editor']);
  const rows = await dlg.locator('.wbui-help-row').allTextContents();
  const text = rows.join('\n');
  for (const needle of ['Rectangle', 'Eyedropper', 'Zoom to fit', 'Toggle theme', 'Group', 'Copy style', 'Command palette', 'Send to back']) assert.ok(text.includes(needle), needle);
  assert.ok(rows.length >= 70, 'lists a lot of shortcuts: ' + rows.length);
  assert.ok((await dlg.locator('.wbui-help-keys').first().textContent()).length > 0);
  assert.ok(/Ctrl|Cmd/.test(text), 'platform modifier rendered');
  assert.equal(await page.evaluate(() => document.activeElement.className.includes('wbui-help-search')), true, 'search focused');
  await page.keyboard.type('flip');
  const shown = await dlg.locator('.wbui-help-row:not([hidden])').allTextContents();
  assert.equal(shown.length, 2);
  assert.ok(shown.every((t) => /Flip/.test(t)));
  assert.equal(await dlg.locator('.wbui-help-col[data-section="Tools"]').isVisible(), false, 'empty sections collapse');
  await page.fill('.wbui-help-search', 'zzzz');
  assert.equal(await dlg.locator('.wbui-help-empty').isVisible(), true);
  await page.fill('.wbui-help-search', 'ctrl+shift');
  assert.ok((await dlg.locator('.wbui-help-row:not([hidden])').count()) > 3, 'search matches key text');
  await page.keyboard.press('Escape');
  await page.waitForSelector('dialog.wbui-help', { state: 'detached' });
  assert.equal(await page.evaluate(() => window.wb.tool), 'select', 'Esc did not leak to the engine tool state');
  await page.click('[data-action="help"]');
  assert.equal(await page.locator('dialog.wbui-help[open]').count(), 1, 'help button opens it too');
  await page.click('[data-action="help-close"]');
  await page.waitForSelector('dialog.wbui-help', { state: 'detached' });
});

// ------------------------------------------------------------------ modes
await test('read-only (view mode): only hand and laser stay enabled; panel hidden; zen hides chrome', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(100, 100)]);
  assert.equal(await props(page).isVisible(), true);
  await page.evaluate(() => window.wb.setMode({ view: true }));
  const enabled = await page.evaluate(() => [...document.querySelectorAll('.wbui-toolbar [data-tool]')].filter((b) => !b.disabled).map((b) => b.dataset.tool));
  assert.deepEqual(enabled, ['hand']);
  assert.equal(await page.locator('[data-action="tool-lock"]').isDisabled(), true);
  assert.equal(await props(page).isVisible(), false);
  await page.click('[data-action="more-tools"]');
  const more = await page.evaluate(() => [...document.querySelectorAll('.wbui-popover [data-tool]')].filter((b) => !b.disabled).map((b) => b.dataset.tool));
  assert.deepEqual(more, ['laser']);
  await page.click('.wbui-popover button[data-tool="laser"]');
  assert.equal(await page.evaluate(() => window.wb.tool), 'laser');
  await page.evaluate(() => window.wb.setMode({ view: false }));
  assert.equal(await page.locator('.wbui-toolbar button[data-tool="rect"]').isEnabled(), true);
  await page.evaluate(() => window.wb.setMode({ zen: true }));
  for (const c of ['.wbui-toolbar', '.wbui-zoombar', '.wbui-help-btn']) assert.equal(await page.locator(c).isVisible(), false, c + ' hidden in zen');
  await page.evaluate(() => window.wb.setMode({ zen: false }));
  assert.equal(await page.locator('.wbui-toolbar').isVisible(), true);
});

await test('chrome never blocks the canvas between panels', async (page) => {
  const hit = (x, y) => page.evaluate(([x, y]) => document.elementFromPoint(x, y).closest('[data-wb-canvas], .wb-canvas, svg') !== null && !document.elementFromPoint(x, y).closest('.wbui-root'), [x, y]);
  assert.equal(await hit(300, 300), true);
  assert.equal(await hit(1200, 40), true, 'beside the toolbar');
  assert.equal(await page.evaluate(() => getComputedStyle(document.querySelector('.wbui-root')).pointerEvents), 'none');
});

await test('theme: chrome follows the whiteboard theme (dark)', async (page) => {
  await prep(page);
  await insert(page, [rectSpec(500, 250)]);
  assert.equal(await page.locator('.wbui-toolbar-root').getAttribute('data-wbui-theme'), 'dark');
  const bg = await page.locator('.wbui-toolbar').evaluate((e) => getComputedStyle(e).backgroundColor);
  const [r, g, b] = bg.match(/\d+/g).map(Number);
  assert.ok(r + g + b < 300, 'dark toolbar background: ' + bg);
  await page.evaluate(() => window.wb.toggleTheme());
  assert.equal(await page.locator('.wbui-toolbar-root').getAttribute('data-wbui-theme'), 'light');
}, { query: '?theme=dark' });

// ------------------------------------------------------------------ screenshots (looked at by a human; asserted only to exist)
for (const theme of ['light', 'dark']) {
  for (const [w, h] of [[1440, 900], [800, 600]]) {
    await test(`screenshot ${theme} ${w}x${h}`, async (page) => {
      await prep(page);
      await insert(page, [
        rectSpec(420, 160, { text: 'API', backgroundColor: '#a5d8ff', fillStyle: 'solid' }),
        { type: 'ellipse', props: { x: 640, y: 160, w: 140, h: 90, text: 'Client' } },
        { type: 'sticky', props: { x: 420, y: 330, w: 150, h: 120, text: 'Note' } },
        { type: 'arrow', props: { x: 560, y: 205, points: [[0, 0], [80, 0]], w: 80, h: 0 } },
      ], false);
      await page.evaluate(() => window.wb.select([window.wb.board.items[0].id]));
      await page.click('[data-action="stats"]');
      await page.screenshot({ path: path.join(shots, `chrome-${theme}-${w}x${h}.png`) });
      await page.click('[data-prop-chip="strokeColor"]').catch(() => {});
      await page.waitForTimeout(250);
      await page.screenshot({ path: path.join(shots, `chrome-${theme}-${w}x${h}-palette.png`) });
      await page.keyboard.press('Escape');
      await page.evaluate(() => window.wb.select([window.wb.board.items[3].id]));
      await page.click('[data-prop-picker="endArrowhead"]').catch(() => {});
      await page.waitForTimeout(250);
      await page.screenshot({ path: path.join(shots, `chrome-${theme}-${w}x${h}-arrow.png`) });
      await page.keyboard.press('Escape');
      await page.evaluate(() => window.wb.select([]));
      await page.mouse.move(700, 500); await page.keyboard.press('?');
      await page.waitForTimeout(250);
      await page.screenshot({ path: path.join(shots, `chrome-${theme}-${w}x${h}-help.png`) });
      assert.ok(fs.existsSync(path.join(shots, `chrome-${theme}-${w}x${h}.png`)));
    }, { query: `?theme=${theme}`, viewport: { width: w, height: h } });
  }
}

await browser.close();
server.close();
assert.deepEqual(errors, [], 'no page errors: ' + errors.join('; '));
console.log(`ui-chrome e2e: ${passed} tests passed (screenshots in ${shots})`);

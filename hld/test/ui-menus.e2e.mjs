// Browser tests for the whiteboard menus, dialogs, command palette, find bar and library (agent D).
// Run: node test/ui-menus.e2e.mjs   (HLD_SHOTS=dir to keep screenshots; HLD_CHROME for the chromium path)
import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';

const here = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(here, '..');
const SHOTS = process.env.HLD_SHOTS || '';
if (SHOTS) fs.mkdirSync(SHOTS, { recursive: true });
const TMP = fs.mkdtempSync(path.join(os.tmpdir(), 'wbm-'));

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

const PAGE = `<!doctype html><html lang="en"><head><meta charset="utf-8"><title>menus</title>
<link rel="stylesheet" href="/styles.css"><link rel="stylesheet" href="/wb/engine.css"><link rel="stylesheet" href="/wb/chrome.css"><link rel="stylesheet" href="/wb/menus.css">
<style>html,body{margin:0;height:100%;overflow:hidden;background:var(--bg)} #host{position:absolute;inset:0;overflow:hidden;background:var(--board,#fff)} #stage{position:absolute;inset:0}
#lib{position:absolute;top:12px;right:12px;bottom:12px;width:300px;z-index:40}</style></head>
<body><div id="host"><div id="stage"></div><div id="lib" hidden></div></div>
<script type="module">
import { Whiteboard } from '/wb/engine.js';
import { mountMenu } from '/wb/ui-menu.js';
import { mountContextMenu } from '/wb/ui-context.js';
import { mountPalette, mountSearch, buildCommands, ENGINE_ACTIONS } from '/wb/ui-palette.js';
import { mountLibrary } from '/wb/ui-library.js';
import { openExportDialog } from '/wb/ui-export.js';
import { createElement } from '/shared/board.js';
const q = new URLSearchParams(location.search);
const host = document.getElementById('host');
const wb = new Whiteboard(document.getElementById('stage'), { theme: q.get('theme') === 'dark' ? 'dark' : 'light' });
window.wb = wb; window.hookLog = []; window.mods = { buildCommands, ENGINE_ACTIONS, openExportDialog, createElement };
window.parts = {
  menu: mountMenu(wb, host, { onOpen: (b, m) => hookLog.push(['open', m.name, b.items.length]), onSave: () => hookLog.push(['save']), fileName: () => window.__fileName || 'scene', extra: [{ label: 'Back to home', action: () => hookLog.push(['home']) }] }),
  context: mountContextMenu(wb, host), palette: mountPalette(wb, host), search: mountSearch(wb, host),
  library: mountLibrary(wb, document.getElementById('lib'), { storageKey: 'wbm-test-lib' }),
};
window.addShape = (type, x, y, text, extra = {}) => wb.insertElements([createElement(type, { x, y, w: 160, h: 80, text, ...extra })], { at: { x: x + 80, y: y + 40 }, select: false })[0];
window.ready = true;
</script></body></html>`;

const MIME = { '.js': 'text/javascript', '.css': 'text/css', '.html': 'text/html', '.json': 'application/json', '.svg': 'image/svg+xml' };
const server = http.createServer((req, res) => {
  const u = new URL(req.url, 'http://x');
  if (u.pathname === '/__page') { res.writeHead(200, { 'content-type': 'text/html' }); res.end(PAGE); return; }
  let file;
  if (u.pathname.startsWith('/shared/')) file = path.join(ROOT, 'shared', u.pathname.slice(8));
  else file = path.join(ROOT, 'public', u.pathname);
  if (!file.startsWith(ROOT) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) { res.writeHead(404); res.end('nf'); return; }
  res.writeHead(200, { 'content-type': MIME[path.extname(file)] || 'application/octet-stream' });
  fs.createReadStream(file).pipe(res);
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const BASE = `http://127.0.0.1:${server.address().port}`;

const browser = await chromium.launch({ executablePath: findChrome(), args: ['--no-sandbox'] });
const results = [];
const pageErrors = [];

async function newPage(theme = 'light', { viewport = { width: 1280, height: 800 } } = {}) {
  const ctx = await browser.newContext({ viewport, acceptDownloads: true, colorScheme: theme });
  await ctx.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: BASE });
  const page = await ctx.newPage();
  page.setDefaultTimeout(8000);
  page.on('pageerror', (e) => pageErrors.push(e.message));
  page.on('console', (m) => { if (m.type() === 'error' && !/404/.test(m.text())) pageErrors.push(m.text()); });
  await page.goto(`${BASE}/__page?theme=${theme}`);
  await page.waitForFunction(() => window.ready === true);
  return page;
}
const shot = async (page, name) => { if (SHOTS) await page.screenshot({ path: path.join(SHOTS, `${name}.png`) }); };
const wbEval = (page, fn, arg) => page.evaluate(fn, arg);
const readDownload = async (dl) => fs.readFileSync(await dl.path());
const pngSize = (buf) => ({ w: buf.readUInt32BE(16), h: buf.readUInt32BE(20) });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function test(name, fn) {
  if (process.env.ONLY && !name.includes(process.env.ONLY)) return;
  const t0 = Date.now();
  try { await fn(); results.push([name, true, Date.now() - t0]); console.log(`ok   ${name}`); } catch (err) {
    results.push([name, false, Date.now() - t0]); console.log(`FAIL ${name}\n     ${String(err && err.stack || err).split('\n').slice(0, 6).join('\n     ')}`);
  }
}

const seed = (page) => page.evaluate(() => {
  addShape('rect', 100, 140, 'API Gateway'); addShape('database', 420, 140, 'Orders DB'); addShape('ellipse', 260, 330, 'Client');
});

const EXCALIDRAW = {
  type: 'excalidraw', version: 2, source: 'test',
  elements: [
    { id: 'a', type: 'rectangle', x: 10, y: 10, width: 160, height: 80, angle: 0, strokeColor: '#e03131', backgroundColor: '#a5d8ff', fillStyle: 'solid', strokeWidth: 2, strokeStyle: 'solid', roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: { type: 3 }, seed: 11, version: 1, versionNonce: 1, isDeleted: false, boundElements: [{ type: 'text', id: 't' }], updated: 1, link: null, locked: false },
    { id: 't', type: 'text', x: 40, y: 38, width: 90, height: 25, angle: 0, strokeColor: '#1e1e1e', backgroundColor: 'transparent', fillStyle: 'solid', strokeWidth: 1, strokeStyle: 'solid', roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: null, seed: 12, version: 1, versionNonce: 2, isDeleted: false, boundElements: null, updated: 1, link: null, locked: false, text: 'Imported box', fontSize: 20, fontFamily: 1, textAlign: 'center', verticalAlign: 'middle', containerId: 'a', originalText: 'Imported box', lineHeight: 1.25 },
    { id: 'b', type: 'ellipse', x: 300, y: 20, width: 100, height: 100, angle: 0, strokeColor: '#1e1e1e', backgroundColor: '#b2f2bb', fillStyle: 'hachure', strokeWidth: 2, strokeStyle: 'solid', roughness: 1, opacity: 100, groupIds: [], frameId: null, roundness: null, seed: 13, version: 1, versionNonce: 3, isDeleted: false, boundElements: null, updated: 1, link: null, locked: false },
  ],
  appState: { viewBackgroundColor: '#f8f9fa' }, files: {},
};
const fixtureExcalidraw = path.join(TMP, 'sample.excalidraw');
fs.writeFileSync(fixtureExcalidraw, JSON.stringify(EXCALIDRAW));
const fixtureBad = path.join(TMP, 'bad.json');
fs.writeFileSync(fixtureBad, '{ nope');

// ======================================================================================================================
await test('main menu: open, sections, keyboard, check marks, extras', async () => {
  const page = await newPage();
  await page.click('[data-action="menu"]');
  assert.ok(await page.locator('.wbm-main-menu').isVisible());
  for (const a of ['open', 'save', 'export-image', 'copy-png', 'copy-svg', 'find', 'palette', 'clear', 'theme', 'grid', 'snap', 'zen', 'view-mode', 'stats', 'shortcuts']) {
    assert.equal(await page.locator(`.wbm-main-menu [data-action="${a}"]`).count(), 1, `menu item ${a}`);
  }
  assert.equal(await page.locator('.wbm-main-menu [role="menuitem"], .wbm-main-menu [role="menuitemcheckbox"]').first().textContent().then((t) => /Back to home/.test(t)), true, 'extra entry at the top');
  assert.equal(await page.locator('.wbm-main-menu [data-action="background"]').count(), 5);
  await shot(page, 'menu-light');
  // keyboard: arrows move, Escape closes and restores focus to the button
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'extra-0');
  await page.keyboard.press('ArrowDown');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'open');
  await page.keyboard.press('End');
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'shortcuts');
  await page.keyboard.press('Escape');
  assert.ok(!(await page.locator('.wbm-main-menu').isVisible()));
  assert.equal(await page.evaluate(() => document.activeElement.dataset.action), 'menu');
  // extra hook
  await page.click('[data-action="menu"]'); await page.click('[data-action="extra-0"]');
  assert.deepEqual(await page.evaluate(() => hookLog.filter((l) => l[0] === 'home')), [['home']]);
  // grid toggles with a check, stays open
  await page.click('[data-action="menu"]');
  await page.click('.wbm-main-menu [data-action="grid"]');
  assert.equal(await wbEval(page, () => wb.mode.grid), true);
  assert.equal(await page.getAttribute('.wbm-main-menu [data-action="grid"]', 'aria-checked'), 'true');
  assert.ok(await page.locator('.wbm-main-menu').isVisible());
  await page.click('.wbm-main-menu [data-action="snap"]');
  assert.equal(await wbEval(page, () => wb.mode.snap), true);
  // background swatch + custom colour
  await page.click('.wbm-main-menu [data-action="background"][data-value="#fffce8"]');
  assert.equal(await wbEval(page, () => wb.mode.background), '#fffce8');
  assert.equal(await page.getAttribute('[data-action="background"][data-value="#fffce8"]', 'aria-checked'), 'true');
  await page.locator('[data-action="background-custom"]').evaluate((i) => { i.value = '#ffd8a8'; i.dispatchEvent(new Event('input', { bubbles: true })); });
  assert.equal(await wbEval(page, () => wb.mode.background), '#ffd8a8');
  // zen closes the menu; view mode too
  await page.click('.wbm-main-menu [data-action="zen"]');
  assert.equal(await wbEval(page, () => wb.mode.zen), true);
  assert.ok(!(await page.locator('.wbm-main-menu').isVisible()));
  await wbEval(page, () => wb.toggleZen());
  await page.click('[data-action="menu"]'); await page.click('.wbm-main-menu [data-action="view-mode"]');
  assert.equal(await wbEval(page, () => wb.mode.view), true);
  await wbEval(page, () => wb.toggleViewMode());
  // theme
  await page.click('[data-action="menu"]');
  assert.match(await page.textContent('.wbm-main-menu [data-action="theme"]'), /Light/);
  await page.click('.wbm-main-menu [data-action="theme"]');
  assert.equal(await wbEval(page, () => wb.mode.theme), 'dark');
  assert.match(await page.textContent('.wbm-main-menu [data-action="theme"]'), /Dark/);
  await seed(page);
  await shot(page, 'menu-dark-engine');
  // outside click closes
  await page.mouse.click(700, 500);
  assert.ok(!(await page.locator('.wbm-main-menu').isVisible()));
  await page.context().close();
});

await test('menu in dark app theme', async () => {
  const page = await newPage('dark');
  await seed(page);
  await page.click('[data-action="menu"]');
  await shot(page, 'menu-dark');
  await page.context().close();
});

await test('save to file downloads .hld.json named from hooks', async () => {
  const page = await newPage();
  await seed(page);
  await page.click('[data-action="menu"]');
  const [dl] = await Promise.all([page.waitForEvent('download'), page.click('.wbm-main-menu [data-action="save"]')]);
  assert.equal(dl.suggestedFilename(), 'scene.hld.json');
  const data = JSON.parse((await readDownload(dl)).toString());
  assert.equal(data.type, 'hld-whiteboard');
  assert.equal(data.items.length, 3);
  assert.deepEqual(await page.evaluate(() => hookLog.filter((l) => l[0] === 'save')), [['save']]);
  // Ctrl+S via the engine request
  const [dl2] = await Promise.all([page.waitForEvent('download'), page.evaluate(() => wb._emit('request', { name: 'save' }))]);
  assert.equal(dl2.suggestedFilename(), 'scene.hld.json');
  await page.context().close();
});

await test('open: excalidraw fixture, confirm before replacing, bad file toast, round trip', async () => {
  const page = await newPage();
  // empty scene: opens straight away
  await page.click('[data-action="menu"]');
  let [fc] = await Promise.all([page.waitForEvent('filechooser'), page.click('.wbm-main-menu [data-action="open"]')]);
  await fc.setFiles(fixtureExcalidraw);
  await page.waitForFunction(() => wb.board.items.length > 0);
  const info = await wbEval(page, () => ({ n: wb.board.items.length, labels: wb.board.items.map((e) => e.text).filter(Boolean), bg: wb.mode.background, log: hookLog.filter((l) => l[0] === 'open') }));
  assert.ok(info.labels.includes('Imported box'), JSON.stringify(info));
  assert.equal(info.bg, '#f8f9fa');
  assert.equal(info.log.length, 1); assert.equal(info.log[0][1], 'sample');
  await shot(page, 'after-open');
  // non-empty: confirm dialog; cancel keeps
  await page.evaluate(() => addShape('rect', 500, 400, 'Extra'));
  const before = await wbEval(page, () => wb.board.items.length);
  [fc] = await Promise.all([page.waitForEvent('filechooser'), page.evaluate(() => { parts.menu.openFile(); })]);
  await fc.setFiles(fixtureExcalidraw);
  await page.waitForSelector('.wbm-confirm');
  await shot(page, 'confirm-open');
  await page.click('[data-action="confirm-cancel"]');
  await page.waitForSelector('.wbm-confirm', { state: 'detached' });
  assert.equal(await wbEval(page, () => wb.board.items.length), before);
  [fc] = await Promise.all([page.waitForEvent('filechooser'), page.evaluate(() => { parts.menu.openFile(); })]);
  await fc.setFiles(fixtureExcalidraw);
  await page.waitForSelector('.wbm-confirm');
  await page.click('[data-action="confirm-ok"]');
  await page.waitForFunction((b) => wb.board.items.length !== b, before);
  assert.ok(!(await wbEval(page, () => wb.board.items.some((e) => e.text === 'Extra'))));
  // bad file -> error toast, board untouched
  [fc] = await Promise.all([page.waitForEvent('filechooser'), page.evaluate(() => { parts.menu.openFile(); })]);
  await fc.setFiles(fixtureBad);
  await page.waitForSelector('.wbm-toast-error');
  assert.match(await page.textContent('.wbm-toast-error'), /bad\.json/);
  // round trip: save then open into a cleared board
  await page.evaluate(() => addShape('diamond', 40, 300, 'Round trip'));
  const [dl] = await Promise.all([page.waitForEvent('download'), page.evaluate(() => parts.menu.save())]);
  const saved = path.join(TMP, 'saved.hld.json');
  fs.writeFileSync(saved, await readDownload(dl));
  const n = await wbEval(page, () => wb.board.items.length);
  await wbEval(page, () => wb.clear());
  [fc] = await Promise.all([page.waitForEvent('filechooser'), page.evaluate(() => { parts.menu.openFile(); })]);
  await fc.setFiles(saved);
  await page.waitForFunction((k) => wb.board.items.length === k, n);
  assert.ok(await wbEval(page, () => wb.board.items.some((e) => e.text === 'Round trip')));
  await page.context().close();
});

await test('drag and drop a scene file onto the board offers to open it', async () => {
  const page = await newPage();
  await seed(page);
  await page.evaluate((json) => {
    const dt = new DataTransfer();
    dt.items.add(new File([json], 'dropped.excalidraw', { type: '' }));
    const host = document.getElementById('host');
    host.dispatchEvent(new DragEvent('dragenter', { bubbles: true, dataTransfer: dt }));
    host.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: dt }));
    host.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt }));
  }, JSON.stringify(EXCALIDRAW));
  await page.waitForSelector('.wbm-confirm');
  await page.click('[data-action="confirm-ok"]');
  await page.waitForFunction(() => wb.board.items.some((e) => e.text === 'Imported box'));
  await page.context().close();
});

await test('clear canvas: confirm dialog, undoable', async () => {
  const page = await newPage();
  await seed(page);
  await page.click('[data-action="menu"]'); await page.click('.wbm-main-menu [data-action="clear"]');
  await page.waitForSelector('.wbm-confirm');
  await page.click('[data-action="confirm-cancel"]');
  assert.equal(await wbEval(page, () => wb.board.items.length), 3);
  await page.click('[data-action="menu"]'); await page.click('.wbm-main-menu [data-action="clear"]');
  await page.waitForSelector('.wbm-confirm');
  await page.click('[data-action="confirm-ok"]');
  await page.waitForFunction(() => wb.board.items.length === 0);
  await wbEval(page, () => wb.undo());
  assert.equal(await wbEval(page, () => wb.board.items.length), 3);
  await page.context().close();
});

await test('export dialog: preview, options, PNG scale, SVG, embed scene, clipboard, saves', async () => {
  const page = await newPage();
  // empty board message
  await page.evaluate(() => parts.menu.close());
  await page.keyboard.press('Control+Shift+E');
  await page.waitForSelector('.wbm-export[open]');
  assert.ok(await page.locator('.wbm-export-empty').isVisible());
  assert.ok(await page.locator('.wbm-export [data-action="export-png"]').isDisabled());
  await page.keyboard.press('Escape');
  await page.waitForSelector('.wbm-export', { state: 'detached' });

  await seed(page);
  await page.click('[data-action="menu"]'); await page.click('.wbm-main-menu [data-action="export-image"]');
  await page.waitForSelector('.wbm-export[open]');
  await page.waitForFunction(() => { const i = document.querySelector('.wbm-export-img'); return i && i.complete && i.naturalWidth > 50; });
  await shot(page, 'export-light');
  assert.ok(await page.locator('[data-opt="selectionOnly"]').isDisabled(), 'only-selected disabled with nothing selected');
  const dl = async (action) => { const [d] = await Promise.all([page.waitForEvent('download'), page.click(`.wbm-export [data-action="${action}"]`)]); return d; };
  await page.fill('[data-opt="name"]', 'my diagram');
  await page.click('[data-opt="scale"][data-value="1"]');
  let d1 = await dl('export-png');
  assert.equal(d1.suggestedFilename(), 'my diagram.png');
  const s1 = pngSize(await readDownload(d1));
  await page.click('[data-opt="scale"][data-value="2"]');
  const s2 = pngSize(await readDownload(await dl('export-png')));
  await page.click('[data-opt="scale"][data-value="3"]');
  const s3 = pngSize(await readDownload(await dl('export-png')));
  assert.ok(s1.w > 200 && s1.h > 100, JSON.stringify(s1));
  assert.ok(Math.abs(s2.w - 2 * s1.w) <= 2 && Math.abs(s2.h - 2 * s1.h) <= 2, `2x ${JSON.stringify(s2)} vs ${JSON.stringify(s1)}`);
  assert.ok(Math.abs(s3.w - 3 * s1.w) <= 3, `3x ${JSON.stringify(s3)}`);
  // svg + embed
  let svgD = await dl('export-svg');
  assert.equal(svgD.suggestedFilename(), 'my diagram.svg');
  const svg = (await readDownload(svgD)).toString();
  assert.match(svg, /^<svg/); assert.ok(!svg.includes('hld-scene'));
  await page.check('[data-opt="embed"]');
  const svg2 = (await readDownload(await dl('export-svg'))).toString();
  assert.match(svg2, /<metadata id="hld-scene">[A-Za-z0-9+/=]+<\/metadata>/);
  fs.writeFileSync(path.join(TMP, 'embedded.svg'), svg2);
  // background off -> checkerboard + transparent png header size same
  await page.uncheck('[data-opt="background"]');
  await page.waitForSelector('.wbm-export-stage[data-checker="1"]');
  const svg3 = (await readDownload(await dl('export-svg'))).toString();
  assert.ok(svg3.length > 0 && svg3 !== svg2.replace(/<metadata[\s\S]*?<\/metadata>/, ''));
  await page.check('[data-opt="background"]');
  // dark mode option changes svg
  await page.uncheck('[data-opt="embed"]');
  const svgLight = (await readDownload(await dl('export-svg'))).toString();
  await page.check('[data-opt="dark"]');
  const svgDark = (await readDownload(await dl('export-svg'))).toString();
  assert.notEqual(svgLight, svgDark);
  await page.uncheck('[data-opt="dark"]');
  // clipboard
  await page.click('.wbm-export [data-action="copy-svg"]');
  await page.waitForSelector('.wbm-toast');
  assert.match(await page.evaluate(() => navigator.clipboard.readText()), /^<svg/);
  await page.click('.wbm-export [data-action="copy-png"]');
  await page.waitForFunction(() => /PNG/.test(document.querySelector('.wbm-toasts')?.textContent || ''));
  assert.deepEqual(await page.evaluate(async () => (await navigator.clipboard.read())[0].types), ['image/png']);
  // scene saves
  const hld = JSON.parse((await readDownload(await dl('save-hld'))).toString());
  assert.equal(hld.items.length, 3);
  const exd = await dl('save-excalidraw');
  assert.equal(exd.suggestedFilename(), 'my diagram.excalidraw');
  const exj = JSON.parse((await readDownload(exd)).toString());
  assert.equal(exj.type, 'excalidraw'); assert.ok(exj.elements.length >= 3);
  await page.keyboard.press('Escape');
  await page.waitForSelector('.wbm-export', { state: 'detached' });

  // selection only: enabled with selection, smaller output
  await page.evaluate(() => wb.select([wb.board.items[0].id]));
  await page.keyboard.press('Control+Shift+E');
  await page.waitForSelector('.wbm-export[open]');
  assert.ok(await page.locator('[data-opt="selectionOnly"]').isEnabled());
  await page.click('[data-opt="scale"][data-value="1"]');
  const full = pngSize(await readDownload(await dl('export-png')));
  await page.check('[data-opt="selectionOnly"]');
  const part = pngSize(await readDownload(await dl('export-png')));
  assert.ok(part.w < full.w && part.h < full.h, `${JSON.stringify(part)} < ${JSON.stringify(full)}`);
  await page.context().close();

  // the embedded svg re-opens as a scene
  const p2 = await newPage();
  const [fc] = await Promise.all([p2.waitForEvent('filechooser'), p2.evaluate(() => { parts.menu.openFile(); })]);
  await fc.setFiles(path.join(TMP, 'embedded.svg'));
  await p2.waitForFunction(() => wb.board.items.length === 3);
  await p2.context().close();
});

await test('export dialog dark app theme screenshot', async () => {
  const page = await newPage('dark');
  await seed(page);
  await page.evaluate(() => wb.toggleTheme());
  await page.keyboard.press('Control+Shift+E');
  await page.waitForSelector('.wbm-export[open]');
  await page.waitForFunction(() => { const i = document.querySelector('.wbm-export-img'); return i && i.complete && i.naturalWidth > 50; });
  await page.uncheck('[data-opt="background"]');
  await sleep(300);
  await shot(page, 'export-dark');
  await page.context().close();
});

await test('context menu: canvas and element menus, submenus, effects, positioning', async () => {
  const page = await newPage();
  await seed(page);
  const pos = await page.evaluate(() => { const e = wb.board.items[0]; const p = wb.worldToClient(e.x + e.w / 2, e.y + e.h / 2); return p; });
  // canvas menu on empty space
  await page.mouse.click(900, 600, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="canvas"]');
  for (const a of ['paste', 'select-all', 'grid', 'zen', 'view-mode']) assert.equal(await page.locator(`.wbm-ctx [data-action="${a}"]`).count(), 1, a);
  await shot(page, 'context-canvas');
  await page.click('.wbm-ctx [data-action="select-all"]');
  assert.equal(await wbEval(page, () => wb.selection.size), 3);
  assert.ok(!(await page.locator('.wbm-ctx').count()));
  await wbEval(page, () => wb.clearSelection());
  // element menu
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="element"]');
  assert.equal(await wbEval(page, () => wb.selection.size), 1);
  assert.equal(await page.getAttribute('.wbm-ctx [data-action="group"]', 'aria-disabled'), 'true');
  assert.equal(await page.getAttribute('.wbm-ctx [data-action="align"]', 'aria-disabled'), 'true');
  await page.hover('.wbm-ctx [data-action="layers"]');
  await page.waitForSelector('.wbm-ctx[data-menu="submenu"]');
  await shot(page, 'context-element-layers');
  await page.click('.wbm-ctx [data-action="bring-front"]');
  assert.equal(await wbEval(page, () => wb.board.items.at(-1).text), 'API Gateway');
  // lock, then label flips to Unlock
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="element"]');
  await page.click('.wbm-ctx [data-action="lock"]');
  assert.equal(await wbEval(page, () => wb.board.items.at(-1).locked), true);
  await wbEval(page, () => wb.unlockAll());
  // duplicate, delete
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.click('.wbm-ctx [data-action="duplicate"]');
  assert.equal(await wbEval(page, () => wb.board.items.length), 4);
  await page.mouse.click(pos.x + 20, pos.y + 20, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="element"]');
  await page.click('.wbm-ctx [data-action="delete"]');
  assert.equal(await wbEval(page, () => wb.board.items.length), 3);
  // multi selection: group enabled, align submenu
  await wbEval(page, () => wb.selectAll());
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="element"]');
  assert.equal(await page.getAttribute('.wbm-ctx [data-action="group"]', 'aria-disabled'), null);
  await page.hover('.wbm-ctx [data-action="align"]');
  await page.click('.wbm-ctx [data-action="align-left"]');
  const xs = await wbEval(page, () => wb.board.items.map((e) => Math.round(e.x)));
  assert.equal(new Set(xs).size, 1, `aligned left ${xs}`);
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.click('.wbm-ctx [data-action="group"]');
  assert.ok(await wbEval(page, () => wb.board.items.every((e) => e.groupIds.length === 1)));
  // link prompt
  await wbEval(page, () => wb.select([wb.board.items[0].id]));
  await page.mouse.click(pos.x, pos.y, { button: 'right' }).catch(() => {});
  await page.keyboard.press('Escape');
  // Esc / outside click close
  await page.mouse.click(900, 600, { button: 'right' });
  await page.waitForSelector('.wbm-ctx');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('.wbm-ctx').count(), 0);
  await page.mouse.click(900, 600, { button: 'right' });
  await page.waitForSelector('.wbm-ctx');
  await page.mouse.click(200, 200);
  assert.equal(await page.locator('.wbm-ctx').count(), 0);
  // positioned inside the viewport near the corner, submenu flips
  await wbEval(page, (p) => { wb.select([wb.board.items[0].id]); const e = wb.board.items[0]; const c = wb.worldToClient(e.x + e.w / 2, e.y + e.h / 2); wb.pan(1272 - c.x, 792 - c.y); }, pos);
  await page.mouse.click(1272, 792, { button: 'right' });
  await page.waitForSelector('.wbm-ctx');
  let r = await page.locator('.wbm-ctx').first().boundingBox();
  assert.ok(r.x >= 0 && r.y >= 0 && r.x + r.width <= 1280 && r.y + r.height <= 800, JSON.stringify(r));
  await page.hover('.wbm-ctx [data-action="layers"]');
  await page.waitForSelector('.wbm-ctx[data-menu="submenu"]');
  r = await page.locator('.wbm-ctx[data-menu="submenu"]').boundingBox();
  assert.ok(r.x >= 0 && r.x + r.width <= 1280 && r.y + r.height <= 800, JSON.stringify(r));
  await page.keyboard.press('Escape');
  await page.context().close();
});

await test('context menu keyboard + link dialog', async () => {
  const page = await newPage();
  await seed(page);
  const pos = await page.evaluate(() => { const e = wb.board.items[0]; return wb.worldToClient(e.x + e.w / 2, e.y + e.h / 2); });
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  await page.waitForSelector('.wbm-ctx[data-menu="element"]');
  await page.click('.wbm-ctx [data-action="link"]');
  await page.waitForSelector('.wbm-prompt');
  await page.fill('[data-action="prompt-input"]', 'https://example.com/x');
  await page.click('[data-action="prompt-ok"]');
  await page.waitForFunction(() => wb.board.items[0].link === 'https://example.com/x');
  await page.mouse.click(pos.x, pos.y, { button: 'right' });
  assert.match(await page.textContent('.wbm-ctx [data-action="link"]'), /Edit link/);
  await page.keyboard.press('ArrowDown'); await page.keyboard.press('Escape');
  await page.context().close();
});

await test('command palette: fuzzy run, recent first, every tool and action present', async () => {
  const page = await newPage();
  await seed(page);
  await page.keyboard.press('Control+/');
  await page.waitForSelector('.wbm-palette[open]');
  await shot(page, 'palette-light');
  assert.ok((await page.locator('.wbm-palette-row').count()) > 70);
  await page.keyboard.type('rect');
  assert.equal(await page.locator('.wbm-palette-row').first().getAttribute('data-cmd'), 'tool:rect');
  await page.keyboard.press('Enter');
  await page.waitForSelector('.wbm-palette', { state: 'detached' });
  assert.equal(await wbEval(page, () => wb.tool), 'rect');
  // fuzzy: "tgrid" -> toggle grid
  await page.keyboard.press('Control+Shift+P');
  await page.waitForSelector('.wbm-palette[open]');
  await page.keyboard.type('tgl grid');
  assert.equal(await page.locator('.wbm-palette-row').first().getAttribute('data-cmd'), 'grid');
  await page.keyboard.press('ArrowDown'); await page.keyboard.press('ArrowUp');
  await page.keyboard.press('Enter');
  await page.waitForFunction(() => wb.mode.grid === true);
  // recents first
  await page.keyboard.press('Control+/');
  await page.waitForSelector('.wbm-palette[open]');
  assert.equal(await page.locator('.wbm-palette-row').first().getAttribute('data-cmd'), 'grid');
  assert.equal(await page.locator('.wbm-palette-group').first().textContent(), 'Recent');
  assert.equal(await page.locator('.wbm-palette-row').nth(1).getAttribute('data-cmd'), 'tool:rect');
  // click a row; Esc closes
  await page.keyboard.type('zen');
  await page.click('[data-cmd="zen"]');
  await page.waitForFunction(() => wb.mode.zen === true);
  await wbEval(page, () => wb.toggleZen());
  await page.keyboard.press('Control+/');
  await page.waitForSelector('.wbm-palette[open]');
  await page.keyboard.press('Escape');
  await page.waitForSelector('.wbm-palette', { state: 'detached' });
  // coverage: every tool, every engine action
  const cov = await page.evaluate(() => {
    const cmds = mods.buildCommands(wb, document.getElementById('host'));
    const ids = new Set(cmds.map((c) => c.id));
    const tools = ['hand', 'select', 'rect', 'diamond', 'ellipse', 'arrow', 'line', 'freedraw', 'text', 'image', 'eraser', 'database', 'sticky', 'frame', 'laser', 'bucket', 'eyedropper'];
    const missingTools = tools.filter((t) => !ids.has(`tool:${t}`));
    const called = new Set();
    const skip = new Set(['link', 'clear-canvas', 'export-image', 'save', 'open', 'library', 'library-add', 'library-import', 'library-export', 'find', 'help', 'stats']);
    const orig = {};
    for (const m of mods.ENGINE_ACTIONS) { orig[m] = wb[m]; wb[m] = function (...a) { called.add(m); return undefined; }; }
    for (const c of cmds) if (!skip.has(c.id)) { try { c.run(); } catch (e) { /* ignore */ } }
    for (const m of mods.ENGINE_ACTIONS) wb[m] = orig[m];
    const must = mods.ENGINE_ACTIONS.filter((m) => !['setLink', 'clear'].includes(m));
    return { missingTools, missingActions: must.filter((m) => !called.has(m)), categories: [...new Set(cmds.map((c) => c.category))] };
  });
  assert.deepEqual(cov.missingTools, []);
  assert.deepEqual(cov.missingActions, []);
  assert.deepEqual(cov.categories.sort(), ['Edit', 'Export', 'Library', 'Tools', 'View']);
  await page.context().close();
});

await test('command palette dark screenshot', async () => {
  const page = await newPage('dark');
  await seed(page);
  await page.keyboard.press('Control+/');
  await page.waitForSelector('.wbm-palette[open]');
  await page.keyboard.type('al');
  await shot(page, 'palette-dark');
  await page.context().close();
});

await test('find on canvas: count, stepping, case-insensitive, Esc restores view', async () => {
  const page = await newPage();
  await page.evaluate(() => { addShape('rect', 100, 100, 'Apple service'); addShape('rect', 2500, 100, 'apple pie'); addShape('rect', 100, 1800, 'Banana'); addShape('text', 600, 600, 'APPLE', { w: 90, h: 30 }); });
  await page.evaluate(() => wb.zoomToFit());
  const view0 = await wbEval(page, () => ({ ...wb.view }));
  await page.evaluate(() => wb.setZoom(1));
  const view1 = await wbEval(page, () => ({ ...wb.view }));
  await page.keyboard.press('Control+f');
  await page.waitForSelector('.wbm-search:not([hidden])');
  assert.equal(await page.evaluate(() => document.activeElement.className), 'wbm-search-input');
  await page.keyboard.type('apple');
  await page.waitForFunction(() => /1 \/ 3/.test(document.querySelector('.wbm-search-count').textContent));
  const first = await wbEval(page, () => [...wb.selection][0]);
  await shot(page, 'search');
  await page.keyboard.press('Enter');
  assert.match(await page.textContent('.wbm-search-count'), /2 \/ 3/);
  const second = await wbEval(page, () => [...wb.selection][0]);
  assert.notEqual(first, second);
  await page.keyboard.press('Enter'); await page.keyboard.press('Enter');
  assert.match(await page.textContent('.wbm-search-count'), /1 \/ 3/);
  await page.keyboard.press('Shift+Enter');
  assert.match(await page.textContent('.wbm-search-count'), /3 \/ 3/);
  await page.click('[data-action="search-next"]');
  assert.match(await page.textContent('.wbm-search-count'), /1 \/ 3/);
  await page.click('[data-action="search-prev"]');
  assert.match(await page.textContent('.wbm-search-count'), /3 \/ 3/);
  await page.fill('.wbm-search-input', 'zzz');
  await page.waitForFunction(() => /0 \/ 0/.test(document.querySelector('.wbm-search-count').textContent));
  await page.fill('.wbm-search-input', 'banana');
  await page.waitForFunction(() => /1 \/ 1/.test(document.querySelector('.wbm-search-count').textContent));
  await page.keyboard.press('Escape');
  assert.ok(await page.locator('.wbm-search').isHidden());
  const view2 = await wbEval(page, () => ({ ...wb.view }));
  assert.ok(Math.abs(view2.x - view1.x) < 0.5 && Math.abs(view2.y - view1.y) < 0.5 && Math.abs(view2.zoom - view1.zoom) < 1e-6, `${JSON.stringify(view2)} vs ${JSON.stringify(view1)}`);
  void view0;
  // menu entry opens it too
  await page.click('[data-action="menu"]'); await page.click('.wbm-main-menu [data-action="find"]');
  await page.waitForSelector('.wbm-search:not([hidden])');
  await page.context().close();
});

await test('library: built-in tiles, search, click + drag insert, my library persists, rename, delete+undo, export/import', async () => {
  const page = await newPage();
  await page.evaluate(() => window.dispatchEvent(new CustomEvent('wbm:request', { detail: { wb, name: 'library' } })));
  await page.waitForSelector('#lib:not([hidden]) .wbm-library');
  const tiles = await page.locator('[data-action="library-item"][data-source="builtin"]').count();
  assert.ok(tiles >= 20, `builtin tiles ${tiles}`);
  assert.ok((await page.locator('.wbm-lib-cat').count()) >= 5);
  await page.waitForFunction(() => [...document.querySelectorAll('.wbm-lib-thumb')].every((i) => i.complete && i.naturalWidth > 0));
  assert.match(await page.getAttribute('.wbm-lib-thumb', 'src'), /^data:image\/svg\+xml/);
  await shot(page, 'library-light');
  // search
  await page.fill('[data-action="library-search"]', 'cache');
  const filtered = await page.locator('[data-action="library-item"]').count();
  assert.ok(filtered >= 1 && filtered < tiles, `filtered ${filtered}`);
  await page.fill('[data-action="library-search"]', '');
  // click inserts at the viewport centre
  const before = await wbEval(page, () => wb.board.items.length);
  await page.click('[data-action="library-item"]');
  const after = await wbEval(page, () => wb.board.items.length);
  assert.ok(after > before);
  const c = await wbEval(page, () => { const ids = [...wb.selection]; const els = wb.board.items.filter((e) => ids.includes(e.id)); const x0 = Math.min(...els.map((e) => e.x)); const x1 = Math.max(...els.map((e) => e.x + e.w)); const y0 = Math.min(...els.map((e) => e.y)); const y1 = Math.max(...els.map((e) => e.y + e.h)); const p = wb.worldToClient((x0 + x1) / 2, (y0 + y1) / 2); const r = document.getElementById('stage').getBoundingClientRect(); return { dx: p.x - (r.left + r.width / 2), dy: p.y - (r.top + r.height / 2) }; });
  assert.ok(Math.abs(c.dx) < 40 && Math.abs(c.dy) < 40, JSON.stringify(c));
  // drag onto the canvas at a given point
  await wbEval(page, () => { wb.setBoard({ version: 2, items: [], nextId: 1 }); });
  const tile = page.locator('[data-action="library-item"]').nth(2);
  await tile.dragTo(page.locator('#stage'), { targetPosition: { x: 300, y: 400 } });
  await page.waitForFunction(() => wb.board.items.length > 0);
  const d = await wbEval(page, () => { const els = wb.board.items; const x0 = Math.min(...els.map((e) => e.x)); const x1 = Math.max(...els.map((e) => e.x + e.w)); const y0 = Math.min(...els.map((e) => e.y)); const y1 = Math.max(...els.map((e) => e.y + e.h)); const p = wb.worldToClient((x0 + x1) / 2, (y0 + y1) / 2); const r = document.getElementById('stage').getBoundingClientRect(); return { x: p.x - r.left, y: p.y - r.top }; });
  assert.ok(Math.abs(d.x - 300) < 25 && Math.abs(d.y - 400) < 25, `drop point ${JSON.stringify(d)}`);
  // add selection to library
  await page.click('[data-action="library-tab-mine"]');
  assert.match(await page.textContent('.wbm-lib-empty'), /empty/);
  await wbEval(page, () => wb.clearSelection());
  await page.click('[data-action="library-add"]');
  await page.waitForSelector('.wbm-toast');
  assert.match(await page.textContent('.wbm-toasts'), /Select something/);
  await wbEval(page, () => wb.selectAll());
  await page.click('[data-action="library-add"]');
  await page.waitForSelector('.wbm-prompt');
  await page.fill('[data-action="prompt-input"]', 'My stencil');
  await page.click('[data-action="prompt-ok"]');
  await page.waitForSelector('[data-source="mine"]');
  assert.equal(await page.locator('[data-source="mine"] .wbm-lib-name').first().textContent(), 'My stencil');
  // reload -> persisted
  await page.reload();
  await page.waitForFunction(() => window.ready === true);
  await wbEval(page, () => window.parts.library.open());
  await page.click('[data-action="library-tab-mine"]');
  await page.waitForSelector('[data-source="mine"]');
  assert.equal(await page.locator('[data-source="mine"]').count(), 1);
  // insert my item
  await page.click('[data-source="mine"]');
  assert.ok(await wbEval(page, () => wb.board.items.length) > 0);
  // rename
  await page.hover('.wbm-lib-item');
  await page.click('[data-action="library-rename"]');
  await page.fill('[data-action="prompt-input"]', 'Renamed');
  await page.click('[data-action="prompt-ok"]');
  await page.waitForFunction(() => document.querySelector('[data-source="mine"] .wbm-lib-name')?.textContent === 'Renamed');
  // export
  const [dl] = await Promise.all([page.waitForEvent('download'), page.click('[data-action="library-export"]')]);
  const libFile = path.join(TMP, 'out.hldlib.json');
  fs.writeFileSync(libFile, await readDownload(dl));
  const lib = JSON.parse(fs.readFileSync(libFile, 'utf8'));
  assert.equal(lib.items.length, 1); assert.equal(lib.items[0].name, 'Renamed');
  // delete -> undo toast
  await page.hover('.wbm-lib-item');
  await page.click('[data-action="library-delete"]');
  assert.equal(await page.locator('[data-source="mine"]').count(), 0);
  await page.click('.wbm-toast-action');
  assert.equal(await page.locator('[data-source="mine"]').count(), 1);
  // delete for real, then import the exported file (merge) twice -> two items with distinct ids
  await page.hover('.wbm-lib-item'); await page.click('[data-action="library-delete"]');
  const [fc] = await Promise.all([page.waitForEvent('filechooser'), page.click('[data-action="library-import"]')]);
  await fc.setFiles(libFile);
  await page.waitForSelector('[data-source="mine"]');
  const [fc2] = await Promise.all([page.waitForEvent('filechooser'), page.click('[data-action="library-import"]')]);
  await fc2.setFiles(libFile);
  await page.waitForFunction(() => document.querySelectorAll('[data-source="mine"]').length === 2);
  const ids = await page.locator('[data-source="mine"]').evaluateAll((n) => n.map((x) => x.dataset.id));
  assert.equal(new Set(ids).size, 2);
  await shot(page, 'library-mine');
  // corrupt storage does not break mounting
  await page.evaluate(() => localStorage.setItem('wbm-test-lib', '{{{'));
  await page.reload();
  await page.waitForFunction(() => window.ready === true);
  assert.equal(await wbEval(page, () => window.parts.library.getItems().length), 0);
  await page.context().close();
});

await test('library dark screenshot', async () => {
  const page = await newPage('dark');
  await page.evaluate(() => wb.toggleTheme());
  await page.evaluate(() => window.parts.library.open());
  await page.waitForSelector('.wbm-lib-thumb');
  await sleep(300);
  await shot(page, 'library-dark');
  await page.context().close();
});

await test('keyboard-only: Ctrl+O request opens the picker', async () => {
  const page = await newPage();
  const [fc] = await Promise.all([page.waitForEvent('filechooser'), page.evaluate(() => wb._emit('request', { name: 'open' }))]);
  assert.ok(fc);
  await page.context().close();
});

await test('no page errors', async () => {
  assert.deepEqual(pageErrors, []);
});

await browser.close();
server.close();
fs.rmSync(TMP, { recursive: true, force: true });
const failed = results.filter((r) => !r[1]);
console.log(`\n${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 1 : 0);

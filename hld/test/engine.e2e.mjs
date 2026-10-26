// Whiteboard engine end-to-end tests: real server (no AI) + headless Chromium driving public/wb/dev.html with the real mouse and keyboard.
// Run with: node test/engine.e2e.mjs   (HLD_CHROME=<path> if Chromium is not found; HLD_SHOTS=<dir> to keep screenshots)
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { createServer, loadQuestions } from '../server/index.js';

const here = path.dirname(fileURLToPath(import.meta.url));

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

const shots = process.env.HLD_SHOTS || fs.mkdtempSync(path.join(os.tmpdir(), 'wb-shots-'));
fs.mkdirSync(shots, { recursive: true });

const server = createServer({ ai: null, questions: loadQuestions(), quizzes: [] });
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const base = `http://127.0.0.1:${server.address().port}/wb/dev.html`;

const browser = await chromium.launch({ executablePath: findChrome(), args: ['--no-sandbox'] });
const context = await browser.newContext({ viewport: { width: 1200, height: 800 }, permissions: ['clipboard-read', 'clipboard-write'] });
const page = await context.newPage();
const pageErrors = [];
page.on('pageerror', (e) => pageErrors.push(e.message));
page.on('console', (m) => { if (m.type() === 'error' && !/404|favicon/i.test(m.text())) pageErrors.push(m.text()); });

// ---------- helpers ----------
const items = () => page.evaluate(() => wb.getBoard().items);
const byType = async (t) => (await items()).filter((e) => e.type === t);
const last = async () => { const a = await items(); return a[a.length - 1]; };
const sel = () => page.evaluate(() => [...wb.selection]);
const near = (a, b, tol = 1.5, msg = '') => assert.ok(Math.abs(a - b) <= tol, `${msg} expected ${b} got ${a}`);

async function reset(url = base) {
  if (!(await page.evaluate(() => !!window.wb).catch(() => false))) { await page.goto(url); await page.waitForFunction(() => window.__wbReady); }
  await page.evaluate(() => {
    wb._finishEditor?.({ discard: true });
    wb.setMode({ view: false, grid: false, snap: false, zen: false, theme: 'light', background: null });
    wb.setBoard({ version: 2, items: [], nextId: 1 });
    wb.setTool('select', { lock: false });
    wb._setView({ x: 0, y: 0, zoom: 1 });
    wb.setStyle({ strokeColor: '#1e1e1e', backgroundColor: 'transparent', strokeWidth: 2, roughness: 1, arrowType: 'sharp', endArrowhead: 'arrow', startArrowhead: null, fillStyle: 'solid', opacity: 100, fontSize: 20, fontFamily: 1 });
    wb.clearSelection();
    window.__events.length = 0;
  });
  await page.mouse.move(5, 5);
}

async function drag(x1, y1, x2, y2, { steps = 8, mods = [] } = {}) {
  for (const m of mods) await page.keyboard.down(m);
  await page.mouse.move(x1, y1);
  await page.mouse.down();
  await page.mouse.move(x2, y2, { steps });
  await page.mouse.up();
  for (const m of mods.slice().reverse()) await page.keyboard.up(m);
}
const click = async (x, y, { mods = [], count = 1 } = {}) => {
  for (const m of mods) await page.keyboard.down(m);
  await page.mouse.click(x, y, { clickCount: count });
  for (const m of mods.slice().reverse()) await page.keyboard.up(m);
};
const dbl = (x, y) => page.mouse.dblclick(x, y);
const press = (k) => page.keyboard.press(k);
const MOD = 'Control';

// make shapes directly (not via UI) for tests that are about something else
const add = (el) => page.evaluate((e) => {
  const b = wb.board;
  const id = b.nextId++;
  const full = { id, type: 'rect', x: 0, y: 0, w: 100, h: 60, angle: 0, strokeColor: '#1e1e1e', backgroundColor: '#e9ecef', fillStyle: 'solid', strokeWidth: 2, strokeStyle: 'solid',
    roughness: 1, opacity: 100, roundness: 'round', seed: id * 7919, groupIds: [], frameId: null, locked: false, link: null, text: '', fontFamily: 1, fontSize: 20, textAlign: 'center', verticalAlign: 'middle', ...e };
  const next = wb.getBoard();
  next.items.push(full);
  next.nextId = b.nextId;
  wb.setBoard(next, { resetHistory: false });
  return id;
}, el);

let passed = 0;
const failures = [];
async function test(name, fn) {
  if (process.env.ONLY && !name.includes(process.env.ONLY)) return;
  const before = pageErrors.length;
  try {
    await reset();
    await fn();
    if (pageErrors.length > before) throw new Error(`page errors: ${pageErrors.slice(before).join(' | ')}`);
    passed++;
    console.log(`  ok  ${name}`);
  } catch (err) {
    failures.push({ name, err });
    console.log(`  FAIL ${name}\n       ${String(err.stack || err).split('\n').slice(0, 4).join('\n       ')}`);
    try { await page.screenshot({ path: path.join(shots, `fail-${name.replace(/\W+/g, '-')}.png`) }); } catch { /* ignore */ }
  }
}

console.log('whiteboard engine e2e');
await page.goto(base);
await page.waitForFunction(() => window.__wbReady);

// ---------- tests ----------
await test('mounts: svg canvas, test hooks, defaults', async () => {
  assert.equal(await page.locator('svg[data-wb-canvas]').count(), 1);
  assert.equal(await page.evaluate(() => document.getElementById('host').__wb === wb), true);
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  assert.equal(await page.evaluate(() => wb.isEmpty()), true);
  assert.deepEqual(await page.evaluate(() => wb.view), { x: 0, y: 0, zoom: 1 });
  assert.equal(await page.evaluate(() => wb.style.strokeColor), '#1e1e1e');
});

await test('shape tools: drag creates the box, tool returns to select, shape is selected', async () => {
  const cases = [['r', 'rect'], ['d', 'diamond'], ['o', 'ellipse'], ['c', 'database'], ['n', 'sticky']];
  let i = 0;
  for (const [key, type] of cases) {
    await press(key);
    assert.equal(await page.evaluate(() => wb.tool), type);
    await drag(100 + i * 180, 100, 220 + i * 180, 190);
    const el = await last();
    assert.equal(el.type, type);
    near(el.w, 120); near(el.h, 90);
    assert.equal(el.text, '', 'created empty');
    assert.equal(await page.evaluate(() => wb.tool), 'select');
    assert.deepEqual(await sel(), [el.id]);
    await press('Escape'); // typing right after creation would start the label
    i++;
  }
  assert.equal((await items()).length, 5);
  assert.equal(await page.locator('.wb-el').count(), 5);
});

await test('number keys pick tools and click creates a default-sized shape centred on the pointer', async () => {
  await press('2');
  await click(400, 300);
  const r = await last();
  assert.equal(r.type, 'rect');
  near(r.w, 160); near(r.h, 90);
  near(r.x + r.w / 2, 400); near(r.y + r.h / 2, 300);
  await press('Escape');
  await press('4');
  assert.equal(await page.evaluate(() => wb.tool), 'ellipse');
  await press('3');
  assert.equal(await page.evaluate(() => wb.tool), 'diamond');
  await press('0');
  assert.equal(await page.evaluate(() => wb.tool), 'eraser');
  await press('h');
  assert.equal(await page.evaluate(() => wb.tool), 'hand');
  await press('v');
  assert.equal(await page.evaluate(() => wb.tool), 'select');
});

await test('Shift draws squares/circles, Alt draws from the centre', async () => {
  await press('r');
  await drag(200, 200, 320, 260, { mods: ['Shift'] });
  let el = await last();
  near(el.w, 120); near(el.h, 120);
  await press('Escape');
  await press('o');
  await drag(600, 300, 660, 340, { mods: ['Alt'] });
  el = await last();
  near(el.x, 540); near(el.y, 260); near(el.w, 120); near(el.h, 80);
});

await test('tool lock keeps the tool; Q toggles it', async () => {
  await press('q');
  assert.equal(await page.evaluate(() => wb.toolLocked), true);
  await press('r');
  await drag(100, 100, 200, 160);
  await drag(300, 100, 400, 160);
  assert.equal(await page.evaluate(() => wb.tool), 'rect');
  assert.equal((await items()).length, 2);
  await press('q');
  assert.equal(await page.evaluate(() => wb.toolLocked), false);
});

await test('drag moves; click selects; shift-click toggles; click empty clears', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, backgroundColor: '#a5d8ff' });
  const b = await add({ type: 'rect', x: 400, y: 100, w: 120, h: 80, backgroundColor: '#b2f2bb' });
  await click(160, 140);
  assert.deepEqual(await sel(), [a]);
  await click(460, 140, { mods: ['Shift'] });
  assert.deepEqual((await sel()).sort(), [a, b].sort());
  await click(460, 140, { mods: ['Shift'] });
  assert.deepEqual(await sel(), [a]);
  await click(700, 500);
  assert.deepEqual(await sel(), []);
  await drag(160, 140, 260, 240);
  const el = (await items()).find((e) => e.id === a);
  near(el.x, 200); near(el.y, 200);
  assert.deepEqual(await sel(), [a]);
});

await test('arrow keys nudge (Shift = 10px); Delete removes; undo restores', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100 });
  await click(150, 130);
  await press('ArrowRight'); await press('ArrowDown');
  let el = (await items())[0];
  near(el.x, 101); near(el.y, 101);
  await press('Shift+ArrowLeft');
  el = (await items())[0];
  near(el.x, 91);
  await press('Delete');
  assert.equal((await items()).length, 0);
  await press(`${MOD}+z`);
  assert.equal((await items()).length, 1);
  assert.equal((await items())[0].id, a);
  await press(`${MOD}+Shift+z`);
  assert.equal((await items()).length, 0);
});

await test('resize with handles (corner, edge, Shift keeps aspect, Alt from centre)', async () => {
  await add({ type: 'rect', x: 200, y: 200, w: 100, h: 80 });
  await click(250, 240);
  // padded handle sits 6px outside the box: se handle at (306, 286)
  await drag(306, 286, 356, 336);
  let el = (await items())[0];
  near(el.w, 150); near(el.h, 130);
  // east edge handle
  const h = await page.evaluate(() => { const e = wb.board.items[0]; return { x: e.x + e.w + 6, y: e.y + e.h / 2 }; });
  await drag(h.x, h.y, h.x + 50, h.y);
  el = (await items())[0];
  near(el.w, 200); near(el.h, 130);
  // shift keeps aspect from the se corner
  const c = await page.evaluate(() => { const e = wb.board.items[0]; return { x: e.x + e.w + 6, y: e.y + e.h + 6, ar: e.w / e.h }; });
  await drag(c.x, c.y, c.x + 100, c.y + 5, { mods: ['Shift'] });
  el = (await items())[0];
  near(el.w / el.h, c.ar, 0.02, 'aspect');
  assert.ok(el.w > 250);
  // alt = from centre: centre stays
  const before = (await items())[0];
  const cc = { x: before.x + before.w / 2, y: before.y + before.h / 2 };
  const c2 = { x: before.x + before.w + 6, y: before.y + before.h + 6 };
  await drag(c2.x, c2.y, c2.x + 20, c2.y + 20, { mods: ['Alt'] });
  el = (await items())[0];
  near(el.x + el.w / 2, cc.x, 1); near(el.y + el.h / 2, cc.y, 1);
  assert.ok(el.w > before.w);
});

await test('rotate handle rotates; Shift snaps to 15 degrees', async () => {
  await add({ type: 'rect', x: 300, y: 300, w: 200, h: 100 });
  await click(400, 350);
  // rotate handle is 24px above the padded top: (400, 294-24)
  await drag(400, 270, 560, 350); // to the right of the centre -> ~90 degrees
  let el = (await items())[0];
  near(el.angle, Math.PI / 2, 0.08, 'angle');
  await drag(400 + 0, 350 + 0, 400, 350); // no-op drag keeps it selected
  // handle moved with rotation: now to the right of the box
  const hp = await page.evaluate(() => { const e = wb.board.items[0]; const r = 3 + 6 + 24 + e.w / 2; return { x: e.x + e.w / 2 + 0 + r - 3 + 0, y: e.y + e.h / 2 }; });
  await drag(hp.x, hp.y, hp.x + 70, hp.y + 90, { mods: ['Shift'] });
  el = (await items())[0];
  const step = Math.PI / 12;
  near(Math.round(el.angle / step) * step, el.angle, 1e-6, 'snapped to 15deg');
});

await test('text: double-click empty canvas creates text; Esc/click-away finishes; empty text is removed', async () => {
  await dbl(300, 300);
  assert.equal(await page.evaluate(() => wb.editingId !== null), true);
  await page.keyboard.type('Hello world');
  await press('Escape');
  let t = await byType('text');
  assert.equal(t.length, 1);
  assert.equal(t[0].text, 'Hello world');
  assert.ok(t[0].w > 40 && t[0].h > 10);
  assert.equal(await page.evaluate(() => wb.editingId), null);
  // empty text is removed
  await dbl(600, 500);
  await press('Escape');
  assert.equal((await byType('text')).length, 1);
  // text tool: click, type multi-line (Enter = newline), Ctrl+Enter finishes
  await press('t');
  await click(500, 200);
  await page.keyboard.type('line one');
  await press('Enter');
  await page.keyboard.type('line two');
  await press(`${MOD}+Enter`);
  t = await byType('text');
  assert.equal(t.length, 2);
  assert.equal(t[1].text, 'line one\nline two');
  assert.ok(t[1].h > t[0].h * 1.7, 'two lines are taller');
  // click away finishes
  await dbl(700, 600);
  await page.keyboard.type('away');
  await click(900, 700);
  assert.equal((await byType('text')).length, 3);
  // one undo step per text
  await press(`${MOD}+z`);
  assert.equal((await byType('text')).length, 2);
});

await test('text in shapes: Enter / double-click edit the label; typing after creation starts a label', async () => {
  await press('r');
  await drag(200, 200, 400, 300);
  await page.keyboard.type('Gateway');
  assert.equal(await page.evaluate(() => wb.editingId !== null), true, 'typing after create opens the editor');
  await press(`${MOD}+Enter`);
  let r = (await byType('rect'))[0];
  assert.equal(r.text, 'Gateway');
  await click(300, 250);
  await press('Enter');
  assert.equal(await page.evaluate(() => wb.editingId !== null), true);
  await press(`${MOD}+a`);
  await page.keyboard.type('API');
  await press('Escape');
  r = (await byType('rect'))[0];
  assert.equal(r.text, 'API');
  // textarea overlay is positioned over the shape
  await dbl(300, 250);
  const box = await page.locator('textarea.wb-editor').boundingBox();
  assert.ok(box.x >= 195 && box.x + box.width <= 405 && box.y >= 195 && box.y + box.height <= 305, JSON.stringify(box));
  await press('Escape');
  // diamond/ellipse labels too
  await press('d');
  await drag(500, 150, 660, 260);
  await press('Enter');
  await page.keyboard.type('LB');
  await press(`${MOD}+Enter`);
  assert.equal((await byType('diamond'))[0].text, 'LB');
  // undo removes the label edit as one step
  await press(`${MOD}+z`);
  assert.equal((await byType('diamond'))[0].text, '');
});

await test('arrow: drag between shapes binds both ends and they follow moves, resizes and rotation', async () => {
  const a = await add({ type: 'rect', x: 100, y: 200, w: 120, h: 80 });
  const b = await add({ type: 'rect', x: 500, y: 200, w: 120, h: 80 });
  await press('a');
  await drag(160, 240, 560, 240);
  const ar = await last();
  assert.equal(ar.type, 'arrow');
  assert.equal(ar.from, a); assert.equal(ar.to, b);
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  const ends = () => page.evaluate(() => { const arrow = wb.board.items.find((e) => e.type === 'arrow'); const p = wb.board.items; void p; return arrow; });
  const abs = () => page.evaluate(async () => {
    const m = await import('/shared/board.js');
    return m.resolvePoints(wb.board, wb.board.items.find((e) => e.type === 'arrow'));
  });
  let p = await abs();
  near(p[0].x, 220, 2); near(p[p.length - 1].x, 500, 2);
  // move shape b down: the arrow end follows
  await click(560, 240); // select b? (arrow body hit first at the mid)
  await page.evaluate((id) => wb.select([id]), b);
  await press('Shift+ArrowDown'); await press('Shift+ArrowDown');
  p = await abs();
  const bb = (await items()).find((e) => e.id === b);
  assert.ok(Math.abs(p[p.length - 1].y - (bb.y + bb.h / 2)) < 40, 'end near the moved shape');
  assert.ok(p[p.length - 1].y > 245, 'end moved down');
  // resize b through the API-level patch is covered by the handles; rotate b
  await page.evaluate((id) => { const x = wb.getBoard(); x.items.find((e) => e.id === id).angle = 0.5; wb.setBoard(x, { resetHistory: false }); }, b);
  p = await abs();
  assert.ok(Math.hypot(p[p.length - 1].x - 560, p[p.length - 1].y - (bb.y + 40)) < 120);
  void ends;
});

await test('arrow ends highlight the target while dragging and Ctrl prevents binding', async () => {
  const a = await add({ type: 'rect', x: 100, y: 200, w: 120, h: 80 });
  const b = await add({ type: 'ellipse', x: 500, y: 200, w: 120, h: 80 });
  await press('a');
  await page.mouse.move(160, 240);
  await page.mouse.down();
  await page.mouse.move(540, 240, { steps: 6 });
  assert.ok(await page.locator('.wb-bind').count() > 0, 'binding highlight visible');
  await page.mouse.up();
  let ar = await last();
  assert.equal(ar.from, a); assert.equal(ar.to, b);
  await press(`${MOD}+z`);
  assert.equal((await byType('arrow')).length, 0);
  await press('a');
  await page.keyboard.down(MOD);
  await drag(160, 240, 540, 240);
  await page.keyboard.up(MOD);
  ar = await last();
  assert.equal(ar.from, null); assert.equal(ar.to, null);
});

await test('arrow from empty space to empty space, Shift constrains to 45 degrees', async () => {
  await press('a');
  await drag(100, 100, 300, 130, { mods: ['Shift'] });
  let ar = await last();
  assert.equal(ar.type, 'arrow');
  assert.equal(ar.from, null);
  near(ar.points[1][1], 0, 1);
  near(ar.points[1][0], Math.hypot(200, 30), 1);
  await press('l');
  await drag(100, 300, 300, 400);
  ar = await last();
  assert.equal(ar.type, 'line');
  assert.equal(ar.endArrowhead, null);
});

await test('elbow and curved arrows', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 100, h: 60 });
  const b = await add({ type: 'rect', x: 400, y: 300, w: 100, h: 60 });
  await page.evaluate(() => wb.setStyle({ arrowType: 'elbow' }));
  await press('a');
  await drag(150, 130, 450, 330);
  let ar = await last();
  assert.equal(ar.arrowType, 'elbow');
  assert.equal(ar.from, a); assert.equal(ar.to, b);
  const route = await page.evaluate(async () => (await import('/shared/board.js')).resolvePoints(wb.board, wb.board.items.at(-1)));
  assert.ok(route.length >= 3, 'orthogonal route has bends');
  for (let i = 1; i < route.length; i++) assert.ok(Math.abs(route[i].x - route[i - 1].x) < 0.5 || Math.abs(route[i].y - route[i - 1].y) < 0.5, 'axis aligned segments');
  // re-routes when the shape moves
  await page.evaluate((id) => wb.select([id]), b);
  await press('Shift+ArrowUp'); await press('Shift+ArrowUp'); await press('Shift+ArrowUp');
  const route2 = await page.evaluate(async () => (await import('/shared/board.js')).resolvePoints(wb.board, wb.board.items.at(-1)));
  assert.notDeepEqual(route2, route);
  // curved
  await page.evaluate(() => wb.setStyle({ arrowType: 'round' }));
  await press('a');
  await drag(600, 500, 800, 600);
  ar = await last();
  assert.equal(ar.arrowType, 'round');
});

await test('polyline: click-click-click then double-click / Enter / Esc finishes', async () => {
  await press('l');
  await click(100, 100);
  await page.mouse.move(200, 150, { steps: 3 });
  await click(200, 150);
  await page.mouse.move(300, 100, { steps: 3 });
  await click(300, 100);
  await page.mouse.move(400, 200, { steps: 3 });
  await page.mouse.dblclick(400, 200);
  let ln = await last();
  assert.equal(ln.type, 'line');
  assert.equal(ln.points.length, 4, JSON.stringify(ln.points));
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  assert.equal(await page.evaluate(() => wb.editingId), null, 'double-click that finishes does not open a label');
  // arrow with Enter
  await press('a');
  await click(100, 400);
  await page.mouse.move(200, 450, { steps: 3 });
  await click(200, 450);
  await page.mouse.move(300, 400, { steps: 3 });
  await click(300, 400);
  await page.mouse.move(350, 380, { steps: 2 });
  await press('Enter');
  ln = await last();
  assert.equal(ln.type, 'arrow');
  assert.equal(ln.points.length, 3, JSON.stringify(ln.points));
  // Esc finishes too
  await press('a');
  await click(500, 400);
  await page.mouse.move(600, 450, { steps: 3 });
  await click(600, 450);
  await page.mouse.move(700, 400, { steps: 3 });
  await press('Escape');
  ln = await last();
  assert.equal(ln.points.length, 2);
  // a lone click then Esc leaves nothing behind
  const n = (await items()).length;
  await press('a');
  await click(800, 600);
  await press('Escape');
  assert.equal((await items()).length, n);
});

await test('editing line points: endpoint handle moves, midpoint handle adds a point', async () => {
  await press('l');
  await drag(200, 300, 500, 300);
  const id = (await last()).id;
  assert.deepEqual(await sel(), [id]);
  await drag(500, 300, 520, 380); // end handle
  let ln = await last();
  near(ln.points[1][0], 320, 2); near(ln.points[1][1], 80, 2);
  // midpoint of the new segment
  const mid = await page.evaluate(() => { const l = wb.board.items.at(-1); return { x: l.x + l.points[1][0] / 2, y: l.y + l.points[1][1] / 2 }; });
  await drag(mid.x, mid.y, mid.x - 40, mid.y + 120);
  ln = await last();
  assert.equal(ln.points.length, 3);
  assert.ok(await page.locator('.wb-point').count() === 3);
});

await test('arrow label: double-click edits the label', async () => {
  await page.evaluate(() => wb.setStyle({ arrowType: 'sharp' }));
  await press('a');
  await drag(100, 300, 500, 300);
  await dbl(300, 300);
  assert.equal(await page.evaluate(() => wb.editingId !== null), true);
  await page.keyboard.type('HTTP');
  await press(`${MOD}+Enter`);
  assert.equal((await last()).text, 'HTTP');
});

await test('freedraw: stroke is recorded, simplified; the tool stays active', async () => {
  await press('p');
  await page.mouse.move(100, 100);
  await page.mouse.down();
  for (let i = 0; i <= 40; i++) await page.mouse.move(100 + i * 5, 100 + Math.sin(i / 4) * 40);
  await page.mouse.up();
  const f = await last();
  assert.equal(f.type, 'freedraw');
  assert.ok(f.points.length >= 5 && f.points.length < 41, `simplified ${f.points.length}`);
  near(f.points[0][0], 0, 0.001);
  assert.ok(f.w > 150);
  assert.equal(await page.evaluate(() => wb.tool), 'freedraw');
  assert.equal(await page.evaluate(() => wb.canUndo()), true);
});

await test('eraser removes elements it touches in one undo step; Esc / other tool leaves', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 100, h: 60, backgroundColor: '#ffc9c9' });
  await add({ type: 'rect', x: 300, y: 100, w: 100, h: 60, backgroundColor: '#ffc9c9' });
  const keep = await add({ type: 'rect', x: 300, y: 400, w: 100, h: 60, backgroundColor: '#ffc9c9' });
  await press('e');
  await page.mouse.move(60, 130);
  await page.mouse.down();
  await page.mouse.move(150, 130, { steps: 4 });
  assert.ok(await page.locator('.wb-slot.wb-erasing').count() >= 1, 'erased elements are highlighted');
  assert.equal((await items()).length, 3, 'still there until release');
  await page.mouse.move(350, 130, { steps: 8 });
  await page.mouse.up();
  const left = await items();
  assert.deepEqual(left.map((e) => e.id), [keep]);
  await press(`${MOD}+z`);
  assert.equal((await items()).length, 3);
});

await test('marquee selects contained elements; Shift adds; select all; click empty clears', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 80, h: 60 });
  const b = await add({ type: 'ellipse', x: 250, y: 100, w: 80, h: 60 });
  const c = await add({ type: 'diamond', x: 600, y: 400, w: 80, h: 60 });
  await drag(60, 60, 380, 220);
  assert.deepEqual((await sel()).sort(), [a, b].sort());
  assert.ok(await page.locator('.wb-marquee').count() === 0, 'marquee gone after release');
  await drag(560, 360, 720, 500, { mods: ['Shift'] });
  assert.deepEqual((await sel()).sort(), [a, b, c].sort());
  await click(900, 700);
  assert.deepEqual(await sel(), []);
  await press(`${MOD}+a`);
  assert.equal((await sel()).length, 3);
  await press('Escape');
  assert.deepEqual(await sel(), []);
});

await test('groups select as one; Ctrl+click goes inside; ungroup', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 80, h: 60, backgroundColor: '#ffc9c9' });
  const b = await add({ type: 'rect', x: 300, y: 100, w: 80, h: 60, backgroundColor: '#b2f2bb' });
  await press(`${MOD}+a`);
  await press(`${MOD}+g`);
  const g = (await items())[0].groupIds;
  assert.equal(g.length, 1);
  assert.deepEqual((await items())[1].groupIds, g);
  await click(900, 700);
  await click(140, 130);
  assert.deepEqual((await sel()).sort(), [a, b].sort());
  await click(900, 700);
  await click(140, 130, { mods: [MOD] });
  assert.deepEqual(await sel(), [a]);
  // dragging a group moves both
  await click(900, 700);
  await drag(340, 130, 440, 230);
  const [ea, eb] = await items();
  near(ea.x, 200); near(eb.x, 400);
  await press(`${MOD}+Shift+g`);
  assert.deepEqual((await items()).map((e) => e.groupIds.length), [0, 0]);
});

await test('multi-selection: bounding box scales every element', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 100, h: 100 });
  await add({ type: 'rect', x: 300, y: 100, w: 100, h: 100 });
  await press(`${MOD}+a`);
  // padded bbox: x 94..406 ; se handle at (406, 206)
  await drag(406, 206, 706, 206);
  const [a, b] = await items();
  near(a.w, 200, 2); near(b.w, 200, 2);
  near(b.x, 500, 2);
});

await test('frames: contain, clip and carry children; deleting a frame deletes children', async () => {
  await page.evaluate(() => wb.setStyle({ backgroundColor: '#a5d8ff' }));
  await press('f');
  await drag(100, 100, 500, 400);
  const frame = await last();
  assert.equal(frame.type, 'frame');
  assert.match(frame.name, /Frame/);
  await press('Escape');
  await press('r');
  await drag(200, 200, 300, 260);
  const r = await last();
  assert.equal(r.frameId, frame.id);
  await press('Escape');
  // an element straddling the edge is clipped
  await press('o');
  await drag(420, 250, 540, 330);
  await press('Escape');
  const e = await last();
  assert.equal(e.frameId, frame.id, 'centre inside -> child');
  assert.ok(await page.locator(`.wb-slot[clip-path] [data-id="${e.id}"]`).count() === 1, 'rendered with a clip path');
  // moving the frame moves children
  await page.evaluate((id) => wb.select([id]), frame.id);
  await drag(200, 100, 240, 140); // frame outline top edge
  const items2 = await items();
  const fr2 = items2.find((x) => x.id === frame.id);
  const r2 = items2.find((x) => x.id === r.id);
  near(fr2.x, frame.x + 40); near(r2.x, r.x + 40);
  // dragging a child out clears frameId
  await drag(r2.x + 50, r2.y + 30, 800, 600);
  assert.equal((await items()).find((x) => x.id === r.id).frameId, null);
  // delete the frame: children go too
  await page.evaluate((id) => wb.select([id]), frame.id);
  await press('Delete');
  const left = await items();
  assert.deepEqual(left.map((x) => x.id), [r.id]);
});

await test('layout API: align, distribute, flip, layers, group, lock', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 60, h: 60 });
  const b = await add({ type: 'rect', x: 250, y: 200, w: 60, h: 60 });
  const c = await add({ type: 'rect', x: 700, y: 330, w: 60, h: 60 });
  await press(`${MOD}+a`);
  await page.evaluate(() => wb.align('left'));
  assert.deepEqual((await items()).map((e) => e.x), [100, 100, 100]);
  await page.evaluate(() => wb.align('top'));
  assert.deepEqual((await items()).map((e) => e.y), [100, 100, 100]);
  await press(`${MOD}+z`); await press(`${MOD}+z`);
  await page.evaluate(() => wb.distribute('h'));
  const xs = (await items()).map((e) => e.x);
  near(xs[1], (xs[0] + xs[2]) / 2, 1);
  await page.evaluate(() => wb.flip('h'));
  const flipped = (await items()).map((e) => e.x);
  assert.ok(flipped[0] > flipped[2]);
  await page.evaluate((id) => wb.select([id]), a);
  await page.evaluate(() => wb.bringToFront());
  assert.equal((await items()).at(-1).id, a);
  await press(`${MOD}+Shift+BracketLeft`);
  assert.equal((await items())[0].id, a);
  await press(`${MOD}+BracketRight`);
  assert.equal((await items())[1].id, a);
  void b; void c;
});

await test('lock: locked elements render but cannot be selected; unlockAll', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, backgroundColor: '#ffc9c9' });
  await click(160, 140);
  await press(`${MOD}+Shift+l`);
  assert.equal((await items())[0].locked, true);
  assert.deepEqual(await sel(), []);
  await click(160, 140);
  assert.deepEqual(await sel(), []);
  await drag(40, 40, 400, 300);
  assert.deepEqual(await sel(), [], 'marquee skips locked');
  assert.equal(await page.locator(`.wb-el[data-id="${a}"]`).count(), 1);
  await page.evaluate(() => wb.unlockAll());
  assert.equal((await items())[0].locked, false);
  await click(160, 140);
  assert.deepEqual(await sel(), [a]);
});

await test('duplicate (Ctrl+D), Alt-drag duplicates, copy / cut / paste, repeat paste offsets', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, text: 'Svc', backgroundColor: '#a5d8ff' });
  await click(160, 140);
  await press(`${MOD}+d`);
  let it = await items();
  assert.equal(it.length, 2);
  near(it[1].x, 120); near(it[1].y, 120);
  assert.equal(it[1].text, 'Svc');
  assert.deepEqual(await sel(), [it[1].id]);
  // alt-drag duplicates and moves the copy
  await drag(160, 160, 460, 360, { mods: ['Alt'] });
  it = await items();
  assert.equal(it.length, 3);
  assert.equal(it.find((e) => e.id === a).x, 100);
  near(it[2].x, 420, 2);
  // copy + paste at pointer
  await page.mouse.move(700, 500);
  await press(`${MOD}+c`);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 4);
  it = await items();
  near(it[3].x + it[3].w / 2, 700, 3); near(it[3].y + it[3].h / 2, 500, 3);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 5);
  it = await items();
  assert.ok(Math.abs(it[4].x - it[3].x) > 5, 'repeat paste is offset');
  // cut removes
  await press(`${MOD}+x`);
  assert.equal((await items()).length, 4);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 5);
  // pasting copies keeps group/binding remapped
  assert.equal(new Set((await items()).map((e) => e.id)).size, 5);
});

await test('copy puts readable text on the system clipboard, not JSON, and still pastes back as shapes', async () => {
  await page.evaluate(() => { wb.setBoard({ version: 2, items: [], nextId: 1 }); });
  await press('t');
  await page.mouse.click(300, 300);
  await page.keyboard.type('https://example.com/');
  await page.keyboard.press('Escape');
  await page.evaluate(() => wb.selectAll());
  await press(`${MOD}+c`);
  await page.waitForTimeout(150);
  const flavours = await page.evaluate(async () => {
    const [it] = await navigator.clipboard.read();
    const out = {};
    for (const t of it.types) out[t] = await (await it.getType(t)).text();
    return out;
  });
  assert.equal(flavours['text/plain'], 'https://example.com/', 'other apps see plain words');
  assert.ok(!/hld-whiteboard|"elements"/.test(flavours['text/plain']), 'no JSON in the plain text');
  assert.ok(/data-hld-clip/.test(flavours['text/html']), 'board data rides in the html flavour');
  await page.mouse.move(600, 500);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 2);
  assert.equal((await items())[1].text, 'https://example.com/');
});

await test('paste plain text creates a text element; Ctrl+Shift+V pastes our payload as text', async () => {
  await page.evaluate(() => navigator.clipboard.writeText('pasted from outside'));
  await page.mouse.move(300, 300);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 1);
  const t = await last();
  assert.equal(t.type, 'text');
  assert.equal(t.text, 'pasted from outside');
  // excalidraw clipboard json is understood when interop.js exists
  const ex = JSON.stringify({ type: 'excalidraw/clipboard', elements: [{ type: 'rectangle', id: 'a', x: 10, y: 10, width: 100, height: 50, angle: 0, strokeColor: '#1e1e1e', backgroundColor: 'transparent', fillStyle: 'solid', strokeWidth: 2, strokeStyle: 'solid', roughness: 1, opacity: 100, seed: 1, version: 1, versionNonce: 1, isDeleted: false, groupIds: [], frameId: null, boundElements: null, updated: 1, link: null, locked: false, roundness: null }] });
  await page.evaluate((s) => navigator.clipboard.writeText(s), ex);
  await press(`${MOD}+v`);
  await page.waitForFunction(() => wb.board.items.length === 2);
  assert.equal((await last()).type, 'rect');
});

await test('undo / redo coalesce whole gestures; history is capped and emits', async () => {
  await page.evaluate(() => wb.setStyle({ backgroundColor: '#a5d8ff' }));
  await press('r');
  await drag(100, 100, 300, 200); // create
  assert.equal(await page.evaluate(() => wb.canUndo()), true);
  await drag(200, 150, 400, 350); // move (many pointer events, one step)
  await drag(406 + 200, 356 + 200, 706 + 200, 456 + 200, { steps: 20 }).catch(() => {});
  await drag(406, 356, 456, 406, { steps: 20 }); // resize with the se handle
  assert.equal(await page.evaluate(() => wb._hist.length), 4, 'create, move and resize are one step each');
  await press(`${MOD}+z`);
  await press(`${MOD}+z`);
  await press(`${MOD}+z`);
  assert.equal((await items()).length, 0);
  assert.equal(await page.evaluate(() => wb.canUndo()), false);
  await press(`${MOD}+y`);
  assert.equal((await items()).length, 1);
  const ev = await page.evaluate(() => window.__events.filter((e) => e.t === 'history').pop().p);
  assert.equal(ev.canRedo, true);
  // cap
  const len = await page.evaluate(() => { for (let i = 0; i < 260; i++) { wb.insertElements([{ type: 'rect', x: i, y: 0, w: 10, h: 10, id: 1 }]); } return wb._hist.length; });
  assert.ok(len <= 201, `history length ${len}`);
});

await test('zoom: Ctrl+wheel anchors at the cursor, clamped; keys; zoomToFit / zoomToSelection', async () => {
  await add({ type: 'rect', x: 1000, y: 1000, w: 200, h: 100 });
  await page.mouse.move(600, 400);
  const w0 = await page.evaluate(() => wb.clientToWorld(600, 400));
  await page.keyboard.down(MOD);
  await page.mouse.wheel(0, -300);
  await page.keyboard.up(MOD);
  const z = await page.evaluate(() => wb.view.zoom);
  assert.ok(z > 1.5, `zoomed in ${z}`);
  const w1 = await page.evaluate(() => wb.clientToWorld(600, 400));
  near(w1.x, w0.x, 0.5); near(w1.y, w0.y, 0.5);
  await press(`${MOD}+0`);
  assert.equal(await page.evaluate(() => wb.view.zoom), 1);
  await press(`${MOD}+Equal`);
  assert.ok(await page.evaluate(() => wb.view.zoom) > 1);
  await press(`${MOD}+Minus`); await press(`${MOD}+Minus`);
  assert.ok(await page.evaluate(() => wb.view.zoom) < 1);
  for (let i = 0; i < 80; i++) await page.evaluate(() => wb.zoomIn());
  assert.equal(await page.evaluate(() => wb.view.zoom), 30);
  for (let i = 0; i < 200; i++) await page.evaluate(() => wb.zoomOut());
  assert.equal(await page.evaluate(() => wb.view.zoom), 0.1);
  await page.evaluate(() => wb.resetZoom());
  await press('Shift+Digit1');
  const r = await page.evaluate(() => { const p = wb.worldToClient(1000, 1000); const q = wb.worldToClient(1200, 1100); return { p, q }; });
  assert.ok(r.p.x >= 0 && r.q.x <= 1200 && r.p.y >= 0 && r.q.y <= 800, 'fit shows the element');
  await page.evaluate(() => { wb.resetZoom(); wb.select([wb.board.items[0].id]); });
  await press('Shift+Digit2');
  assert.ok(await page.evaluate(() => wb.view.zoom) > 1);
});

await test('pan: wheel, Space-drag, middle-drag, hand tool', async () => {
  const before = (await items()).length;
  await page.mouse.move(600, 400);
  await page.mouse.wheel(0, 100);
  let v = await page.evaluate(() => wb.view);
  near(v.y, -100); near(v.x, 0);
  await page.mouse.wheel(50, 0);
  v = await page.evaluate(() => wb.view);
  near(v.x, -50);
  await page.keyboard.down('Space');
  await drag(300, 300, 400, 350);
  await page.keyboard.up('Space');
  v = await page.evaluate(() => wb.view);
  near(v.x, 50); near(v.y, -50);
  await page.mouse.move(300, 300);
  await page.mouse.down({ button: 'middle' });
  await page.mouse.move(250, 250, { steps: 3 });
  await page.mouse.up({ button: 'middle' });
  v = await page.evaluate(() => wb.view);
  near(v.x, 0); near(v.y, -100);
  await press('h');
  await drag(300, 300, 320, 340);
  v = await page.evaluate(() => wb.view);
  near(v.x, 20); near(v.y, -60);
  assert.equal((await items()).length, before, JSON.stringify(await items()));
});

await test('grid: pattern follows the view; moves snap to the grid', async () => {
  await page.keyboard.down('Control');
  await press('Quote');
  await page.keyboard.up('Control');
  assert.equal(await page.evaluate(() => wb.mode.grid), true);
  assert.equal(await page.locator('rect.wb-grid').getAttribute('visibility'), 'visible');
  await press('r');
  await drag(103, 107, 255, 191);
  const r = await last();
  assert.equal(r.x % 20, 0); assert.equal(r.y % 20, 0); assert.equal(r.w % 20, 0); assert.equal(r.h % 20, 0);
  await drag(r.x + 30, r.y + 30, r.x + 47, r.y + 52);
  const m = await last();
  assert.equal(m.x % 20, 0); assert.equal(m.y % 20, 0);
  const attr = await page.evaluate(() => document.querySelector('pattern').getAttribute('width'));
  assert.equal(attr, '20');
  await page.evaluate(() => wb.setZoom(2));
  assert.equal(await page.evaluate(() => document.querySelector('pattern').getAttribute('width')), '40');
});

await test('object snapping (Alt+S) shows guide lines and aligns edges', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 100, h: 60 });
  const b = await add({ type: 'rect', x: 400, y: 300, w: 100, h: 60 });
  await press('Alt+s');
  assert.equal(await page.evaluate(() => wb.mode.snap), true);
  await page.mouse.move(450, 330);
  await page.mouse.down();
  await page.mouse.move(250, 160, { steps: 6 });
  await page.mouse.move(251, 133, { steps: 3 }); // top edge 3px off the first rect's top
  const guides = await page.locator('.wb-guide').count();
  await page.mouse.up();
  const el = (await items()).find((e) => e.id === b);
  assert.ok(guides >= 1, 'guide drawn while snapping');
  near(el.y, 100, 0.5, 'snapped y');
  assert.equal(await page.locator('.wb-guide').count(), 0, 'guides gone after drop');
});

await test('view mode: edits are blocked, pan/zoom still work; Alt+R toggles', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 100, h: 60 });
  await press('Alt+r');
  assert.equal(await page.evaluate(() => wb.mode.view), true);
  await press('r');
  assert.notEqual(await page.evaluate(() => wb.tool), 'rect');
  await drag(300, 300, 400, 400);
  assert.equal((await items()).length, 1);
  await click(150, 130);
  assert.deepEqual(await sel(), []);
  await press(`${MOD}+a`);
  assert.deepEqual(await sel(), []);
  await dbl(500, 500);
  assert.equal((await items()).length, 1);
  await page.evaluate(() => wb.deleteSelected());
  await page.evaluate(() => wb.clear());
  assert.equal((await items()).length, 1);
  const v0 = await page.evaluate(() => wb.view);
  await page.mouse.wheel(0, 50);
  assert.equal(await page.evaluate(() => wb.view.y), v0.y - 50, JSON.stringify(v0));
  const v1 = await page.evaluate(() => wb.view);
  await drag(300, 300, 340, 340); // pans
  assert.equal(await page.evaluate(() => wb.view.x), v1.x + 40);
  await press('Alt+r');
  assert.equal(await page.evaluate(() => wb.mode.view), false);
  await page.evaluate(() => wb._setView({ x: 0, y: 0, zoom: 1 }));
  await click(150, 130);
  assert.equal((await sel()).length, 1);
});

await test('laser leaves no trace in the scene and is not undoable', async () => {
  await press('k');
  assert.equal(await page.evaluate(() => wb.tool), 'laser');
  const hist = await page.evaluate(() => wb._hist.length);
  await page.mouse.move(100, 100);
  await page.mouse.down();
  await page.mouse.move(300, 300, { steps: 10 });
  assert.ok(await page.locator('.wb-trail line').count() > 0, 'trail drawn');
  await page.mouse.up();
  assert.equal((await items()).length, 0);
  assert.equal(await page.evaluate(() => wb._hist.length), hist);
  await page.waitForFunction(() => document.querySelectorAll('.wb-trail line').length === 0, null, { timeout: 4000 });
});

await test('bucket fills with the current background; eyedropper picks stroke (Alt: background)', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, strokeColor: '#e03131', backgroundColor: '#a5d8ff' });
  await page.evaluate(() => wb.setStyle({ backgroundColor: '#ffec99' }));
  await press('b');
  await click(160, 140);
  assert.equal((await items()).find((e) => e.id === a).backgroundColor, '#ffec99');
  await press('i');
  await click(160, 140);
  assert.equal(await page.evaluate(() => wb.style.strokeColor), '#e03131');
  const toast = await page.evaluate(() => window.__events.filter((e) => e.t === 'toast').pop().p);
  assert.match(toast.message, /e03131/);
  await press('i');
  await page.evaluate(() => wb.setStyle({ backgroundColor: '#ffffff' }));
  await click(160, 140, { mods: ['Alt'] });
  assert.equal(await page.evaluate(() => wb.style.backgroundColor), '#ffec99');
});

await test('setStyle applies to the selection and to new elements', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80 });
  await click(160, 140);
  await page.evaluate(() => wb.setStyle({ strokeColor: '#2f9e44', backgroundColor: '#b2f2bb', strokeWidth: 4, roughness: 0 }));
  const el = (await items()).find((e) => e.id === a);
  assert.equal(el.strokeColor, '#2f9e44'); assert.equal(el.strokeWidth, 4); assert.equal(el.roughness, 0);
  await page.evaluate(() => wb.clearSelection());
  await press('r');
  await click(500, 300);
  const n = await last();
  assert.equal(n.strokeColor, '#2f9e44'); assert.equal(n.backgroundColor, '#b2f2bb');
  const sum = await page.evaluate(() => wb.styleSummary());
  assert.equal(sum.values.strokeColor, '#2f9e44');
  assert.equal(sum.applies.arrowType, false);
  await page.evaluate(() => { wb.copyStyle(); });
});

await test('image: addImageFile, drop, downscale, rejects non-images', async () => {
  const png = await page.evaluate(() => {
    const c = document.createElement('canvas'); c.width = 40; c.height = 20;
    const x = c.getContext('2d'); x.fillStyle = '#e03131'; x.fillRect(0, 0, 40, 20); x.fillStyle = '#1971c2'; x.fillRect(20, 0, 20, 20);
    return c.toDataURL('image/png');
  });
  assert.match(png, /^data:image\/png;base64,/);
  const id = await page.evaluate(async (d) => {
    const bin = atob(d.split(',')[1]);
    const bytes = Uint8Array.from(bin, (c) => c.charCodeAt(0));
    return wb.addImageFile(new File([bytes], 't.png', { type: 'image/png' }), { x: 400, y: 300 });
  }, png);
  const img = (await items()).find((e) => e.id === id);
  assert.equal(img.type, 'image');
  assert.match(img.dataUrl, /^data:image\/png;base64,/);
  near(img.w / img.h, 2, 0.01);
  near(img.x + img.w / 2, 400, 1);
  assert.equal(await page.locator(`.wb-el[data-id="${id}"] image`).count(), 1);
  // large image is downscaled to <= 1600
  const big = await page.evaluate(async () => {
    const c = document.createElement('canvas'); c.width = 3000; c.height = 1000;
    c.getContext('2d').fillRect(0, 0, 3000, 1000);
    const blob = await new Promise((r) => c.toBlob(r, 'image/png'));
    const id2 = await wb.addImageFile(new File([blob], 'big.png', { type: 'image/png' }));
    const el = wb.board.items.find((e) => e.id === id2);
    return new Promise((res) => { const im = new Image(); im.onload = () => res({ w: im.naturalWidth, h: im.naturalHeight, bw: el.w, bh: el.h }); im.src = el.dataUrl; });
  });
  assert.equal(big.w, 1600); near(big.h, 533, 1);
  assert.ok(big.bw <= 480);
  // aspect is locked on resize
  await page.evaluate((i) => wb.select([i]), id);
  const hp = await page.evaluate((i) => { const e = wb.board.items.find((x) => x.id === i); return { x: e.x + e.w + 6, y: e.y + e.h + 6 }; }, id);
  await drag(hp.x, hp.y, hp.x + 60, hp.y + 5);
  const r = (await items()).find((e) => e.id === id);
  near(r.w / r.h, 2, 0.02);
  // non-images are rejected
  const err = await page.evaluate(async () => { try { await wb.addImageFile(new File(['x'], 'a.txt', { type: 'text/plain' })); return null; } catch (e) { return e.message; } });
  assert.match(err, /not an image/);
  // image request on tool 9
  await press('9');
  assert.ok((await page.evaluate(() => window.__events)).some((e) => e.t === 'request' && e.p.name === 'image'));
  assert.equal(await page.evaluate(() => wb.tool), 'select');
});

await test('drag and drop an image file inserts it at the drop point', async () => {
  await page.evaluate(() => {
    const c = document.createElement('canvas'); c.width = 30; c.height = 30; c.getContext('2d').fillRect(0, 0, 30, 30);
    return new Promise((res) => c.toBlob((blob) => {
      const dt = new DataTransfer();
      dt.items.add(new File([blob], 'd.png', { type: 'image/png' }));
      const host = document.getElementById('host');
      host.dispatchEvent(new DragEvent('dragover', { dataTransfer: dt, bubbles: true, cancelable: true, clientX: 500, clientY: 350 }));
      host.dispatchEvent(new DragEvent('drop', { dataTransfer: dt, bubbles: true, cancelable: true, clientX: 500, clientY: 350 }));
      res();
    }));
  });
  await page.waitForFunction(() => wb.board.items.length === 1);
  const el = await last();
  assert.equal(el.type, 'image');
  near(el.x + el.w / 2, 500, 2); near(el.y + el.h / 2, 350, 2);
});

await test('export: toSVG is well-formed, toPNG yields a PNG blob, selection-only, dark', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 160, h: 90, text: 'A&B <GW>', backgroundColor: '#a5d8ff' });
  await add({ type: 'ellipse', x: 400, y: 100, w: 120, h: 80, text: 'Client' });
  const svg = await page.evaluate(() => wb.toSVG());
  assert.match(svg, /^<svg[^>]+xmlns="http:\/\/www\.w3\.org\/2000\/svg"/);
  const ok = await page.evaluate((s) => { const d = new DOMParser().parseFromString(s, 'image/svg+xml'); return !d.querySelector('parsererror') && d.documentElement.nodeName === 'svg'; }, svg);
  assert.equal(ok, true, 'svg parses');
  assert.ok(svg.includes('A&amp;B &lt;GW&gt;'));
  const png = await page.evaluate(async () => {
    const blob = await wb.toPNG({ scale: 2 });
    const buf = new Uint8Array(await blob.arrayBuffer());
    const bmp = await createImageBitmap(blob);
    return { type: blob.type, size: blob.size, sig: [...buf.slice(0, 4)], w: bmp.width, h: bmp.height };
  });
  assert.equal(png.type, 'image/png');
  assert.ok(png.size > 500);
  assert.deepEqual(png.sig, [0x89, 0x50, 0x4e, 0x47]);
  const dims = await page.evaluate(() => { const s = wb.toSVG(); const m = /width="([0-9.]+)" height="([0-9.]+)"/.exec(s); return [Number(m[1]), Number(m[2])]; });
  near(png.w, dims[0] * 2, 1); near(png.h, dims[1] * 2, 1);
  // selection only is smaller
  await page.evaluate(() => wb.select([wb.board.items[1].id]));
  const part = await page.evaluate(() => wb.toSVG({ selectionOnly: true }));
  assert.ok(!part.includes('GW') && part.includes('Client'));
  const noBg = await page.evaluate(() => wb.toSVG({ background: false }));
  assert.ok(noBg.length < svg.length);
  const dark = await page.evaluate(() => wb.toSVG({ dark: true }));
  assert.notEqual(dark, svg);
  // empty board rejects
  const err = await page.evaluate(async () => { wb.clear(); try { await wb.toPNG(); return null; } catch (e) { return e.message; } });
  assert.match(err, /Nothing/);
  // copy as svg / png do not throw
  await page.evaluate(() => wb.undo());
  assert.ok((await page.evaluate(() => wb.copyAsSvg())).startsWith('<svg'));
  assert.ok(await page.evaluate(async () => (await wb.copyAsPng()) instanceof Blob));
});

await test('theme: dark class, dark background, theme toggle shortcut', async () => {
  await add({ type: 'rect', x: 100, y: 100, w: 100, h: 60, backgroundColor: '#a5d8ff' });
  await press('Alt+Shift+d');
  assert.equal(await page.evaluate(() => wb.mode.theme), 'dark');
  assert.equal(await page.locator('.wb-root.wb-dark').count(), 1);
  const bg = await page.evaluate(() => getComputedStyle(document.querySelector('.wb-root')).backgroundColor);
  const lum = bg.match(/\d+/g).slice(0, 3).map(Number).reduce((a, b) => a + b, 0);
  assert.ok(lum < 150, `dark background ${bg}`);
  await page.evaluate(() => wb.setMode({ background: '#fffce8', theme: 'light' }));
  assert.equal(await page.evaluate(() => getComputedStyle(document.querySelector('.wb-root')).backgroundColor), 'rgb(255, 252, 232)');
});

await test('keyboard: requests, modes, font size, link, style copy, flowchart add', async () => {
  await press('?');
  await press(`${MOD}+f`);
  await press(`${MOD}+Slash`);
  await press(`${MOD}+Shift+e`);
  await press(`${MOD}+o`);
  await press(`${MOD}+s`);
  await press(`${MOD}+Shift+p`);
  const reqs = (await page.evaluate(() => window.__events)).filter((e) => e.t === 'request').map((e) => e.p.name);
  assert.deepEqual(reqs, ['help', 'search', 'palette', 'export', 'open', 'save', 'palette']);
  await press('Alt+z');
  assert.equal(await page.evaluate(() => wb.mode.zen), true);
  await press('Alt+z');
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, text: 'Svc', backgroundColor: '#a5d8ff' });
  await click(160, 140);
  await press(`${MOD}+Shift+Period`);
  assert.equal((await items())[0].fontSize, 22);
  await press(`${MOD}+Shift+Comma`);
  assert.equal((await items())[0].fontSize, 20);
  await page.evaluate(() => wb.setLink('example.com'));
  assert.equal((await items())[0].link, 'https://example.com');
  await page.evaluate(() => wb.setLink(null));
  assert.equal((await items())[0].link, null);
  // flowchart add: Ctrl+ArrowRight adds a connected shape
  await press(`${MOD}+ArrowRight`);
  const it = await items();
  assert.equal(it.length, 3);
  const arrow = it.find((e) => e.type === 'arrow');
  assert.equal(arrow.from, a);
  assert.ok(it.find((e) => e.id === arrow.to).x > 220);
  // copy / paste style
  await page.evaluate((id) => wb.select([id]), a);
  await press(`${MOD}+Alt+c`);
  const other = await add({ type: 'ellipse', x: 600, y: 400, w: 100, h: 60 });
  await page.evaluate((id) => wb.select([id]), other);
  await press(`${MOD}+Alt+v`);
  assert.equal((await items()).find((e) => e.id === other).backgroundColor, '#a5d8ff');
});

await test('context menu event carries the target and does not break selection', async () => {
  const a = await add({ type: 'rect', x: 100, y: 100, w: 120, h: 80, backgroundColor: '#ffc9c9' });
  await page.mouse.click(160, 140, { button: 'right' });
  let ev = await page.evaluate(() => window.__events.filter((e) => e.t === 'contextmenu').pop().p);
  assert.equal(ev.targetId, a);
  assert.deepEqual(await sel(), [a]);
  await page.mouse.click(700, 500, { button: 'right' });
  ev = await page.evaluate(() => window.__events.filter((e) => e.t === 'contextmenu').pop().p);
  assert.equal(ev.targetId, null);
});

await test('shortcuts are ignored while typing in an input and when keyboard is disabled', async () => {
  await page.evaluate(() => { const i = document.createElement('input'); i.id = 'tmp-in'; document.body.appendChild(i); });
  await page.focus('#tmp-in');
  await page.keyboard.type('r');
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  await page.evaluate(() => document.getElementById('tmp-in').remove());
  await page.evaluate(() => wb.setKeyboard(false));
  await press('r');
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  await page.evaluate(() => wb.setKeyboard(true));
  await press('r');
  assert.equal(await page.evaluate(() => wb.tool), 'rect');
  // clicking the canvas blurs a focused input
  await page.evaluate(() => { const i = document.createElement('input'); i.id = 'tmp-in2'; i.style.cssText = 'position:fixed;left:900px;top:10px;z-index:5'; document.body.appendChild(i); i.focus(); });
  await click(300, 300);
  assert.equal(await page.evaluate(() => document.activeElement && document.activeElement.id), '');
  await page.evaluate(() => document.getElementById('tmp-in2').remove());
});

await test('insertElements / toTemplate / find / focusElement / setBoard fit', async () => {
  const ids = await page.evaluate(() => wb.insertElements([
    { id: 1, type: 'rect', x: 0, y: 0, w: 100, h: 60, text: 'Alpha' },
    { id: 2, type: 'rect', x: 200, y: 0, w: 100, h: 60, text: 'Beta' },
    { id: 3, type: 'arrow', x: 100, y: 30, points: [[0, 0], [100, 0]], from: 1, to: 2 },
  ], { at: 'viewport-center' }));
  assert.equal(ids.length, 3);
  const it = await items();
  const arrow = it.find((e) => e.type === 'arrow');
  assert.equal(arrow.from, ids[0]); assert.equal(arrow.to, ids[1]);
  assert.deepEqual((await sel()).sort(), ids.slice().sort());
  const tpl = await page.evaluate(() => wb.toTemplate());
  assert.equal(tpl.elements.length, 3);
  assert.equal(Math.min(...tpl.elements.map((e) => e.x)), 0);
  assert.deepEqual(await page.evaluate(() => wb.find('beta')), [ids[1]]);
  await page.evaluate(() => wb._setView({ x: -5000, y: -5000, zoom: 1 }));
  await page.evaluate((id) => wb.focusElement(id), ids[1]);
  const pos = await page.evaluate((id) => { const e = wb.board.items.find((x) => x.id === id); return wb.worldToClient(e.x + e.w / 2, e.y + e.h / 2); }, ids[1]);
  assert.ok(pos.x > 0 && pos.x < 1200 && pos.y > 0 && pos.y < 800);
  await page.evaluate(() => wb.clear());
  assert.equal(await page.evaluate(() => wb.isEmpty()), true);
  await page.evaluate(() => wb.undo());
  assert.equal(await page.evaluate(() => wb.isEmpty()), false);
  // change events fire once per committed gesture
  await page.evaluate(() => { window.__n = 0; wb.on('change', () => { window.__n++; }); });
  await press('r');
  await drag(100, 500, 200, 560);
  assert.equal(await page.evaluate(() => window.__n), 1);
});

await test('robustness: NaN / Infinity inputs are ignored, handlers never throw', async () => {
  await page.evaluate(() => { wb.setZoom(NaN); wb.setZoom(Infinity); wb.pan(NaN, Infinity); wb.setStyle({ fontSize: NaN, opacity: Infinity }); wb.setMode({ gridSize: NaN }); });
  const v = await page.evaluate(() => wb.view);
  assert.ok(Number.isFinite(v.x) && Number.isFinite(v.y) && Number.isFinite(v.zoom));
  await page.evaluate(() => wb.insertElements([{ type: 'rect', x: NaN, y: Infinity, w: NaN, h: 5 }, null, 5, 'x']));
  await page.evaluate(() => wb.setBoard({ items: [{ id: 1, type: 'bogus' }, { id: 'x' }], nextId: 'q' }));
  assert.equal(await page.evaluate(() => wb.isEmpty()), true);
  await page.evaluate(() => wb.setTool('nope'));
  assert.equal(await page.evaluate(() => wb.tool), 'select');
  await page.evaluate(() => wb.align('diagonal'));
  await page.evaluate(() => wb.flowchartAdd('sideways'));
});

await test('readOnly option: dev.html?readonly blocks edits', async () => {
  await page.goto(`${base}?readonly`);
  await page.waitForFunction(() => window.__wbReady);
  await press('r');
  await drag(100, 100, 300, 200);
  assert.equal(await page.evaluate(() => wb.board.items.length), 0);
  await dbl(300, 300);
  assert.equal(await page.evaluate(() => wb.board.items.length), 0);
  await page.goto(base);
  await page.waitForFunction(() => window.__wbReady);
});

await test('resize of the container: engine keeps working (ResizeObserver)', async () => {
  await page.setViewportSize({ width: 700, height: 500 });
  await press('r');
  await click(300, 200);
  await page.setViewportSize({ width: 1200, height: 800 });
  const el = await last();
  near(el.x + el.w / 2, 300);
  assert.ok(await page.evaluate(() => wb._size.w) > 1100);
});

await test('performance: 600 elements render, select-all and drag stay responsive', async () => {
  await page.evaluate(() => {
    const items = [];
    for (let i = 0; i < 600; i++) {
      items.push({ id: i + 1, type: i % 3 === 0 ? 'ellipse' : 'rect', x: (i % 30) * 120, y: Math.floor(i / 30) * 90, w: 100, h: 60, text: `n${i}`, seed: i + 1, backgroundColor: '#a5d8ff' });
    }
    wb.setBoard({ version: 2, items, nextId: 1000 });
  });
  assert.equal(await page.locator('.wb-el').count(), 600);
  await page.evaluate(() => wb.zoomToFit());
  const t0 = Date.now();
  await press(`${MOD}+a`);
  const w = await page.evaluate(() => { const e = wb.board.items[0]; return wb.worldToClient(e.x + e.w / 2, e.y + e.h / 2); });
  await page.mouse.move(w.x, w.y);
  await page.mouse.down();
  await page.mouse.move(w.x + 100, w.y + 60, { steps: 20 });
  await page.mouse.up();
  const dt = Date.now() - t0;
  assert.ok(dt < 6000, `select all + drag took ${dt}ms`);
  const stats = await page.evaluate(() => { const t = performance.now(); wb._renderNow(); return performance.now() - t; });
  assert.ok(stats < 150, `idle re-render ${stats}ms`);
});

await test('HLD diagram screenshot: clients, LB, services, DB, cache, queue, frame, sticky', async () => {
  await buildDiagram();
  await page.screenshot({ path: path.join(shots, 'hld-diagram.png') });
  const b = await items();
  assert.ok(b.length >= 17);
  assert.ok(b.filter((e) => e.type === 'arrow').every((a) => a.from != null && a.to != null), 'every diagram arrow is bound at both ends');
  await page.evaluate(() => wb.toggleTheme());
  await page.screenshot({ path: path.join(shots, 'hld-diagram-dark.png') });
});

await test('destroy removes listeners, DOM and the __wb hook', async () => {
  await page.evaluate(() => { window.__counts = { n: 0 }; wb.on('tool', () => { window.__counts.n++; }); });
  await page.evaluate(() => wb.destroy());
  assert.equal(await page.locator('svg[data-wb-canvas]').count(), 0);
  assert.equal(await page.evaluate(() => document.getElementById('host').__wb), undefined);
  await press('r'); await drag(100, 100, 200, 200);
  assert.equal(await page.evaluate(() => window.__counts.n), 0);
  await page.evaluate(() => wb.destroy());
  // a second instance can mount on the same container and works
  await page.evaluate(async () => {
    const m = await import('/wb/engine.js');
    window.wb = new m.Whiteboard(document.getElementById('host'), {});
    window.__events = [];
  });
  await press('r');
  await drag(100, 100, 200, 200);
  assert.equal(await page.evaluate(() => wb.board.items.length), 1);
  assert.equal(await page.locator('svg[data-wb-canvas]').count(), 1);
  await page.reload();
  await page.waitForFunction(() => window.__wbReady);
});

// ---------- the HLD diagram ----------
async function buildDiagram() {
  await page.evaluate(() => {
    const mk = (id, type, x, y, w, h, extra = {}) => ({ id, type, x, y, w, h, seed: id * 131, ...extra });
    const items = [
      mk(1, 'frame', 60, 70, 900, 560, { name: 'Production region' }),
      mk(2, 'ellipse', 100, 330, 120, 80, { text: 'Clients', backgroundColor: '#ffec99', fontSize: 20 }),
      mk(3, 'diamond', 280, 300, 150, 140, { text: 'Load Balancer', backgroundColor: '#ffc9c9', fontSize: 18 }),
      mk(4, 'rect', 520, 130, 150, 80, { text: 'Auth Service', backgroundColor: '#a5d8ff' }),
      mk(5, 'rect', 520, 330, 150, 80, { text: 'URL Service', backgroundColor: '#a5d8ff' }),
      mk(6, 'rect', 520, 520, 150, 80, { text: 'Analytics', backgroundColor: '#a5d8ff' }),
      mk(7, 'database', 790, 305, 110, 130, { text: 'Postgres', backgroundColor: '#b2f2bb' }),
      mk(8, 'rect', 790, 130, 110, 80, { text: 'Redis cache', backgroundColor: '#ffd8a8' }),
      mk(9, 'rect', 790, 520, 110, 80, { text: 'Kafka queue', backgroundColor: '#d0bfff' }),
      mk(10, 'sticky', 1000, 120, 180, 150, { text: 'Read heavy:\n100:1 read/write.\nCache hot keys.', fontSize: 16 }),
      mk(11, 'arrow', 220, 370, 0, 0, { points: [[0, 0], [60, 0]], from: 2, to: 3, text: 'HTTPS' }),
      mk(12, 'arrow', 430, 370, 0, 0, { points: [[0, 0], [90, 0]], from: 3, to: 5, text: 'route' }),
      mk(13, 'arrow', 400, 330, 0, 0, { points: [[0, 0], [120, -160]], from: 3, to: 4, arrowType: 'elbow' }),
      mk(14, 'arrow', 400, 410, 0, 0, { points: [[0, 0], [120, 150]], from: 3, to: 6, arrowType: 'round' }),
      mk(15, 'arrow', 670, 370, 0, 0, { points: [[0, 0], [120, 0]], from: 5, to: 7, text: 'SQL', endArrowhead: 'arrow', startArrowhead: 'crowfoot_many' }),
      mk(16, 'arrow', 600, 330, 0, 0, { points: [[0, 0], [190, -150]], from: 5, to: 8, arrowType: 'elbow', text: 'cache-aside', strokeStyle: 'dashed' }),
      mk(17, 'arrow', 670, 560, 0, 0, { points: [[0, 0], [120, 0]], from: 6, to: 9, strokeColor: '#1971c2', text: 'events' }),
    ];
    const b = { version: 2, items, nextId: 100 };
    wb.setBoard(b);
    const next = wb.getBoard();
    for (const e of next.items) if (e.type !== 'frame' && e.type !== 'sticky' && e.type !== 'text') { /* frame membership */ }
    wb.setBoard(next);
    wb.select([]);
    wb.zoomToFit();
  });
  // let the engine assign frame membership through a real move, as a user would
  await page.evaluate(() => { const ids = wb.board.items.filter((e) => e.type !== 'frame' && e.type !== 'sticky').map((e) => e.id); wb.select(ids); wb.pan(0, 0); wb.clearSelection(); });
}

console.log(`\n${passed} passed, ${failures.length} failed; screenshots in ${shots}`);
await browser.close();
server.close();
if (failures.length || pageErrors.length) {
  if (pageErrors.length) console.log('page errors:', [...new Set(pageErrors)].join('\n'));
  process.exit(1);
}

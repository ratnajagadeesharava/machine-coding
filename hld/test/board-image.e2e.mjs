// The image tool on the standalone board: toolbar button -> file picker -> picture on the canvas.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { chromium } from 'playwright-core';
import { createAi } from '../server/ai.js';
import { createServer, loadQuestions } from '../server/index.js';
import { createMockClient } from '../server/mock.js';

const server = createServer({ ai: createAi({ client: createMockClient(), model: 'mock', effort: 'none' }), questions: loadQuestions(), quizzes: [], mode: 'mock', model: 'mock' });
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const base = fs.readdirSync('/opt/pw-browsers').filter((n) => n.startsWith('chromium-')).sort().reverse()[0];
const browser = await chromium.launch({ executablePath: process.env.HLD_CHROME || `/opt/pw-browsers/${base}/chrome-linux/chrome`, args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 1300, height: 800 } });
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));
page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
let code = 0;
try {
  await page.goto(`http://127.0.0.1:${server.address().port}/board.html`);
  await page.waitForSelector('[data-wb-canvas]');
  const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAoAAAAKCAYAAACNMs+9AAAAFklEQVR4nGP8z8Dwn4EIwDiqkL4KAQDp6hHpe8Cc1gAAAABJRU5ErkJggg==', 'base64');
  const file = path.join(os.tmpdir(), 'wb-test.png');
  fs.writeFileSync(file, png);
  const [chooser] = await Promise.all([page.waitForEvent('filechooser'), page.click('[data-tool="image"]')]);
  await chooser.setFiles(file);
  await page.waitForFunction(() => document.querySelector('.wbx-stage')?.__wb?.getBoard().items.some((i) => i.type === 'image'));
  assert.equal(await page.locator('.wb-el image').count(), 1, 'image is drawn');
  const loaded = await page.evaluate(() => new Promise((res) => { const im = document.querySelector('.wb-el image'); const t = new Image(); t.onload = () => res(true); t.onerror = () => res(false); t.src = im.getAttribute('href') || im.getAttributeNS('http://www.w3.org/1999/xlink', 'href'); }));
  assert.equal(loaded, true, 'image data decodes');
  assert.deepEqual(errors, []);
  console.log('board image e2e ok');
} catch (e) { console.error(e); code = 1; }
await browser.close(); server.close();
process.exit(code);

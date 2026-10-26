// End-to-end smoke test: real server in mock mode + headless Chromium driving the actual UI.
// Run with: npm run e2e   (needs a Chromium; set HLD_CHROME to its path if it is not found automatically)
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright-core';
import { createAi } from '../server/ai.js';
import { createServer, loadQuestions } from '../server/index.js';
import { loadQuizzes } from '../server/quiz.js';
import { createMockClient } from '../server/mock.js';

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

const server = createServer({
  ai: createAi({ client: createMockClient(), model: 'mock', effort: 'none' }),
  questions: loadQuestions(), quizzes: loadQuizzes(path.join(here, '..', 'quizzes')), mode: 'mock', model: 'mock',
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const url = `http://127.0.0.1:${server.address().port}/`;
const shots = process.env.HLD_SHOTS;

const browser = await chromium.launch({ executablePath: findChrome(), args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 1500, height: 940 } });
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));
page.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });

const wbItems = () => page.evaluate(() => document.querySelector('.wbx-stage')?.__wb?.getBoard().items.length ?? -1);
const hasBoard = async () => page.waitForSelector('[data-wb-canvas]', { timeout: 8000 }).then(() => true, () => false);
const apiCalls = [];
page.on('request', (r) => { const u = new URL(r.url()); if (u.pathname.startsWith('/api/')) apiCalls.push(u.pathname); });
const drawRect = async (x1, y1, x2, y2) => {
  const box = await page.locator('#board').boundingBox();
  await page.click('[data-tool="rect"]');
  await page.mouse.move(box.x + x1, box.y + y1);
  await page.mouse.down();
  await page.mouse.move(box.x + (x1 + x2) / 2, box.y + (y1 + y2) / 2, { steps: 4 });
  await page.mouse.move(box.x + x2, box.y + y2, { steps: 4 });
  await page.mouse.up();
  await page.keyboard.press('Escape');
};

try {
  // ---------- home ----------
  await page.goto(url);
  await page.waitForSelector('.bq-card');
  assert.equal(await page.locator('.bq-card').count(), 26, 'all questions listed');
  assert.match(await page.textContent('h1'), /Practise system design, your way/);
  assert.equal(await page.locator('.nav a.on').textContent(), 'Home');
  assert.deepEqual(await page.locator('.nav a').allTextContents(), ['Home', 'Whiteboard', 'Quizzes']);
  assert.match(await page.getAttribute('#entry-board', 'href'), /board\.html/);
  await page.fill('#q-search', 'rate limiter');
  assert.equal(await page.locator('.bq-card').count(), 1);
  await page.fill('#q-search', '');
  await page.click('#q-diff button[data-diff="Hard"]');
  const hard = await page.locator('.bq-card').count();
  assert.ok(hard > 5 && hard < 26);
  await page.click('#q-diff button[data-diff="All"]');
  await page.selectOption('#q-category', 'Messaging');
  const msg = await page.locator('.bq-card').count();
  assert.ok(msg >= 2 && msg < 26);
  await page.selectOption('#q-category', 'All');
  if (shots) await page.screenshot({ path: path.join(shots, 'home.png'), fullPage: true });

  // ---------- bank question -> pre-start dialog -> room ----------
  await page.click('.bq-card:has-text("URL shortener")');
  await page.waitForSelector('#start-dialog[open]');
  assert.match(await page.textContent('#sd-prompt'), /turns long URLs into short links/);
  assert.equal(await page.locator('#sd-assume li').count(), 4);
  await page.selectOption('#sd-level', 'staff');
  await page.selectOption('#sd-style', 'tough');
  await page.fill('#sd-minutes', '30');
  await page.click('#sd-start');
  await page.waitForURL(/room\.html\?q=url-shortener/);
  await page.waitForSelector('#q-title:not(:has-text("Loading"))');
  assert.match(await page.textContent('#q-title'), /URL shortener/);
  assert.match(await page.textContent('#timer'), /30:00/, 'the time override applies');
  assert.match(await page.textContent('#q-chips'), /Staff/);
  assert.match(await page.textContent('#q-chips'), /Tough/);
  // the side panels can be dragged wider, the choice survives a reload, and double-click resets it
  {
    const width = (sel) => page.evaluate((q) => Math.round(document.querySelector(q).getBoundingClientRect().width), sel);
    const before = await width('#q-panel');
    const bar = await page.locator('.resizer-right').boundingBox();
    await page.mouse.move(bar.x + 3, bar.y + 200); await page.mouse.down(); await page.mouse.move(bar.x + 123, bar.y + 200, { steps: 4 }); await page.mouse.up();
    const wider = await width('#q-panel');
    assert.ok(wider >= before + 100, `question panel grew (${before} -> ${wider})`);
    await page.reload();
    await page.waitForSelector('#q-title:not(:has-text("Loading"))');
    assert.equal(await width('#q-panel'), wider, 'width is remembered');
    await page.dblclick('.resizer-right');
    assert.equal(await width('#q-panel'), before, 'double-click resets');
  }

  // left panel shows only the question, and closes/reopens
  assert.equal(await page.locator('#q-panel').isVisible(), true);
  const panelText = await page.textContent('#q-panel');
  assert.match(panelText, /turns long URLs into short links/);
  assert.match(panelText, /100M new links per month/);
  assert.doesNotMatch(panelText, /rate limiter|news feed|Search questions/, 'no question list in the room');
  await page.click('#q-close');
  assert.equal(await page.locator('#q-panel').isVisible(), false);
  assert.equal(await page.locator('#q-tab').isVisible(), true);
  await page.reload();
  await page.waitForSelector('#q-title:not(:has-text("Loading"))');
  assert.equal(await page.locator('#q-panel').isVisible(), false, 'closed state is remembered');
  await page.click('#q-tab');
  assert.equal(await page.locator('#q-panel').isVisible(), true);

  // bot panel: while closed there are no AI requests at all
  await page.click('#btn-start');
  await page.waitForSelector('#btn-pause:not([hidden])');
  await page.click('#bot-close');
  assert.equal(await page.locator('#bot-panel').isVisible(), false);
  assert.equal(await page.locator('#bot-tab').isVisible(), true);
  apiCalls.length = 0;
  const canvas = await hasBoard();
  if (canvas) {
    await drawRect(300, 250, 460, 340);
    await drawRect(600, 250, 760, 340);
  }
  await page.waitForTimeout(1200);
  assert.deepEqual(apiCalls.filter((p) => /\/api\/(chat|watch|evaluate)/.test(p)), [], 'Aria is off: no AI requests');
  await page.click('#bot-tab');
  assert.equal(await page.locator('#bot-panel').isVisible(), true);

  // whiteboard drawing in the room (skipped until the engine exists)
  if (canvas) {
    assert.ok((await wbItems()) >= 2, 'drawn shapes are on the board');
    const n = await wbItems();
    await page.waitForTimeout(600);
    await page.reload();
    await hasBoard();
    assert.equal(await wbItems(), n, 'board is restored from the browser');
    assert.match(await page.textContent('#aria-board'), /2 components/);
  } else console.warn('skipped: whiteboard engine not available');

  // chat reaches the server with the question; the mock reply proves which question and settings it saw
  await page.click('[data-quick="review"]');
  await page.waitForFunction(() => /Mock review/.test(document.body.textContent));
  await page.waitForFunction(() => !document.querySelector('#chat-send').disabled);
  const last = await page.locator('.msg.assistant .bubble').last().textContent();
  assert.match(last, /\[Q: Design a URL shortener\] \[staff\/tough\]/, last);
  await page.fill('#chat-input', 'What is the read to write ratio?');
  await page.press('#chat-input', 'Enter');
  await page.waitForFunction(() => /Mock Aria here/.test(document.body.textContent));
  await page.waitForFunction(() => !document.querySelector('#chat-send').disabled);
  if (shots) await page.screenshot({ path: path.join(shots, 'interview.png') });

  // end and feedback report
  await page.click('#btn-pause');
  await page.click('#btn-end');
  await page.click('#confirm-yes');
  await page.waitForSelector('#report .verdict');
  assert.equal(await page.locator('#report .area').count(), 7);
  assert.match(await page.textContent('#report'), /Mock report/);
  if (shots) await page.screenshot({ path: path.join(shots, 'report.png') });
  await page.click('#report-close');
  assert.equal(await page.locator('#btn-feedback:not([hidden])').count(), 1);

  // Home lists the finished session under Continue
  await page.click('#btn-home');
  await page.waitForSelector('.bq-card');
  assert.match(await page.textContent('#continue-list'), /URL shortener/);

  // ---------- unknown question redirects home with a notice ----------
  await page.goto(`${url}room.html?q=does-not-exist`);
  await page.waitForSelector('.bq-card');
  assert.match(await page.textContent('#notice'), /could not be found/);
  await page.goto(`${url}room.html`);
  await page.waitForSelector('.bq-card');

  // ---------- custom question ----------
  await page.click('#c-start');
  assert.match(await page.textContent('#e-title'), /title/i);
  await page.fill('#c-title', 'Design a zebra crossing scheduler');
  await page.fill('#c-prompt', 'Schedule pedestrian crossings for a city of five million people.');
  await page.fill('#c-assumptions', '2,000 junctions\nSensors fail sometimes');
  await page.selectOption('#c-time', '30');
  await page.selectOption('#c-level', 'mid');
  await page.selectOption('#c-style', 'supportive');
  await page.fill('#c-focus', 'sensor failure');
  await page.click('#c-save');
  assert.equal(await page.locator('.hm-custom').count(), 1);
  await page.click('.hm-custom button:has-text("Edit")');
  assert.equal(await page.inputValue('#c-title'), 'Design a zebra crossing scheduler');
  assert.equal(await page.inputValue('#c-time'), '30');
  await page.fill('#c-title', 'Design a zebra crossing planner');
  await page.click('#c-start');
  await page.waitForURL(/room\.html\?custom=custom-/);
  await page.waitForSelector('#q-title:not(:has-text("Loading"))');
  assert.match(await page.textContent('#q-panel'), /zebra crossing planner/);
  assert.match(await page.textContent('#q-panel'), /Sensors fail sometimes/);
  assert.match(await page.textContent('#q-panel'), /sensor failure/);
  assert.match(await page.textContent('#timer'), /30:00/);
  await page.click('#btn-start');
  await page.fill('#chat-input', 'Hello there');
  await page.press('#chat-input', 'Enter');
  await page.waitForFunction(() => /\[Q: Design a zebra crossing planner\] \[mid\/supportive\]/.test(document.body.textContent));
  await page.click('#btn-home');
  await page.waitForSelector('.bq-card');
  assert.equal(await page.locator('.hm-custom').count(), 1, 'custom question is listed');
  assert.match(await page.textContent('#continue-list'), /zebra crossing planner/);
  await page.click('.hm-custom button:has-text("Delete")');
  await page.click('#confirm-yes');
  assert.equal(await page.locator('.hm-custom').count(), 0);

  // ---------- standalone whiteboard: no API calls at all, even when /api is blocked ----------
  const wbPage = await browser.newPage({ viewport: { width: 1300, height: 860 } });
  const wbReqs = [];
  const wbErrors = [];
  wbPage.on('request', (r) => wbReqs.push(new URL(r.url()).pathname));
  wbPage.on('pageerror', (e) => wbErrors.push(e.message));
  await wbPage.route('**/api/**', (route) => route.abort());
  await wbPage.goto(`${url}board.html`);
  assert.equal(await wbPage.locator('.nav a.on').textContent(), 'Whiteboard');
  const wbReady = await wbPage.waitForSelector('[data-wb-canvas]', { timeout: 8000 }).then(() => true, () => false);
  if (wbReady) {
    const box = await wbPage.locator('#board').boundingBox();
    await wbPage.click('[data-tool="rect"]');
    await wbPage.mouse.move(box.x + 300, box.y + 250);
    await wbPage.mouse.down();
    await wbPage.mouse.move(box.x + 450, box.y + 330, { steps: 5 });
    await wbPage.mouse.up();
    await wbPage.waitForTimeout(700);
    assert.equal(await wbPage.evaluate(() => document.querySelector('.wbx-stage').__wb.getBoard().items.length), 1);
    await wbPage.reload();
    await wbPage.waitForSelector('[data-wb-canvas]');
    assert.equal(await wbPage.evaluate(() => document.querySelector('.wbx-stage').__wb.getBoard().items.length), 1, 'scratch board persists');
    if (shots) await wbPage.screenshot({ path: path.join(shots, 'board.png') });
  } else console.warn('skipped: whiteboard engine not available');
  assert.deepEqual(wbReqs.filter((p) => p.startsWith('/api/')), [], 'board.html never calls the API');
  assert.deepEqual(wbErrors, []);
  await wbPage.close();

  // ---------- quiz page ----------
  const quizDef = await (await fetch(`${url}api/quizzes/scaling-reads`)).json();
  const byPrompt = new Map(quizDef.questions.map((q) => [q.prompt, q]));
  await page.goto(url);
  await page.click('.nav a[href="/quiz.html"]');
  await page.waitForSelector('.card');
  assert.match(await page.textContent('.card:has-text("Scaling Reads") h3'), /Scaling Reads/);
  assert.equal(await page.locator('.topic-card').count(), 3, 'each series is one topic card, not eight quizzes');
  assert.equal(await page.locator('#btn-quit:not([hidden])').count(), 0);

  const answer = async (right) => {
    const prompt = (await page.textContent('#q-prompt')).trim();
    const q = byPrompt.get(prompt);
    assert.ok(q, `unknown prompt: ${prompt}`);
    const texts = (await page.locator('.opt .txt').allTextContents()).map((t) => t.trim());
    const text = (o) => o.text;
    const want = right ? q.options.filter((o) => q.correct.includes(o.id)) : [q.options.find((o) => !q.correct.includes(o.id))];
    for (const o of want) {
      const idx = texts.findIndex((t) => t === text(o));
      assert.ok(idx >= 0, `option not found: ${o.text}`);
      await page.locator('.opt').nth(idx).click();
    }
    await page.click('#btn-check');
    return q;
  };

  await page.click('.card .btn:has-text("Quick 10")');
  await page.waitForSelector('#q-options .opt');
  assert.match(await page.textContent('#run-count'), /Question 1 of 10/);
  assert.equal(await page.locator('#btn-check').isDisabled(), true, 'cannot check without choosing');

  // question 1 right, question 2 wrong (and ask the tutor about it), the rest right except one more
  let wrongCount = 0;
  for (let i = 0; i < 10; i++) {
    const right = !(i === 1 || i === 6);
    if (!right) wrongCount += 1;
    await answer(right);
    assert.match(await page.textContent('#q-verdict'), right ? /Correct!/ : /Not quite|Partly right/);
    assert.equal(await page.locator('.opt .why').count() > 0, true, 'every option explains itself after checking');
    if (i === 1) {
      await page.click('#btn-tutor');
      await page.waitForFunction(() => /Mock tutor: let us look at/.test(document.querySelector('#tutor-text')?.textContent || ''));
      await page.fill('#tutor-input', 'Can you give an example?');
      await page.press('#tutor-input', 'Enter');
      await page.waitForFunction(() => /Mock tutor follow-up: Can you give an example\?/.test(document.querySelector('#tutor-text')?.textContent || ''));
    }
    await page.click('#btn-next');
  }
  await page.waitForSelector('#view-result:not([hidden])');
  assert.match(await page.textContent('#ring-pct'), /80%/);
  assert.match(await page.textContent('#result-sub'), /8 of 10 correct/);
  assert.ok((await page.locator('#by-concept .bc-row').count()) >= 3);
  assert.equal(await page.locator('#review .rv').count(), wrongCount, 'review shows only wrong answers by default');
  await page.uncheck('#only-wrong');
  assert.equal(await page.locator('#review .rv').count(), 10);
  if (shots) await page.screenshot({ path: path.join(shots, 'quiz-result.png') });

  // retry the two missed questions, then everything is on the home page with a best score
  await page.click('#btn-retry-missed');
  await page.waitForSelector('#q-options .opt');
  assert.match(await page.textContent('#run-count'), /Question 1 of 2/);
  await answer(true); await page.click('#btn-next');
  await answer(true); await page.click('#btn-next');
  await page.waitForSelector('#view-result:not([hidden])');
  assert.match(await page.textContent('#ring-pct'), /100%/);
  assert.equal(await page.locator('#btn-retry-missed:not([hidden])').count(), 0, 'nothing left to retry');
  await page.click('#btn-home');
  assert.equal(await page.locator('.card .btn:has-text("Retry missed")').count(), 0);

  // exam mode: no feedback until the end
  await page.click('.card .btn:has-text("Quick 10")');
  await page.waitForSelector('#q-options .opt');
  await page.uncheck('#instant');
  assert.equal(await page.locator('#btn-check:not([hidden])').count(), 0);
  await page.locator('.opt').first().click();
  await page.click('#btn-next');
  assert.match(await page.textContent('#run-count'), /Question 2 of 10/);
  assert.equal(await page.locator('#q-feedback:not([hidden])').count(), 0, 'no feedback in exam mode');
  await page.click('#btn-quit');
  await page.click('#confirm-yes');
  await page.waitForSelector('.card');
  await page.check('#instant').catch(() => {});

  // generate a quiz with the (mock) AI, start it, delete it
  await page.fill('#gen-topic', 'Database sharding');
  await page.selectOption('#gen-count', '5');
  await page.click('#gen-btn');
  await page.waitForFunction(() => /Created/.test(document.querySelector('#gen-status')?.textContent || ''));
  await page.click('.topic-card:has-text("My generated quizzes") .btn');
  await page.waitForSelector('.card:has-text("AI generated")');
  assert.equal(await page.locator('.card:has-text("AI generated")').count(), 1);
  await page.click('.card:has-text("AI generated") .btn:has-text("Start")');
  await page.waitForSelector('#q-options .opt');
  assert.match(await page.textContent('#run-count'), /Question 1 of 5/);
  await page.click('#btn-quit');
  await page.click('#confirm-yes');
  await page.waitForSelector('.card');
  await page.click('.card:has-text("AI generated") .btn:has-text("Delete")');
  await page.click('#confirm-yes');
  await page.waitForFunction(() => !document.querySelector('.card .badge.ai') && document.querySelectorAll('.topic-card').length === 3);

  assert.deepEqual(errors, [], `browser errors: ${errors.join(' | ')}`);
  console.log('e2e ok');
} catch (err) {
  if (shots) await page.screenshot({ path: path.join(shots, 'failure.png') });
  console.error(err);
  process.exitCode = 1;
} finally {
  await browser.close();
  server.close();
}

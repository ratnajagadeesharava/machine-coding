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

try {
  await page.goto(url);
  await page.waitForSelector('.q-item');
  assert.equal(await page.locator('.q-item').count(), 26, 'all questions listed');
  assert.match(await page.textContent('#q-title'), /URL shortener/);
  assert.match(await page.textContent('#timer'), /45:00/);

  // filter and switch question
  await page.fill('#q-search', 'rate');
  assert.equal(await page.locator('.q-item').count(), 1);
  await page.click('.q-item');
  assert.match(await page.textContent('#q-title'), /rate limiter/);
  assert.match(await page.textContent('#timer'), /35:00/);
  await page.fill('#q-search', '');
  await page.click('#q-diff button[data-diff="Hard"]');
  assert.ok((await page.locator('.q-item').count()) > 5);
  await page.click('#q-diff button[data-diff="All"]');
  await page.click('.q-item:has-text("URL shortener")');

  // start the interview
  await page.click('#btn-start');
  await page.waitForSelector('#btn-pause:not([hidden])');
  assert.match(await page.textContent('#pill-text'), /watching/i);

  // draw: rectangle, database, then an arrow between them
  const box = await page.locator('#board').boundingBox();
  const drag = async (x1, y1, x2, y2) => {
    await page.mouse.move(box.x + x1, box.y + y1);
    await page.mouse.down();
    await page.mouse.move(box.x + (x1 + x2) / 2, box.y + (y1 + y2) / 2, { steps: 4 });
    await page.mouse.move(box.x + x2, box.y + y2, { steps: 4 });
    await page.mouse.up();
  };
  await page.click('[data-tool="rect"]');
  await page.mouse.click(box.x + 250, box.y + 320);
  await page.keyboard.type('API Server');
  await page.keyboard.press('Enter');
  await page.click('[data-tool="database"]');
  await page.mouse.click(box.x + 620, box.y + 320);
  await page.keyboard.type('Links DB');
  await page.keyboard.press('Enter');
  await page.click('[data-tool="arrow"]');
  await drag(285, 320, 600, 322);
  const stats = await page.textContent('#aria-board');
  assert.match(stats, /2 components · 1 connection in view/, stats);
  await page.click('[data-tool="select"]');

  // chat: quick action + free text, streamed from the (mock) server with the board attached
  await page.click('[data-quick="review"]');
  await page.waitForFunction(() => /connection\(s\) on your board/.test(document.body.textContent));
  await page.waitForFunction(() => !document.querySelector('#chat-send').disabled);
  const lastBubble = await page.locator('.msg.assistant .bubble').last().textContent();
  assert.match(lastBubble, /Mock review: I can see 2 component\(s\) and 1 connection\(s\)/, lastBubble);
  await page.fill('#chat-input', 'What is the read to write ratio?');
  await page.press('#chat-input', 'Enter');
  await page.waitForFunction(() => /Mock Aria here/.test(document.body.textContent));
  await page.waitForFunction(() => !document.querySelector('#chat-send').disabled);

  if (shots) await page.screenshot({ path: path.join(shots, 'interview.png') });

  // undo / persistence across reload
  await page.click('#btn-undo');
  assert.match(await page.textContent('#aria-board'), /2 components · 0 connections/);
  await page.reload();
  await page.waitForSelector('.q-item');
  assert.match(await page.textContent('#q-title'), /URL shortener/, 'last question restored');
  assert.match(await page.textContent('#aria-board'), /2 components/, 'board restored');
  assert.match(await page.textContent('#aria-status-text'), /Paused/, 'running session comes back paused');

  // resume, end, feedback report
  await page.click('#btn-pause');
  await page.click('#btn-end');
  await page.click('#confirm-yes');
  await page.waitForSelector('#report .verdict');
  assert.match(await page.textContent('#report'), /Mock report: the final board had 2 component/);
  assert.equal(await page.locator('#report .area').count(), 7);
  if (shots) await page.screenshot({ path: path.join(shots, 'report.png') });
  await page.click('#report-close');
  assert.equal(await page.locator('#btn-feedback:not([hidden])').count(), 1);


  // ---------- quiz page ----------
  const quizDef = await (await fetch(`${url}api/quizzes/scaling-reads`)).json();
  const byPrompt = new Map(quizDef.questions.map((q) => [q.prompt, q]));
  await page.goto(url);
  await page.click('.nav a[href="/quiz.html"]');
  await page.waitForSelector('.card');
  assert.match(await page.textContent('.card h3'), /Scaling Reads/);
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
  assert.equal(await page.locator('.card:has-text("AI generated")').count(), 1);
  await page.click('.card:has-text("AI generated") .btn:has-text("Start")');
  await page.waitForSelector('#q-options .opt');
  assert.match(await page.textContent('#run-count'), /Question 1 of 5/);
  await page.click('#btn-quit');
  await page.click('#confirm-yes');
  await page.waitForSelector('.card');
  await page.click('.card:has-text("AI generated") .btn:has-text("Delete")');
  await page.click('#confirm-yes');
  await page.waitForFunction(() => document.querySelectorAll('.card').length === 1);

  assert.deepEqual(errors, [], `browser errors: ${errors.join(' | ')}`);
  console.log('e2e ok');
} catch (err) {
  if (shots) await page.screenshot({ path: path.join(shots, 'failure.png') });
  console.error(err);
  process.exitCode = 1;
} finally {
  if (shots) {
    await page.goto(url);
    await page.waitForSelector('.q-item');
    await page.screenshot({ path: path.join(shots, 'home.png') });
  }
  await browser.close();
  server.close();
}

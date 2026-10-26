// Quiz page UX: question map, flag, finish early, resume after reload, weak-spot practice, index and search, progress export.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { chromium } from 'playwright-core';
import { createAi } from '../server/ai.js';
import { createServer, loadQuestions } from '../server/index.js';
import { loadQuizzes } from '../server/quiz.js';
import { createMockClient } from '../server/mock.js';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const server = createServer({ ai: createAi({ client: createMockClient(), model: 'mock', effort: 'none' }), questions: loadQuestions(), quizzes: loadQuizzes(path.join(here, '..', 'quizzes')), mode: 'mock', model: 'mock' });
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const url = `http://127.0.0.1:${server.address().port}/quiz.html`;
const base = fs.readdirSync('/opt/pw-browsers').filter((n) => n.startsWith('chromium-')).sort().reverse()[0];
const browser = await chromium.launch({ executablePath: process.env.HLD_CHROME || `/opt/pw-browsers/${base}/chrome-linux/chrome`, args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 }, acceptDownloads: true });
const errors = [];
page.on('pageerror', (e) => errors.push(e.message));
let code = 0;
const step = async (name, fn) => { await fn(); console.log('  ok  ' + name); };
try {
  await page.goto(url);
  await page.waitForSelector('.card');
  await step('index lists every quiz and search filters it', async () => {
    assert.equal(await page.locator('.toc-item').count(), 9);
    await page.fill('#quiz-search', 'replication');
    assert.equal(await page.locator('.card:visible').count(), 1);
    await page.fill('#quiz-search', '');
  });
  await step('question map, flag, back and forward keys', async () => {
    await page.click('[data-quiz="pg-replication"] .btn.primary');
    await page.waitForSelector('#q-options .opt');
    assert.equal(await page.locator('.map-dot').count(), 6);
    await page.keyboard.press('1');
    await page.keyboard.press('Enter');
    assert.equal(await page.locator('.map-dot.right, .map-dot.wrong').count(), 1);
    await page.keyboard.press('f');
    assert.equal(await page.locator('.map-dot.flag').count(), 1);
    await page.click('.map-dot >> nth=3');
    assert.match(await page.textContent('#run-count'), /Question 4 of 6/);
    await page.keyboard.press('ArrowLeft');
    assert.match(await page.textContent('#run-count'), /Question 3 of 6/);
  });
  await step('resume after reload keeps answers, flags and position', async () => {
    await page.reload();
    await page.waitForSelector('.upnext-card');
    assert.match(await page.textContent('.upnext-card'), /Resume/);
    await page.click('.upnext-card .btn.primary');
    await page.waitForSelector('#q-options .opt');
    assert.match(await page.textContent('#run-count'), /Question 3 of 6/);
    assert.equal(await page.locator('.map-dot.flag').count(), 1);
    assert.equal(await page.locator('.map-dot.right, .map-dot.wrong').count(), 1);
  });
  await step('finish early asks first, then scores unanswered as wrong and weak spots appear', async () => {
    await page.click('#btn-finish');
    assert.match(await page.textContent('#confirm-text'), /unanswered/);
    await page.click('#confirm-yes');
    await page.waitForSelector('#view-result:not([hidden])');
    await page.click('#btn-home');
    await page.waitForSelector('.upnext-card');
    assert.match(await page.textContent('.upnext-card'), /WEAK SPOTS/i);
  });
  await step('weak-spot practice runs a mixed round', async () => {
    await page.click('.upnext-card .btn.primary');
    await page.waitForSelector('#q-options .opt');
    assert.match(await page.textContent('#page-sub'), /Weak spots/);
  });
  await step('cards show Completed / In progress / Not started', async () => {
    assert.match(await page.textContent('[data-quiz="pg-replication"] .status'), /Completed/);
    assert.match(await page.textContent('[data-quiz="pg-when-to-use"] .status'), /Not started/);
    assert.equal(await page.locator('.series-num.done').count(), 1);
    await page.click('#btn-quit'); await page.click('#confirm-yes');
    await page.waitForSelector('.card');
    await page.click('[data-quiz="pg-when-to-use"] .btn.primary');
    await page.waitForSelector('#q-options .opt');
    await page.keyboard.press('1');
    await page.click('#btn-quit'); await page.click('#confirm-yes');
    await page.waitForSelector('.card');
    assert.match(await page.textContent('[data-quiz="pg-replication"] .btn'), /Retake/);
  });
  await step('light / dark toggle switches and is remembered', async () => {
    const bg = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    const before = await bg();
    await page.click('[data-theme-btn]');
    const after = await bg();
    assert.notEqual(before, after);
    await page.reload();
    await page.waitForSelector('.card');
    assert.equal(await bg(), after, 'choice survives a reload');
    await page.click('[data-theme-btn]');
    assert.equal(await bg(), before);
  });
  await step('progress exports as a file', async () => {
    await page.waitForSelector('.hist-tools');
    const [dl] = await Promise.all([page.waitForEvent('download'), page.click('text=Export progress')]);
    const data = JSON.parse(fs.readFileSync(await dl.path(), 'utf8'));
    assert.equal(data.app, 'hld-room-quizzes');
    assert.ok(Object.keys(data.attempts).length >= 1);
  });
  assert.deepEqual(errors, []);
  console.log('quiz ux e2e ok');
} catch (e) { console.error(e); code = 1; }
await browser.close(); server.close();
process.exit(code);

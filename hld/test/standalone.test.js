import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

// The whiteboard (public/wb) and the shared data model must not know about the server, the AI or interview questions.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dirs = ['public/wb', 'shared'];
const files = dirs.flatMap((d) => (fs.existsSync(path.join(root, d)) ? fs.readdirSync(path.join(root, d)).filter((f) => f.endsWith('.js')).map((f) => path.join(d, f)) : []));
// shared/quiz.js belongs to the quiz feature, not the whiteboard.
const scanned = files.filter((f) => f !== path.join('shared', 'quiz.js'));

const FORBIDDEN = [
  [/\/api\//, '/api/'], [/\bfetch\s*\(/, 'fetch('], [/\bapp\.js\b/, 'app.js'], [/\broom\.js\b/, 'room.js'], [/\bquiz\.js\b/, 'quiz.js'],
  [/\bquestions?\b/i, 'question'], [/(?<![-\w])aria(?![-\w])/i, 'Aria (the bot; aria-* attributes are fine)'], [/\banthropic\b/i, 'anthropic'], [/\bhome\.js\b|\bstore\.js\b/, 'page module'],
];

test('there are whiteboard files to scan', () => {
  assert.ok(scanned.length >= 3, scanned.join(','));
});

for (const f of scanned) {
  test(`${f} is standalone`, () => {
    const text = fs.readFileSync(path.join(root, f), 'utf8');
    for (const [re, name] of FORBIDDEN) {
      const m = re.exec(text);
      assert.equal(m, null, `${f} mentions ${name}${m ? ` near "${text.slice(Math.max(0, m.index - 30), m.index + 40).replace(/\n/g, ' ')}"` : ''}`);
    }
  });
}

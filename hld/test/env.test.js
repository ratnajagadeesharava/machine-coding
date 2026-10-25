import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { loadEnvFile, parseEnv } from '../server/env.js';

test('parseEnv handles comments, quotes, export and inline comments', () => {
  const out = parseEnv(['# comment', '', 'A=1', 'B = two words ', 'C="quoted # not a comment"', "D='single'", 'export E=5', 'F=6 # trailing', '1BAD=x', 'noequals', 'G='].join('\n'));
  assert.deepEqual(out, { A: '1', B: 'two words', C: 'quoted # not a comment', D: 'single', E: '5', F: '6', G: '' });
});

test('parseEnv copes with CRLF line endings', () => {
  assert.deepEqual(parseEnv('A=1\r\nB=2\r\n'), { A: '1', B: '2' });
});

test('loadEnvFile never overrides variables that are already set', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hld-env-'));
  const file = path.join(dir, '.env');
  fs.writeFileSync(file, 'KEEP=file\nNEW=file\nEMPTY=file\n');
  const env = { KEEP: 'shell', EMPTY: '' };
  assert.equal(loadEnvFile(file, env), true);
  assert.deepEqual(env, { KEEP: 'shell', NEW: 'file', EMPTY: 'file' });
  assert.equal(loadEnvFile(path.join(dir, 'missing'), env), false);
});

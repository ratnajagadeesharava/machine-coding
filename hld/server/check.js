// `npm run check`: makes two tiny real API calls (streaming chat + structured output) to confirm your key and model work.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import Anthropic from '@anthropic-ai/sdk';
import { createAi, DEFAULT_MODEL, friendlyError } from './ai.js';
import { loadEnvFile } from './env.js';
import { loadQuestions } from './index.js';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
loadEnvFile(path.join(root, '.env'));

if (!process.env.ANTHROPIC_API_KEY) {
  console.error('ANTHROPIC_API_KEY is not set. Copy .env.example to .env and add your key (https://console.anthropic.com/).');
  process.exit(1);
}
const model = process.env.HLD_MODEL || DEFAULT_MODEL;
const ai = createAi({ client: new Anthropic(), model, effort: process.env.HLD_EFFORT || 'default' });
const question = loadQuestions()[0];
const board = { nextId: 3, items: [{ id: 1, type: 'rect', x: 0, y: 0, w: 120, h: 70, text: 'API Server' }, { id: 2, type: 'database', x: 300, y: 0, w: 110, h: 90, text: 'Links DB' }] };

try {
  console.log(`Model: ${model}`);
  let reply = '';
  for await (const t of ai.chat({
    question, board, elapsedMs: 60_000, status: 'running', mode: 'chat',
    messages: [{ role: 'user', content: 'In one sentence: what should I clarify first?' }],
  })) reply += t;
  console.log('Chat (streaming) OK:', reply.trim().slice(0, 200));
  const w = await ai.watch({ question, board, elapsedMs: 60_000, status: 'running', messages: [], sinceAriaMs: null });
  console.log('Watch (structured output) OK:', JSON.stringify(w));
  console.log('All good - run "npm start" and open the URL it prints.');
} catch (err) {
  console.error('Check failed:', friendlyError(err).message);
  if (err?.status) console.error(`(HTTP ${err.status})`);
  process.exit(1);
}

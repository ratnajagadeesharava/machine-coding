import assert from 'node:assert/strict';
import test from 'node:test';
import Anthropic from '@anthropic-ai/sdk';
import { AriaError, cleanEvaluation, createAi, friendlyError } from '../server/ai.js';
import { emptyBoard } from '../shared/board.js';

const question = {
  title: 'Design X', difficulty: 'Medium', minutes: 45, prompt: 'Build X.', assumptions: ['a'], level: 'senior', style: 'balanced',
};
const board = { nextId: 3, items: [{ id: 1, type: 'rect', x: 0, y: 0, w: 100, h: 60, text: 'API' }, { id: 2, type: 'database', x: 200, y: 0, w: 100, h: 80, text: 'DB' }] };

// A fake SDK client that records every request.
function fakeClient({ deltas = ['Hello', ' there'], stop = 'end_turn', createText = '{}', createStop = 'end_turn' } = {}) {
  const calls = { stream: [], create: [] };
  return {
    calls,
    messages: {
      stream(params, opts) {
        calls.stream.push({ params, opts });
        const events = deltas.map((text) => ({ type: 'content_block_delta', delta: { type: 'text_delta', text } }));
        return {
          async *[Symbol.asyncIterator]() { yield { type: 'message_start' }; yield* events; },
          finalMessage: async () => ({ stop_reason: stop, content: [{ type: 'text', text: deltas.join('') }] }),
          abort() { calls.aborted = true; },
        };
      },
      async create(params) {
        calls.create.push(params);
        return { stop_reason: createStop, content: [{ type: 'text', text: createText }] };
      },
    },
  };
}

const run = async (gen) => { let out = ''; for await (const t of gen) out += t; return out; };
const chatArgs = (extra = {}) => ({
  question, board, elapsedMs: 120_000, status: 'running', mode: 'chat',
  messages: [{ role: 'assistant', content: 'welcome' }, { role: 'user', content: 'What is the scale?' }], ...extra,
});

test('chat streams text deltas and sends a well-formed request', async () => {
  const client = fakeClient();
  const ai = createAi({ client, model: 'claude-opus-5-5' });
  assert.equal(await run(ai.chat(chatArgs())), 'Hello there');
  const { params } = client.calls.stream[0];
  assert.equal(params.model, 'claude-opus-5-5');
  assert.ok(params.max_tokens >= 1000);
  assert.equal(params.system[0].type, 'text');
  assert.deepEqual(params.system[0].cache_control, { type: 'ephemeral' });
  assert.match(params.system[0].text, /QUESTION: Design X/);
  assert.doesNotMatch(params.system[0].text, /HIDDEN RUBRIC/);
  assert.equal(params.messages[0].role, 'user', 'leading assistant greeting is dropped');
  const last = params.messages[params.messages.length - 1];
  assert.equal(last.role, 'user');
  assert.match(last.content, /What is the scale\?/);
  assert.match(last.content, /2 component\(s\)/);
  assert.match(last.content, /"API"/);
  assert.deepEqual(params.output_config, { effort: 'low' });
});

test('requests never use parameters the current models reject', async () => {
  const client = fakeClient({ createText: JSON.stringify({ speak: false, message: '' }) });
  const ai = createAi({ client });
  await run(ai.chat(chatArgs()));
  await ai.watch({ question, board, elapsedMs: 0, status: 'running', messages: [] });
  for (const p of [client.calls.stream[0].params, client.calls.create[0]]) {
    for (const banned of ['temperature', 'top_p', 'top_k', 'budget_tokens', 'tool_choice', 'prefill']) assert.ok(!(banned in p), banned);
    assert.ok(!('thinking' in p) || p.thinking.type === 'adaptive');
  }
});

test('effort setting: default per task, explicit override, or none', async () => {
  const check = async (effort) => {
    const client = fakeClient({ createText: JSON.stringify({ verdict: 'Good', summary: 's', areas: [], strengths: [], gaps: [], next_steps: [] }) });
    const ai = createAi({ client, effort });
    await run(ai.chat(chatArgs()));
    await ai.evaluate({ question, board, elapsedMs: 1, messages: [] });
    return [client.calls.stream[0].params.output_config?.effort, client.calls.create[0].output_config.effort];
  };
  assert.deepEqual(await check('default'), ['low', 'high']);
  assert.deepEqual(await check('medium'), ['medium', 'medium']);
  assert.deepEqual(await check('none'), [undefined, undefined]);
});

test('chat refuses to run without a user turn', async () => {
  const ai = createAi({ client: fakeClient() });
  await assert.rejects(run(ai.chat(chatArgs({ messages: [{ role: 'assistant', content: 'hi' }] }))), AriaError);
});

test('a model refusal surfaces as a friendly error after streaming', async () => {
  const ai = createAi({ client: fakeClient({ stop: 'refusal' }) });
  await assert.rejects(run(ai.chat(chatArgs())), (e) => e instanceof AriaError && /declined/.test(e.message));
});

test('aborting the request signal aborts the upstream stream', async () => {
  const client = fakeClient();
  const ai = createAi({ client });
  const ctl = new AbortController();
  const gen = ai.chat(chatArgs({ signal: ctl.signal }));
  await gen.next();
  ctl.abort();
  assert.equal(client.calls.aborted, true);
  assert.equal(client.calls.stream[0].opts.signal, ctl.signal);
});

test('watch uses structured output and stays quiet unless told to speak', async () => {
  const speak = fakeClient({ createText: JSON.stringify({ speak: true, message: ' What about the cache? ' }) });
  const out = await createAi({ client: speak }).watch({ question, board, elapsedMs: 1, status: 'running', messages: [], sinceAriaMs: 90_000 });
  assert.deepEqual(out, { speak: true, message: 'What about the cache?' });
  const req = speak.calls.create[0];
  assert.equal(req.output_config.format.type, 'json_schema');
  assert.equal(req.output_config.format.schema.additionalProperties, false);
  assert.match(req.messages[0].content, /Silent check-in/);
  assert.match(req.messages[0].content, /01:30 ago/);

  const empty = fakeClient({ createText: JSON.stringify({ speak: true, message: '   ' }) });
  assert.equal((await createAi({ client: empty }).watch({ question, board, elapsedMs: 1, status: 'running', messages: [] })).speak, false);
  const junk = fakeClient({ createText: 'not json' });
  assert.deepEqual(await createAi({ client: junk }).watch({ question, board, elapsedMs: 1, status: 'running', messages: [] }), { speak: false, message: '' });
  const refused = fakeClient({ createStop: 'refusal' });
  assert.equal((await createAi({ client: refused }).watch({ question, board, elapsedMs: 1, status: 'running', messages: [] })).speak, false);
});

test('evaluate returns a cleaned, complete report and includes transcript and final board', async () => {
  const raw = {
    verdict: 'Strong', summary: 'Solid.', strengths: ['good'], gaps: [], next_steps: ['practise'],
    areas: [{ name: 'Architecture', score: 9, comment: 'nice' }, { name: 'Scaling', score: -2, comment: 'none' }],
  };
  const client = fakeClient({ createText: JSON.stringify(raw) });
  const out = await createAi({ client }).evaluate({ question, board, elapsedMs: 600_000, messages: [{ role: 'user', content: 'Let us start with requirements' }] });
  assert.equal(out.areas.length, 7);
  assert.equal(out.areas.find((a) => a.name === 'Architecture').score, 5, 'scores are clamped');
  assert.equal(out.areas.find((a) => a.name === 'Scaling').score, 0);
  assert.equal(out.areas.find((a) => a.name === 'Requirements').score, 0, 'missing areas default to zero');
  const content = client.calls.create[0].messages[0].content;
  assert.match(content, /Candidate: Let us start with requirements/);
  assert.match(content, /"API"/);
  assert.equal(client.calls.create[0].output_config.format.schema.properties.verdict.enum.length, 4);
});

test('evaluate reports unusable output and refusals as errors', async () => {
  await assert.rejects(createAi({ client: fakeClient({ createText: '<<<' }) }).evaluate({ question, board: emptyBoard(), elapsedMs: 0, messages: [] }), AriaError);
  await assert.rejects(createAi({ client: fakeClient({ createStop: 'refusal' }) }).evaluate({ question, board: emptyBoard(), elapsedMs: 0, messages: [] }), AriaError);
});

test('cleanEvaluation falls back safely', () => {
  const e = cleanEvaluation({ verdict: 'Legendary', summary: 5, areas: 'x', strengths: [1, 'ok', ' '], gaps: null });
  assert.equal(e.verdict, 'Mixed');
  assert.equal(e.summary, '');
  assert.deepEqual(e.strengths, ['ok']);
  assert.deepEqual(e.gaps, []);
  assert.equal(e.areas.length, 7);
});

test('friendlyError maps SDK errors to safe messages and never echoes secrets', () => {
  const mk = (status, message = 'secret sk-ant-123') => Anthropic.APIError.generate(status, { error: { message } }, message, new Headers());
  assert.equal(friendlyError(mk(401)).status, 401);
  assert.match(friendlyError(mk(401)).message, /API key was rejected/);
  assert.equal(friendlyError(mk(429)).status, 429);
  assert.equal(friendlyError(mk(403)).status, 403);
  assert.match(friendlyError(mk(404)).message, /model was not found/);
  assert.equal(friendlyError(mk(500)).status, 502);
  assert.equal(friendlyError(new Error('boom sk-ant-xyz')).status, 500);
  for (const e of [mk(401), mk(429), mk(500), new Error('sk-ant-xyz')]) assert.doesNotMatch(friendlyError(e).message, /sk-ant/);
  assert.equal(friendlyError(new AriaError('custom', 418)).status, 418);
});

// Claude integration: streaming chat, silent watching, and the end-of-interview evaluation.
import Anthropic from '@anthropic-ai/sdk';
import {
  buildContext, chatSystem, evalSystem, normaliseMessages, transcriptText, watchSystem, withContext, formatClock,
} from './prompts.js';
import {
  explainContext, explainSystem, generateSystem, generateUser, GENERATE_SCHEMA, normaliseGenerated,
} from './quiz.js';

export const DEFAULT_MODEL = 'claude-sonnet-5-5';
// Models the browser may choose between (sent in the x-hld-model header). Anything else is ignored.
export const MODELS = [
  { id: 'claude-sonnet-5-5', label: 'Sonnet 5.5', note: 'Balanced speed, quality and cost (default)' },
  { id: 'claude-opus-5-5', label: 'Opus 5.5', note: 'Strongest answers, highest cost' },
  { id: 'claude-haiku-5-5', label: 'Haiku 5.5', note: 'Fastest and cheapest' },
  { id: 'claude-fable-5-1', label: 'Fable 5.1', note: 'Most capable of the family' },
];

export class AriaError extends Error {
  constructor(message, status = 502) {
    super(message);
    this.name = 'AriaError';
    this.status = status;
  }
}

const EVAL_AREAS = ['Requirements', 'Estimation', 'API design', 'Data model', 'Architecture', 'Scaling', 'Trade-offs'];

export const WATCH_SCHEMA = {
  type: 'object',
  properties: {
    speak: { type: 'boolean' },
    message: { type: 'string' },
  },
  required: ['speak', 'message'],
  additionalProperties: false,
};

export const EVAL_SCHEMA = {
  type: 'object',
  properties: {
    verdict: { type: 'string', enum: ['Strong', 'Good', 'Mixed', 'Needs work'] },
    summary: { type: 'string' },
    areas: {
      type: 'array',
      items: {
        type: 'object',
        properties: { name: { type: 'string' }, score: { type: 'integer' }, comment: { type: 'string' } },
        required: ['name', 'score', 'comment'],
        additionalProperties: false,
      },
    },
    strengths: { type: 'array', items: { type: 'string' } },
    gaps: { type: 'array', items: { type: 'string' } },
    next_steps: { type: 'array', items: { type: 'string' } },
  },
  required: ['verdict', 'summary', 'areas', 'strengths', 'gaps', 'next_steps'],
  additionalProperties: false,
};

// Map any SDK / network failure to something safe to show the user.
export function friendlyError(err) {
  if (err instanceof AriaError) return { status: err.status, message: err.message };
  if (err instanceof Anthropic.AuthenticationError) return { status: 401, message: 'The Anthropic API key was rejected. Check the key in Settings (or ANTHROPIC_API_KEY in hld/.env).' };
  if (err instanceof Anthropic.PermissionDeniedError) return { status: 403, message: 'This API key is not allowed to use the configured model.' };
  if (err instanceof Anthropic.NotFoundError) return { status: 502, message: 'The configured model was not found. Check HLD_MODEL.' };
  if (err instanceof Anthropic.RateLimitError) return { status: 429, message: 'Anthropic rate limit reached. Wait a few seconds and try again.' };
  if (err instanceof Anthropic.BadRequestError && /credit balance/i.test(String(err.message))) return { status: 402, message: 'Your Anthropic account is out of credit, so the API refused the request. Add credit under Plans & Billing at console.anthropic.com (an API key and a Claude chat subscription are billed separately), then try again. Demo mode (npm run demo) works without credit.' };
  if (err instanceof Anthropic.BadRequestError) return { status: 502, message: 'The request was rejected by the API (' + String(err.message).slice(0, 160) + ').' };
  if (err instanceof Anthropic.APIConnectionError) return { status: 502, message: 'Could not reach the Anthropic API. Check your network connection.' };
  if (err instanceof Anthropic.APIError) return { status: 502, message: 'The Anthropic API returned an error (' + (err.status ?? 'unknown') + ').' };
  return { status: 500, message: 'Something went wrong talking to the AI.' };
}

function effortFor(kind, setting) {
  if (setting === 'none') return undefined;
  if (setting && setting !== 'default') return setting;
  if (kind === 'evaluate') return 'high';
  if (kind === 'generate') return 'medium';
  return 'low';
}

function cachedSystem(text) {
  return [{ type: 'text', text, cache_control: { type: 'ephemeral' } }];
}

function textOf(message) {
  return (message.content || []).filter((b) => b.type === 'text').map((b) => b.text).join('');
}

function checkStop(message) {
  if (message.stop_reason === 'refusal') throw new AriaError('Aria declined to answer that. Try rephrasing your question.', 422);
}

export function createAi({ client, model = DEFAULT_MODEL, effort = 'default' }) {
  function request(kind, system, messages, maxTokens, extra = {}) {
    const e = effortFor(kind, effort);
    const output_config = { ...(extra.output_config || {}) };
    if (e) output_config.effort = e;
    const params = { model, max_tokens: maxTokens, system: cachedSystem(system), messages, ...extra };
    if (Object.keys(output_config).length) params.output_config = output_config;
    else delete params.output_config;
    return params;
  }

  // Async generator of text deltas. Callers can pass an AbortSignal to stop the upstream request.
  async function* chat({ question, messages, board, elapsedMs, status, mode, signal }) {
    const history = normaliseMessages(messages);
    if (!history.length || history[history.length - 1].role !== 'user') throw new AriaError('Say something to Aria first.', 400);
    const context = buildContext({ board, elapsedMs, totalMs: question.minutes * 60000, status, mode });
    const params = request('chat', chatSystem(question), withContext(history, context), 4000);
    const stream = client.messages.stream(params, signal ? { signal } : undefined);
    if (signal) signal.addEventListener('abort', () => stream.abort?.(), { once: true });
    for await (const event of stream) {
      if (event.type === 'content_block_delta' && event.delta.type === 'text_delta') yield event.delta.text;
    }
    checkStop(await stream.finalMessage());
  }

  async function watch({ question, messages, board, elapsedMs, status, sinceAriaMs }) {
    const history = normaliseMessages(messages);
    const recent = history.slice(-6);
    const extra = `Aria last spoke ${sinceAriaMs == null ? 'never' : formatClock(sinceAriaMs) + ' ago'}. Recent conversation:\n${transcriptText(recent)}`;
    const context = buildContext({ board, elapsedMs, totalMs: question.minutes * 60000, status, mode: 'chat', extra });
    const messagesOut = [{ role: 'user', content: `Silent check-in. Should you say something right now?\n\n${context}` }];
    const params = request('watch', watchSystem(question), messagesOut, 2000, {
      output_config: { format: { type: 'json_schema', schema: WATCH_SCHEMA } },
    });
    const response = await client.messages.create(params);
    if (response.stop_reason === 'refusal') return { speak: false, message: '' };
    try {
      const parsed = JSON.parse(textOf(response));
      const message = typeof parsed.message === 'string' ? parsed.message.trim().slice(0, 600) : '';
      return { speak: parsed.speak === true && message.length > 0, message };
    } catch {
      return { speak: false, message: '' };
    }
  }

  async function evaluate({ question, messages, board, elapsedMs }) {
    const history = normaliseMessages(messages);
    const context = buildContext({ board, elapsedMs, totalMs: question.minutes * 60000, status: 'ended', mode: 'chat' });
    const content = `Write the feedback report.\n\nTranscript:\n${transcriptText(history)}\n\nFinal state:\n${context}`;
    const params = request('evaluate', evalSystem(question), [{ role: 'user', content }], 8000, {
      output_config: { format: { type: 'json_schema', schema: EVAL_SCHEMA } },
    });
    const response = await client.messages.create(params);
    checkStop(response);
    let parsed;
    try {
      parsed = JSON.parse(textOf(response));
    } catch {
      throw new AriaError('The feedback report came back in an unexpected format. Try ending the interview again.', 502);
    }
    return cleanEvaluation(parsed);
  }

  // Streams a tutor-style explanation of a quiz question; `history` carries earlier follow-up turns (assistant first).
  async function* explain({ question, selected, history, signal }) {
    const messages = [{ role: 'user', content: `${explainContext(question, selected)}\n\nExplain this to me.` }];
    for (const m of Array.isArray(history) ? history.slice(-12) : []) {
      if (!m || (m.role !== 'user' && m.role !== 'assistant') || typeof m.content !== 'string' || !m.content.trim()) continue;
      const content = m.content.trim().slice(0, 4000);
      const last = messages[messages.length - 1];
      if (last.role === m.role) last.content += `\n\n${content}`;
      else messages.push({ role: m.role, content });
    }
    if (messages[messages.length - 1].role !== 'user') throw new AriaError('Ask a question first.', 400);
    const params = request('explain', explainSystem(), messages, 3000);
    const stream = client.messages.stream(params, signal ? { signal } : undefined);
    if (signal) signal.addEventListener('abort', () => stream.abort?.(), { once: true });
    for await (const event of stream) {
      if (event.type === 'content_block_delta' && event.delta.type === 'text_delta') yield event.delta.text;
    }
    checkStop(await stream.finalMessage());
  }

  async function generateQuiz({ topic, notes, count, level }) {
    const params = request('generate', generateSystem(), [{ role: 'user', content: generateUser({ topic, notes, count, level }) }], 16000, {
      output_config: { format: { type: 'json_schema', schema: GENERATE_SCHEMA } },
    });
    const response = await client.messages.create(params);
    checkStop(response);
    let parsed;
    try {
      parsed = JSON.parse(textOf(response));
    } catch {
      throw new AriaError('The quiz came back in an unexpected format. Please try again.', 502);
    }
    try {
      return normaliseGenerated(parsed, { topic, requested: count });
    } catch (err) {
      throw new AriaError(`${err.message} Please try again.`, 502);
    }
  }

  return { chat, watch, evaluate, explain, generateQuiz, model };
}

const list = (v, n) => (Array.isArray(v) ? v.filter((s) => typeof s === 'string' && s.trim()).map((s) => s.trim().slice(0, 400)).slice(0, n) : []);

export function cleanEvaluation(e) {
  const byName = new Map((Array.isArray(e.areas) ? e.areas : []).map((a) => [String(a?.name || ''), a]));
  const areas = EVAL_AREAS.map((name) => {
    const a = byName.get(name);
    const score = Number.isFinite(a?.score) ? Math.max(0, Math.min(5, Math.round(a.score))) : 0;
    return { name, score, comment: typeof a?.comment === 'string' ? a.comment.trim().slice(0, 500) : '' };
  });
  const verdicts = ['Strong', 'Good', 'Mixed', 'Needs work'];
  return {
    verdict: verdicts.includes(e.verdict) ? e.verdict : 'Mixed',
    summary: typeof e.summary === 'string' ? e.summary.trim().slice(0, 900) : '',
    areas,
    strengths: list(e.strengths, 6),
    gaps: list(e.gaps, 6),
    next_steps: list(e.next_steps, 6),
  };
}

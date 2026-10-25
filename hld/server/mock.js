// Offline stand-in for the Anthropic client, used by `npm run demo` (HLD_MOCK=1) and by the tests.
// It is deterministic and reads the same prompts the real model would, so the whole app can be exercised without an API key.

function lastUserText(params) {
  const msgs = params.messages || [];
  const last = msgs[msgs.length - 1];
  if (!last) return '';
  return typeof last.content === 'string' ? last.content : last.content.map((b) => b.text || '').join('');
}

function parseContext(text) {
  const stats = /(\d+) component\(s\), (\d+) connection\(s\)/.exec(text);
  const candidate = text.split('<context>')[0].trim();
  const mode = /Mode: The candidate pressed "([^"]+)"/.exec(text)?.[1] || '';
  const isolated = /Components with no arrows attached: (.+)/.exec(text)?.[1];
  const empty = text.includes('The whiteboard is empty.');
  return {
    candidate,
    mode,
    components: stats ? Number(stats[1]) : 0,
    connections: stats ? Number(stats[2]) : 0,
    isolated,
    empty,
  };
}

export function mockReply(params) {
  const c = parseContext(lastUserText(params));
  const board = c.empty ? 'I can see an empty whiteboard.' : `I can see ${c.components} component(s) and ${c.connections} connection(s) on your board.`;
  if (c.mode === 'Give me a hint') return `Mock hint: think about what happens to your design when traffic grows ten times. ${board}`;
  if (c.mode === 'Review my board') return `Mock review: ${board} What is the first thing that would break under load?`;
  if (c.mode === 'Clarify requirements') return `Mock clarification: pin down the scale, the read/write ratio and the latency target first. ${board}`;
  return `Mock Aria here. You said: "${c.candidate.slice(0, 120)}". ${board}`;
}

function mockWatch(params) {
  const c = parseContext(lastUserText(params));
  if (c.isolated) return { speak: true, message: `Mock watch: ${c.isolated.split(',')[0].trim()} has no arrows yet - how does it talk to the rest of the system?` };
  return { speak: false, message: '' };
}

function mockEvaluation(params) {
  const c = parseContext(lastUserText(params).split('Final state:')[1] || '');
  const score = Math.min(5, c.components);
  const areas = ['Requirements', 'Estimation', 'API design', 'Data model', 'Architecture', 'Scaling', 'Trade-offs'];
  return {
    verdict: c.components >= 4 ? 'Good' : 'Needs work',
    summary: `Mock report: the final board had ${c.components} component(s) and ${c.connections} connection(s).`,
    areas: areas.map((name) => ({ name, score: name === 'Architecture' ? score : 1, comment: `Mock comment on ${name.toLowerCase()}.` })),
    strengths: c.components ? ['You put components on the board.'] : [],
    gaps: ['Mock mode gives canned feedback only.'],
    next_steps: ['Run with a real ANTHROPIC_API_KEY for real feedback.'],
  };
}

function isTutor(params) {
  const sys = params.system;
  const text = Array.isArray(sys) ? sys.map((b) => b.text || '').join('') : String(sys || '');
  return text.includes('quiz tutor');
}

function mockExplain(params) {
  const text = lastUserText(params);
  const concept = /Concept: (.+)/.exec(text)?.[1] || 'this concept';
  const chose = /Learner chose: (.+)/.exec(text)?.[1] || '(nothing)';
  if (!text.includes('Explain this to me.')) return `Mock tutor follow-up: ${text.slice(0, 80)}`;
  return `Mock tutor: let us look at ${concept}. You chose ${chose}. The reference explanation is the key idea to remember.`;
}

function mockQuiz(params) {
  const content = lastUserText(params);
  const topic = /Topic: (.+)/.exec(content)?.[1] || 'Mock topic';
  const count = Number(/exactly (\d+) questions/.exec(content)?.[1]) || 5;
  const questions = Array.from({ length: count }, (_, i) => ({
    concept: `Mock concept ${(i % 3) + 1}`,
    type: 'single',
    difficulty: (i % 3) + 1,
    prompt: `Mock question ${i + 1} about ${topic}: which option is correct?`,
    options: ['The right answer', 'A plausible distractor', 'Another distractor', 'A third distractor'].map((text, k) => ({ text: `${text} (${i + 1})`, why: k === 0 ? 'This is the right one.' : 'This one is wrong.' })),
    correct_indexes: [i % 4],
    explanation: 'Mock explanation for the generated question.',
  }));
  // keep the correct option text consistent with its position
  for (const q of questions) {
    const c = q.correct_indexes[0];
    [q.options[0], q.options[c]] = [q.options[c], q.options[0]];
  }
  return { title: `Mock quiz: ${topic.slice(0, 40)}`, description: 'A canned quiz from demo mode.', questions };
}

export function createMockClient({ delayMs = 3 } = {}) {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  return {
    messages: {
      stream(params) {
        const text = isTutor(params) ? mockExplain(params) : mockReply(params);
        let aborted = false;
        const iterator = (async function* () {
          for (const word of text.split(/(?<= )/)) {
            if (aborted) return;
            await sleep(delayMs);
            yield { type: 'content_block_delta', delta: { type: 'text_delta', text: word } };
          }
        })();
        return {
          [Symbol.asyncIterator]: () => iterator,
          finalMessage: async () => ({ stop_reason: 'end_turn', content: [{ type: 'text', text }] }),
          abort: () => { aborted = true; },
        };
      },
      async create(params) {
        const props = params.output_config?.format?.schema?.properties || {};
        const body = 'speak' in props ? mockWatch(params) : 'questions' in props ? mockQuiz(params) : mockEvaluation(params);
        return { stop_reason: 'end_turn', content: [{ type: 'text', text: JSON.stringify(body) }] };
      },
    },
  };
}

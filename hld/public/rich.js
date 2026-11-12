// Question text with code in it: ```lang fenced blocks become snippet cards (language label, syntax colours, copy button) and `inline code` becomes a
// monospace chip. Everything is built with DOM nodes (never innerHTML) because quiz text can come from an AI.

const FENCE = /```([A-Za-z0-9+#_.-]*)[ \t]*\n([\s\S]*?)\n?```/g;
const INLINE = /`([^`\n]+)`/g;

const KW = {
  java: 'abstract assert boolean break byte case catch char class const continue default do double else enum extends final finally float for goto if implements import instanceof int interface long native new package private protected public record return sealed permits short static strictfp super switch synchronized this throw throws transient try var void volatile while yield null true false',
  python: 'and as assert async await break class continue def del elif else except finally for from global if import in is lambda nonlocal not or pass raise return try while with yield None True False self cls match case',
  sql: 'select from where group by having order limit offset join inner left right full outer cross on as and or not in exists between like is null insert into values update set delete create table index unique primary key foreign references alter add drop column constraint default union all distinct case when then else end with recursive over partition rows range unbounded preceding following current begin commit rollback savepoint lock for share nowait skip locked using explain analyze true false asc desc count sum avg min max',
  js: 'async await break case catch class const continue default delete do else export extends finally for function if import in instanceof let new of return static super switch this throw try typeof var void while yield null undefined true false',
  bash: 'if then else elif fi for while do done case esac function in echo export cd ls cat grep awk sed curl docker kubectl git npm pip python java mvn redis-cli psql kafka-topics',
  yaml: 'true false null yes no',
  json: 'true false null',
  http: 'GET POST PUT PATCH DELETE HEAD OPTIONS HTTP',
};
const ALIAS = { py: 'python', python3: 'python', javascript: 'js', typescript: 'js', ts: 'js', sh: 'bash', shell: 'bash', zsh: 'bash', yml: 'yaml', postgres: 'sql', postgresql: 'sql', mysql: 'sql', mongosh: 'js', kotlin: 'java', c: 'java', cpp: 'java' };
const COMMENT = { java: '//[^\\n]*|/\\*[\\s\\S]*?\\*/', js: '//[^\\n]*|/\\*[\\s\\S]*?\\*/', python: '#[^\\n]*', bash: '#[^\\n]*', yaml: '#[^\\n]*', sql: '--[^\\n]*|/\\*[\\s\\S]*?\\*/' };
const cache = new Map();

function tokenizer(lang) {
  if (cache.has(lang)) return cache.get(lang);
  const kws = (KW[lang] || '').split(' ').filter(Boolean);
  const parts = [];
  if (COMMENT[lang]) parts.push(`(?<c>${COMMENT[lang]})`);
  parts.push(lang === 'python' ? '(?<s>"""[\\s\\S]*?"""|\'\'\'[\\s\\S]*?\'\'\'|[fFrRbB]?"(?:\\\\.|[^"\\\\\\n])*"|[fFrRbB]?\'(?:\\\\.|[^\'\\\\\\n])*\')' : '(?<s>"(?:\\\\.|[^"\\\\\\n])*"|\'(?:\\\\.|[^\'\\\\\\n])*\'|`(?:\\\\.|[^`\\\\])*`)');
  parts.push('(?<a>@[A-Za-z_][\\w.]*)');
  parts.push('(?<n>\\b\\d[\\d_]*(?:\\.\\d+)?(?:[eE][+-]?\\d+)?[LlFfDd]?\\b)');
  if (kws.length) parts.push(`(?<k>\\b(?:${kws.map((k) => k.replace(/[-.]/g, '\\$&')).join('|')})\\b)`);
  if (lang === 'java' || lang === 'js' || lang === 'python') parts.push('(?<t>\\b[A-Z][A-Za-z0-9_]*\\b)');
  if (lang === 'sql') parts.push('(?<f>\\b[A-Za-z_]+(?=\\())');
  const re = new RegExp(parts.join('|'), lang === 'sql' || lang === 'http' ? 'gi' : 'g');
  cache.set(lang, re);
  return re;
}

export function highlight(h, code, langName) {
  const lang = ALIAS[langName] || langName;
  if (!KW[lang] && !COMMENT[lang]) return [code];
  const re = new RegExp(tokenizer(lang));
  const out = [];
  let last = 0;
  for (const m of code.matchAll(re)) {
    if (m.index > last) out.push(code.slice(last, m.index));
    const g = m.groups;
    const kind = g.c != null ? 'c' : g.s != null ? 's' : g.a != null ? 'a' : g.n != null ? 'n' : g.k != null ? 'k' : g.t != null ? 't' : 'f';
    out.push(h('span', { class: `tok-${kind}`, text: m[0] }));
    last = m.index + m[0].length;
  }
  if (last < code.length) out.push(code.slice(last));
  return out;
}

const LABEL = { java: 'Java', python: 'Python', py: 'Python', sql: 'SQL', js: 'JavaScript', javascript: 'JavaScript', yaml: 'YAML', yml: 'YAML', json: 'JSON', bash: 'Shell', sh: 'Shell', http: 'HTTP', text: 'Pseudocode', pseudocode: 'Pseudocode', mongosh: 'mongosh', xml: 'XML' };

function copyButton(h, code) {
  const btn = h('button', { class: 'snip-copy', type: 'button', 'aria-label': 'Copy code', text: 'Copy' });
  btn.addEventListener('click', async (e) => {
    e.stopPropagation();
    try { await navigator.clipboard.writeText(code); btn.textContent = 'Copied'; } catch { btn.textContent = 'Press Ctrl+C'; }
    setTimeout(() => { btn.textContent = 'Copy'; }, 1500);
  });
  return btn;
}

function snippet(h, lang, code) {
  const name = (lang || '').toLowerCase();
  return h('figure', { class: 'snip' },
    h('figcaption', {}, h('span', { class: 'snip-lang', text: LABEL[name] || (name ? name : 'Code') }), copyButton(h, code)),
    h('pre', { tabindex: '0' }, h('code', {}, ...highlight(h, code, name))));
}

function inline(h, text) {
  const out = [];
  let last = 0;
  for (const m of text.matchAll(INLINE)) {
    if (m.index > last) out.push(text.slice(last, m.index));
    out.push(h('code', { class: 'ic', text: m[1] }));
    last = m.index + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

// Returns an array of nodes: paragraphs (blank-line separated), snippet cards, and inline code chips. Plain text without any of that stays one text node
// inside a single <span> so short options and prompts look exactly as before.
export function richText(h, text) {
  const src = String(text ?? '');
  if (!src.includes('`')) return [src];
  const out = [];
  let last = 0;
  const flushProse = (chunk) => {
    const paras = chunk.split(/\n{2,}/).map((p) => p.replace(/^\n+|\n+$/g, '')).filter((p) => p.trim());
    if (paras.length === 1 && !out.length && last === 0 && chunk === src) { out.push(...inline(h, paras[0])); return; }
    for (const p of paras) out.push(h('p', { class: 'rt-p' }, ...inline(h, p)));
  };
  for (const m of src.matchAll(FENCE)) {
    if (m.index > last) flushProse(src.slice(last, m.index));
    out.push(snippet(h, m[1], m[2]));
    last = m.index + m[0].length;
  }
  if (last < src.length) flushProse(src.slice(last));
  return out;
}

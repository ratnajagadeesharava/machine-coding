// Lets the user paste their own Claude API key. It lives only in this browser (localStorage) and is sent to the
// local server in an x-anthropic-key header on /api/ calls; the server never stores or logs it.
// Import it for its side effects: it adds a "Claude key" button to the page header and wraps fetch.
const STORE = 'hld-anthropic-key';
const KEY_RE = /^sk-ant-[A-Za-z0-9_-]{10,400}$/;

export function getKey() { try { return localStorage.getItem(STORE) || ''; } catch { return ''; } }
export function setKey(k) { try { if (k) localStorage.setItem(STORE, k); else localStorage.removeItem(STORE); return true; } catch { return false; } }
export const looksLikeKey = (k) => KEY_RE.test(k);
const mask = (k) => (k.length > 14 ? `${k.slice(0, 7)}...${k.slice(-4)}` : '');

const realFetch = window.fetch.bind(window);
window.fetch = (input, init = {}) => {
  const key = getKey();
  const url = typeof input === 'string' ? input : input && input.url;
  if (key && typeof url === 'string' && url.startsWith('/api/')) {
    const headers = new Headers(init.headers || (input && input.headers) || {});
    headers.set('x-anthropic-key', key);
    return realFetch(input, { ...init, headers });
  }
  return realFetch(input, init);
};

function h(tag, attrs = {}, ...kids) {
  const n = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) { if (k === 'text') n.textContent = v; else if (k.startsWith('on')) n.addEventListener(k.slice(2), v); else n.setAttribute(k, v); }
  n.append(...kids);
  return n;
}

function openDialog(trigger) {
  const dlg = h('dialog', { class: 'keydlg', 'aria-labelledby': 'keydlg-title' });
  const input = h('input', { type: 'password', id: 'keydlg-input', autocomplete: 'off', spellcheck: 'false', placeholder: 'sk-ant-...' });
  const msg = h('p', { class: 'keydlg-msg', role: 'status' });
  const show = h('input', { type: 'checkbox', id: 'keydlg-show' });
  show.addEventListener('change', () => { input.type = show.checked ? 'text' : 'password'; });
  const cur = getKey();
  const status = h('p', { class: 'keydlg-cur', text: cur ? `Saved in this browser: ${mask(cur)}` : 'No key saved in this browser.' });
  const save = h('button', { type: 'button', class: 'btn primary', text: 'Save key' });
  const remove = h('button', { type: 'button', class: 'btn', text: 'Remove' });
  const close = h('button', { type: 'button', class: 'btn', text: 'Close' });
  if (!cur) remove.disabled = true;
  save.addEventListener('click', () => {
    const v = input.value.trim();
    if (!looksLikeKey(v)) { msg.textContent = 'That does not look like an Anthropic key. It should start with sk-ant-.'; msg.className = 'keydlg-msg err'; input.focus(); return; }
    if (!setKey(v)) { msg.textContent = 'This browser blocked local storage, so the key cannot be saved.'; msg.className = 'keydlg-msg err'; return; }
    location.reload();
  });
  remove.addEventListener('click', () => { setKey(''); location.reload(); });
  close.addEventListener('click', () => dlg.close());
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); save.click(); } });
  dlg.append(
    h('h2', { id: 'keydlg-title', text: 'Claude API key' }),
    h('p', { class: 'muted', text: 'Paste your own key to use the AI features. It is stored only in this browser and sent to the local HLD Room server with each request. It is never written to disk by the server. If the server already has a key in .env, that one is used instead.' }),
    status,
    h('label', { for: 'keydlg-input', text: 'API key' }), input,
    h('label', { class: 'keydlg-show', for: 'keydlg-show' }, show, ' Show key'),
    msg,
    h('div', { class: 'keydlg-actions' }, remove, close, save),
  );
  dlg.addEventListener('close', () => { dlg.remove(); trigger.focus(); });
  document.body.append(dlg);
  dlg.showModal();
  input.focus();
}

function mountButton() {
  const bar = document.querySelector('.topbar');
  if (!bar || bar.querySelector('[data-keybtn]')) return;
  let actions = bar.querySelector('.actions');
  if (!actions) { actions = h('div', { class: 'actions' }); bar.append(actions); }
  const has = !!getKey();
  const b = h('button', { type: 'button', class: 'btn keybtn' + (has ? ' has-key' : ''), 'data-keybtn': '', title: 'Set your Claude API key', 'aria-label': 'Claude API key' });
  b.innerHTML = '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="8" cy="15" r="4"/><path d="M11 12l9-9M16 7l3 3M14 9l2 2"/></svg>';
  b.append(h('span', { text: has ? 'Key set' : 'Add key' }));
  b.addEventListener('click', () => openDialog(b));
  actions.prepend(b);
}
if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mountButton); else mountButton();

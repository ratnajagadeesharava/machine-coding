// Small DOM helpers for the Quizzes page. Everything is built with DOM nodes (never innerHTML for text) because quiz text can come from an AI.

export function h(tag, props = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'text') el.textContent = v;
    else if (k === 'dataset') Object.assign(el.dataset, v);
    else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
    else if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? '' : String(v));
  }
  for (const kid of kids.flat(Infinity)) if (kid != null && kid !== false) el.append(kid);
  return el;
}
export const $ = (sel, root = document) => root.querySelector(sel);

// Icons: fixed strings written here (not user text), so innerHTML is safe.
const ICONS = {
  x: '<path d="M6 6l12 12M18 6L6 18"/>',
  check: '<path d="M5 12.5l4.5 4.5L19 7.5"/>',
  flag: '<path d="M5 21V4M5 4h11l-2 4 2 4H5"/>',
  spark: '<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/><path d="M19 16l.7 2 2 .7-2 .7-.7 2-.7-2-2-.7 2-.7z"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>',
  chevron: '<path d="M9 6l6 6-6 6"/>',
  back: '<path d="M15 6l-6 6 6 6"/>',
  more: '<circle cx="5" cy="12" r="1.2"/><circle cx="12" cy="12" r="1.2"/><circle cx="19" cy="12" r="1.2"/>',
  clock: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l2.5 2M9 2h6"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  play: '<path d="M7 5l12 7-12 7z"/>',
  bolt: '<path d="M13 2L4 14h7l-1 8 9-12h-7z"/>',
  target: '<circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="5"/><circle cx="12" cy="12" r="1"/>',
  db: '<ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v14c0 1.7 3.6 3 8 3s8-1.3 8-3V5M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3"/>',
  home: '<path d="M3 11l9-8 9 8M5 10v10h14V10"/>',
  list: '<path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"/>',
  retry: '<path d="M3 12a9 9 0 1 0 3-6.7M3 4v5h5"/>',
  next: '<path d="M5 12h14M13 6l6 6-6 6"/>',
};
export function icon(name, size = 18, cls = '') {
  const s = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  s.setAttribute('viewBox', '0 0 24 24'); s.setAttribute('width', String(size)); s.setAttribute('height', String(size));
  s.setAttribute('fill', 'none'); s.setAttribute('stroke', 'currentColor'); s.setAttribute('stroke-width', '2');
  s.setAttribute('stroke-linecap', 'round'); s.setAttribute('stroke-linejoin', 'round'); s.setAttribute('aria-hidden', 'true');
  if (cls) s.setAttribute('class', cls);
  s.innerHTML = ICONS[name] || '';
  return s;
}

export const fmtClock = (sec) => { const s = Math.max(0, Math.round(sec)); return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`; };
export const plural = (n, one, many = `${one}s`) => `${n} ${n === 1 ? one : many}`;
export const ago = (t) => {
  const m = Math.round((Date.now() - t) / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m} min ago`;
  const hrs = Math.round(m / 60);
  if (hrs < 24) return `${hrs} h ago`;
  const d = Math.round(hrs / 24);
  return d === 1 ? 'yesterday' : `${d} days ago`;
};

// ---------- toast ----------
let toastTimer = null;
export function toast(message, kind = '') {
  let el = document.getElementById('toast');
  if (!el) { el = h('div', { id: 'toast', class: 'toast', role: 'status' }); document.body.append(el); }
  el.className = `toast show ${kind}`;
  el.textContent = message;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { el.classList.remove('show'); }, 4200);
}

// ---------- dialogs (one <dialog> per call, resolved with the chosen action's value) ----------
export function dialog({ title, body, actions = [{ label: 'OK', value: true, kind: 'primary' }], wide = false, onOpen }) {
  return new Promise((resolve) => {
    const dlg = h('dialog', { class: 'dlg' + (wide ? ' wide' : '') });
    const result = { value: null };
    const finish = (v) => { result.value = v; dlg.close(); };
    dlg.append(
      h('div', { class: 'dlg-head' }, h('h2', { text: title || '' }), h('button', { class: 'icon-btn', type: 'button', 'aria-label': 'Close', onclick: () => finish(null) }, icon('x', 18))),
      h('div', { class: 'dlg-body' }, body || null),
      actions.length ? h('div', { class: 'dlg-actions' }, ...actions.map((a) => h('button', { class: `btn ${a.kind || ''}`, type: 'button', text: a.label, onclick: () => (a.onclick ? a.onclick(finish) : finish(a.value)) }))) : null);
    dlg.addEventListener('close', () => { dlg.remove(); resolve(result.value); });
    dlg.addEventListener('click', (e) => { if (e.target === dlg) finish(null); });
    document.body.append(dlg);
    dlg.showModal();
    if (onOpen) onOpen(dlg, finish);
  });
}
export const confirmDialog = (text, yes = 'Confirm', { danger = false, title = 'Are you sure?' } = {}) =>
  dialog({ title, body: h('p', { text }), actions: [{ label: 'Cancel', value: false }, { label: yes, value: true, kind: danger ? 'danger' : 'primary' }] }).then((v) => v === true);

// ---------- small popover menu anchored to a button ----------
export function menuButton(label, items) {
  const wrap = h('div', { class: 'menu-wrap' });
  const btn = h('button', { class: 'icon-btn', type: 'button', 'aria-haspopup': 'menu', 'aria-expanded': 'false', 'aria-label': label }, icon('more', 18));
  const pop = h('div', { class: 'menu', role: 'menu', hidden: true });
  const close = () => { pop.hidden = true; btn.setAttribute('aria-expanded', 'false'); document.removeEventListener('click', outside, true); };
  const outside = (e) => { if (!wrap.contains(e.target)) close(); };
  for (const it of items) pop.append(h('button', { class: 'menu-item' + (it.danger ? ' danger' : ''), type: 'button', role: 'menuitem', text: it.label, onclick: () => { close(); it.onclick(); } }));
  btn.addEventListener('click', (e) => { e.stopPropagation(); if (pop.hidden) { pop.hidden = false; btn.setAttribute('aria-expanded', 'true'); document.addEventListener('click', outside, true); } else close(); });
  wrap.append(btn, pop);
  return wrap;
}

// ---------- SSE over fetch (the server streams explain replies) ----------
export async function streamSse(url, body, onDelta) {
  const res = await fetch(url, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
  if (!res.ok) {
    let msg = `Request failed (${res.status}).`;
    try { msg = (await res.json()).error || msg; } catch { /* keep default */ }
    throw new Error(msg);
  }
  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buf = '';
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    buf += decoder.decode(value, { stream: true });
    let idx;
    while ((idx = buf.indexOf('\n\n')) >= 0) {
      const block = buf.slice(0, idx);
      buf = buf.slice(idx + 2);
      const ev = /^event: (.+)$/m.exec(block)?.[1];
      const data = JSON.parse(/^data: (.+)$/m.exec(block)?.[1] || '{}');
      if (ev === 'delta') onDelta(data.text);
      else if (ev === 'error') throw new Error(data.message || 'Claude could not answer.');
    }
  }
}

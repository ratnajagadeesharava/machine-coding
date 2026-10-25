// Shared helpers for the whiteboard menus/dialogs/library (agent D): tiny DOM builder, menu icons, shortcut formatting,
// toasts, confirm/prompt dialogs and a request bus. No dynamic innerHTML anywhere.

const SVG_NS = 'http://www.w3.org/2000/svg';

export const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent || '');

/** Build a DOM node. props: class, text, on:{evt:fn}, any other key -> attribute (false/null/undefined skipped, true -> ''). */
export function el(tag, props = {}, children = []) {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') node.className = v;
    else if (k === 'text') node.textContent = v;
    else if (k === 'on') for (const [evt, fn] of Object.entries(v)) node.addEventListener(evt, fn);
    else node.setAttribute(k, v === true ? '' : String(v));
  }
  for (const c of [].concat(children)) if (c != null && c !== false) node.append(c);
  return node;
}

// 24x24 stroke icons: arrays of path data.
const PATHS = {
  menu: ['M4 6h16', 'M4 12h16', 'M4 18h16'],
  open: ['M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'],
  save: ['M5 3h11l4 4v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z', 'M8 3v5h7V3', 'M8 21v-7h8v7'],
  image: ['M4 5h16a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1z', 'M3 16l5-5 4 4 3-3 6 6', 'M15.5 9.5h.01'],
  copy: ['M9 9h10a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V10a1 1 0 0 1 1-1z', 'M16 9V5a1 1 0 0 0-1-1H5a1 1 0 0 0-1 1v10a1 1 0 0 0 1 1h3'],
  cut: ['M6 3l12 14', 'M18 3L6 17', 'M6 17a2.5 2.5 0 1 0 .01 0', 'M18 17a2.5 2.5 0 1 0 .01 0'],
  paste: ['M9 4h6v3H9z', 'M15 5.5h2a1 1 0 0 1 1 1V20a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1V6.5a1 1 0 0 1 1-1h2'],
  search: ['M11 4a7 7 0 1 0 .01 0', 'M16.5 16.5L21 21'],
  command: ['M9 6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3z'],
  trash: ['M4 7h16', 'M9 7V4h6v3', 'M6 7l1 13h10l1-13', 'M10 11v6', 'M14 11v6'],
  sun: ['M12 8a4 4 0 1 0 .01 0', 'M12 2v2', 'M12 20v2', 'M2 12h2', 'M20 12h2', 'M5 5l1.5 1.5', 'M17.5 17.5L19 19', 'M5 19l1.5-1.5', 'M17.5 6.5L19 5'],
  moon: ['M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z'],
  grid: ['M4 4h16v16H4z', 'M4 9.33h16', 'M4 14.67h16', 'M9.33 4v16', 'M14.67 4v16'],
  magnet: ['M5 3v8a7 7 0 0 0 14 0V3h-4v8a3 3 0 0 1-6 0V3z', 'M5 7h4', 'M15 7h4'],
  zen: ['M4 9V5h4', 'M16 5h4v4', 'M20 15v4h-4', 'M8 19H4v-4'],
  eye: ['M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z', 'M12 9.5a2.5 2.5 0 1 0 .01 0'],
  stats: ['M5 20V10', 'M12 20V4', 'M19 20v-7'],
  help: ['M12 3a9 9 0 1 0 .01 0', 'M9.5 9.5a2.5 2.5 0 1 1 3.5 2.3c-.7.4-1 .9-1 1.7', 'M12 17h.01'],
  link: ['M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1', 'M14 10a4 4 0 0 0-5.7 0l-3 3A4 4 0 0 0 11 18.7l1-1'],
  lock: ['M6 11h12a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1z', 'M8 11V8a4 4 0 0 1 8 0v3'],
  unlock: ['M6 11h12a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1z', 'M8 11V8a4 4 0 0 1 7.5-2'],
  layers: ['M12 3l9 5-9 5-9-5z', 'M3 12.5l9 5 9-5', 'M3 17l9 5 9-5'],
  align: ['M4 4v16', 'M8 7h12', 'M8 14h7'],
  chevron: ['M9 6l6 6-6 6'],
  check: ['M5 12.5l4.5 4.5L19 7.5'],
  close: ['M6 6l12 12', 'M18 6L6 18'],
  plus: ['M12 5v14', 'M5 12h14'],
  upload: ['M12 16V4', 'M7 9l5-5 5 5', 'M5 20h14'],
  download: ['M12 4v12', 'M7 11l5 5 5-5', 'M5 20h14'],
  rename: ['M4 20h4L19 9l-4-4L4 16z', 'M13.5 6.5l4 4'],
  library: ['M5 4h4v16H5z', 'M11 4h4v16h-4z', 'M17 6l3.5 1-3.5 12.5-3.5-1z'],
  group: ['M4 4h8v8H4z', 'M12 12h8v8h-8z'],
  ungroup: ['M4 4h6v6H4z', 'M14 14h6v6h-6z'],
  flip: ['M12 3v18', 'M8 7L3 12l5 5z', 'M16 7l5 5-5 5z'],
  pen: ['M4 20l1-4L16 5l3 3L8 19z'],
  tool: ['M4 20l4-1L19 8l-3-3L5 16z', 'M14 7l3 3'],
  undo: ['M9 14L4 9l5-5', 'M4 9h10a6 6 0 0 1 0 12h-3'],
  redo: ['M15 14l5-5-5-5', 'M20 9H10a6 6 0 0 0 0 12h3'],
  clock: ['M12 3a9 9 0 1 0 .01 0', 'M12 7v5l3 2'],
  arrowUp: ['M12 19V5', 'M6 11l6-6 6 6'],
  arrowDown: ['M12 5v14', 'M6 13l6 6 6-6'],
  home: ['M4 11l8-7 8 7', 'M6 10v10h12V10'],
  external: ['M14 4h6v6', 'M20 4l-9 9', 'M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5'],
  palette: ['M12 3a9 9 0 1 0 0 18c1.5 0 2-1 1.5-2-.6-1.2.2-2.5 1.6-2.5H17a4 4 0 0 0 4-4C21 7 17 3 12 3z', 'M7.5 11h.01', 'M10 7.5h.01', 'M14.5 7.5h.01'],
};

export function mi(name, size = 16) {
  const svg = document.createElementNS(SVG_NS, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('width', String(size));
  svg.setAttribute('height', String(size));
  svg.setAttribute('fill', 'none');
  svg.setAttribute('stroke', 'currentColor');
  svg.setAttribute('stroke-width', '1.8');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  svg.setAttribute('aria-hidden', 'true');
  svg.classList.add('wbm-icon');
  for (const d of PATHS[name] || PATHS.help) {
    const p = document.createElementNS(SVG_NS, 'path');
    p.setAttribute('d', d);
    svg.append(p);
  }
  return svg;
}
export const ICON_NAMES = Object.keys(PATHS);

/** 'Mod+Shift+E' -> 'Ctrl+Shift+E' (or the mac glyph form). */
export function fmtShortcut(s, mac = isMac) {
  if (!s) return '';
  const parts = String(s).split('+');
  if (mac) {
    const m = { Mod: '⌘', Ctrl: '⌃', Shift: '⇧', Alt: '⌥', Backspace: '⌫', Delete: '⌦' };
    return parts.map((p) => m[p] || p).join('');
  }
  return parts.map((p) => (p === 'Mod' ? 'Ctrl' : p)).join('+');
}

/** Fire a "UI request" (open the library, find bar, ...) to the sibling modules mounted on this wb. */
export function requestUi(wb, name, detail = {}) {
  window.dispatchEvent(new CustomEvent('wbm:request', { detail: { wb, name, ...detail } }));
}
/** Listen to engine `request` events AND requests fired with requestUi. Returns an unsubscribe function. */
export function onRequest(wb, fn) {
  const offEngine = wb.on ? wb.on('request', (e) => fn(e && e.name, e || {})) : () => {};
  const h = (e) => { if (e.detail && e.detail.wb === wb) fn(e.detail.name, e.detail); };
  window.addEventListener('wbm:request', h);
  return () => { offEngine && offEngine(); window.removeEventListener('wbm:request', h); };
}

// ---- toasts -------------------------------------------------------------------------------------------------------
let toastBox = null;
export function showToast(message, kind = 'info', action = null) {
  if (typeof document === 'undefined') return;
  if (!toastBox || !toastBox.isConnected) {
    toastBox = el('div', { class: 'wbm-toasts', role: 'status', 'aria-live': 'polite', popover: 'manual' });
    document.body.append(toastBox);
  }
  const t = el('div', { class: `wbm-toast wbm-toast-${kind}`, 'data-kind': kind }, [el('span', { text: message })]);
  let timer = null;
  const dismiss = () => { clearTimeout(timer); t.remove(); if (!toastBox.children.length) try { toastBox.hidePopover(); } catch { /* ignore */ } };
  if (action) {
    t.append(el('button', { type: 'button', class: 'wbm-toast-action', text: action.label, on: { click: () => { dismiss(); action.run(); } } }));
  }
  toastBox.append(t);
  while (toastBox.children.length > 4) toastBox.firstChild.remove();
  try { toastBox.hidePopover(); } catch { /* ignore */ }
  try { toastBox.showPopover(); } catch { /* popover API unsupported: plain fixed box */ }
  timer = setTimeout(dismiss, action ? 7000 : 4000);
}
const toastWired = new WeakSet();
/** Render the engine's `toast` events (once per wb, whichever module mounts first). */
export function ensureToasts(wb) {
  if (toastWired.has(wb) || !wb.on) return;
  toastWired.add(wb);
  wb.on('toast', (e) => e && e.message && showToast(e.message, e.kind === 'error' ? 'error' : 'info'));
}

// ---- dialogs ------------------------------------------------------------------------------------------------------
/** Create a modal <dialog> shell. Returns { dialog, body, footer, close(value), result: Promise<value> }. */
export function makeDialog({ title, className = '', width } = {}) {
  const dialog = el('dialog', { class: `wbm-dialog ${className}`.trim(), 'aria-label': title || 'Dialog' });
  if (width) dialog.style.setProperty('--wbm-dialog-w', width);
  const closeBtn = el('button', { type: 'button', class: 'wbm-icon-btn', 'aria-label': 'Close', title: 'Close', 'data-action': 'dialog-close' }, [mi('close', 16)]);
  const header = el('div', { class: 'wbm-dialog-head' }, [el('h2', { class: 'wbm-dialog-title', text: title || '' }), closeBtn]);
  const body = el('div', { class: 'wbm-dialog-body' });
  const footer = el('div', { class: 'wbm-dialog-foot' });
  dialog.append(header, body, footer);
  let resolve;
  const result = new Promise((r) => { resolve = r; });
  let value;
  const close = (v) => { value = v; if (dialog.open) dialog.close(); else finish(); };
  const finish = () => { dialog.remove(); resolve(value); };
  dialog.addEventListener('close', finish);
  dialog.addEventListener('cancel', () => { value = undefined; });
  closeBtn.addEventListener('click', () => close(undefined));
  // click on the backdrop closes
  dialog.addEventListener('mousedown', (e) => { if (e.target === dialog) close(undefined); });
  document.body.append(dialog);
  const show = () => { try { dialog.showModal(); } catch { dialog.setAttribute('open', ''); } return result; };
  return { dialog, body, footer, close, result, show };
}

export function confirmDialog({ title = 'Are you sure?', message = '', confirmLabel = 'OK', cancelLabel = 'Cancel', danger = false } = {}) {
  const d = makeDialog({ title, className: 'wbm-confirm', width: '26rem' });
  d.body.append(el('p', { class: 'wbm-confirm-msg', text: message }));
  const cancel = el('button', { type: 'button', class: 'wbm-btn', 'data-action': 'confirm-cancel', text: cancelLabel, on: { click: () => d.close(false) } });
  const ok = el('button', { type: 'button', class: `wbm-btn ${danger ? 'wbm-btn-danger' : 'wbm-btn-primary'}`, 'data-action': 'confirm-ok', text: confirmLabel, on: { click: () => d.close(true) } });
  d.footer.append(cancel, ok);
  const p = d.show();
  (danger ? cancel : ok).focus();
  return p.then((v) => v === true);
}

export function promptDialog({ title = 'Name', label = '', value = '', placeholder = '', confirmLabel = 'OK', allowEmpty = false } = {}) {
  const d = makeDialog({ title, className: 'wbm-prompt', width: '26rem' });
  const input = el('input', { type: 'text', class: 'wbm-input', value, placeholder, 'aria-label': label || title, 'data-action': 'prompt-input' });
  const form = el('form', { class: 'wbm-prompt-form', on: { submit: (e) => { e.preventDefault(); if (allowEmpty || input.value.trim()) d.close(input.value.trim()); } } },
    [label ? el('label', { class: 'wbm-label', text: label }) : null, input]);
  d.body.append(form);
  d.footer.append(
    el('button', { type: 'button', class: 'wbm-btn', text: 'Cancel', on: { click: () => d.close(null) } }),
    el('button', { type: 'button', class: 'wbm-btn wbm-btn-primary', 'data-action': 'prompt-ok', text: confirmLabel, on: { click: () => form.requestSubmit() } }),
  );
  const p = d.show();
  input.focus(); input.select();
  return p.then((v) => (v === undefined ? null : v));
}

/** Position a fixed popover near (x, y) keeping it inside the viewport. */
export function placeInViewport(node, x, y, margin = 8) {
  node.style.left = '0px'; node.style.top = '0px';
  const r = node.getBoundingClientRect();
  const vw = window.innerWidth; const vh = window.innerHeight;
  let left = x; let top = y;
  if (left + r.width + margin > vw) left = Math.max(margin, vw - r.width - margin);
  if (top + r.height + margin > vh) top = Math.max(margin, vh - r.height - margin);
  node.style.left = `${Math.round(left)}px`; node.style.top = `${Math.round(top)}px`;
}

export function debounce(fn, ms) {
  let t = null;
  const d = (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); };
  d.cancel = () => clearTimeout(t);
  return d;
}

/** Download helper fallbacks used when interop.js helpers are missing (they should not be). */
export function safeName(name, fallback = 'whiteboard') {
  const n = String(name || '').replace(/[\\/:*?"<>|\u0000-\u001f]+/g, ' ').trim().replace(/\s+/g, ' ').slice(0, 80);
  return n || fallback;
}

// ---- cross-module helpers ---------------------------------------------------------------------------------------
const scopeOf = (host) => (host && host.closest && host.closest('.wbx-host')) || document;
/** Stats readout lives in ui-zoom.js (a button with data-action="stats"); use engine mode.stats when the engine has it. */
export function statsOn(wb, host) {
  if (wb.mode && 'stats' in wb.mode) return !!wb.mode.stats;
  const b = scopeOf(host).querySelector('[data-action="stats"]');
  return !!b && b.getAttribute('aria-pressed') === 'true';
}
export function toggleStats(wb, host) {
  if (wb.mode && 'stats' in wb.mode) { wb.setMode({ stats: !wb.mode.stats }); return; }
  const b = scopeOf(host).querySelector('[data-action="stats"]');
  if (b) b.click(); else requestUi(wb, 'stats');
}
/** Open the shortcuts dialog (ui-help.js) if present, else fall back to a `help` request. */
export async function openHelpDialog(wb) {
  try {
    const m = await import('./ui-help.js');
    if (m && typeof m.openHelp === 'function') { m.openHelp(wb); return; }
  } catch { /* not available */ }
  requestUi(wb, 'help');
}

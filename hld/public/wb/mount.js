// Assembles the whiteboard: the engine plus every UI module, laid out around the canvas, with local autosave.
//
//   import { mountWhiteboard } from './wb/mount.js';
//   const app = mountWhiteboard(containerEl, { storageKey: 'hld-wb:scratch' });
//
// This file is part of the standalone whiteboard: it knows nothing about interviews, servers or any other page.
// Every module is loaded with a guarded dynamic import, so a missing or broken optional piece (say the library)
// degrades to a console warning instead of taking the whole page down.
import { emptyBoard, sanitizeBoard } from '../../shared/board.js';

async function optional(load, name) {
  try { return await load(); } catch (err) { console.warn(`[whiteboard] ${name} is unavailable:`, err); return null; }
}
const M = {
  engine: await optional(() => import('./engine.js'), 'engine'),
  toolbar: await optional(() => import('./ui-toolbar.js'), 'toolbar'),
  properties: await optional(() => import('./ui-properties.js'), 'properties panel'),
  zoom: await optional(() => import('./ui-zoom.js'), 'zoom bar'),
  help: await optional(() => import('./ui-help.js'), 'help'),
  menu: await optional(() => import('./ui-menu.js'), 'menu'),
  context: await optional(() => import('./ui-context.js'), 'context menu'),
  palette: await optional(() => import('./ui-palette.js'), 'command palette'),
  library: await optional(() => import('./ui-library.js'), 'library'),
};

const STYLESHEETS = ['engine.css', 'chrome.css', 'menus.css', 'mount.css'];
const LIBRARY_KEY = 'hld-wb:library';
const SAVE_DELAY = 400;
const CHANGE_DELAY = 300;

function injectStyles() {
  if (typeof document === 'undefined') return;
  for (const name of STYLESHEETS) {
    if (document.querySelector(`link[data-wb-css="${name}"]`)) continue;
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = new URL(`./${name}`, import.meta.url).href;
    link.dataset.wbCss = name;
    document.head.append(link);
  }
}

function debounce(fn, ms) {
  let t = null;
  const wrapped = (...args) => { clearTimeout(t); t = setTimeout(() => { t = null; fn(...args); }, ms); };
  wrapped.flush = (...args) => { if (t !== null) { clearTimeout(t); t = null; fn(...args); } };
  wrapped.cancel = () => { clearTimeout(t); t = null; };
  return wrapped;
}

const el = (tag, cls, attrs = {}) => {
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  for (const [k, v] of Object.entries(attrs)) { if (v === true) n.setAttribute(k, ''); else if (v !== false && v != null) n.setAttribute(k, v); }
  return n;
};

// ---- local storage helpers (every access guarded: private windows, blocked storage, quota) ----
const store = {
  get(key) { try { return localStorage.getItem(key); } catch { return null; } },
  set(key, value) { try { localStorage.setItem(key, value); return true; } catch { return false; } },
};

function loadSavedBoard(key) {
  const raw = store.get(key);
  if (!raw) return null;
  try {
    const data = JSON.parse(raw);
    return sanitizeBoard(data && data.board ? data.board : data);
  } catch { return null; } // corrupt JSON: start from the provided/empty board
}

function loadSavedMode(key) {
  const raw = store.get(`${key}:mode`);
  if (!raw) return {};
  try {
    const m = JSON.parse(raw);
    const out = {};
    if (m && (m.theme === 'light' || m.theme === 'dark')) out.theme = m.theme;
    if (m && typeof m.background === 'string' && /^#[0-9a-fA-F]{3,8}$/.test(m.background)) out.background = m.background;
    if (m && typeof m.grid === 'boolean') out.grid = m.grid;
    if (m && typeof m.snap === 'boolean') out.snap = m.snap;
    return out;
  } catch { return {}; }
}

const prefersDark = () => { try { return window.matchMedia('(prefers-color-scheme: dark)').matches; } catch { return false; } };

export function mountWhiteboard(container, options = {}) {
  injectStyles();
  const {
    board = null, onChange = null, storageKey = null, readOnly = false, menuExtra = [],
    library = true, menu = true, toolbar = true, properties = true, zoom = true, stats = true, keyboard = true,
  } = options;

  container.classList.add('wbx-host');
  container.replaceChildren();
  const cleanups = [];
  let destroyed = false;

  // Inline layout so the engine measures a real size even before the stylesheets finish loading.
  if (getComputedStyle(container).position === 'static') container.style.position = 'relative';
  const stage = el('div', 'wbx-stage');
  stage.style.cssText = 'position:absolute;inset:0';
  container.append(stage);

  // Engine missing: keep the page alive with a message and a minimal, storage-backed API.
  if (!M.engine || !M.engine.Whiteboard) {
    const note = el('p', 'wbx-missing');
    note.textContent = 'The whiteboard could not be loaded.';
    container.append(note);
    let current = (storageKey && loadSavedBoard(storageKey)) || (board ? sanitizeBoard(board) : emptyBoard());
    return {
      wb: null, ready: true, destroy() { container.replaceChildren(); container.classList.remove('wbx-host'); },
      setBoard(b) { current = sanitizeBoard(b); }, getBoard: () => current, setKeyboard() {}, on: () => () => {}, toggleLibrary() {},
    };
  }

  // ---- initial board and mode ----
  const saved = storageKey ? loadSavedBoard(storageKey) : null;
  const initialBoard = saved || (board ? sanitizeBoard(board) : emptyBoard());
  const savedMode = storageKey ? loadSavedMode(storageKey) : {};
  const theme = options.theme || savedMode.theme || (prefersDark() ? 'dark' : 'light');
  const wb = new M.engine.Whiteboard(stage, {
    board: initialBoard, readOnly, theme, background: savedMode.background, grid: savedMode.grid ?? false, keyboard,
  });
  if (savedMode.snap !== undefined && wb.setMode) wb.setMode({ snap: savedMode.snap });

  // ---- toast (autosave problems) ----
  let toastTimer = null;
  const toastBox = el('div', 'wbx-toast', { role: 'status', 'aria-live': 'polite', hidden: true });
  container.append(toastBox);
  function toast(message) {
    toastBox.textContent = message;
    toastBox.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toastBox.hidden = true; }, 5000);
  }

  // ---- autosave ----
  let quotaWarned = false;
  const saveNow = () => {
    if (!storageKey || destroyed) return;
    const ok = store.set(storageKey, JSON.stringify(wb.getBoard()));
    if (!ok && !quotaWarned) { quotaWarned = true; toast('Board too large to save in this browser'); }
    if (ok) quotaWarned = false;
  };
  const saveDebounced = debounce(saveNow, SAVE_DELAY);
  const saveMode = debounce(() => {
    if (!storageKey) return;
    const m = wb.mode || {};
    store.set(`${storageKey}:mode`, JSON.stringify({ theme: m.theme, background: m.background, grid: !!m.grid, snap: !!m.snap }));
  }, 200);
  const emitChange = onChange ? debounce(() => { try { onChange(wb.getBoard()); } catch (err) { console.error(err); } }, CHANGE_DELAY) : null;

  cleanups.push(wb.on('change', () => { saveDebounced(); if (emitChange) emitChange(); }));
  cleanups.push(wb.on('mode', () => { saveMode(); syncChrome(); }));
  const flush = () => { saveDebounced.flush(); saveMode.flush(); if (emitChange) emitChange.flush(); };
  window.addEventListener('pagehide', flush);
  cleanups.push(() => window.removeEventListener('pagehide', flush));

  // ---- chrome layout ----
  const chrome = el('div', 'wbx-chrome');
  chrome.style.cssText = 'position:absolute;inset:0;pointer-events:none;z-index:5';
  const topHost = el('div', 'wbx-top');
  const leftHost = el('div', 'wbx-left');
  const menuHost = el('div', 'wbx-menu');
  const zoomHost = el('div', 'wbx-zoom');
  const libHost = el('div', 'wbx-library', { hidden: true, 'aria-label': 'Shape library' });
  const helpHost = el('div', 'wbx-help');
  const zenPill = el('button', 'wbx-zen-exit', { type: 'button', hidden: true, 'data-action': 'exit-zen' });
  zenPill.textContent = 'Exit zen mode';
  // The chrome modules position themselves inside full-size layers; set inline so it holds before mount.css arrives.
  for (const layer of [topHost, leftHost, zoomHost, helpHost]) layer.style.cssText = 'position:absolute;inset:0;pointer-events:none';
  menuHost.style.cssText = 'position:absolute;top:12px;left:12px;pointer-events:auto';
  libHost.style.position = 'absolute';
  chrome.append(topHost, leftHost, menuHost, zoomHost, libHost, helpHost);
  container.append(chrome, zenPill);

  zenPill.addEventListener('click', () => { wb.toggleZen(); });

  function syncChrome() {
    const m = wb.mode || {};
    container.classList.toggle('wbx-zen', !!m.zen);
    container.classList.toggle('wbx-view', !!m.view || !!readOnly);
    container.dataset.theme = m.theme === 'dark' ? 'dark' : 'light';
    zenPill.hidden = !m.zen;
  }

  const parts = [];
  function mountPart(name, enabled, fn) {
    if (!enabled) return;
    if (!fn) { console.warn(`[whiteboard] ${name} is unavailable`); return; }
    try { const part = fn(); if (part && typeof part.destroy === 'function') parts.push(part); } catch (err) { console.warn(`[whiteboard] ${name} failed to mount:`, err); }
  }

  let libraryOpen = false;
  function setLibraryOpen(open) {
    libraryOpen = !!open && !!M.library && library;
    libHost.hidden = !libraryOpen;
    container.classList.toggle('wbx-library-open', libraryOpen);
    const btn = topHost.querySelector('[data-action="library"], [data-tool="library"], .wbui-library');
    if (btn) btn.setAttribute('aria-pressed', String(libraryOpen));
  }
  const toggleLibrary = () => setLibraryOpen(!libraryOpen);

  const T = M.toolbar;
  mountPart('toolbar', toolbar, T && T.mountToolbar && (() => T.mountToolbar(wb, topHost, { library: !!(library && M.library), onLibrary: toggleLibrary })));
  mountPart('properties panel', properties, M.properties && M.properties.mountProperties && (() => M.properties.mountProperties(wb, leftHost)));
  mountPart('zoom bar', zoom, M.zoom && M.zoom.mountZoomBar && (() => M.zoom.mountZoomBar(wb, zoomHost, { stats })));
  mountPart('help', true, M.help && M.help.mountHelp && (() => M.help.mountHelp(wb, helpHost)));
  mountPart('menu', menu, M.menu && M.menu.mountMenu && (() => M.menu.mountMenu(wb, menuHost, {
    extra: Array.isArray(menuExtra) ? menuExtra : [],
    onOpen: () => { saveDebounced(); },
    onSave: () => {},
  })));
  mountPart('context menu', true, M.context && M.context.mountContextMenu && (() => M.context.mountContextMenu(wb, container)));
  mountPart('command palette', true, M.palette && M.palette.mountPalette && (() => M.palette.mountPalette(wb, container)));
  mountPart('find bar', true, M.palette && M.palette.mountSearch && (() => M.palette.mountSearch(wb, container)));
  mountPart('library', library, M.library && M.library.mountLibrary && (() => M.library.mountLibrary(wb, libHost, { storageKey: LIBRARY_KEY })));

  // The library button and the `library` request (from the palette or the menu) toggle the right-hand panel.
  cleanups.push(wb.on('request', (e) => { if (e && e.name === 'library') toggleLibrary(); }));

  // The image tool (toolbar button, key 9, palette) asks for a picture: open the file picker and drop it in the centre of the view.
  cleanups.push(wb.on('request', async (e) => {
    if (!e || e.name !== 'image') return;
    try {
      const { pickFile } = await import('./interop.js');
      const file = await pickFile('image/*');
      if (file) await wb.addImageFile(file);
      else if (wb.setTool) wb.setTool('select');
    } catch (err) { console.warn('[whiteboard] image insert failed:', err); }
  }));
  const onUiRequest = (e) => { if (e.detail && e.detail.wb === wb && e.detail.name === 'library') toggleLibrary(); };
  window.addEventListener('wbm:request', onUiRequest);
  cleanups.push(() => window.removeEventListener('wbm:request', onUiRequest));

  syncChrome();

  // ---- public API ----
  const api = {
    wb,
    ready: true,
    getBoard: () => wb.getBoard(),
    setBoard(next, opts = {}) {
      wb.setBoard(sanitizeBoard(next), { resetHistory: true, fit: opts.fit !== false });
      saveNow();
    },
    setKeyboard(on) { if (wb.setKeyboard) wb.setKeyboard(!!on); },
    on: (type, fn) => wb.on(type, fn),
    toggleLibrary,
    destroy() {
      if (destroyed) return;
      flush();
      destroyed = true;
      saveDebounced.cancel();
      for (const p of parts) { try { p.destroy(); } catch { /* ignore */ } }
      for (const off of cleanups) { try { off(); } catch { /* ignore */ } }
      clearTimeout(toastTimer);
      try { if (wb.destroy) wb.destroy(); } catch { /* ignore */ }
      container.replaceChildren();
      container.classList.remove('wbx-host', 'wbx-zen', 'wbx-view', 'wbx-library-open');
      delete container.__wb;
    },
  };
  return api;
}

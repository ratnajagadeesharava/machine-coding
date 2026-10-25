// Library panel (agent D): built-in "System design" stencils + persisted "My library"; click or drag to insert.
import { BUILTIN_LIBRARY } from './library-data.js';
import { boardToSvgString } from './render.js';
import {
  serializeLibrary, parseLibrary, libraryItemFromElements, instantiateLibraryItem, downloadText, pickFile, readFileText,
} from './interop.js';
import { el, mi, promptDialog, confirmDialog, showToast, ensureToasts, onRequest, requestUi, trackTheme } from './menu-icons.js';

export const DRAG_TYPE = 'application/x-hld-library-item';
const DEFAULT_KEY = 'hld-wb:library';

/** Pure: group library items by category, keeping first-seen order. */
export function groupByCategory(items) {
  const map = new Map();
  for (const it of items) {
    const c = it.category || 'Other';
    if (!map.has(c)) map.set(c, []);
    map.get(c).push(it);
  }
  return [...map].map(([category, list]) => ({ category, items: list }));
}
/** Pure: case-insensitive name/category filter. */
export function filterItems(items, q) {
  const s = String(q || '').trim().toLowerCase();
  if (!s) return items;
  return items.filter((it) => `${it.name} ${it.category || ''}`.toLowerCase().includes(s));
}

export function thumbnailSrc(item, dark = false) {
  try {
    const svg = boardToSvgString({ version: 2, items: item.elements, nextId: 1_000_000 }, { background: null, dark, padding: 8 });
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  } catch { return ''; }
}

export function mountLibrary(wb, host, { storageKey = DEFAULT_KEY } = {}) {
  ensureToasts(wb);
  const managed = host.classList.contains('wbx-library'); // mount.js shows/hides this host itself
  let mine = [];
  let tab = 'builtin';
  let query = '';
  let destroyed = false;

  // ---- persistence
  const load = () => {
    try {
      const raw = localStorage.getItem(storageKey);
      mine = raw ? parseLibrary(raw) : [];
    } catch { mine = []; }
  };
  const save = () => {
    try { localStorage.setItem(storageKey, serializeLibrary(mine)); return true; } catch {
      showToast('Could not save your library (storage is full or blocked)', 'error');
      return false;
    }
  };

  // ---- DOM
  const root = el('section', { class: `wbm-library ${managed ? '' : 'wbm-library-float'}`.trim(), 'aria-label': 'Shape library', 'data-wbm': 'library' });
  const closeBtn = el('button', { type: 'button', class: 'wbm-icon-btn', 'data-action': 'library-close', 'aria-label': 'Close library', title: 'Close library' }, [mi('close', 16)]);
  const head = el('div', { class: 'wbm-lib-head' }, [el('h2', { class: 'wbm-lib-title', text: 'Library' }), closeBtn]);
  const tabBtns = {};
  const mkTab = (id, label) => {
    const b = el('button', { type: 'button', class: 'wbm-tab', role: 'tab', id: `wbm-libtab-${id}`, 'data-action': `library-tab-${id}`, text: label, on: { click: () => setTab(id) } });
    tabBtns[id] = b; return b;
  };
  const tabs = el('div', { class: 'wbm-tabs', role: 'tablist', 'aria-label': 'Library sections' }, [mkTab('builtin', 'System design'), mkTab('mine', 'My library')]);
  tabs.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); const n = tab === 'builtin' ? 'mine' : 'builtin'; setTab(n); tabBtns[n].focus(); }
  });
  const search = el('input', { type: 'search', class: 'wbm-input wbm-lib-search', 'aria-label': 'Search library', placeholder: 'Search shapes', 'data-action': 'library-search', autocomplete: 'off' });
  search.addEventListener('input', () => { query = search.value; render(); });
  const bodyEl = el('div', { class: 'wbm-lib-body', role: 'tabpanel' });
  const foot = el('div', { class: 'wbm-lib-foot' });
  const addBtn = el('button', { type: 'button', class: 'wbm-btn wbm-btn-primary', 'data-action': 'library-add', on: { click: () => addSelection() } }, [mi('plus', 14), el('span', { text: 'Add selection to library' })]);
  const impBtn = el('button', { type: 'button', class: 'wbm-btn', 'data-action': 'library-import', title: 'Import a .hldlib.json or .excalidrawlib file', on: { click: () => importFlow() } }, [mi('upload', 14), el('span', { text: 'Import' })]);
  const expBtn = el('button', { type: 'button', class: 'wbm-btn', 'data-action': 'library-export', title: 'Export your library as .hldlib.json', on: { click: () => exportFlow() } }, [mi('download', 14), el('span', { text: 'Export' })]);
  foot.append(addBtn, el('div', { class: 'wbm-lib-foot-row' }, [impBtn, expBtn]));
  root.append(head, tabs, search, bodyEl, foot);
  host.append(root);
  const offTheme = trackTheme(wb, root);

  const dark = () => (wb.mode && wb.mode.theme) === 'dark';
  const thumbCache = new Map();
  const thumb = (item) => {
    const key = `${item.id}|${dark() ? 'd' : 'l'}|${item.elements.length}`;
    if (!thumbCache.has(key)) thumbCache.set(key, thumbnailSrc(item, dark()));
    return thumbCache.get(key);
  };

  const allById = (src, id) => (src === 'mine' ? mine : BUILTIN_LIBRARY).find((i) => i.id === id);

  const insertItem = (item, at = 'viewport-center') => {
    if (!item) return;
    try {
      const els = instantiateLibraryItem(wb.board, item, { x: 0, y: 0 });
      wb.insertElements(els, { at, select: true });
    } catch (err) { showToast(`Could not insert "${item.name}": ${(err && err.message) || err}`, 'error'); }
  };

  const tile = (item, src) => {
    const img = el('img', { class: 'wbm-lib-thumb', alt: '', draggable: 'false', src: thumb(item) });
    const btn = el('button', {
      type: 'button', class: 'wbm-lib-tile', draggable: 'true', title: `${item.name} - click or drag onto the canvas`, 'aria-label': `Insert ${item.name}`,
      'data-action': 'library-item', 'data-id': item.id, 'data-source': src,
      on: {
        click: () => insertItem(item),
        dragstart: (e) => {
          e.dataTransfer.setData(DRAG_TYPE, JSON.stringify({ src, id: item.id }));
          e.dataTransfer.setData('text/plain', item.name);
          e.dataTransfer.effectAllowed = 'copy';
          try { e.dataTransfer.setDragImage(img, img.width / 2, img.height / 2); } catch { /* ignore */ }
        },
      },
    }, [el('span', { class: 'wbm-lib-thumbbox' }, [img]), el('span', { class: 'wbm-lib-name', text: item.name })]);
    const wrap = el('div', { class: 'wbm-lib-item' }, [btn]);
    if (src === 'mine') {
      wrap.append(el('div', { class: 'wbm-lib-actions' }, [
        el('button', { type: 'button', class: 'wbm-icon-btn wbm-icon-btn-sm', 'data-action': 'library-rename', 'aria-label': `Rename ${item.name}`, title: 'Rename', on: { click: () => renameItem(item) } }, [mi('rename', 13)]),
        el('button', { type: 'button', class: 'wbm-icon-btn wbm-icon-btn-sm', 'data-action': 'library-delete', 'aria-label': `Delete ${item.name}`, title: 'Delete', on: { click: () => deleteItem(item) } }, [mi('trash', 13)]),
      ]));
    }
    return wrap;
  };

  function render() {
    for (const [id, b] of Object.entries(tabBtns)) { b.setAttribute('aria-selected', String(id === tab)); b.tabIndex = id === tab ? 0 : -1; }
    bodyEl.setAttribute('aria-labelledby', `wbm-libtab-${tab}`);
    root.dataset.tab = tab;
    foot.hidden = tab !== 'mine';
    bodyEl.textContent = '';
    if (tab === 'builtin') {
      const groups = groupByCategory(filterItems(BUILTIN_LIBRARY, query));
      if (!groups.length) bodyEl.append(el('p', { class: 'wbm-lib-empty', text: 'No shapes match your search.' }));
      for (const g of groups) {
        bodyEl.append(el('h3', { class: 'wbm-lib-cat', text: g.category }));
        bodyEl.append(el('div', { class: 'wbm-lib-grid' }, g.items.map((it) => tile(it, 'builtin'))));
      }
    } else {
      const list = filterItems(mine, query);
      if (!mine.length) {
        bodyEl.append(el('div', { class: 'wbm-lib-empty' }, [
          el('p', { text: 'Your library is empty.' }),
          el('p', { class: 'wbm-muted', text: 'Select shapes on the canvas and press "Add selection to library" to reuse them anywhere.' }),
        ]));
      } else if (!list.length) bodyEl.append(el('p', { class: 'wbm-lib-empty', text: 'No items match your search.' }));
      else bodyEl.append(el('div', { class: 'wbm-lib-grid' }, list.map((it) => tile(it, 'mine'))));
    }
  }
  function setTab(id) { tab = id; render(); }

  // ---- my library operations
  async function addSelection() {
    let t = null;
    try { t = wb.toTemplate(); } catch { /* handled below */ }
    if (!t || !t.elements || !t.elements.length) { showToast('Select something on the canvas first', 'info'); return; }
    const name = await promptDialog({ title: 'Add to library', label: 'Item name', value: `Item ${mine.length + 1}`, confirmLabel: 'Add' });
    if (name === null) return;
    const item = libraryItemFromElements(wb.board, t.elements, name || `Item ${mine.length + 1}`, 'My library');
    mine.push(item);
    save();
    tab = 'mine'; query = ''; search.value = '';
    render();
    const n = bodyEl.querySelector(`[data-id="${CSS.escape(item.id)}"]`);
    if (n && n.scrollIntoView) n.scrollIntoView({ block: 'nearest' });
    showToast(`Added "${item.name}" to your library`, 'info');
  }
  async function renameItem(item) {
    const name = await promptDialog({ title: 'Rename item', label: 'Item name', value: item.name, confirmLabel: 'Rename' });
    if (!name) return;
    item.name = name.slice(0, 80);
    save(); render();
  }
  function deleteItem(item) {
    const i = mine.indexOf(item);
    if (i < 0) return;
    mine.splice(i, 1);
    save(); render();
    showToast(`Removed "${item.name}"`, 'info', {
      label: 'Undo',
      run: () => { if (!mine.includes(item)) { mine.splice(Math.min(i, mine.length), 0, item); save(); render(); } },
    });
  }
  function merge(items) {
    const ids = new Set(mine.map((m) => m.id));
    let n = 0;
    for (const it of items) {
      let id = it.id;
      if (ids.has(id)) id = `${id}-${Date.now().toString(36)}-${n}`;
      ids.add(id);
      mine.push({ ...it, id, category: it.category || 'My library' });
      n++;
    }
    save();
    return n;
  }
  function importText(text, name = 'file') {
    let items;
    try { items = parseLibrary(text); } catch (err) { showToast(`Could not import ${name}: ${(err && err.message) || err}`, 'error'); return 0; }
    if (!items.length) { showToast('That library file has no items', 'info'); return 0; }
    const n = merge(items);
    tab = 'mine'; render();
    showToast(`Imported ${n} item${n === 1 ? '' : 's'}`, 'info');
    return n;
  }
  async function importFlow() {
    const file = await pickFile('.hldlib.json,.excalidrawlib,.json');
    if (!file) return;
    let text;
    try { text = await readFileText(file); } catch (err) { showToast(`Could not read file: ${(err && err.message) || err}`, 'error'); return; }
    importText(text, file.name);
  }
  function exportFlow() {
    if (!mine.length) { showToast('Your library is empty - add some shapes first', 'info'); return; }
    downloadText('whiteboard.hldlib.json', serializeLibrary(mine), 'application/json');
  }
  async function resetFlow() {
    if (!mine.length) { showToast('Your library is already empty', 'info'); return; }
    const ok = await confirmDialog({ title: 'Reset the library', message: `This deletes all ${mine.length} items in "My library". The built-in shapes are not affected.`, confirmLabel: 'Reset library', danger: true });
    if (!ok) return;
    mine = []; save(); render();
  }

  // ---- open / close
  const isOpen = () => !host.hidden;
  const open = () => { if (!managed) host.hidden = false; };
  const close = () => { if (!managed) host.hidden = true; };
  const toggle = () => { if (managed) requestUi(wb, 'library'); else host.hidden = !host.hidden; };
  closeBtn.addEventListener('click', () => (managed ? requestUi(wb, 'library') : close()));
  root.addEventListener('keydown', (e) => { if (e.key === 'Escape' && !e.defaultPrevented) { e.stopPropagation(); managed ? requestUi(wb, 'library') : close(); } });

  const offReq = onRequest(wb, (name, detail) => {
    if (name === 'library' && !managed) toggle();
    else if (name === 'library-add') addSelection();
    else if (name === 'library-import-pick') importFlow();
    else if (name === 'library-import' && detail && typeof detail.text === 'string') importText(detail.text, detail.name);
    else if (name === 'library-export') exportFlow();
    else if (name === 'library-reset') resetFlow();
  });
  // tell the engine's dedupe: engine `request` events with name 'library' reach onRequest through wb.on; managed hosts handle open/close themselves.

  // ---- drop on canvas
  const canvasOf = (target) => { let n = target; while (n && n.__wb !== wb) n = n.parentElement; return n; };
  const onDragOver = (e) => {
    if (!e.dataTransfer || ![...e.dataTransfer.types].includes(DRAG_TYPE)) return;
    if (canvasOf(e.target)) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; }
  };
  const onDrop = (e) => {
    if (!e.dataTransfer || ![...e.dataTransfer.types].includes(DRAG_TYPE)) return;
    const c = canvasOf(e.target);
    if (!c) return;
    e.preventDefault(); e.stopPropagation();
    let info; try { info = JSON.parse(e.dataTransfer.getData(DRAG_TYPE)); } catch { return; }
    const item = allById(info.src, info.id);
    const at = wb.clientToWorld(e.clientX, e.clientY);
    insertItem(item, at);
  };
  document.addEventListener('dragover', onDragOver, true);
  document.addEventListener('drop', onDrop, true);

  const offMode = wb.on('mode', () => { if (isOpen() || managed) render(); });
  const onStorage = (e) => { if (e.key === storageKey) { load(); render(); } };
  window.addEventListener('storage', onStorage);

  load();
  render();
  return {
    destroy() {
      if (destroyed) return; destroyed = true;
      offReq(); offMode && offMode(); offTheme && offTheme();
      document.removeEventListener('dragover', onDragOver, true); document.removeEventListener('drop', onDrop, true);
      window.removeEventListener('storage', onStorage);
      root.remove();
    },
    open, close, toggle, isOpen,
    // exposed for tests/hosts
    getItems: () => mine.slice(), addSelection, importText, exportLibrary: exportFlow,
  };
}

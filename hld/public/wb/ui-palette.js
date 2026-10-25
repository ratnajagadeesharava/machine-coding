// Command palette and find-on-canvas bar (agent D).
import { TOOLS } from '../../shared/wbconst.js';
import { el, mi, fmtShortcut, makeDialog, confirmDialog, promptDialog, showToast, ensureToasts, onRequest, requestUi, debounce, openHelpDialog, toggleStats } from './menu-icons.js';

export const CATEGORIES = ['Tools', 'Edit', 'View', 'Export', 'Library'];
const RECENT_KEY = 'hld.wb.palette.recent';
const RECENT_MAX = 8;

/** Engine action methods that have a palette command (kept in sync with section 5 of the API doc; a test checks it). */
export const ENGINE_ACTIONS = [
  'undo', 'redo', 'copy', 'cut', 'paste', 'copyAsPng', 'copyAsSvg', 'duplicate', 'deleteSelected', 'selectAll', 'clearSelection', 'group', 'ungroup',
  'toggleLock', 'unlockAll', 'bringToFront', 'bringForward', 'sendBackward', 'sendToBack', 'align', 'distribute', 'flip', 'increaseFontSize', 'decreaseFontSize',
  'setLink', 'flowchartAdd', 'zoomIn', 'zoomOut', 'resetZoom', 'zoomToFit', 'zoomToSelection', 'toggleToolLock', 'toggleGrid', 'toggleSnap', 'toggleZen',
  'toggleViewMode', 'toggleTheme', 'copyStyle', 'pasteStyle', 'clear', 'setTool',
];

// ---- fuzzy ranking (pure) ----------------------------------------------------------------------------------------
const isWordStart = (s, i) => i === 0 || /[\s\-_/:.]/.test(s[i - 1]);
function scoreToken(t, s) {
  const idx = s.indexOf(t);
  if (idx >= 0) return 100 - idx * 0.5 + (idx === 0 ? 30 : isWordStart(s, idx) ? 20 : 0) - (s.length - t.length) * 0.2;
  let si = 0; let score = 0; let prev = -2;
  for (const ch of t) {
    const j = s.indexOf(ch, si);
    if (j < 0) return -1;
    score += 5 + (j === prev + 1 ? 8 : 0) + (isWordStart(s, j) ? 10 : 0);
    prev = j; si = j + 1;
  }
  return score;
}
/** Score a (possibly multi-word) query against text. -1 when it does not match. Higher is better. */
export function fuzzyScore(query, text) {
  const q = String(query || '').toLowerCase().trim();
  if (!q) return 0;
  const s = String(text || '').toLowerCase();
  let total = 0;
  for (const tok of q.split(/\s+/)) {
    const v = scoreToken(tok, s);
    if (v < 0) return -1;
    total += v;
  }
  return total;
}
/** Filter + rank commands. `recent` is a list of ids, most recent first. */
export function rankCommands(commands, query, recent = []) {
  const q = String(query || '').trim();
  const rankOf = (id) => { const i = recent.indexOf(id); return i < 0 ? -1 : i; };
  if (!q) {
    const rec = recent.map((id) => commands.find((c) => c.id === id)).filter(Boolean);
    const rest = commands.filter((c) => !rec.includes(c));
    rest.sort((a, b) => (CATEGORIES.indexOf(a.category) - CATEGORIES.indexOf(b.category)));
    return [...rec.map((c) => ({ cmd: c, group: 'Recent' })), ...rest.map((c) => ({ cmd: c, group: c.category }))];
  }
  const out = [];
  for (const c of commands) {
    const a = fuzzyScore(q, c.label);
    const b = fuzzyScore(q, `${c.category} ${c.label} ${c.keywords || ''}`);
    let s = Math.max(a, b < 0 ? -1 : b * 0.6);
    if (s < 0) continue;
    const r = rankOf(c.id);
    if (r >= 0) s += 25 - r * 2;
    out.push({ cmd: c, score: s, group: c.category });
  }
  out.sort((x, y) => y.score - x.score || x.cmd.label.localeCompare(y.cmd.label));
  return out;
}

// ---- commands ----------------------------------------------------------------------------------------------------
export function buildCommands(wb, host) {
  const list = [];
  const add = (id, category, label, shortcut, run, keywords = '') => list.push({ id, category, label, shortcut, run, keywords });
  const act = (m, ...args) => () => wb[m](...args);

  for (const t of TOOLS) {
    const sc = [t.key && t.key.toUpperCase(), t.num].filter(Boolean).join(' or ');
    add(`tool:${t.id}`, 'Tools', t.label, sc, () => wb.setTool(t.id), `tool ${t.id}`);
  }
  add('tool-lock', 'Tools', 'Toggle tool lock (keep tool active)', 'Q', act('toggleToolLock'), 'sticky lock tool');

  add('undo', 'Edit', 'Undo', 'Mod+Z', act('undo'));
  add('redo', 'Edit', 'Redo', 'Mod+Shift+Z', act('redo'));
  add('cut', 'Edit', 'Cut', 'Mod+X', act('cut'));
  add('copy', 'Edit', 'Copy', 'Mod+C', act('copy'));
  add('paste', 'Edit', 'Paste', 'Mod+V', act('paste'));
  add('duplicate', 'Edit', 'Duplicate selection', 'Mod+D', act('duplicate'));
  add('delete', 'Edit', 'Delete selection', 'Delete', act('deleteSelected'), 'remove erase');
  add('select-all', 'Edit', 'Select all', 'Mod+A', act('selectAll'));
  add('deselect', 'Edit', 'Clear selection', 'Esc', act('clearSelection'), 'deselect');
  add('group', 'Edit', 'Group selection', 'Mod+G', act('group'));
  add('ungroup', 'Edit', 'Ungroup selection', 'Mod+Shift+G', act('ungroup'));
  add('lock', 'Edit', 'Lock / unlock selection', 'Mod+Shift+L', act('toggleLock'));
  add('unlock-all', 'Edit', 'Unlock all elements', '', act('unlockAll'));
  add('front', 'Edit', 'Bring to front', 'Mod+Shift+]', act('bringToFront'), 'layers z-order');
  add('forward', 'Edit', 'Bring forward', 'Mod+]', act('bringForward'), 'layers z-order');
  add('backward', 'Edit', 'Send backward', 'Mod+[', act('sendBackward'), 'layers z-order');
  add('back', 'Edit', 'Send to back', 'Mod+Shift+[', act('sendToBack'), 'layers z-order');
  for (const [dir, label, sc] of [['left', 'Align left', 'Mod+Shift+Left'], ['right', 'Align right', 'Mod+Shift+Right'], ['top', 'Align top', 'Mod+Shift+Up'],
    ['bottom', 'Align bottom', 'Mod+Shift+Down'], ['hcenter', 'Center horizontally', ''], ['vcenter', 'Center vertically', '']]) add(`align:${dir}`, 'Edit', label, sc, act('align', dir), 'align');
  add('distribute:h', 'Edit', 'Distribute horizontally', '', act('distribute', 'h'));
  add('distribute:v', 'Edit', 'Distribute vertically', '', act('distribute', 'v'));
  add('flip:h', 'Edit', 'Flip horizontal', 'Shift+H', act('flip', 'h'), 'mirror');
  add('flip:v', 'Edit', 'Flip vertical', 'Shift+V', act('flip', 'v'), 'mirror');
  add('font-up', 'Edit', 'Increase font size', 'Mod+Shift+>', act('increaseFontSize'), 'text bigger');
  add('font-down', 'Edit', 'Decrease font size', 'Mod+Shift+<', act('decreaseFontSize'), 'text smaller');
  add('copy-style', 'Edit', 'Copy styles', 'Mod+Alt+C', act('copyStyle'));
  add('paste-style', 'Edit', 'Paste styles', 'Mod+Alt+V', act('pasteStyle'));
  add('link', 'Edit', 'Add or edit link', 'Mod+K', async () => {
    const sel = [...wb.selection]; const cur = sel.length === 1 ? (wb.board.items.find((e) => e.id === sel[0]) || {}).link : '';
    const v = await promptDialog({ title: 'Add link', label: 'URL', value: cur || '', placeholder: 'https://', confirmLabel: 'Save', allowEmpty: true });
    if (v !== null) wb.setLink(v || null);
  }, 'hyperlink url');
  for (const [d, label] of [['right', 'right'], ['left', 'left'], ['up', 'above'], ['down', 'below']]) {
    add(`flowchart:${d}`, 'Edit', `Add connected shape ${label}`, `Mod+Arrow ${d}`, act('flowchartAdd', d), 'flowchart connect arrow');
  }
  add('clear-canvas', 'Edit', 'Clear canvas', '', async () => {
    if (wb.isEmpty()) { showToast('The canvas is already empty', 'info'); return; }
    if (await confirmDialog({ title: 'Clear canvas', message: 'This removes everything from the canvas. You can undo it with Ctrl+Z.', confirmLabel: 'Clear canvas', danger: true })) wb.clear();
  }, 'reset empty');

  add('zoom-in', 'View', 'Zoom in', 'Mod+=', act('zoomIn'));
  add('zoom-out', 'View', 'Zoom out', 'Mod+-', act('zoomOut'));
  add('zoom-reset', 'View', 'Reset zoom to 100%', 'Mod+0', act('resetZoom'));
  add('zoom-fit', 'View', 'Zoom to fit all', 'Shift+1', act('zoomToFit'));
  add('zoom-selection', 'View', 'Zoom to selection', 'Shift+2', act('zoomToSelection'));
  add('grid', 'View', 'Toggle grid', "Mod+'", act('toggleGrid'));
  add('snap', 'View', 'Toggle object snapping', 'Alt+S', act('toggleSnap'));
  add('zen', 'View', 'Toggle zen mode', 'Alt+Z', act('toggleZen'));
  add('view-mode', 'View', 'Toggle view mode', 'Alt+R', act('toggleViewMode'), 'read only');
  add('theme', 'View', 'Toggle dark / light theme', 'Alt+Shift+D', act('toggleTheme'), 'dark mode light mode');
  add('stats', 'View', 'Toggle stats', 'Alt+/', () => toggleStats(wb, host), 'statistics');
  add('find', 'View', 'Find on canvas', 'Mod+F', () => requestUi(wb, 'search'), 'search text');
  add('help', 'View', 'Keyboard shortcuts', '?', () => openHelpDialog(wb), 'help keys');

  add('export-image', 'Export', 'Export image...', 'Mod+Shift+E', () => requestUi(wb, 'export'), 'png svg download');
  add('save', 'Export', 'Save to file', 'Mod+S', () => requestUi(wb, 'save'), 'hld.json download');
  add('open', 'Export', 'Open file...', 'Mod+O', () => requestUi(wb, 'open'), 'load import excalidraw');
  add('copy-png', 'Export', 'Copy as PNG', 'Shift+Alt+C', act('copyAsPng'), 'clipboard image');
  add('copy-svg', 'Export', 'Copy as SVG', '', act('copyAsSvg'), 'clipboard');

  add('library', 'Library', 'Toggle library panel', '', () => requestUi(wb, 'library'), 'shapes stencils');
  add('library-add', 'Library', 'Add selection to library', '', () => requestUi(wb, 'library-add'));
  add('library-import', 'Library', 'Import library...', '', () => requestUi(wb, 'library-import-pick'));
  add('library-export', 'Library', 'Export library', '', () => requestUi(wb, 'library-export'));
  return list;
}

function loadRecent() {
  try { const v = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'); return Array.isArray(v) ? v.filter((x) => typeof x === 'string').slice(0, RECENT_MAX) : []; } catch { return []; }
}
function saveRecent(id) {
  try {
    const next = [id, ...loadRecent().filter((x) => x !== id)].slice(0, RECENT_MAX);
    localStorage.setItem(RECENT_KEY, JSON.stringify(next));
  } catch { /* storage unavailable */ }
}

export function mountPalette(wb, host, { getCommands } = {}) {
  ensureToasts(wb);
  let current = null;

  const open = () => {
    if (current) { current.input.focus(); return current.promise; }
    const commands = buildCommands(wb, host);
    if (typeof getCommands === 'function') {
      try { for (const c of getCommands(wb, commands) || []) if (c && c.id && c.label && typeof c.run === 'function') commands.push({ category: 'Edit', shortcut: '', keywords: '', ...c }); } catch { /* ignore bad host commands */ }
    }
    const recent = loadRecent();
    const d = makeDialog({ title: 'Command palette', className: 'wbm-palette', width: '36rem' });
    d.dialog.querySelector('.wbm-dialog-head').hidden = true;
    const input = el('input', {
      type: 'text', class: 'wbm-palette-input', role: 'combobox', 'aria-expanded': 'true', 'aria-controls': 'wbm-palette-list', 'aria-autocomplete': 'list',
      placeholder: 'Type a command or tool...', 'aria-label': 'Search commands', 'data-action': 'palette-input', autocomplete: 'off', spellcheck: 'false',
    });
    const list = el('div', { class: 'wbm-palette-list', id: 'wbm-palette-list', role: 'listbox', 'aria-label': 'Commands' });
    d.body.append(el('div', { class: 'wbm-palette-search' }, [mi('search', 16), input]), list);
    d.footer.append(el('span', { class: 'wbm-palette-hint', text: 'Up/Down to move, Enter to run, Esc to close' }));
    let rows = []; let sel = 0;

    const render = () => {
      list.textContent = '';
      rows = rankCommands(commands, input.value, recent);
      sel = Math.min(sel, Math.max(0, rows.length - 1));
      if (!rows.length) { list.append(el('div', { class: 'wbm-palette-empty', text: 'No matching commands' })); input.removeAttribute('aria-activedescendant'); return; }
      let lastGroup = null;
      rows.forEach((r, i) => {
        const showHeaders = !input.value.trim();
        if (showHeaders && r.group !== lastGroup) { list.append(el('div', { class: 'wbm-palette-group', role: 'presentation', text: r.group })); lastGroup = r.group; }
        const row = el('div', {
          class: 'wbm-palette-row', role: 'option', id: `wbm-pal-${i}`, 'data-cmd': r.cmd.id, 'aria-selected': String(i === sel),
          on: { click: () => run(i), pointermove: () => { if (sel !== i) { sel = i; mark(); } } },
        }, [
          el('span', { class: 'wbm-palette-label', text: r.cmd.label }),
          showHeaders ? null : el('span', { class: 'wbm-palette-cat', text: r.cmd.category }),
          r.cmd.shortcut ? el('kbd', { class: 'wbm-palette-sc', text: fmtShortcut(r.cmd.shortcut) }) : null,
        ]);
        list.append(row);
      });
      mark(true);
    };
    const mark = (scroll = true) => {
      list.querySelectorAll('[role="option"]').forEach((n, i) => n.setAttribute('aria-selected', String(i === sel)));
      const n = list.querySelector(`#wbm-pal-${sel}`);
      if (n) { input.setAttribute('aria-activedescendant', n.id); if (scroll) n.scrollIntoView({ block: 'nearest' }); }
    };
    const run = (i) => {
      const r = rows[i]; if (!r) return;
      saveRecent(r.cmd.id);
      d.close(r.cmd);
    };
    input.addEventListener('input', () => { sel = 0; render(); });
    input.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown') { e.preventDefault(); sel = rows.length ? (sel + 1) % rows.length : 0; mark(); }
      else if (e.key === 'ArrowUp') { e.preventDefault(); sel = rows.length ? (sel - 1 + rows.length) % rows.length : 0; mark(); }
      else if (e.key === 'Enter') { e.preventDefault(); run(sel); }
    });
    render();
    const promise = d.show().then((cmd) => {
      current = null;
      if (cmd) { try { const r = cmd.run(); if (r && r.catch) r.catch((err) => showToast(String((err && err.message) || err), 'error')); } catch (err) { showToast(String((err && err.message) || err), 'error'); } }
    });
    current = { input, promise, close: () => d.close(undefined) };
    input.focus();
    return promise;
  };

  const off = onRequest(wb, (name) => { if (name === 'palette') open(); });
  return { destroy() { off(); current && current.close(); }, open, close: () => current && current.close() };
}

// ---- find on canvas ----------------------------------------------------------------------------------------------
export function mountSearch(wb, host) {
  ensureToasts(wb);
  const input = el('input', { type: 'text', class: 'wbm-search-input', 'aria-label': 'Find on canvas', placeholder: 'Find on canvas', 'data-action': 'search-input', autocomplete: 'off', spellcheck: 'false' });
  const count = el('span', { class: 'wbm-search-count', 'aria-live': 'polite', text: '' });
  const prev = el('button', { type: 'button', class: 'wbm-icon-btn', 'data-action': 'search-prev', 'aria-label': 'Previous match', title: 'Previous (Shift+Enter)' }, [mi('arrowUp', 15)]);
  const next = el('button', { type: 'button', class: 'wbm-icon-btn', 'data-action': 'search-next', 'aria-label': 'Next match', title: 'Next (Enter)' }, [mi('arrowDown', 15)]);
  const closeBtn = el('button', { type: 'button', class: 'wbm-icon-btn', 'data-action': 'search-close', 'aria-label': 'Close find', title: 'Close (Esc)' }, [mi('close', 15)]);
  const bar = el('div', { class: 'wbm-search', role: 'search', hidden: true }, [mi('search', 15), input, count, prev, next, closeBtn]);
  host.append(bar);

  let ids = []; let idx = 0; let savedView = null; let wasSelection = null;

  const restoreView = () => {
    if (!savedView) return;
    if (typeof wb.setView === 'function') wb.setView(savedView);
    else {
      wb.setZoom(savedView.zoom);
      const v = wb.view; wb.pan(savedView.x - v.x, savedView.y - v.y);
    }
  };
  const label = () => { count.textContent = input.value ? (ids.length ? `${idx + 1} / ${ids.length}` : '0 / 0') : ''; bar.classList.toggle('wbm-search-none', !!input.value && !ids.length); };
  const go = (i, focus = true) => {
    if (!ids.length) { label(); return; }
    idx = (i + ids.length) % ids.length;
    label();
    if (focus) wb.focusElement(ids[idx]);
  };
  const find = (keepId) => {
    const q = input.value.trim();
    ids = q ? wb.find(q) : [];
    const at = keepId != null ? ids.indexOf(keepId) : -1;
    idx = at >= 0 ? at : 0;
    label();
    if (ids.length && at < 0) go(0);
  };
  const onInput = debounce(() => find(), 80);
  input.addEventListener('input', onInput);
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); onInput.cancel(); if (!ids.length) find(); else go(idx + (e.shiftKey ? -1 : 1)); }
    else if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); hide(true); }
  });
  next.addEventListener('click', () => go(idx + 1));
  prev.addEventListener('click', () => go(idx - 1));
  closeBtn.addEventListener('click', () => hide(false));

  const show = () => {
    if (bar.hidden) { savedView = { ...wb.view }; wasSelection = [...wb.selection]; }
    bar.hidden = false;
    input.focus(); input.select();
    if (input.value) find();
  };
  const hide = (restore) => {
    if (bar.hidden) return;
    bar.hidden = true; onInput.cancel();
    if (restore) { restoreView(); if (!wasSelection || !wasSelection.length) wb.clearSelection(); }
    ids = []; label();
    const c = host.querySelector('[data-wb-canvas]'); if (c && c.focus) { try { c.focus({ preventScroll: true }); } catch { /* ignore */ } }
  };
  const offChange = wb.on('change', () => { if (!bar.hidden && input.value) { const keep = ids[idx]; const q = input.value.trim(); const nids = wb.find(q); if (nids.join() !== ids.join()) { ids = nids; idx = Math.max(0, ids.indexOf(keep)); label(); } } });
  const offReq = onRequest(wb, (name) => { if (name === 'search') show(); });
  return { destroy() { offChange && offChange(); offReq(); bar.remove(); }, open: show, close: () => hide(false) };
}

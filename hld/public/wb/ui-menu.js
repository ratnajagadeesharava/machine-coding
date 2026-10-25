// Main (hamburger) menu (agent D): files, export, find, palette, clear, theme, canvas background, view toggles, help.
import { CANVAS_BACKGROUNDS } from '../../shared/wbconst.js';
import { el, mi, fmtShortcut, confirmDialog, showToast, ensureToasts, onRequest, requestUi, trackTheme, safeName, openHelpDialog, toggleStats, statsOn } from './menu-icons.js';
import { openExportDialog, extractSceneFromSvg, sceneAppState } from './ui-export.js';
import { serializeScene, parseScene, downloadText, pickFile, readFileText } from './interop.js';

const SCENE_ACCEPT = '.json,.hld.json,.excalidraw,.excalidrawlib,.svg';
const SCENE_EXT_RE = /\.(json|excalidraw|excalidrawlib|svg)$/i;
const DROP_EXT_RE = /\.(json|excalidraw|excalidrawlib)$/i;
const LIB_EXT_RE = /\.(excalidrawlib|hldlib\.json)$/i;

function looksLikeLibrary(name, text) {
  if (LIB_EXT_RE.test(name || '')) return true;
  const head = String(text).slice(0, 400);
  return /"type"\s*:\s*"(excalidraw-library|excalidrawlib|hld-library)"/.test(head);
}

/** Read a file, parse it as a scene and load it into the board (confirming before replacing content). */
export async function loadSceneFile(wb, hooks, file) {
  let text;
  try { text = readFileText ? await readFileText(file) : await file.text(); } catch (err) { showToast(`Could not read file: ${err.message || err}`, 'error'); return false; }
  const embedded = extractSceneFromSvg(text);
  if (embedded) text = embedded;
  if (looksLikeLibrary(file.name, text)) { requestUi(wb, 'library-import', { text, name: file.name }); return true; }
  let parsed;
  try { parsed = parseScene(text); } catch (err) { showToast(`Could not open ${file.name}: ${err.message || err}`, 'error'); return false; }
  if (!wb.isEmpty()) {
    const ok = await confirmDialog({
      title: 'Open file', message: `Opening "${file.name}" replaces everything on the canvas. You can undo this only by reopening your previous file. Continue?`,
      confirmLabel: 'Replace canvas', danger: true,
    });
    if (!ok) return false;
  }
  wb.setBoard(parsed.board, { fit: true });
  const st = parsed.appState || {};
  const bg = st.background || st.viewBackgroundColor;
  if (typeof bg === 'string' && /^#[0-9a-f]{3,8}$/i.test(bg)) wb.setMode({ background: bg });
  const meta = { name: file.name.replace(/\.(hld\.json|json|excalidraw|svg)$/i, ''), fileName: file.name, appState: st };
  if (hooks.onOpen) hooks.onOpen(parsed.board, meta);
  return true;
}

export function mountMenu(wb, host, hooks = {}) {
  ensureToasts(wb);
  const root = el('div', { class: 'wbm-menu-root' });
  const button = el('button', {
    type: 'button', class: 'wbm-hamburger', 'data-action': 'menu', 'aria-label': 'Main menu', title: 'Main menu',
    'aria-haspopup': 'menu', 'aria-expanded': 'false',
  }, [mi('menu', 20)]);
  const pop = el('div', { class: 'wbm-popover wbm-main-menu', role: 'menu', 'aria-label': 'Main menu', hidden: true });
  root.append(button, pop);
  host.append(root);
  const offTheme = trackTheme(wb, root);

  const mode = () => wb.mode || {};
  const close = (restore = true) => {
    if (pop.hidden) return;
    pop.hidden = true; button.setAttribute('aria-expanded', 'false');
    if (restore) button.focus();
  };

  const openFileFlow = async () => {
    const file = await pickFile(SCENE_ACCEPT);
    if (file) await loadSceneFile(wb, hooks, file);
  };
  const saveFlow = () => {
    const name = safeName(hooks.fileName && hooks.fileName(), 'whiteboard');
    downloadText(`${name}.hld.json`, serializeScene(wb.getBoard(), { appState: sceneAppState(wb) }), 'application/json');
    if (hooks.onSave) hooks.onSave();
    showToast(`Saved ${name}.hld.json`, 'info');
  };
  const clearFlow = async () => {
    if (wb.isEmpty()) { showToast('The canvas is already empty', 'info'); return; }
    const ok = await confirmDialog({ title: 'Clear canvas', message: 'This removes everything from the canvas. You can undo it with Ctrl+Z.', confirmLabel: 'Clear canvas', danger: true });
    if (ok) wb.clear();
  };
  const helpFlow = () => openHelpDialog(wb);
  const statsFlow = () => toggleStats(wb, host);
  const isOn = (k) => () => !!mode()[k];

  /** Menu model. kind: item | group | sep | label | bg */
  const model = () => {
    const out = [];
    const extra = Array.isArray(hooks.extra) ? hooks.extra : [];
    extra.forEach((x, i) => out.push({ kind: 'item', id: `extra-${i}`, label: x.label, icon: x.icon || (x.href ? 'external' : 'home'), href: x.href, run: x.action }));
    if (extra.length) out.push({ kind: 'sep' });
    out.push(
      { kind: 'item', id: 'open', label: 'Open...', icon: 'open', shortcut: 'Mod+O', run: openFileFlow },
      { kind: 'item', id: 'save', label: 'Save to file', icon: 'save', shortcut: 'Mod+S', run: saveFlow },
      { kind: 'item', id: 'export-image', label: 'Export image...', icon: 'image', shortcut: 'Mod+Shift+E', run: () => openExportDialog(wb, hooks) },
      { kind: 'item', id: 'copy-png', label: 'Copy as PNG', icon: 'copy', shortcut: 'Shift+Alt+C', run: () => wb.copyAsPng() },
      { kind: 'item', id: 'copy-svg', label: 'Copy as SVG', icon: 'copy', run: () => wb.copyAsSvg() },
      { kind: 'sep' },
      { kind: 'item', id: 'find', label: 'Find on canvas', icon: 'search', shortcut: 'Mod+F', run: () => requestUi(wb, 'search') },
      { kind: 'item', id: 'palette', label: 'Command palette', icon: 'command', shortcut: 'Mod+/', run: () => requestUi(wb, 'palette') },
      { kind: 'item', id: 'clear', label: 'Clear canvas', icon: 'trash', run: clearFlow },
      { kind: 'item', id: 'reset-library', label: 'Reset the library', icon: 'library', run: () => requestUi(wb, 'library-reset') },
      { kind: 'sep' },
      { kind: 'item', id: 'theme', label: () => `Theme: ${({ dark: 'Dark', light: 'Light', system: 'System' })[mode().theme] || 'Light'}`, icon: () => (mode().theme === 'dark' ? 'moon' : 'sun'), shortcut: 'Alt+Shift+D', keepOpen: true, run: () => wb.toggleTheme() },
      { kind: 'item', id: 'grid', label: 'Grid', icon: 'grid', shortcut: "Mod+'", check: isOn('grid'), keepOpen: true, run: () => wb.toggleGrid() },
      { kind: 'item', id: 'snap', label: 'Object snapping', icon: 'magnet', shortcut: 'Alt+S', check: isOn('snap'), keepOpen: true, run: () => wb.toggleSnap() },
      { kind: 'item', id: 'zen', label: 'Zen mode', icon: 'zen', shortcut: 'Alt+Z', check: isOn('zen'), run: () => wb.toggleZen() },
      { kind: 'item', id: 'view-mode', label: 'View mode', icon: 'eye', shortcut: 'Alt+R', check: isOn('view'), run: () => wb.toggleViewMode() },
      { kind: 'item', id: 'stats', label: 'Stats', icon: 'stats', shortcut: 'Alt+/', check: () => statsOn(wb, host), keepOpen: true, run: statsFlow },
      { kind: 'sep' },
      { kind: 'bg' },
      { kind: 'sep' },
      { kind: 'item', id: 'shortcuts', label: 'Keyboard shortcuts', icon: 'help', shortcut: '?', run: helpFlow },
    );
    return out;
  };

  const refs = []; // { item, node, check, labelNode, iconSlot }
  const bgButtons = [];
  let custom = null;

  const build = () => {
    pop.textContent = '';
    refs.length = 0; bgButtons.length = 0;
    for (const it of model()) {
      if (it.kind === 'sep') { pop.append(el('div', { class: 'wbm-sep', role: 'separator' })); continue; }
      if (it.kind === 'bg') {
        const row = el('div', { class: 'wbm-bg-row', role: 'group', 'aria-label': 'Canvas background', 'data-row': '1' });
        row.append(el('div', { class: 'wbm-menu-label', text: 'Canvas background' }));
        const sw = el('div', { class: 'wbm-swatches', 'data-row': '1' });
        for (const c of CANVAS_BACKGROUNDS) {
          const b = el('button', {
            type: 'button', class: 'wbm-swatch', role: 'menuitemradio', 'data-wbm-item': '1', 'data-action': 'background', 'data-value': c,
            'aria-label': `Background ${c}`, title: c, style: `--sw:${c}`, 'aria-checked': 'false', tabindex: '-1',
            on: { click: () => wb.setMode({ background: c }) },
          });
          bgButtons.push(b); sw.append(b);
        }
        custom = el('input', { type: 'color', class: 'wbm-swatch-color', 'data-action': 'background-custom', 'aria-label': 'Custom background colour', title: 'Custom colour', 'data-wbm-item': '1', tabindex: '-1', value: '#ffffff' });
        custom.addEventListener('input', () => wb.setMode({ background: custom.value }));
        sw.append(custom);
        row.append(sw);
        pop.append(row);
        continue;
      }
      const tag = it.href ? 'a' : 'button';
      const node = el(tag, {
        class: 'wbm-item', role: it.check ? 'menuitemcheckbox' : 'menuitem', 'data-wbm-item': '1', 'data-action': it.id, tabindex: '-1',
        href: it.href, target: it.href ? '_self' : null, type: it.href ? null : 'button',
      });
      const iconSlot = el('span', { class: 'wbm-item-icon' });
      const labelNode = el('span', { class: 'wbm-item-label' });
      const sc = el('span', { class: 'wbm-item-sc', text: fmtShortcut(it.shortcut) });
      const chk = el('span', { class: 'wbm-item-check' }, [mi('check', 14)]);
      node.append(iconSlot, labelNode, sc);
      if (it.check) node.append(chk);
      node.addEventListener('click', (e) => {
        if (it.href && !it.run) return;
        if (it.href) e.preventDefault();
        if (!it.keepOpen) close(false);
        Promise.resolve().then(() => it.run && it.run()).catch((err) => showToast(String((err && err.message) || err), 'error')).finally(() => { if (it.keepOpen) update(); });
      });
      pop.append(node);
      refs.push({ it, node, chk, labelNode, iconSlot });
    }
    update();
  };

  const update = () => {
    for (const r of refs) {
      const lab = typeof r.it.label === 'function' ? r.it.label() : r.it.label;
      if (r.labelNode.textContent !== lab) r.labelNode.textContent = lab;
      const icon = typeof r.it.icon === 'function' ? r.it.icon() : r.it.icon;
      if (r.iconSlot.dataset.icon !== icon) { r.iconSlot.textContent = ''; r.iconSlot.append(mi(icon, 16)); r.iconSlot.dataset.icon = icon; }
      if (r.it.check) {
        const on = !!r.it.check();
        r.node.setAttribute('aria-checked', String(on));
        r.node.classList.toggle('wbm-on', on);
      }
    }
    const bg = String(mode().background || '').toLowerCase();
    let matched = false;
    for (const b of bgButtons) {
      const on = b.dataset.value.toLowerCase() === bg;
      b.setAttribute('aria-checked', String(on)); b.classList.toggle('wbm-on', on); matched = matched || on;
    }
    if (custom) {
      if (/^#[0-9a-f]{6}$/i.test(bg)) custom.value = bg;
      custom.classList.toggle('wbm-on', !!bg && !matched);
    }
  };

  const items = () => [...pop.querySelectorAll('[data-wbm-item]')];
  const focusAt = (i) => { const l = items(); if (l.length) l[(i + l.length) % l.length].focus(); };
  const open = () => {
    build();
    pop.hidden = false; button.setAttribute('aria-expanded', 'true');
    focusAt(0);
  };
  const toggle = () => (pop.hidden ? open() : close());

  button.addEventListener('click', toggle);
  button.addEventListener('keydown', (e) => { if (e.key === 'ArrowDown' && pop.hidden) { e.preventDefault(); open(); } });
  pop.addEventListener('keydown', (e) => {
    const l = items(); const i = l.indexOf(document.activeElement);
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(); }
    else if (e.key === 'ArrowDown') { e.preventDefault(); focusAt(i + 1); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); focusAt(i - 1); }
    else if (e.key === 'Home') { e.preventDefault(); focusAt(0); }
    else if (e.key === 'End') { e.preventDefault(); focusAt(l.length - 1); }
    else if ((e.key === 'ArrowRight' || e.key === 'ArrowLeft') && document.activeElement.parentElement?.dataset.row) {
      e.preventDefault();
      const sib = [...document.activeElement.parentElement.querySelectorAll('[data-wbm-item]')];
      const j = sib.indexOf(document.activeElement);
      sib[(j + (e.key === 'ArrowRight' ? 1 : -1) + sib.length) % sib.length].focus();
    } else if (e.key === 'Tab') close(false);
  });
  const onDocDown = (e) => { if (!pop.hidden && !root.contains(e.target)) close(false); };
  document.addEventListener('pointerdown', onDocDown, true);

  const offMode = wb.on ? wb.on('mode', () => { if (!pop.hidden) update(); }) : null;
  const offReq = onRequest(wb, (name) => {
    if (name === 'open') openFileFlow();
    else if (name === 'save') saveFlow();
    else if (name === 'export') openExportDialog(wb, hooks);
    else if (name === 'menu') toggle();
  });

  // ---- drag and drop a scene file onto the board
  const hint = el('div', { class: 'wbm-drop-hint', 'aria-hidden': 'true' }, [el('div', { text: 'Drop a .hld.json or .excalidraw file to open it' })]);
  hint.hidden = true;
  trackTheme(wb, hint);
  host.append(hint);
  let depth = 0;
  const hasFiles = (e) => e.dataTransfer && [...(e.dataTransfer.types || [])].includes('Files');
  const onEnter = (e) => { if (hasFiles(e)) { depth++; hint.hidden = false; } };
  const onLeave = (e) => { if (hasFiles(e)) { depth = Math.max(0, depth - 1); if (!depth) hint.hidden = true; } };
  const onOver = (e) => { if (hasFiles(e)) e.preventDefault(); };
  const onDrop = (e) => {
    depth = 0; hint.hidden = true;
    const f = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
    if (!f || !DROP_EXT_RE.test(f.name)) return;
    e.preventDefault(); e.stopPropagation();
    loadSceneFile(wb, hooks, f);
  };
  host.addEventListener('dragenter', onEnter, true);
  host.addEventListener('dragleave', onLeave, true);
  host.addEventListener('dragover', onOver, true);
  host.addEventListener('drop', onDrop, true);

  build();
  return {
    destroy() {
      document.removeEventListener('pointerdown', onDocDown, true);
      host.removeEventListener('dragenter', onEnter, true); host.removeEventListener('dragleave', onLeave, true);
      host.removeEventListener('dragover', onOver, true); host.removeEventListener('drop', onDrop, true);
      offMode && offMode(); offTheme && offTheme(); offReq(); root.remove(); hint.remove();
    },
    open, close, toggle, openFile: openFileFlow, save: saveFlow,
  };
}

// Export dialog (agent D): live SVG preview, options, PNG/SVG download, clipboard, .hld.json / .excalidraw save.
import { el, mi, makeDialog, showToast, debounce, safeName, ensureToasts } from './menu-icons.js';
import { serializeScene, exportExcalidraw, downloadText, downloadBlob } from './interop.js';

const META_RE = /<metadata id="hld-scene">([A-Za-z0-9+/=\s]*)<\/metadata>/;

export function utf8ToBase64(text) {
  const bytes = new TextEncoder().encode(text);
  let bin = '';
  for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(bin);
}
export function base64ToUtf8(b64) {
  const bin = atob(b64.replace(/\s+/g, ''));
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}
/** Insert a <metadata id="hld-scene"> element (base64 scene JSON) right after the opening <svg ...> tag. */
export function embedSceneInSvg(svg, sceneText) {
  const meta = `<metadata id="hld-scene">${utf8ToBase64(sceneText)}</metadata>`;
  return svg.replace(/<svg\b[^>]*>/, (m) => m + meta);
}
/** Scene JSON text embedded by embedSceneInSvg, or null. */
export function extractSceneFromSvg(text) {
  const m = META_RE.exec(String(text || ''));
  if (!m) return null;
  try { return base64ToUtf8(m[1]); } catch { return null; }
}

/** Editor state that travels with a saved scene. */
export function sceneAppState(wb) {
  const m = wb.mode || {};
  return { background: m.background, grid: !!m.grid, gridSize: m.gridSize, theme: m.theme };
}

export function openExportDialog(wb, hooks = {}) {
  ensureToasts(wb);
  const mode = wb.mode || {};
  const opts = { background: true, dark: mode.theme === 'dark', scale: 2, selectionOnly: false, embed: false };
  const d = makeDialog({ title: 'Export image', className: 'wbm-export', width: '52rem' });
  const { dialog } = d;

  // ---- preview
  const previewImg = el('img', { class: 'wbm-export-img', alt: 'Export preview' });
  const empty = el('div', { class: 'wbm-export-empty', text: 'Nothing to export yet. Draw something on the canvas first.', hidden: true });
  const info = el('div', { class: 'wbm-export-info' });
  const stage = el('div', { class: 'wbm-export-stage', 'data-checker': '0' }, [previewImg, empty]);
  const preview = el('div', { class: 'wbm-export-preview' }, [stage, info]);

  // ---- options
  const checks = {};
  const checkRow = (key, label, hint) => {
    const input = el('input', { type: 'checkbox', 'data-opt': key, id: `wbm-ex-${key}` });
    input.checked = !!opts[key];
    input.addEventListener('change', () => { opts[key] = input.checked; refresh(); });
    checks[key] = input;
    return el('label', { class: 'wbm-check-row', for: `wbm-ex-${key}` }, [input, el('span', {}, [el('b', { text: label }), hint ? el('small', { text: hint }) : null])]);
  };
  const scaleBtns = [1, 2, 3].map((s) => el('button', {
    type: 'button', class: 'wbm-seg-btn', 'data-opt': 'scale', 'data-value': String(s), 'aria-pressed': String(opts.scale === s), text: `${s}x`,
    on: { click: () => { opts.scale = s; scaleBtns.forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.value === String(s)))); refresh(); } },
  }));
  const nameInput = el('input', { type: 'text', class: 'wbm-input', id: 'wbm-ex-name', 'data-opt': 'name', value: safeName(hooks.fileName && hooks.fileName()), maxlength: '80', 'aria-label': 'File name' });
  const options = el('div', { class: 'wbm-export-opts' }, [
    el('label', { class: 'wbm-label', for: 'wbm-ex-name', text: 'File name' }), nameInput,
    checkRow('background', 'Background', 'Include the canvas background colour'),
    checkRow('dark', 'Dark mode', 'Use the dark colour mapping'),
    checkRow('selectionOnly', 'Only selected', 'Export just the selected elements'),
    checkRow('embed', 'Embed scene', 'Keep the scene inside the SVG so it can be re-opened'),
    el('div', { class: 'wbm-label', text: 'Scale' }),
    el('div', { class: 'wbm-seg', role: 'group', 'aria-label': 'Scale' }, scaleBtns),
  ]);

  // ---- buttons
  const actions = {};
  const btn = (action, label, icon, cls = '') => {
    const b = el('button', { type: 'button', class: `wbm-btn ${cls}`.trim(), 'data-action': action }, [mi(icon, 15), el('span', { text: label })]);
    actions[action] = b;
    return b;
  };
  const bPng = btn('export-png', 'Export to PNG', 'download', 'wbm-btn-primary');
  const bSvg = btn('export-svg', 'Export to SVG', 'download');
  const bCopyPng = btn('copy-png', 'Copy PNG', 'copy');
  const bCopySvg = btn('copy-svg', 'Copy SVG', 'copy');
  const bHld = btn('save-hld', 'Save as .hld.json', 'save');
  const bExc = btn('save-excalidraw', 'Save as .excalidraw', 'save');
  const bar = el('div', { class: 'wbm-export-actions' }, [
    el('div', { class: 'wbm-export-group' }, [bPng, bSvg]),
    el('div', { class: 'wbm-export-group' }, [bCopyPng, bCopySvg]),
    el('div', { class: 'wbm-export-group' }, [bHld, bExc]),
  ]);
  d.body.append(el('div', { class: 'wbm-export-main' }, [preview, options]), bar);
  d.footer.append(el('span', { class: 'wbm-export-status', 'aria-live': 'polite' }));
  const status = d.footer.firstChild;

  let hasContent = false;
  let busy = false;
  let objUrl = null;
  const baseName = () => safeName(nameInput.value, 'whiteboard');
  const exportOpts = (scale = opts.scale) => ({ selectionOnly: opts.selectionOnly, background: opts.background, dark: opts.dark, scale, padding: 10 });

  const syncControls = () => {
    const sel = wb.selection ? wb.selection.size : 0;
    checks.selectionOnly.disabled = sel === 0;
    if (sel === 0 && opts.selectionOnly) { opts.selectionOnly = false; checks.selectionOnly.checked = false; }
    checks.background.checked = opts.background;
    checks.dark.checked = opts.dark;
    for (const b of Object.values(actions)) b.disabled = busy || (!hasContent && !['save-hld', 'save-excalidraw'].includes(b.dataset.action));
    dialog.dataset.busy = busy ? '1' : '0';
  };
  const getSvg = () => {
    let svg = wb.toSVG(exportOpts());
    if (opts.embed) svg = embedSceneInSvg(svg, serializeScene(wb.getBoard(), { appState: sceneAppState(wb) }));
    return svg;
  };
  const dimsOf = (svg) => {
    const w = /<svg\b[^>]*\swidth="([\d.]+)/.exec(svg); const h = /<svg\b[^>]*\sheight="([\d.]+)/.exec(svg);
    return w && h ? [Math.round(+w[1]), Math.round(+h[1])] : null;
  };
  const renderPreview = () => {
    const sel = wb.selection ? wb.selection.size : 0;
    const board = wb.getBoard();
    hasContent = !wb.isEmpty() && !(opts.selectionOnly && sel === 0) && board.items.length > 0;
    stage.dataset.checker = !opts.background ? '1' : '0';
    empty.hidden = hasContent;
    previewImg.hidden = !hasContent;
    info.textContent = '';
    if (hasContent) {
      try {
        const svg = wb.toSVG({ ...exportOpts(1), scale: 1 });
        if (objUrl) URL.revokeObjectURL(objUrl);
        objUrl = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
        previewImg.src = objUrl;
        const dim = dimsOf(svg);
        if (dim) info.textContent = `${dim[0] * opts.scale} x ${dim[1] * opts.scale} px`;
      } catch (err) {
        hasContent = false; empty.hidden = false; previewImg.hidden = true;
        empty.textContent = `Preview failed: ${err.message || err}`;
      }
    }
    syncControls();
  };
  const refresh = debounce(renderPreview, 120);

  const run = async (label, fn) => {
    if (busy) return;
    busy = true; status.textContent = `${label}...`; syncControls();
    try { await fn(); status.textContent = ''; } catch (err) {
      status.textContent = '';
      showToast(`${label} failed: ${(err && err.message) || err}`, 'error');
    } finally { busy = false; syncControls(); }
  };

  actions['export-png'].addEventListener('click', () => run('Exporting PNG', async () => {
    const blob = await wb.toPNG(exportOpts());
    downloadBlob(`${baseName()}.png`, blob);
    showToast('PNG exported', 'info');
  }));
  actions['export-svg'].addEventListener('click', () => run('Exporting SVG', async () => {
    downloadText(`${baseName()}.svg`, getSvg(), 'image/svg+xml');
    showToast('SVG exported', 'info');
  }));
  actions['copy-png'].addEventListener('click', () => run('Copying PNG', async () => {
    if (!navigator.clipboard || !window.ClipboardItem) throw new Error('image clipboard is not available in this browser');
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': wb.toPNG(exportOpts()) })]);
    showToast('Copied PNG to clipboard', 'info');
  }));
  actions['copy-svg'].addEventListener('click', () => run('Copying SVG', async () => {
    if (!navigator.clipboard) throw new Error('clipboard is not available');
    await navigator.clipboard.writeText(getSvg());
    showToast('Copied SVG to clipboard', 'info');
  }));
  actions['save-hld'].addEventListener('click', () => run('Saving', async () => {
    downloadText(`${baseName()}.hld.json`, serializeScene(wb.getBoard(), { appState: sceneAppState(wb) }), 'application/json');
    showToast('Saved .hld.json', 'info');
  }));
  actions['save-excalidraw'].addEventListener('click', () => run('Saving', async () => {
    downloadText(`${baseName()}.excalidraw`, JSON.stringify(exportExcalidraw(wb.getBoard()), null, 2), 'application/json');
    showToast('Saved .excalidraw', 'info');
  }));

  const offs = [];
  if (wb.on) for (const t of ['change', 'selection', 'mode']) offs.push(wb.on(t, () => refresh()));
  dialog.addEventListener('close', () => { offs.forEach((o) => o && o()); refresh.cancel(); if (objUrl) URL.revokeObjectURL(objUrl); });

  renderPreview();
  const p = d.show();
  (hasContent ? bPng : nameInput).focus();
  return Object.assign(p, { dialog });
}

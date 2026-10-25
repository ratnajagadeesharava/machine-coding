// Bottom-left cluster: zoom out / % / zoom in / fit | undo redo | stats. mountZoomBar(wb, host) -> { destroy }
import { selectionBounds } from '/shared/board.js';
import { h, icon, ensureChromeCss, followTheme, guardKeys, rovingKeys } from './icons.js';

const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/i.test(navigator.platform || '');
const M = isMac ? 'Cmd' : 'Ctrl';

export function mountZoomBar(wb, host) {
  ensureChromeCss();
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  const root = h('div', { class: 'wbui-root wbui-zoom-root' });
  const btn = (action, ic, title, run, extra = {}) => {
    const b = h('button', { type: 'button', class: 'wbui-btn', dataset: { action }, title, 'aria-label': title.replace(/ \(.*\)$/, ''), ...extra }, icon(ic, 18));
    b.addEventListener('click', run);
    return b;
  };
  const out = btn('zoom-out', 'zoomMinus', `Zoom out (${M}+-)`, () => wb.zoomOut());
  const pct = h('button', { type: 'button', class: 'wbui-btn wbui-pct', dataset: { action: 'zoom-reset' }, title: `Reset zoom (${M}+0)`, 'aria-label': 'Reset zoom', text: '100%' });
  pct.addEventListener('click', () => wb.resetZoom());
  const zin = btn('zoom-in', 'zoomPlus', `Zoom in (${M}++)`, () => wb.zoomIn());
  const fit = btn('zoom-fit', 'fit', 'Fit to screen (Shift+1)', () => wb.zoomToFit());
  const undo = btn('undo', 'undo', `Undo (${M}+Z)`, () => wb.undo());
  const redo = btn('redo', 'redo', `Redo (${M}+Shift+Z)`, () => wb.redo());
  const stats = btn('stats', 'stats', 'Stats (Alt+/)', () => toggleStats(), { 'aria-pressed': 'false' });
  const zoomGrp = h('div', { class: 'wbui-cluster', role: 'group', 'aria-label': 'Zoom' }, out, pct, zin, fit);
  const histGrp = h('div', { class: 'wbui-cluster', role: 'group', 'aria-label': 'History' }, undo, redo);
  const statGrp = h('div', { class: 'wbui-cluster', role: 'group', 'aria-label': 'Statistics' }, stats);
  rovingKeys(zoomGrp); rovingKeys(histGrp);
  const bar = h('div', { class: 'wbui-zoombar' }, zoomGrp, histGrp, statGrp);

  const panel = h('div', { class: 'wbui-panel wbui-stats', role: 'region', 'aria-label': 'Statistics', hidden: true });
  const dl = (rows) => rows.map(([k, v]) => h('div', { class: 'wbui-stat', dataset: { stat: k.toLowerCase() } }, h('span', { class: 'wbui-stat-k', text: k }), h('span', { class: 'wbui-stat-v', text: v })));
  root.append(panel, bar);
  host.append(root);
  guardKeys(root);

  let open = false;
  function toggleStats() { open = !open; update(); }

  const f = (n) => (Math.round(n * 10) / 10).toString();
  function renderStats() {
    const board = wb.board;
    const items = board.items;
    const sel = items.filter((e) => wb.selection.has(e.id));
    const rows = [h('div', { class: 'wbui-stats-title', text: 'Stats' }),
      ...dl([['Elements', String(items.length)], ['Zoom', Math.round(wb.view.zoom * 100) + '%']])];
    rows.push(h('div', { class: 'wbui-stats-title', text: sel.length ? `Selection (${sel.length})` : 'Selection' }));
    if (!sel.length) rows.push(h('div', { class: 'wbui-stat-empty', text: 'Nothing selected' }));
    else {
      const b = selectionBounds(board, sel);
      const list = [['Width', f(b.w)], ['Height', f(b.h)], ['X', f(b.x)], ['Y', f(b.y)]];
      if (sel.length === 1) list.push(['Angle', f(((sel[0].angle || 0) * 180) / Math.PI % 360) + '°']);
      rows.push(...dl(list));
    }
    panel.replaceChildren(...rows);
  }

  function update() {
    const mode = wb.mode || {};
    root.hidden = !!mode.zen;
    pct.textContent = Math.round(wb.view.zoom * 100) + '%';
    undo.disabled = !wb.canUndo();
    redo.disabled = !wb.canRedo();
    stats.setAttribute('aria-pressed', open ? 'true' : 'false');
    stats.classList.toggle('is-active', open);
    panel.hidden = !open;
    if (open) renderStats();
  }
  let scheduled = false;
  const schedule = () => { if (scheduled) return; scheduled = true; queueMicrotask(() => { scheduled = false; update(); }); };
  const offs = ['view', 'history', 'selection', 'change', 'mode'].map((t) => wb.on(t, schedule));
  offs.push(followTheme(wb, root));
  update();

  return { update, toggleStats, destroy() { offs.forEach((o) => o && o()); root.remove(); } };
}

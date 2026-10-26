// Drag handles for the room's side panels: pull the edge to make the question or Aria panel wider or narrower.
// Keyboard: focus the handle and use the left/right arrows (Shift = bigger steps); double-click resets the width.
// Widths are remembered in this browser.
const STORE = 'hld-room-widths';
const MIN = 240;
const read = () => { try { return JSON.parse(localStorage.getItem(STORE)) || {}; } catch { return {}; } };
const write = (v) => { try { localStorage.setItem(STORE, JSON.stringify(v)); } catch { /* storage blocked */ } };
// Never wider than 55% of the window, and always leave the whiteboard at least 320px.
const maxWidth = (panel) => {
  const others = [...document.querySelectorAll('.room-body .panel')].filter((p) => p !== panel && !p.hidden).reduce((n, p) => n + p.getBoundingClientRect().width, 0);
  return Math.max(MIN, Math.min(Math.round(window.innerWidth * 0.55), window.innerWidth - others - 320 - 40));
};

// panel: the <aside>; edge: 'right' for a left-hand panel (handle on its right edge), 'left' for a right-hand panel.
export function makeResizable(panel, key, edge, fallback) {
  if (!panel) return null;
  const saved = read()[key];
  const clamp = (w) => Math.min(maxWidth(panel), Math.max(MIN, Math.round(w)));
  const apply = (w) => { panel.style.width = `${clamp(w)}px`; };
  if (Number.isFinite(saved)) apply(saved);

  const bar = document.createElement('div');
  bar.className = `resizer resizer-${edge}`;
  bar.setAttribute('role', 'separator');
  bar.setAttribute('aria-orientation', 'vertical');
  bar.setAttribute('aria-label', `Resize ${key} panel`);
  bar.title = 'Drag to resize, double-click to reset';
  bar.tabIndex = 0;
  if (edge === 'right') panel.after(bar); else panel.before(bar);

  const current = () => panel.getBoundingClientRect().width;
  const save = () => { const v = read(); v[key] = clamp(current()); write(v); bar.setAttribute('aria-valuenow', String(Math.round(current()))); };
  const sync = () => { bar.hidden = panel.hidden; };
  new MutationObserver(sync).observe(panel, { attributes: true, attributeFilter: ['hidden'] });
  sync();

  let start = null;
  bar.addEventListener('pointerdown', (e) => {
    if (e.button !== 0) return;
    start = { x: e.clientX, w: current() };
    bar.setPointerCapture(e.pointerId);
    document.body.classList.add('resizing');
    e.preventDefault();
  });
  bar.addEventListener('pointermove', (e) => {
    if (!start) return;
    const dx = e.clientX - start.x;
    apply(edge === 'right' ? start.w + dx : start.w - dx);
    window.dispatchEvent(new Event('resize')); // let the whiteboard re-measure
  });
  const stop = () => { if (!start) return; start = null; document.body.classList.remove('resizing'); save(); };
  bar.addEventListener('pointerup', stop);
  bar.addEventListener('pointercancel', stop);
  bar.addEventListener('dblclick', () => {
    panel.style.width = '';
    const v = read(); delete v[key]; write(v);
    window.dispatchEvent(new Event('resize'));
  });
  bar.addEventListener('keydown', (e) => {
    if (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight') return;
    e.preventDefault();
    const step = e.shiftKey ? 80 : 20;
    const grow = (e.key === 'ArrowRight') === (edge === 'right');
    apply(current() + (grow ? step : -step));
    window.dispatchEvent(new Event('resize'));
    save();
  });
  window.addEventListener('resize', () => { if (panel.style.width) apply(parseFloat(panel.style.width)); });
  return bar;
}

// Top-centre tool palette (Excalidraw style). mountToolbar(wb, host, { library?, onLibrary? }) -> { destroy }
import { TOOLS } from '/shared/wbconst.js';
import { h, icon, popover, rovingKeys, ensureChromeCss, followTheme, guardKeys, watchWidth } from './icons.js';

const MAIN = ['hand', 'select', 'rect', 'diamond', 'ellipse', 'arrow', 'line', 'freedraw', 'text', 'image', 'eraser'];
const MORE = ['database', 'sticky', 'frame', 'laser', 'bucket', 'eyedropper'];
const VIEW_OK = new Set(['hand', 'laser']);

export function toolTitle(t) {
  const keys = [t.key ? t.key.toUpperCase() : null, t.num].filter(Boolean);
  return keys.length ? `${t.label} - ${keys.join(' or ')}` : t.label;
}

export function mountToolbar(wb, host, options = {}) {
  ensureChromeCss();
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  const byId = new Map(TOOLS.map((t) => [t.id, t]));
  const buttons = new Map(); // tool id -> button (main row)
  const moreButtons = new Map();
  let morePop = null;

  const root = h('div', { class: 'wbui-root wbui-toolbar-root' });
  const bar = h('div', { class: 'wbui-toolbar', role: 'toolbar', 'aria-label': 'Drawing tools', 'aria-orientation': 'horizontal' });
  root.append(bar);

  const toolBtn = (id, extra = {}) => {
    const t = byId.get(id);
    const btn = h('button', {
      type: 'button', class: 'wbui-btn wbui-tool', dataset: { tool: id }, title: toolTitle(t),
      'aria-label': t.label, 'aria-pressed': 'false', tabindex: '-1', ...extra,
    }, icon(id, 20), t.num ? h('span', { class: 'wbui-kbd', 'aria-hidden': 'true', text: t.num }) : (t.key ? h('span', { class: 'wbui-kbd', 'aria-hidden': 'true', text: t.key.toUpperCase() }) : null));
    btn.addEventListener('click', () => { closeMore(false); wb.setTool(id); });
    return btn;
  };
  const sep = () => h('span', { class: 'wbui-sep', role: 'separator', 'aria-orientation': 'vertical' });

  const lockBtn = h('button', {
    type: 'button', class: 'wbui-btn wbui-toollock', dataset: { action: 'tool-lock' }, title: 'Keep selected tool active after drawing - Q',
    'aria-label': 'Keep selected tool active after drawing', 'aria-pressed': 'false', tabindex: '-1',
  }, icon('unlock', 20));
  lockBtn.addEventListener('click', () => wb.toggleToolLock());
  bar.append(lockBtn, sep());

  for (const id of MAIN) {
    const b = toolBtn(id);
    buttons.set(id, b);
    bar.append(b);
  }
  bar.append(sep());

  const moreBtn = h('button', {
    type: 'button', class: 'wbui-btn wbui-more', dataset: { action: 'more-tools' }, title: 'More tools', 'aria-label': 'More tools',
    'aria-haspopup': 'true', 'aria-expanded': 'false', tabindex: '-1',
  }, icon('moreTools', 20), h('span', { class: 'wbui-caret' }, icon('chevronDown', 10)));
  bar.append(moreBtn);

  let libBtn = null;
  if (options.library) {
    bar.append(sep());
    libBtn = h('button', {
      type: 'button', class: 'wbui-btn wbui-lib', dataset: { action: 'library' }, title: 'Library', 'aria-label': 'Library', tabindex: '-1',
    }, icon('library', 20));
    libBtn.addEventListener('click', () => { if (options.onLibrary) options.onLibrary(); });
    bar.append(libBtn);
  }

  function closeMore(restore) {
    if (morePop) { const p = morePop; morePop = null; p.close(restore); }
  }
  moreBtn.addEventListener('click', () => {
    if (morePop) { closeMore(true); return; }
    moreButtons.clear();
    const grid = h('div', { class: 'wbui-moregrid', role: 'group', 'aria-label': 'More tools' });
    for (const id of MORE) {
      const t = byId.get(id);
      const b = h('button', {
        type: 'button', class: 'wbui-btn wbui-tool wbui-tool-wide', dataset: { tool: id }, title: toolTitle(t), 'aria-label': t.label, 'aria-pressed': 'false',
      }, icon(id, 20), h('span', { class: 'wbui-tool-name', text: t.label }), t.key ? h('span', { class: 'wbui-kbd-inline', text: t.key.toUpperCase() }) : null);
      b.addEventListener('click', () => { closeMore(false); wb.setTool(id); });
      moreButtons.set(id, b);
      grid.append(b);
    }
    rovingKeys(grid);
    morePop = popover(root, moreBtn, grid, { className: 'wbui-pop-more', label: 'More tools', onClose: () => { morePop = null; moreButtons.clear(); } });
    update();
    const first = grid.querySelector('button:not([disabled])');
    if (first) first.focus();
  });

  // keyboard: roving tabindex across the bar
  rovingKeys(bar);
  guardKeys(root);
  bar.addEventListener('focusin', (e) => {
    const b = e.target.closest && e.target.closest('button');
    if (b && bar.contains(b)) setRoving(b);
  });
  function setRoving(active) {
    for (const b of bar.querySelectorAll('button')) b.setAttribute('tabindex', b === active ? '0' : '-1');
  }

  host.append(root);

  function update() {
    const view = !!(wb.mode && wb.mode.view);
    root.hidden = !!(wb.mode && wb.mode.zen);
    const tool = wb.tool;
    const locked = !!wb.toolLocked;
    bar.classList.toggle('is-view', view);
    for (const [id, b] of [...buttons, ...moreButtons]) {
      const on = id === tool;
      b.setAttribute('aria-pressed', on ? 'true' : 'false');
      b.classList.toggle('is-active', on);
      b.classList.toggle('is-locked', on && locked);
      b.disabled = view && !VIEW_OK.has(id);
    }
    // the "more tools" button mirrors the active hidden tool
    const inMore = MORE.includes(tool);
    moreBtn.classList.toggle('is-active', inMore);
    moreBtn.classList.toggle('is-locked', inMore && locked);
    const wantIcon = inMore ? tool : 'moreTools';
    if (moreBtn.dataset.icon !== wantIcon) {
      moreBtn.dataset.icon = wantIcon;
      moreBtn.firstChild.replaceWith(icon(wantIcon, 20));
    }
    moreBtn.title = inMore ? `More tools (${byId.get(tool).label})` : 'More tools';
    lockBtn.setAttribute('aria-pressed', locked ? 'true' : 'false');
    lockBtn.classList.toggle('is-active', locked);
    lockBtn.disabled = view;
    const want = locked ? 'lock' : 'unlock';
    if (lockBtn.dataset.icon !== want) { lockBtn.dataset.icon = want; lockBtn.firstChild.replaceWith(icon(want, 20)); }
    // roving tabindex: keep the active tool tabbable unless focus is inside
    if (!bar.contains(document.activeElement)) {
      const cur = buttons.get(tool) || (inMore ? moreBtn : null) || buttons.get('select');
      setRoving(cur && !cur.disabled ? cur : [...bar.querySelectorAll('button:not([disabled])')][0]);
    }
    if (morePop) morePop.place();
  }

  const offs = [wb.on('tool', update), wb.on('mode', update), followTheme(wb, root), watchWidth(host, root, 640)];
  update();

  return {
    update,
    destroy() {
      offs.forEach((o) => o && o());
      closeMore(false);
      root.remove();
    },
  };
}

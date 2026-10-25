// Right-click context menus (agent D): canvas menu and element menu, with Layers/Align submenus.
import { el, mi, fmtShortcut, placeInViewport, promptDialog, showToast, ensureToasts, currentTheme } from './menu-icons.js';

/** Entries for the menu shown on empty canvas. */
export function canvasEntries(wb) {
  const m = wb.mode || {};
  return [
    { id: 'paste', label: 'Paste', icon: 'paste', shortcut: 'Mod+V', run: () => wb.paste() },
    { id: 'select-all', label: 'Select all', shortcut: 'Mod+A', run: () => wb.selectAll() },
    { sep: true },
    { id: 'grid', label: 'Show grid', icon: 'grid', shortcut: "Mod+'", check: !!m.grid, run: () => wb.toggleGrid() },
    { id: 'snap', label: 'Object snapping', icon: 'magnet', shortcut: 'Alt+S', check: !!m.snap, run: () => wb.toggleSnap() },
    { id: 'zen', label: 'Zen mode', icon: 'zen', shortcut: 'Alt+Z', check: !!m.zen, run: () => wb.toggleZen() },
    { id: 'view-mode', label: 'View mode', icon: 'eye', shortcut: 'Alt+R', check: !!m.view, run: () => wb.toggleViewMode() },
  ];
}

/** Entries for the menu shown on one or more elements. */
export function elementEntries(wb) {
  const ids = [...(wb.selection || [])];
  const board = wb.board;
  const els = ids.map((id) => board.items.find((e) => e.id === id)).filter(Boolean);
  const n = els.length;
  const anyGrouped = els.some((e) => (e.groupIds || []).length);
  const allLocked = n > 0 && els.every((e) => e.locked);
  const link = n === 1 ? els[0].link : null;
  const align = (dir) => () => wb.align(dir);
  return [
    { id: 'cut', label: 'Cut', icon: 'cut', shortcut: 'Mod+X', run: () => wb.cut() },
    { id: 'copy', label: 'Copy', icon: 'copy', shortcut: 'Mod+C', run: () => wb.copy() },
    { id: 'paste', label: 'Paste', icon: 'paste', shortcut: 'Mod+V', run: () => wb.paste() },
    { id: 'copy-png', label: 'Copy to clipboard as PNG', shortcut: 'Shift+Alt+C', run: () => wb.copyAsPng() },
    { id: 'copy-svg', label: 'Copy to clipboard as SVG', run: () => wb.copyAsSvg() },
    { id: 'copy-style', label: 'Copy styles', shortcut: 'Mod+Alt+C', run: () => wb.copyStyle() },
    { id: 'paste-style', label: 'Paste styles', shortcut: 'Mod+Alt+V', run: () => wb.pasteStyle() },
    { sep: true },
    {
      id: 'link', label: link ? 'Edit link' : 'Add link', icon: 'link', shortcut: 'Mod+K', disabled: n !== 1,
      run: async () => {
        const v = await promptDialog({ title: link ? 'Edit link' : 'Add link', label: 'URL', value: link || '', placeholder: 'https://', confirmLabel: 'Save', allowEmpty: true });
        if (v === null) return;
        wb.setLink(v || null);
      },
    },
    { id: 'duplicate', label: 'Duplicate', shortcut: 'Mod+D', run: () => wb.duplicate() },
    { id: 'delete', label: 'Delete', icon: 'trash', shortcut: 'Delete', run: () => wb.deleteSelected() },
    { sep: true },
    { id: 'group', label: 'Group', icon: 'group', shortcut: 'Mod+G', disabled: n < 2, run: () => wb.group() },
    { id: 'ungroup', label: 'Ungroup', icon: 'ungroup', shortcut: 'Mod+Shift+G', disabled: !anyGrouped, run: () => wb.ungroup() },
    { id: 'lock', label: allLocked ? 'Unlock' : 'Lock', icon: allLocked ? 'unlock' : 'lock', shortcut: 'Mod+Shift+L', run: () => wb.toggleLock() },
    {
      id: 'layers', label: 'Layers', icon: 'layers', submenu: [
        { id: 'send-back', label: 'Send to back', shortcut: 'Mod+Shift+[', run: () => wb.sendToBack() },
        { id: 'send-backward', label: 'Send backward', shortcut: 'Mod+[', run: () => wb.sendBackward() },
        { id: 'bring-forward', label: 'Bring forward', shortcut: 'Mod+]', run: () => wb.bringForward() },
        { id: 'bring-front', label: 'Bring to front', shortcut: 'Mod+Shift+]', run: () => wb.bringToFront() },
      ],
    },
    { id: 'flip-h', label: 'Flip horizontal', icon: 'flip', shortcut: 'Shift+H', run: () => wb.flip('h') },
    { id: 'flip-v', label: 'Flip vertical', shortcut: 'Shift+V', run: () => wb.flip('v') },
    {
      id: 'align', label: 'Align', icon: 'align', disabled: n < 2, submenu: [
        { id: 'align-left', label: 'Align left', shortcut: 'Mod+Shift+Left', run: align('left') },
        { id: 'align-hcenter', label: 'Center horizontally', run: align('hcenter') },
        { id: 'align-right', label: 'Align right', shortcut: 'Mod+Shift+Right', run: align('right') },
        { id: 'align-top', label: 'Align top', shortcut: 'Mod+Shift+Up', run: align('top') },
        { id: 'align-vcenter', label: 'Center vertically', run: align('vcenter') },
        { id: 'align-bottom', label: 'Align bottom', shortcut: 'Mod+Shift+Down', run: align('bottom') },
        { sep: true },
        { id: 'distribute-h', label: 'Distribute horizontally', disabled: n < 3, run: () => wb.distribute('h') },
        { id: 'distribute-v', label: 'Distribute vertically', disabled: n < 3, run: () => wb.distribute('v') },
      ],
    },
  ];
}

export function mountContextMenu(wb, host) {
  ensureToasts(wb);
  let stack = []; // open menu nodes, root first
  let restoreFocus = null;
  let offDismiss = [];

  const closeFrom = (depth) => { while (stack.length > depth) stack.pop().remove(); };
  const closeAll = (restore = true) => {
    closeFrom(0);
    offDismiss.forEach((f) => f()); offDismiss = [];
    if (restore && restoreFocus && restoreFocus.isConnected) { try { restoreFocus.focus({ preventScroll: true }); } catch { /* ignore */ } }
    restoreFocus = null;
  };

  const itemsOf = (menu) => [...menu.querySelectorAll(':scope > [data-wbm-item]:not([aria-disabled="true"])')];

  function build(entries, depth, kind) {
    const menu = el('div', { class: `wbm-popover wbm-ctx ${depth ? 'wbm-ctx-sub' : ''}`, role: 'menu', 'data-menu': depth ? 'submenu' : kind, 'aria-label': depth ? 'Submenu' : 'Context menu', 'data-wbm-theme': currentTheme() });
    menu.style.position = 'fixed';
    for (const e of entries) {
      if (e.sep) { menu.append(el('div', { class: 'wbm-sep', role: 'separator' })); continue; }
      const node = el('button', {
        type: 'button', class: 'wbm-item', role: e.check !== undefined ? 'menuitemcheckbox' : 'menuitem', 'data-wbm-item': '1', 'data-action': e.id, tabindex: '-1',
        'aria-disabled': e.disabled ? 'true' : null, 'aria-haspopup': e.submenu ? 'menu' : null, 'aria-expanded': e.submenu ? 'false' : null,
        'aria-checked': e.check !== undefined ? String(!!e.check) : null,
      }, [
        el('span', { class: 'wbm-item-icon' }, e.icon ? [mi(e.icon, 16)] : []),
        el('span', { class: 'wbm-item-label', text: e.label }),
        e.submenu ? el('span', { class: 'wbm-item-chev' }, [mi('chevron', 14)]) : el('span', { class: 'wbm-item-sc', text: fmtShortcut(e.shortcut) }),
        e.check ? el('span', { class: 'wbm-item-check' }, [mi('check', 14)]) : null,
      ]);
      if (e.check) node.classList.add('wbm-on');
      node.addEventListener('pointerenter', () => {
        if (e.disabled) return;
        const idx = stack.indexOf(menu);
        closeFrom(idx + 1);
        if (e.submenu) openSub(node, e, idx + 1); else node.focus({ preventScroll: true });
      });
      node.addEventListener('click', (ev) => {
        ev.preventDefault();
        if (e.disabled) return;
        if (e.submenu) { openSub(node, e, stack.indexOf(menu) + 1, true); return; }
        closeAll();
        Promise.resolve().then(() => e.run && e.run()).catch((err) => showToast(String((err && err.message) || err), 'error'));
      });
      menu.append(node);
    }
    menu.addEventListener('keydown', (ev) => {
      const l = itemsOf(menu); const i = l.indexOf(document.activeElement);
      const idx = stack.indexOf(menu);
      if (ev.key === 'ArrowDown') { ev.preventDefault(); l[(i + 1) % l.length]?.focus(); }
      else if (ev.key === 'ArrowUp') { ev.preventDefault(); l[(i - 1 + l.length) % l.length]?.focus(); }
      else if (ev.key === 'Home') { ev.preventDefault(); l[0]?.focus(); }
      else if (ev.key === 'End') { ev.preventDefault(); l[l.length - 1]?.focus(); }
      else if (ev.key === 'ArrowRight' && document.activeElement?.getAttribute('aria-haspopup')) { ev.preventDefault(); document.activeElement.click(); }
      else if (ev.key === 'ArrowLeft' && idx > 0) { ev.preventDefault(); const parent = stack[idx - 1]; closeFrom(idx); parent.querySelector('[aria-expanded="true"]')?.focus(); }
      else if (ev.key === 'Escape') { ev.preventDefault(); ev.stopPropagation(); if (idx > 0) { const parent = stack[idx - 1]; closeFrom(idx); parent.querySelector('[aria-expanded="true"]')?.focus(); } else closeAll(); }
      else if (ev.key === 'Tab') { ev.preventDefault(); closeAll(); }
    });
    return menu;
  }

  function openSub(anchor, entry, depth, focus = false) {
    closeFrom(depth);
    for (const n of anchor.parentElement.querySelectorAll('[aria-expanded="true"]')) n.setAttribute('aria-expanded', 'false');
    anchor.setAttribute('aria-expanded', 'true');
    const sub = build(entry.submenu, depth, 'submenu');
    document.body.append(sub);
    stack.push(sub);
    const r = anchor.getBoundingClientRect();
    placeInViewport(sub, r.right - 4, r.top - 6);
    const sr = sub.getBoundingClientRect();
    if (sr.left < r.right - 6) placeInViewport(sub, r.left - sr.width + 4, r.top - 6); // flipped left when it did not fit
    if (focus) itemsOf(sub)[0]?.focus();
  }

  function show(entries, x, y, kind) {
    closeAll(false);
    restoreFocus = document.activeElement;
    const menu = build(entries, 0, kind);
    document.body.append(menu);
    stack = [menu];
    placeInViewport(menu, x, y);
    itemsOf(menu)[0]?.focus({ preventScroll: true });
    const down = (e) => { if (!stack.some((m) => m.contains(e.target))) closeAll(false); };
    const away = () => closeAll(false);
    const key = (e) => { if (e.key === 'Escape' && !stack.some((m) => m.contains(document.activeElement))) closeAll(); };
    document.addEventListener('pointerdown', down, true);
    window.addEventListener('wheel', away, { passive: true, capture: true });
    window.addEventListener('scroll', away, true);
    window.addEventListener('blur', away);
    window.addEventListener('resize', away);
    document.addEventListener('keydown', key, true);
    offDismiss = [
      () => document.removeEventListener('pointerdown', down, true),
      () => window.removeEventListener('wheel', away, { capture: true }),
      () => window.removeEventListener('scroll', away, true),
      () => window.removeEventListener('blur', away),
      () => window.removeEventListener('resize', away),
      () => document.removeEventListener('keydown', key, true),
    ];
  }

  const off = wb.on('contextmenu', (e) => {
    const { clientX, clientY, targetId } = e;
    if (targetId != null) {
      if (!wb.selection.has(targetId)) {
        if (typeof wb.select === 'function') wb.select([targetId]);
        else if (typeof wb.setSelection === 'function') wb.setSelection([targetId]);
      }
      show(elementEntries(wb), clientX, clientY, 'element');
    } else {
      show(canvasEntries(wb), clientX, clientY, 'canvas');
    }
  });

  return { destroy() { off && off(); closeAll(false); }, close: () => closeAll(), show };
}

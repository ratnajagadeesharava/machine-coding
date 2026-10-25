// Left-hand properties panel (Excalidraw style). mountProperties(wb, host) -> { destroy }
// Shown when something is selected or the active tool draws; each row only when wb.styleSummary().applies[prop].
import {
  STROKE_COLORS, BACKGROUND_COLORS, STICKY_COLORS, EXTENDED_COLORS, FILL_STYLES, ARROWHEADS, FONT_FAMILIES, FONT_SIZES, LIMITS,
} from '/shared/wbconst.js';
import { h, icon, popover, headIconName, ensureChromeCss, followTheme, guardKeys, rovingKeys } from './icons.js';

const MIXED = Symbol.for('wb.mixed');
const NO_PANEL_TOOLS = new Set(['hand', 'select', 'eraser', 'laser', 'eyedropper']);
const NARROW = 760;
const RECENT_KEY = 'wbui.recentColors';

const OPTION_ROWS = [
  { prop: 'fillStyle', label: 'Fill', opts: [['solid', 'fillSolid', 'Solid'], ['hachure', 'fillHachure', 'Hachure'], ['cross-hatch', 'fillCross', 'Cross-hatch'], ['zigzag', 'fillZigzag', 'Zigzag']] },
  { prop: 'strokeWidth', label: 'Stroke width', opts: [[1, 'widthThin', 'Thin'], [2, 'widthBold', 'Bold'], [4, 'widthExtra', 'Extra bold']] },
  { prop: 'strokeStyle', label: 'Stroke style', opts: [['solid', 'strokeSolid', 'Solid'], ['dashed', 'strokeDashed', 'Dashed'], ['dotted', 'strokeDotted', 'Dotted']] },
  { prop: 'roughness', label: 'Sloppiness', opts: [[0, 'sloppy0', 'Architect'], [1, 'sloppy1', 'Artist'], [2, 'sloppy2', 'Cartoonist']] },
  { prop: 'roundness', label: 'Edges', opts: [['sharp', 'edgeSharp', 'Sharp'], ['round', 'edgeRound', 'Round']] },
  { prop: 'arrowType', label: 'Arrow type', opts: [['sharp', 'arrowSharp', 'Sharp arrow'], ['round', 'arrowRound', 'Curved arrow'], ['elbow', 'arrowElbow', 'Elbow arrow']] },
];
const AFTER_HEADS = [
  { prop: 'fontFamily', label: 'Font family', opts: [[1, 'fontHand', FONT_FAMILIES[1].name], [2, 'fontNormal', FONT_FAMILIES[2].name], [3, 'fontCode', FONT_FAMILIES[3].name]] },
];
const TEXT_ROWS = [
  { prop: 'textAlign', label: 'Text align', opts: [['left', 'textLeft', 'Align left'], ['center', 'textCenter', 'Align centre'], ['right', 'textRight', 'Align right']] },
  { prop: 'verticalAlign', label: 'Vertical align', opts: [['top', 'vTop', 'Align top'], ['middle', 'vMiddle', 'Align middle'], ['bottom', 'vBottom', 'Align bottom']] },
];
const LAYERS = [
  ['send-to-back', 'toBack', 'Send to back', 'Ctrl+Shift+[', 'sendToBack'],
  ['send-backward', 'backward', 'Send backward', 'Ctrl+[', 'sendBackward'],
  ['bring-forward', 'forward', 'Bring forward', 'Ctrl+]', 'bringForward'],
  ['bring-to-front', 'toFront', 'Bring to front', 'Ctrl+Shift+]', 'bringToFront'],
];
const ALIGNS = [
  ['align-top', 'alignTop', 'Align top', 'top'], ['align-vcenter', 'alignVCenter', 'Centre vertically', 'vcenter'], ['align-bottom', 'alignBottom', 'Align bottom', 'bottom'],
  ['align-left', 'alignLeft', 'Align left', 'left'], ['align-hcenter', 'alignHCenter', 'Centre horizontally', 'hcenter'], ['align-right', 'alignRight', 'Align right', 'right'],
];

const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/i.test(navigator.platform || '');
const mod = isMac ? 'Cmd' : 'Ctrl';
const fixMod = (s) => s.replace('Ctrl', mod);

const sameColor = (a, b) => typeof a === 'string' && typeof b === 'string' && a.toLowerCase() === b.toLowerCase();

/** Normalise user typed hex: '#abc' / 'ABCDEF' / '#aabbccdd' -> '#aabbcc' form, or null when invalid. */
export function normalizeHex(text) {
  const t = String(text || '').trim().replace(/^#/, '').toLowerCase();
  if (/^[0-9a-f]{3}$/.test(t)) return '#' + t.split('').map((c) => c + c).join('');
  if (/^[0-9a-f]{6}$/.test(t) || /^[0-9a-f]{8}$/.test(t)) return '#' + t;
  return null;
}

/** Validate a link typed by the user: http(s) only. Returns the normalised URL string or null. */
export function normalizeLink(text) {
  const t = String(text || '').trim();
  if (!t || t.length > LIMITS.link) return null;
  const tryParse = (s) => { try { const u = new URL(s); return /^https?:$/.test(u.protocol) ? u.href : null; } catch { return null; } };
  if (/^[a-z][a-z0-9+.-]*:/i.test(t)) return tryParse(t);
  return /^[^\s/]+\.[^\s/]+/.test(t) ? tryParse('https://' + t) : null;
}

function loadRecent() {
  try { const v = JSON.parse(localStorage.getItem(RECENT_KEY) || '[]'); return Array.isArray(v) ? v.filter((c) => typeof c === 'string').slice(0, 8) : []; } catch { return []; }
}
function saveRecent(list) { try { localStorage.setItem(RECENT_KEY, JSON.stringify(list.slice(0, 8))); } catch { /* storage unavailable */ } }

export function mountProperties(wb, host) {
  ensureChromeCss();
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';

  const root = h('div', { class: 'wbui-root wbui-props-root' });
  const toggle = h('button', { type: 'button', class: 'wbui-btn wbui-props-toggle', dataset: { action: 'props-toggle' }, 'aria-expanded': 'false', 'aria-label': 'Properties', title: 'Properties' }, icon('sliders', 18), h('span', { text: 'Properties' }));
  const panel = h('div', { class: 'wbui-panel wbui-props', role: 'region', 'aria-label': 'Properties', tabindex: '-1' });
  root.append(toggle, panel);
  host.append(root);

  /** @type {{el:HTMLElement, prop?:string, value?:any, kind:string}[]} */
  const toggles = []; // pressed-state controls
  const rows = new Map(); // row key -> element
  const chips = new Map(); // colour prop -> chip button
  let narrow = false;
  let collapsed = true;
  let pop = null;
  let recent = loadRecent();

  const closePop = (restore = false) => { if (pop) { const p = pop; pop = null; p.close(restore); } };
  const HALF = new Set(['strokeWidth', 'strokeStyle', 'roughness', 'roundness']);
  const sectionRow = (key, label, body, extra) => {
    const el = h('section', { class: 'wbui-row' + (HALF.has(key) ? ' is-half' : ''), dataset: { row: key }, 'aria-label': label }, h('div', { class: 'wbui-label', text: label }), body);
    if (extra) extra(el);
    rows.set(key, el);
    panel.append(el);
    return el;
  };
  const iconBtn = (name, label, attrs, onclick, size = 18) => {
    const b = h('button', { type: 'button', class: 'wbui-btn wbui-opt', title: label, 'aria-label': label, ...attrs }, icon(name, size));
    b.addEventListener('click', onclick);
    return b;
  };
  const set = (prop, value) => { wb.setStyle({ [prop]: value }); };

  // ---- option rows (segmented icon buttons)
  const optionGroup = (def) => {
    const grp = h('div', { class: 'wbui-seg', role: 'group', 'aria-label': def.label, dataset: { group: def.prop } });
    for (const [value, ic, label] of def.opts) {
      const b = iconBtn(ic, label, { 'aria-pressed': 'false', dataset: { prop: def.prop, value: String(value) } }, () => set(def.prop, value));
      toggles.push({ el: b, prop: def.prop, value, kind: 'opt' });
      grp.append(b);
    }
    rovingKeys(grp);
    return grp;
  };
  const optionRow = (def) => sectionRow(def.prop, def.label, optionGroup(def));
  // horizontal + vertical text alignment share one row (saves a lot of height)
  const textAlignRow = () => sectionRow('textAlign', 'Text align', h('div', { class: 'wbui-seg wbui-seg-pair' }, optionGroup(TEXT_ROWS[0]), h('span', { class: 'wbui-vsep wbui-vsep-pair' }), optionGroup(TEXT_ROWS[1])));

  // ---- colour rows
  function colorRow(prop, label) {
    const quick = h('div', { class: 'wbui-swatches', role: 'group', 'aria-label': label });
    const holder = { quick, colors: [] };
    const chip = h('button', {
      type: 'button', class: 'wbui-chip', dataset: { propChip: prop }, 'aria-label': `${label}: more colours`, title: `${label} - more colours`,
      'aria-haspopup': 'dialog', 'aria-expanded': 'false',
    }, h('span', { class: 'wbui-chip-fill' }));
    chip.addEventListener('click', () => (pop && pop.owner === chip ? closePop(true) : openPalette(prop, label, chip)));
    chips.set(prop, chip);
    const body = h('div', { class: 'wbui-colorline' }, quick, h('span', { class: 'wbui-vsep' }), chip);
    sectionRow(prop, label, body);
    colorHolders.set(prop, holder);
  }
  const colorHolders = new Map();
  function fillQuick(prop) {
    const holder = colorHolders.get(prop);
    let colors = prop === 'strokeColor' ? STROKE_COLORS : BACKGROUND_COLORS;
    if (prop === 'backgroundColor' && stickyContext()) colors = STICKY_COLORS;
    const key = colors.join(',');
    if (holder.key === key) return;
    holder.key = key;
    for (let i = toggles.length - 1; i >= 0; i--) if (toggles[i].kind === 'swatch' && toggles[i].prop === prop) toggles.splice(i, 1);
    holder.quick.replaceChildren();
    for (const c of colors) {
      const b = h('button', {
        type: 'button', class: 'wbui-swatch' + (c === 'transparent' ? ' is-transparent' : ''), dataset: { prop, value: c },
        title: c === 'transparent' ? 'Transparent' : c, 'aria-label': c === 'transparent' ? 'Transparent' : c, 'aria-pressed': 'false',
        style: c === 'transparent' ? null : { '--wbui-sw': c },
      });
      b.addEventListener('click', () => set(prop, c));
      toggles.push({ el: b, prop, value: c, kind: 'swatch' });
      holder.quick.append(b);
    }
  }
  function stickyContext() {
    const ids = [...wb.selection];
    if (!ids.length) return wb.tool === 'sticky';
    return ids.every((id) => { const el = wb.board.items.find((e) => e.id === id); return el && el.type === 'sticky'; });
  }

  function openPalette(prop, label, anchor) {
    closePop(false);
    const cur = () => wb.styleSummary().values[prop];
    const content = h('div', { class: 'wbui-palette' });
    const pick = (c, remember = true) => {
      wb.setStyle({ [prop]: c });
      if (remember && c !== 'transparent') { recent = [c, ...recent.filter((x) => !sameColor(x, c))].slice(0, 8); saveRecent(recent); renderRecent(); }
      syncPal();
    };
    const mkSw = (c, extraClass = '') => {
      const b = h('button', {
        type: 'button', class: 'wbui-swatch wbui-swatch-sm ' + extraClass + (c === 'transparent' ? ' is-transparent' : ''), dataset: { prop, value: c, pal: '' },
        title: c, 'aria-label': c, 'aria-pressed': 'false', style: c === 'transparent' ? null : { '--wbui-sw': c },
      });
      b.addEventListener('click', () => pick(c));
      return b;
    };
    const basics = h('div', { class: 'wbui-pal-basics' });
    if (prop === 'backgroundColor') basics.append(mkSw('transparent'));
    for (const c of ['#ffffff', '#1e1e1e']) basics.append(mkSw(c));
    const hues = Object.keys(EXTENDED_COLORS);
    const grid = h('div', { class: 'wbui-pal-grid', style: { gridTemplateColumns: `repeat(${hues.length}, 1fr)` } });
    for (let i = 0; i < 5; i++) for (const hue of hues) grid.append(mkSw(EXTENDED_COLORS[hue][i]));
    const hex = h('input', {
      type: 'text', class: 'wbui-hex', maxlength: '9', spellcheck: 'false', autocomplete: 'off', 'aria-label': `${label} hex value`, dataset: { propHex: prop }, placeholder: 'rrggbb',
    });
    const err = h('div', { class: 'wbui-hex-err', 'aria-live': 'polite' });
    hex.addEventListener('input', () => {
      const raw = hex.value.trim();
      if (!raw) { hex.removeAttribute('aria-invalid'); err.textContent = ''; return; }
      const ok = normalizeHex(raw) || (prop === 'backgroundColor' && raw.toLowerCase() === 'transparent' ? 'transparent' : null);
      if (ok) { hex.removeAttribute('aria-invalid'); err.textContent = ''; pick(ok, false); }
      else { hex.setAttribute('aria-invalid', 'true'); err.textContent = 'Use a hex colour like #1971c2'; }
    });
    hex.addEventListener('change', () => { const ok = normalizeHex(hex.value); if (ok) pick(ok, true); });
    hex.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); const ok = normalizeHex(hex.value); if (ok) { pick(ok, true); } } });
    const recentBox = h('div', { class: 'wbui-pal-recent' });
    function renderRecent() {
      recentBox.replaceChildren();
      recentBox.parentElement && (recentBox.parentElement.hidden = !recent.length);
      for (const c of recent) recentBox.append(mkSw(c));
      syncPal();
    }
    const recentWrap = h('div', { class: 'wbui-pal-sec' }, h('div', { class: 'wbui-label', text: 'Recent' }), recentBox);
    content.append(
      h('div', { class: 'wbui-pal-sec' }, h('div', { class: 'wbui-label', text: label }), basics, grid),
      h('div', { class: 'wbui-pal-sec' }, h('div', { class: 'wbui-label', text: 'Hex' }), h('div', { class: 'wbui-hexline' }, h('span', { class: 'wbui-hash', text: '#' }), hex), err),
      recentWrap,
    );
    function syncPal() {
      const v = cur();
      for (const b of content.querySelectorAll('[data-pal]')) b.setAttribute('aria-pressed', v !== MIXED && sameColor(v, b.dataset.value) ? 'true' : 'false');
      if (document.activeElement !== hex) hex.value = typeof v === 'string' && v !== 'transparent' ? v.replace(/^#/, '') : (v === 'transparent' ? 'transparent' : '');
    }
    const me = popover(root, anchor, content, { side: 'right', className: 'wbui-pop-palette', label: `${label} colours`, onClose: () => { if (pop === me) pop = null; unsub(); } });
    me.owner = anchor;
    pop = me;
    const unsub = wb.on('style', syncPal);
    renderRecent();
    const first = content.querySelector('[aria-pressed="true"]') || content.querySelector('button');
    if (first) first.focus();
  }

  // ---- arrowheads
  function headRow() {
    const mk = (prop, label, flip) => {
      const b = h('button', {
        type: 'button', class: 'wbui-btn wbui-headbtn', dataset: { propPicker: prop }, title: label, 'aria-label': label, 'aria-haspopup': 'dialog', 'aria-expanded': 'false',
      }, h('span', { class: 'wbui-headicon' + (flip ? ' is-flip' : '') }), icon('chevronDown', 12));
      b.addEventListener('click', () => (pop && pop.owner === b ? closePop(true) : openHeads(prop, label, flip, b)));
      return b;
    };
    const dual = h('input', { type: 'checkbox', class: 'wbui-dual', 'aria-label': 'Double-pointed arrow', dataset: { dual: '' } });
    dual.addEventListener('change', () => {
      const end = wb.styleSummary().values.endArrowhead;
      const head = end && end !== MIXED ? end : 'arrow';
      wb.setStyle(dual.checked ? { startArrowhead: head, endArrowhead: head } : { startArrowhead: null, endArrowhead: head });
    });
    const wrap = h('div', { class: 'wbui-heads' },
      h('label', { class: 'wbui-dual-row' }, dual, h('span', { text: 'Double-pointed' })),
      h('div', { class: 'wbui-headcol', dataset: { headcol: 'start' } }, h('span', { class: 'wbui-sublabel', text: 'Start' }), mk('startArrowhead', 'Start arrowhead', true)),
      h('div', { class: 'wbui-headcol' }, h('span', { class: 'wbui-sublabel', text: 'End' }), mk('endArrowhead', 'End arrowhead', false)),
    );
    sectionRow('arrowheads', 'Arrowheads', wrap);
  }
  function openHeads(prop, label, flip, anchor) {
    closePop(false);
    const grid = h('div', { class: 'wbui-headgrid', role: 'group', 'aria-label': label });
    const cur = () => wb.styleSummary().values[prop];
    for (const a of ARROWHEADS) {
      const b = h('button', {
        type: 'button', class: 'wbui-btn wbui-opt wbui-headopt' + (flip ? ' is-flip' : ''), dataset: { prop, value: String(a.id), pal: '' }, title: a.label, 'aria-label': a.label, 'aria-pressed': 'false',
      }, icon(headIconName(a.id), 26));
      b.addEventListener('click', () => {
        const v = wb.styleSummary().values;
        const isDual = v.startArrowhead && v.startArrowhead !== MIXED; // both ends move together while double-pointed
        wb.setStyle(isDual && a.id ? { startArrowhead: a.id, endArrowhead: a.id } : { [prop]: a.id });
        closePop(true);
      });
      grid.append(b);
    }
    const sync = () => {
      const v = cur();
      for (const b of grid.querySelectorAll('button')) b.setAttribute('aria-pressed', v !== MIXED && b.dataset.value === String(v === undefined ? null : v) ? 'true' : 'false');
    };
    sync();
    rovingKeys(grid);
    const me = popover(root, anchor, grid, { side: 'right', className: 'wbui-pop-heads', label, onClose: () => { if (pop === me) pop = null; } });
    me.owner = anchor;
    pop = me;
    const first = grid.querySelector('[aria-pressed="true"]') || grid.querySelector('button');
    if (first) first.focus();
  }

  // ---- font size
  function fontSizeRow() {
    const grp = h('div', { class: 'wbui-seg', role: 'group', 'aria-label': 'Font size' });
    for (const [k, v] of Object.entries(FONT_SIZES)) {
      const b = h('button', { type: 'button', class: 'wbui-btn wbui-opt wbui-textopt', title: `${k} (${v}px)`, 'aria-label': `Font size ${k}`, 'aria-pressed': 'false', dataset: { prop: 'fontSize', value: String(v) } }, k);
      b.addEventListener('click', () => set('fontSize', v));
      toggles.push({ el: b, prop: 'fontSize', value: v, kind: 'opt' });
      grp.append(b);
    }
    const num = h('input', { type: 'number', class: 'wbui-num', min: String(LIMITS.fontSize[0]), max: String(LIMITS.fontSize[1]), step: '1', 'aria-label': 'Custom font size', title: 'Custom font size (Ctrl+Shift+< / >)', dataset: { propNum: 'fontSize' } });
    num.addEventListener('change', () => {
      const n = Math.round(Number(num.value));
      if (Number.isFinite(n)) wb.setStyle({ fontSize: Math.max(LIMITS.fontSize[0], Math.min(LIMITS.fontSize[1], n)) });
    });
    rovingKeys(grp);
    sectionRow('fontSize', 'Font size', h('div', { class: 'wbui-sizeline' }, grp, num));
    numInput = num;
  }
  let numInput = null;

  // ---- opacity
  let opRange = null;
  let opOut = null;
  function opacityRow() {
    opRange = h('input', { type: 'range', class: 'wbui-range', min: '0', max: '100', step: '5', value: '100', 'aria-label': 'Opacity', dataset: { propRange: 'opacity' } });
    opOut = h('output', { class: 'wbui-opval', 'aria-hidden': 'true', text: '100' });
    opRange.addEventListener('input', () => {
      opOut.textContent = opRange.value;
      opRange.style.setProperty('--wbui-fill', opRange.value + '%');
      wb.setStyle({ opacity: Number(opRange.value) });
    });
    sectionRow('opacity', 'Opacity', h('div', { class: 'wbui-opline' }, opRange, opOut));
  }

  // ---- actions rows
  const actionBtns = new Map();
  function actionRow(key, label, defs, cls = '') {
    const grp = h('div', { class: 'wbui-seg wbui-actions ' + cls, role: 'group', 'aria-label': label });
    for (const [action, ic, title, run, hint] of defs) {
      const b = iconBtn(ic, hint ? `${title} (${fixMod(hint)})` : title, { dataset: { action }, 'aria-label': title }, run);
      actionBtns.set(action, b);
      grp.append(b);
    }
    rovingKeys(grp);
    sectionRow(key, label, grp);
  }

  // ---- link dialog
  function openLinkDialog(anchor) {
    const ids = [...wb.selection];
    const first = wb.board.items.find((e) => e.id === ids[0]);
    const current = (first && first.link) || '';
    const input = h('input', { type: 'text', class: 'wbui-input', value: current, placeholder: 'https://example.com', spellcheck: 'false', autocomplete: 'off', 'aria-label': 'Link URL', dataset: { linkInput: '' } });
    const err = h('div', { class: 'wbui-hex-err', role: 'alert' });
    const save = h('button', { type: 'submit', class: 'wbui-btn wbui-primary', dataset: { action: 'link-save' }, text: 'Save' });
    const cancel = h('button', { type: 'button', class: 'wbui-btn wbui-secondary', dataset: { action: 'link-cancel' }, text: 'Cancel' });
    const remove = current ? h('button', { type: 'button', class: 'wbui-btn wbui-danger', dataset: { action: 'link-remove' }, text: 'Remove link' }) : null;
    const form = h('form', { method: 'dialog', class: 'wbui-dlg-form' },
      h('h2', { class: 'wbui-dlg-title', text: current ? 'Edit link' : 'Add link' }),
      h('label', { class: 'wbui-label' }, 'URL (http or https)'), input, err,
      h('div', { class: 'wbui-dlg-actions' }, remove, h('span', { class: 'wbui-spacer' }), cancel, save));
    const dlg = h('dialog', { class: 'wbui-dialog', 'aria-label': 'Link' }, form);
    root.append(dlg);
    const done = () => { dlg.close(); dlg.remove(); if (anchor && anchor.isConnected) anchor.focus({ preventScroll: true }); };
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      const v = input.value.trim();
      if (!v) { wb.setLink(null); done(); return; }
      const url = normalizeLink(v);
      if (!url) { err.textContent = 'Enter a valid http:// or https:// address.'; input.setAttribute('aria-invalid', 'true'); input.focus(); return; }
      wb.setLink(url);
      done();
    });
    cancel.addEventListener('click', done);
    if (remove) remove.addEventListener('click', () => { wb.setLink(null); done(); });
    dlg.addEventListener('cancel', (e) => { e.preventDefault(); done(); });
    dlg.addEventListener('keydown', (e) => e.stopPropagation());
    input.addEventListener('input', () => { err.textContent = ''; input.removeAttribute('aria-invalid'); });
    dlg.showModal();
    input.focus();
    input.select();
  }

  // ---- build rows in Excalidraw order
  colorRow('strokeColor', 'Stroke');
  colorRow('backgroundColor', 'Background');
  optionRow(OPTION_ROWS[0]);
  for (const def of OPTION_ROWS.slice(1, 6)) optionRow(def);
  headRow();
  optionRow(AFTER_HEADS[0]);
  fontSizeRow();
  textAlignRow();
  opacityRow();
  actionRow('layers', 'Layers', LAYERS.map(([a, ic, t, k, m]) => [a, ic, t, () => wb[m](), k]));
  actionRow('align', 'Align', ALIGNS.map(([a, ic, t, dir]) => [a, ic, t, () => wb.align(dir)]));
  actionRow('distribute', 'Distribute', [
    ['distribute-h', 'distH', 'Distribute horizontally', () => wb.distribute('h')],
    ['distribute-v', 'distV', 'Distribute vertically', () => wb.distribute('v')],
  ]);
  actionRow('actions', 'Actions', [
    ['duplicate', 'duplicate', 'Duplicate', () => wb.duplicate(), 'Ctrl+D'],
    ['delete', 'trash', 'Delete', () => wb.deleteSelected(), 'Delete'],
    ['group', 'group', 'Group', () => wb.group(), 'Ctrl+G'],
    ['ungroup', 'ungroup', 'Ungroup', () => wb.ungroup(), 'Ctrl+Shift+G'],
    ['toggle-lock', 'lock', 'Lock', () => wb.toggleLock(), 'Ctrl+Shift+L'],
    ['link', 'link', 'Link', () => openLinkDialog(actionBtns.get('link')), 'Ctrl+K'],
    ['flip-h', 'flipH', 'Flip horizontal', () => wb.flip('h'), 'Shift+H'],
    ['flip-v', 'flipV', 'Flip vertical', () => wb.flip('v'), 'Shift+V'],
    ['copy-style', 'copyStyle', 'Copy style', () => wb.copyStyle(), 'Ctrl+Alt+C'],
    ['paste-style', 'pasteStyle', 'Paste style', () => wb.pasteStyle(), 'Ctrl+Alt+V'],
  ], 'wbui-actions-wrap');

  // ---- update
  function selectedElements() {
    const items = wb.board.items;
    const sel = wb.selection;
    return items.filter((e) => sel.has(e.id));
  }
  function update() {
    root.hidden = false;
    const mode = wb.mode || {};
    const sel = wb.selection;
    const tool = wb.tool;
    const view = !!mode.view;
    const drawing = !NO_PANEL_TOOLS.has(tool);
    const wanted = !view && !mode.zen && (sel.size > 0 || drawing);
    const sum = wb.styleSummary();
    const { values, applies } = sum;
    const els = sel.size ? selectedElements() : [];
    const count = sel.size;

    const show = (key, on) => { const r = rows.get(key); if (r) r.hidden = !on; return on; };
    let any = false;
    for (const prop of ['strokeColor', 'backgroundColor']) { if (show(prop, !!applies[prop])) { any = true; fillQuick(prop); } }
    for (const def of [...OPTION_ROWS, ...AFTER_HEADS]) any = show(def.prop, !!applies[def.prop]) || any;
    any = show('textAlign', !!(applies.textAlign || applies.verticalAlign)) || any;
    for (const prop of ['textAlign', 'verticalAlign']) { const g = panel.querySelector(`[data-group="${prop}"]`); if (g) g.hidden = !applies[prop]; }
    const pairSep = panel.querySelector('.wbui-vsep-pair');
    if (pairSep) pairSep.hidden = !(applies.textAlign && applies.verticalAlign);
    any = show('arrowheads', !!(applies.startArrowhead || applies.endArrowhead)) || any;
    any = show('fontSize', !!applies.fontSize) || any;
    any = show('opacity', !!applies.opacity) || any;
    const hasSel = count > 0;
    any = show('layers', hasSel) || any;
    any = show('align', count >= 2) || any;
    any = show('distribute', count >= 3) || any;
    any = show('actions', hasSel) || any;

    for (const t of toggles) {
      const row = t.el.closest('[data-row]');
      if (row && row.hidden) continue;
      const v = values[t.prop];
      let on;
      if (v === MIXED || v === undefined) on = false;
      else if (t.kind === 'swatch') on = sameColor(v, t.value);
      else on = v === t.value;
      t.el.setAttribute('aria-pressed', on ? 'true' : 'false');
      t.el.classList.toggle('is-on', on);
    }
    for (const [prop, chip] of chips) {
      const v = values[prop];
      const fill = chip.firstChild;
      const mixed = v === MIXED || v === undefined;
      chip.classList.toggle('is-mixed', mixed);
      chip.classList.toggle('is-transparent', v === 'transparent');
      const quickMatch = typeof v === 'string' && (prop === 'strokeColor' ? STROKE_COLORS : stickyContext() ? STICKY_COLORS : BACKGROUND_COLORS).some((c) => sameColor(c, v));
      chip.classList.toggle('is-custom', !mixed && !quickMatch);
      fill.style.setProperty('--wbui-sw', mixed || v === 'transparent' ? 'transparent' : String(v));
      chip.dataset.value = mixed ? '' : String(v);
    }
    // single / double-pointed toggle
    {
      const dual = panel.querySelector('[data-dual]');
      const startCol = panel.querySelector('[data-headcol="start"]');
      if (dual) {
        const sv = values.startArrowhead;
        dual.indeterminate = sv === MIXED;
        dual.checked = sv !== MIXED && !!sv;
        dual.disabled = !applies.startArrowhead;
        if (startCol) startCol.hidden = !dual.checked && sv !== MIXED;
      }
    }
    // arrowhead picker previews
    for (const prop of ['startArrowhead', 'endArrowhead']) {
      const btn = panel.querySelector(`[data-prop-picker="${prop}"]`);
      if (!btn) continue;
      const slot = btn.querySelector('.wbui-headicon');
      const v = values[prop];
      const name = v === MIXED ? 'moreTools' : headIconName(v === undefined ? null : v);
      if (slot.dataset.icon !== name) { slot.dataset.icon = name; slot.replaceChildren(icon(name, 26)); }
      btn.disabled = !applies[prop];
      btn.dataset.value = v === MIXED ? 'mixed' : String(v === undefined ? null : v);
    }
    // font size custom input
    if (numInput) {
      const v = values.fontSize;
      numInput.value = typeof v === 'number' ? String(v) : '';
      numInput.placeholder = v === MIXED ? '-' : '';
    }
    // opacity
    if (opRange) {
      const v = values.opacity;
      const num = typeof v === 'number' ? v : (v === MIXED ? 100 : 100);
      opRange.value = String(num);
      opRange.style.setProperty('--wbui-fill', opRange.value + '%');
      opOut.textContent = v === MIXED ? '-' : String(Math.round(typeof v === 'number' ? v : 100));
      opRange.classList.toggle('is-mixed', v === MIXED);
    }
    // action states
    const inGroup = els.some((e) => e.groupIds && e.groupIds.length);
    const showAct = (a, on) => { const b = actionBtns.get(a); if (b) b.hidden = !on; };
    showAct('group', count >= 2);
    showAct('ungroup', inGroup);
    showAct('link', count >= 1);
    const allLocked = els.length > 0 && els.every((e) => e.locked);
    const lockBtn = actionBtns.get('toggle-lock');
    if (lockBtn) {
      lockBtn.setAttribute('aria-pressed', allLocked ? 'true' : 'false');
      lockBtn.title = (allLocked ? 'Unlock' : 'Lock') + ` (${fixMod('Ctrl+Shift+L')})`;
      lockBtn.setAttribute('aria-label', allLocked ? 'Unlock' : 'Lock');
      const want = allLocked ? 'unlock' : 'lock';
      if (lockBtn.dataset.icon !== want) { lockBtn.dataset.icon = want; lockBtn.firstChild.replaceWith(icon(want, 18)); }
    }
    const linkBtn = actionBtns.get('link');
    if (linkBtn) linkBtn.classList.toggle('is-on', els.length > 0 && els.every((e) => e.link));

    // visibility + responsive collapse
    const visible = wanted && any;
    root.classList.toggle('is-narrow', narrow);
    toggle.hidden = !(narrow && visible);
    panel.hidden = !visible || (narrow && collapsed);
    root.classList.toggle('is-hidden', !visible);
    toggle.setAttribute('aria-expanded', narrow && !collapsed ? 'true' : 'false');
    if (!visible) closePop(false);
    if (pop) pop.place();
  }

  let scheduled = false;
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    queueMicrotask(() => { scheduled = false; update(); });
  };

  toggle.addEventListener('click', () => { collapsed = !collapsed; if (collapsed) closePop(false); update(); if (!collapsed) panel.focus({ preventScroll: true }); });
  guardKeys(root);

  const ro = typeof ResizeObserver !== 'undefined' ? new ResizeObserver(() => {
    const n = host.clientWidth > 0 && host.clientWidth < NARROW;
    if (n !== narrow) { narrow = n; if (n) collapsed = true; update(); }
  }) : null;
  if (ro) ro.observe(host);
  narrow = host.clientWidth > 0 && host.clientWidth < NARROW;

  const offs = ['selection', 'style', 'tool', 'change', 'mode', 'history'].map((t) => wb.on(t, schedule));
  offs.push(wb.on('request', (e) => { if (e && e.name === 'link' && wb.selection.size && !(wb.mode && wb.mode.view)) openLinkDialog(actionBtns.get('link')); }));
  offs.push(followTheme(wb, root));
  update();

  return {
    update,
    destroy() {
      offs.forEach((o) => o && o());
      if (ro) ro.disconnect();
      closePop(false);
      root.remove();
    },
  };
}
